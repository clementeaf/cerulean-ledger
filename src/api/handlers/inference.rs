//! Optimistic ML Oracle endpoints — verifiable inference claims.
//!
//! Phase 1 (MVP): submit claims, finalize after dispute window, query.
//!
//! Endpoints:
//! - POST /inference/submit         — submit an inference claim (staked oracle)
//! - POST /inference/finalize/{id}  — finalize a claim after dispute window
//! - GET  /inference/claims         — list claims (filter by status/oracle/model)
//! - GET  /inference/claims/{id}    — get claim details
//! - GET  /inference/models         — list known model hashes

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use crate::storage::traits::{ClaimStatus, InferenceClaim};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::collections::HashMap;

/// Default dispute window: 24 hours.
const DEFAULT_DISPUTE_WINDOW_SECS: u64 = 86_400;
/// Default minimum stake to submit inference claims.
const DEFAULT_MIN_ORACLE_STAKE: u64 = 5_000;

fn err_dto(code: &str, msg: &str) -> ErrorDto {
    ErrorDto {
        code: code.to_string(),
        message: msg.to_string(),
        field: None,
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn dispute_window() -> u64 {
    std::env::var("INFERENCE_DISPUTE_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_DISPUTE_WINDOW_SECS)
}

fn min_oracle_stake() -> u64 {
    std::env::var("INFERENCE_MIN_ORACLE_STAKE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MIN_ORACLE_STAKE)
}

#[derive(Deserialize)]
pub struct SubmitInferenceRequest {
    /// Oracle ID (must be registered and staked).
    pub oracle_id: String,
    /// SHA3-256 hash of model weights (64 hex chars).
    pub model_hash: String,
    /// Human-readable model version.
    pub model_version: String,
    /// SHA3-256 hash of input data (64 hex chars).
    pub input_hash: String,
    /// Optional URI for challengers to fetch input.
    pub input_uri: Option<String>,
    /// Inference output (JSON-serialized).
    pub output: String,
    /// SHA3-256 of output (64 hex chars).
    pub output_hash: String,
    /// Ed25519 signature over `"inference:{id}:{model_hash}:{output_hash}"`.
    pub signature: String,
    /// Ed25519 public key (hex, 64 chars = 32 bytes).
    pub public_key: String,
}

#[derive(Deserialize)]
pub struct ListClaimsQuery {
    pub status: Option<String>,
    pub oracle_id: Option<String>,
    pub model_hash: Option<String>,
}

fn parse_status(s: &str) -> Option<ClaimStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(ClaimStatus::Pending),
        "finalized" => Some(ClaimStatus::Finalized),
        "disputed" => Some(ClaimStatus::Disputed),
        "slashed" => Some(ClaimStatus::Slashed),
        "rejected" => Some(ClaimStatus::Rejected),
        _ => None,
    }
}

/// Verify an Ed25519 signature (reuses the same pattern as alias handler).
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

/// POST /api/v1/inference/submit
#[post("/inference/submit")]
pub async fn submit_inference(
    state: web::Data<AppState>,
    body: web::Json<SubmitInferenceRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Validate hex fields
    for (name, val, len) in [
        ("model_hash", &body.model_hash, 64),
        ("input_hash", &body.input_hash, 64),
        ("output_hash", &body.output_hash, 64),
    ] {
        if val.len() != len || hex::decode(val).is_err() {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                err_dto(
                    "INVALID_HASH",
                    &format!("{name} must be {len} hex characters"),
                ),
                400,
            )));
        }
    }

    // Verify oracle is registered
    let oracle_registry = state
        .oracle_registry
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if oracle_registry.get_oracle(&body.oracle_id).is_none() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("ORACLE_NOT_REGISTERED", "oracle is not registered"),
            400,
        )));
    }
    drop(oracle_registry);

    // Verify oracle is staked above minimum
    if let Some(validator) = state.staking_manager.get_validator(&body.oracle_id) {
        if validator.staked_amount < min_oracle_stake() {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                err_dto(
                    "INSUFFICIENT_STAKE",
                    &format!(
                        "oracle must stake at least {} tokens (current: {})",
                        min_oracle_stake(),
                        validator.staked_amount
                    ),
                ),
                400,
            )));
        }
    } else {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "ORACLE_NOT_STAKED",
                "oracle must be staked to submit claims",
            ),
            400,
        )));
    }

    // Generate claim ID and verify signature.
    // Oracle signs "inference:submit:{model_hash}:{output_hash}" (claim_id is server-generated).
    let claim_id = uuid::Uuid::new_v4().to_string();
    let submit_msg = format!("inference:submit:{}:{}", body.model_hash, body.output_hash);
    if !verify_ed25519(&body.public_key, submit_msg.as_bytes(), &body.signature) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    let now = now_secs();
    let claim = InferenceClaim {
        id: claim_id,
        oracle_id: body.oracle_id.clone(),
        model_hash: body.model_hash.clone(),
        model_version: body.model_version.clone(),
        input_hash: body.input_hash.clone(),
        input_uri: body.input_uri.clone(),
        output: body.output.clone(),
        output_hash: body.output_hash.clone(),
        timestamp: now,
        signature: body.signature.clone(),
        status: ClaimStatus::Pending,
        dispute_deadline: now + dispute_window(),
        finalized_at: None,
    };

    store
        .write_inference_claim(&claim)
        .map_err(|e| ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Created().json(ApiResponse::success(
        serde_json::json!({
            "id": claim.id,
            "status": "Pending",
            "dispute_deadline": claim.dispute_deadline,
        }),
        trace,
    )))
}

/// POST /api/v1/inference/finalize/{id}
#[post("/inference/finalize/{id}")]
pub async fn finalize_inference(
    state: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let mut claim = match store.read_inference_claim(&id) {
        Ok(c) => c,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("NOT_FOUND", "inference claim not found"),
                404,
            )));
        }
    };

    if claim.status != ClaimStatus::Pending {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
            err_dto(
                "NOT_PENDING",
                &format!("claim is {:?}, not Pending", claim.status),
            ),
            409,
        )));
    }

    let now = now_secs();
    if now < claim.dispute_deadline {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "DISPUTE_WINDOW_OPEN",
                &format!(
                    "dispute window ends in {} seconds",
                    claim.dispute_deadline - now
                ),
            ),
            400,
        )));
    }

    claim.status = ClaimStatus::Finalized;
    claim.finalized_at = Some(now);

    store
        .write_inference_claim(&claim)
        .map_err(|e| ApiError::StorageError {
            reason: e.to_string(),
        })?;

    // Update oracle reputation
    {
        let mut registry = state
            .oracle_registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(node) = registry.nodes.get_mut(&claim.oracle_id) {
            node.update_reputation(true);
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "id": claim.id,
            "status": "Finalized",
            "finalized_at": claim.finalized_at,
        }),
        trace,
    )))
}

/// GET /api/v1/inference/claims
#[get("/inference/claims")]
pub async fn list_claims(
    state: web::Data<AppState>,
    query: web::Query<ListClaimsQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let status = query.status.as_deref().and_then(parse_status);
    let claims = store
        .list_inference_claims(
            status.as_ref(),
            query.oracle_id.as_deref(),
            query.model_hash.as_deref(),
        )
        .map_err(|e| ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(claims, trace)))
}

/// GET /api/v1/inference/claims/{id}
#[get("/inference/claims/{id}")]
pub async fn get_claim(
    state: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    match store.read_inference_claim(&id) {
        Ok(claim) => Ok(HttpResponse::Ok().json(ApiResponse::success(claim, trace))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("NOT_FOUND", "inference claim not found"),
            404,
        ))),
    }
}

/// GET /api/v1/inference/models
#[get("/inference/models")]
pub async fn list_models(state: web::Data<AppState>, req: HttpRequest) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let claims =
        store
            .list_inference_claims(None, None, None)
            .map_err(|e| ApiError::StorageError {
                reason: e.to_string(),
            })?;

    // Aggregate by model_hash
    let mut models: HashMap<String, ModelSummary> = HashMap::new();
    for claim in &claims {
        let entry = models
            .entry(claim.model_hash.clone())
            .or_insert_with(|| ModelSummary {
                model_hash: claim.model_hash.clone(),
                model_version: claim.model_version.clone(),
                total_claims: 0,
                finalized: 0,
                pending: 0,
            });
        entry.total_claims += 1;
        match claim.status {
            ClaimStatus::Finalized => entry.finalized += 1,
            ClaimStatus::Pending => entry.pending += 1,
            _ => {}
        }
    }

    let summaries: Vec<ModelSummary> = models.into_values().collect();
    Ok(HttpResponse::Ok().json(ApiResponse::success(summaries, trace)))
}

#[derive(serde::Serialize)]
struct ModelSummary {
    model_hash: String,
    model_version: String,
    total_claims: u64,
    finalized: u64,
    pending: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_secs_returns_reasonable_value() {
        assert!(now_secs() > 1_700_000_000);
    }

    #[test]
    fn parse_status_variants() {
        assert_eq!(parse_status("pending"), Some(ClaimStatus::Pending));
        assert_eq!(parse_status("Finalized"), Some(ClaimStatus::Finalized));
        assert_eq!(parse_status("DISPUTED"), Some(ClaimStatus::Disputed));
        assert_eq!(parse_status("unknown"), None);
    }
}
