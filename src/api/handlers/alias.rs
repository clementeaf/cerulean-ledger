//! Alias registry endpoints — zero-knowledge alias system.
//!
//! The node stores only SHA3-256 commitments, never plaintext aliases.
//! Clients compute commitments and encrypted aliases locally.
//!
//! Endpoints:
//! - POST /alias/register — register an alias commitment
//! - POST /alias/resolve — resolve commitment to DID
//! - POST /alias/revoke  — revoke an alias (15-day cooldown)

use crate::api::errors::{ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use crate::storage::traits::AliasEntry;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// 15-day cooldown in seconds before a revoked alias can be re-registered.
const REVOKE_COOLDOWN_SECS: u64 = 15 * 24 * 60 * 60;

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
pub struct AliasRegisterRequest {
    pub did: String,
    /// Ed25519 public key (hex, 64 chars = 32 bytes).
    pub public_key: String,
    /// SHA3-256 commitment: hex(SHA3-256(salt || alias)), 64 hex chars.
    pub commitment: String,
    /// Deterministic salt: hex, 32 chars (16 bytes).
    pub salt: String,
    /// AES-256-GCM encrypted alias (hex). Opaque to the node.
    pub encrypted_alias: String,
    /// Ed25519 signature over the commitment, hex-encoded.
    pub signature: String,
}

#[derive(Deserialize)]
pub struct AliasResolveRequest {
    /// SHA3-256 commitment to look up.
    pub commitment: String,
}

#[derive(Deserialize)]
pub struct AliasRevokeRequest {
    pub did: String,
    /// Ed25519 public key (hex, 64 chars = 32 bytes).
    pub public_key: String,
    pub commitment: String,
    /// Ed25519 signature over the commitment, hex-encoded.
    pub signature: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/v1/alias/register
#[post("/alias/register")]
pub async fn alias_register(
    state: web::Data<AppState>,
    body: web::Json<AliasRegisterRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Validate commitment format: 64 hex chars (32 bytes SHA3-256)
    if body.commitment.len() != 64 || hex::decode(&body.commitment).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("INVALID_COMMITMENT", "commitment must be 64 hex characters"),
            400,
        )));
    }

    // Validate salt format: 32 hex chars (16 bytes)
    if body.salt.len() != 32 || hex::decode(&body.salt).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("INVALID_SALT", "salt must be 32 hex characters"),
            400,
        )));
    }

    // Verify Ed25519 signature over commitment
    if !verify_ed25519(
        &body.public_key,
        body.commitment.as_bytes(),
        &body.signature,
    ) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    // Check if DID already has an active alias
    if let Ok(existing) = store.read_alias_by_did(&body.did) {
        if existing.status == "active" {
            return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                err_dto(
                    "ALIAS_EXISTS",
                    "this DID already has an active alias; revoke it first",
                ),
                409,
            )));
        }
    }

    // Check if commitment already exists
    if let Ok(existing) = store.read_alias(&body.commitment) {
        if existing.status == "active" {
            return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                err_dto("ALIAS_TAKEN", "this alias commitment is already registered"),
                409,
            )));
        }
        // Revoked — check cooldown
        if existing.status == "revoked" {
            if let Some(revoked_at) = existing.revoked_at {
                if now_secs() < revoked_at + REVOKE_COOLDOWN_SECS {
                    return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                        err_dto(
                            "ALIAS_COOLDOWN",
                            "this alias is in a 15-day cooldown after revocation",
                        ),
                        409,
                    )));
                }
            }
        }
    }

    let entry = AliasEntry {
        commitment: body.commitment.clone(),
        did: body.did.clone(),
        salt: body.salt.clone(),
        encrypted_alias: body.encrypted_alias.clone(),
        registered_at: now_secs(),
        status: "active".to_string(),
        revoked_at: None,
    };

    store
        .write_alias(&entry)
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "commitment": body.commitment,
            "did": body.did,
        }),
        trace,
    )))
}

/// POST /api/v1/alias/resolve
#[post("/alias/resolve")]
pub async fn alias_resolve(
    state: web::Data<AppState>,
    body: web::Json<AliasResolveRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    match store.read_alias(&body.commitment) {
        Ok(entry) if entry.status == "active" => {
            // Extract address from DID (did:bc:<address>)
            let address = entry
                .did
                .strip_prefix("did:bc:")
                .unwrap_or(&entry.did)
                .to_string();
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                serde_json::json!({
                    "did": entry.did,
                    "address": address,
                }),
                trace,
            )))
        }
        Ok(entry) if entry.status == "revoked" => Ok(HttpResponse::Gone().json(
            ApiResponse::<()>::error(err_dto("ALIAS_REVOKED", "this alias has been revoked"), 410),
        )),
        _ => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("NOT_FOUND", "alias not found"),
            404,
        ))),
    }
}

/// GET /api/v1/alias/by-did/{did}
#[get("/alias/by-did/{did}")]
pub async fn alias_by_did(
    state: web::Data<AppState>,
    did: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    match store.read_alias_by_did(&did) {
        Ok(entry) if entry.status == "active" => Ok(HttpResponse::Ok().json(ApiResponse::success(
            serde_json::json!({
                "commitment": entry.commitment,
                "did": entry.did,
                "encrypted_alias": entry.encrypted_alias,
                "registered_at": entry.registered_at,
            }),
            trace,
        ))),
        Ok(entry) if entry.status == "revoked" => Ok(HttpResponse::Gone().json(
            ApiResponse::<()>::error(err_dto("ALIAS_REVOKED", "this alias has been revoked"), 410),
        )),
        _ => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("NOT_FOUND", "no alias found for this DID"),
            404,
        ))),
    }
}

/// POST /api/v1/alias/revoke
#[post("/alias/revoke")]
pub async fn alias_revoke(
    state: web::Data<AppState>,
    body: web::Json<AliasRevokeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Read the existing alias
    let entry = match store.read_alias(&body.commitment) {
        Ok(e) => e,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("NOT_FOUND", "alias not found"),
                404,
            )));
        }
    };

    // Only the owner can revoke
    if entry.did != body.did {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            err_dto("FORBIDDEN", "only the alias owner can revoke it"),
            403,
        )));
    }

    if entry.status == "revoked" {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
            err_dto("ALREADY_REVOKED", "this alias is already revoked"),
            409,
        )));
    }

    // Verify Ed25519 signature
    if !verify_ed25519(
        &body.public_key,
        body.commitment.as_bytes(),
        &body.signature,
    ) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    // Mark as revoked
    let revoked = AliasEntry {
        status: "revoked".to_string(),
        revoked_at: Some(now_secs()),
        ..entry
    };
    store
        .write_alias(&revoked)
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "commitment": body.commitment,
            "status": "revoked",
        }),
        trace,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_secs_returns_reasonable_value() {
        let t = now_secs();
        assert!(t > 1_700_000_000); // After 2023
    }

    #[test]
    fn verify_ed25519_rejects_bad_hex() {
        assert!(!verify_ed25519("not_hex", b"msg", "not_hex"));
    }

    #[test]
    fn verify_ed25519_rejects_wrong_length() {
        assert!(!verify_ed25519("abcd", b"msg", "abcd"));
    }
}
