//! Optimistic ML Oracle endpoints — verifiable inference claims.
//!
//! Phase 1: submit claims, finalize after dispute window, query.
//! Phase 2: challenge claims, resolve disputes (slash/reward).
//!
//! Endpoints:
//! - POST /inference/submit         — submit an inference claim (staked oracle)
//! - POST /inference/challenge      — challenge a pending claim (staked)
//! - POST /inference/finalize/{id}  — finalize a claim after dispute window
//! - GET  /inference/claims         — list claims (filter by status/oracle/model)
//! - GET  /inference/claims/{id}    — get claim details
//! - GET  /inference/models         — list known model hashes

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use crate::storage::traits::{ClaimStatus, InferenceChallenge, InferenceClaim, OutputTolerance};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::collections::HashMap;

/// Default dispute window: 24 hours.
const DEFAULT_DISPUTE_WINDOW_SECS: u64 = 86_400;
/// Default minimum stake to submit inference claims.
const DEFAULT_MIN_ORACLE_STAKE: u64 = 5_000;
/// Default bond required to challenge a claim.
const DEFAULT_CHALLENGE_BOND: u64 = 1_000;

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

fn challenge_bond() -> u64 {
    std::env::var("INFERENCE_CHALLENGE_BOND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_CHALLENGE_BOND)
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
    /// Tolerance mode: "exact" (default), {"numeric": threshold}, {"cosine": min_similarity}.
    #[serde(default)]
    pub tolerance: OutputTolerance,
}

#[derive(Deserialize)]
pub struct ChallengeInferenceRequest {
    /// Claim ID to challenge.
    pub claim_id: String,
    /// Challenger's address (must be staked).
    pub challenger_id: String,
    /// Re-executed output (JSON-serialized).
    pub challenger_output: String,
    /// SHA3-256 of the challenger's output (64 hex chars).
    pub challenger_output_hash: String,
    /// Ed25519 signature over `"challenge:{claim_id}:{challenger_output_hash}"`.
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

/// Compare two inference outputs using the claim's tolerance mode.
///
/// Returns `true` if the outputs are considered equivalent (no fraud).
fn resolve_outputs(
    tolerance: &OutputTolerance,
    oracle_output: &str,
    oracle_hash: &str,
    challenger_output: &str,
    challenger_hash: &str,
) -> bool {
    match tolerance {
        OutputTolerance::Exact => oracle_hash == challenger_hash,
        OutputTolerance::Numeric { threshold } => {
            // Parse both outputs as f64, compare within threshold
            match (
                parse_numeric_output(oracle_output),
                parse_numeric_output(challenger_output),
            ) {
                (Some(a), Some(b)) => (a - b).abs() <= *threshold,
                // If either can't be parsed, fall back to exact hash comparison
                _ => oracle_hash == challenger_hash,
            }
        }
        OutputTolerance::Cosine { min_similarity } => {
            // Parse both outputs as f64 vectors, compute cosine similarity
            match (
                parse_vector_output(oracle_output),
                parse_vector_output(challenger_output),
            ) {
                (Some(a), Some(b)) if a.len() == b.len() && !a.is_empty() => {
                    cosine_similarity(&a, &b) >= *min_similarity
                }
                // If either can't be parsed or lengths differ, fall back to exact
                _ => oracle_hash == challenger_hash,
            }
        }
    }
}

/// Parse a JSON output as a single f64 number.
fn parse_numeric_output(output: &str) -> Option<f64> {
    // Try direct number parse
    if let Ok(v) = output.trim().parse::<f64>() {
        return Some(v);
    }
    // Try JSON with "result" field
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(n) = obj.get("result").and_then(|v| v.as_f64()) {
            return Some(n);
        }
        // Try root-level number
        if let Some(n) = obj.as_f64() {
            return Some(n);
        }
    }
    None
}

/// Parse a JSON output as a vector of f64.
fn parse_vector_output(output: &str) -> Option<Vec<f64>> {
    if let Ok(arr) = serde_json::from_str::<Vec<f64>>(output) {
        return Some(arr);
    }
    // Try JSON with "embedding" or "vector" field
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(output) {
        for key in &["embedding", "vector", "result"] {
            if let Some(arr) = obj.get(key).and_then(|v| v.as_array()) {
                let nums: Option<Vec<f64>> = arr.iter().map(|v| v.as_f64()).collect();
                if let Some(v) = nums {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Cosine similarity between two vectors. Returns 0.0 for zero-magnitude vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
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
        tolerance: body.tolerance.clone(),
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

/// POST /api/v1/inference/challenge
#[post("/inference/challenge")]
pub async fn challenge_inference(
    state: web::Data<AppState>,
    body: web::Json<ChallengeInferenceRequest>,
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

    // Validate output hash format
    if body.challenger_output_hash.len() != 64 || hex::decode(&body.challenger_output_hash).is_err()
    {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "INVALID_HASH",
                "challenger_output_hash must be 64 hex characters",
            ),
            400,
        )));
    }

    // Read the claim being challenged
    let mut claim = match store.read_inference_claim(&body.claim_id) {
        Ok(c) => c,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("NOT_FOUND", "inference claim not found"),
                404,
            )));
        }
    };

    // Only pending claims can be challenged
    if claim.status != ClaimStatus::Pending {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
            err_dto(
                "NOT_PENDING",
                &format!("claim is {:?}, not Pending", claim.status),
            ),
            409,
        )));
    }

    // Must be within dispute window
    let now = now_secs();
    if now >= claim.dispute_deadline {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("DISPUTE_WINDOW_CLOSED", "dispute window has expired"),
            400,
        )));
    }

    // Cannot self-challenge
    if body.challenger_id == claim.oracle_id {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("SELF_CHALLENGE", "cannot challenge your own claim"),
            400,
        )));
    }

    // Verify challenger is staked
    let bond = challenge_bond();
    if let Some(validator) = state.staking_manager.get_validator(&body.challenger_id) {
        if validator.staked_amount < bond {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                err_dto(
                    "INSUFFICIENT_STAKE",
                    &format!(
                        "challenger must stake at least {} tokens (current: {})",
                        bond, validator.staked_amount
                    ),
                ),
                400,
            )));
        }
    } else {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "CHALLENGER_NOT_STAKED",
                "challenger must be staked to submit challenges",
            ),
            400,
        )));
    }

    // Verify Ed25519 signature
    let challenge_msg = format!(
        "challenge:{}:{}",
        body.claim_id, body.challenger_output_hash
    );
    if !verify_ed25519(&body.public_key, challenge_msg.as_bytes(), &body.signature) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    // Create the challenge
    let challenge_id = uuid::Uuid::new_v4().to_string();

    // Resolution: compare outputs using claim's tolerance mode
    let outputs_match = resolve_outputs(
        &claim.tolerance,
        &claim.output,
        &claim.output_hash,
        &body.challenger_output,
        &body.challenger_output_hash,
    );
    let challenge_succeeded = !outputs_match;

    let challenge = InferenceChallenge {
        id: challenge_id.clone(),
        claim_id: body.claim_id.clone(),
        challenger_id: body.challenger_id.clone(),
        challenger_output: body.challenger_output.clone(),
        challenger_output_hash: body.challenger_output_hash.clone(),
        bond,
        timestamp: now,
        signature: body.signature.clone(),
        succeeded: Some(challenge_succeeded),
    };

    store
        .write_inference_challenge(&challenge)
        .map_err(|e| ApiError::StorageError {
            reason: e.to_string(),
        })?;

    // Update claim status based on resolution
    if challenge_succeeded {
        // Oracle was wrong — slash oracle, reward challenger
        claim.status = ClaimStatus::Slashed;

        // Slash oracle reputation
        {
            let mut registry = state
                .oracle_registry
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(node) = registry.nodes.get_mut(&claim.oracle_id) {
                node.update_reputation(false);
                node.update_reputation(false); // Double penalty for fraud
            }
        }

        crate::audit::emit_if_present(
            &state.audit_store,
            crate::audit::AuditAction::TokenStaked, // closest available action
            &claim.oracle_id,
            Some(format!(
                "inference_slashed: claim={}, challenger={}",
                body.claim_id, body.challenger_id
            )),
        );
    } else {
        // Challenge failed — oracle was correct
        claim.status = ClaimStatus::Rejected;

        // Reward oracle reputation
        {
            let mut registry = state
                .oracle_registry
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(node) = registry.nodes.get_mut(&claim.oracle_id) {
                node.update_reputation(true);
            }
        }
    }

    store
        .write_inference_claim(&claim)
        .map_err(|e| ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "challenge_id": challenge_id,
            "claim_id": body.claim_id,
            "succeeded": challenge_succeeded,
            "claim_status": format!("{:?}", claim.status),
            "oracle_id": claim.oracle_id,
            "challenger_id": body.challenger_id,
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
