//! Invitation endpoints — invite users to governance proposals via alias.
//!
//! Endpoints:
//! - POST /governance/invitations          — create an invitation
//! - GET  /governance/invitations?voter=X  — list invitations for a voter (by commitment)
//! - POST /governance/invitations/respond  — accept or reject an invitation

use crate::api::errors::{ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use crate::storage::traits::Invitation;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Max proposal IDs per invitation (anti-spam).
const MAX_PROPOSALS_PER_INVITATION: usize = 20;

fn err_dto(code: &str, msg: &str) -> ErrorDto {
    ErrorDto {
        code: code.to_string(),
        message: msg.to_string(),
        field: None,
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Verify an Ed25519 signature over a message.
fn verify_ed25519(public_key_hex: &str, message: &[u8], signature_hex: &str) -> bool {
    let pub_bytes = match hex::decode(public_key_hex) {
        Ok(b) if b.len() == 32 => b,
        _ => return false,
    };
    let sig_bytes = match hex::decode(signature_hex) {
        Ok(b) if b.len() == 64 => b,
        _ => return false,
    };
    use pqc_crypto_module::legacy::ed25519::{Signature, Verifier, VerifyingKey};
    match (
        pub_bytes
            .as_slice()
            .try_into()
            .ok()
            .and_then(|b: &[u8; 32]| VerifyingKey::from_bytes(b).ok()),
        Signature::from_slice(&sig_bytes).ok(),
    ) {
        (Some(vk), Some(sig)) => vk.verify(message, &sig).is_ok(),
        _ => false,
    }
}

// ── Request types ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateInvitationRequest {
    pub from_did: String,
    /// Ed25519 public key of the inviter (hex, 64 chars = 32 bytes).
    pub public_key: String,
    /// Alias commitment of the invitee.
    pub to_commitment: String,
    /// Proposal IDs to invite the user to (max 20).
    pub proposal_ids: Vec<u64>,
    /// Ed25519 signature from the inviter (hex).
    pub signature: String,
}

#[derive(Deserialize)]
pub struct InvitationQuery {
    /// Voter address or commitment to filter by.
    pub voter: Option<String>,
}

#[derive(Deserialize)]
pub struct RespondInvitationRequest {
    pub invitation_id: String,
    /// Ed25519 public key of the invitee (hex, 64 chars = 32 bytes).
    pub public_key: String,
    pub accepted: bool,
    /// Ed25519 signature from the invitee (hex).
    pub signature: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/v1/governance/invitations
#[post("/governance/invitations")]
pub async fn create_invitation(
    state: web::Data<AppState>,
    body: web::Json<CreateInvitationRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Validate proposal count
    if body.proposal_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("INVALID_PROPOSALS", "at least one proposal_id is required"),
            400,
        )));
    }
    if body.proposal_ids.len() > MAX_PROPOSALS_PER_INVITATION {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "TOO_MANY_PROPOSALS",
                &format!("max {MAX_PROPOSALS_PER_INVITATION} proposal_ids per invitation"),
            ),
            400,
        )));
    }

    // Verify the invitee commitment exists and is active
    match store.read_alias(&body.to_commitment) {
        Ok(entry) if entry.status == "active" => {}
        _ => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto(
                    "ALIAS_NOT_FOUND",
                    "invitee alias commitment not found or not active",
                ),
                404,
            )));
        }
    }

    // Rate limit: max 5 invitations per hour per from_did
    let recent = store
        .list_invitations_by_sender(&body.from_did)
        .unwrap_or_default();
    let one_hour_ago = now_secs().saturating_sub(3600);
    let recent_count = recent
        .iter()
        .filter(|i| i.created_at > one_hour_ago)
        .count();
    if recent_count >= 5 {
        return Ok(
            HttpResponse::TooManyRequests().json(ApiResponse::<()>::error(
                err_dto("RATE_LIMITED", "max 5 invitations per hour per DID"),
                429,
            )),
        );
    }

    // Verify Ed25519 signature from the inviter
    // Sign over: to_commitment + comma-separated proposal IDs
    let sign_payload = format!(
        "{}:{}",
        body.to_commitment,
        body.proposal_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    if !verify_ed25519(&body.public_key, sign_payload.as_bytes(), &body.signature) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    let invitation = Invitation {
        id: uuid::Uuid::new_v4().to_string(),
        from_did: body.from_did.clone(),
        to_commitment: body.to_commitment.clone(),
        proposal_ids: body.proposal_ids.clone(),
        signature: body.signature.clone(),
        created_at: now_secs(),
        responded: false,
        accepted: false,
    };

    store.write_invitation(&invitation).map_err(|e| {
        crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        }
    })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "invitation_id": invitation.id,
            "from_did": invitation.from_did,
            "to_commitment": invitation.to_commitment,
            "proposal_ids": invitation.proposal_ids,
        }),
        trace,
    )))
}

/// GET /api/v1/governance/invitations?voter={commitment}
#[get("/governance/invitations")]
pub async fn list_invitations(
    state: web::Data<AppState>,
    query: web::Query<InvitationQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let invitations = if let Some(ref voter) = query.voter {
        store
            .list_invitations_by_commitment(voter)
            .unwrap_or_default()
    } else {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("MISSING_PARAM", "voter query parameter is required"),
            400,
        )));
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "invitations": invitations }),
        trace,
    )))
}

/// POST /api/v1/governance/invitations/respond
#[post("/governance/invitations/respond")]
pub async fn respond_invitation(
    state: web::Data<AppState>,
    body: web::Json<RespondInvitationRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let invitation = match store.read_invitation(&body.invitation_id) {
        Ok(inv) => inv,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("NOT_FOUND", "invitation not found"),
                404,
            )));
        }
    };

    if invitation.responded {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
            err_dto(
                "ALREADY_RESPONDED",
                "this invitation has already been responded to",
            ),
            409,
        )));
    }

    // Verify signature from the invitee (public_key provided in request)
    let sign_payload = format!("{}:{}", body.invitation_id, body.accepted);
    if !verify_ed25519(&body.public_key, sign_payload.as_bytes(), &body.signature) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    // Update invitation
    let updated = Invitation {
        responded: true,
        accepted: body.accepted,
        ..invitation
    };
    store
        .write_invitation(&updated)
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "invitation_id": body.invitation_id,
            "accepted": body.accepted,
        }),
        trace,
    )))
}
