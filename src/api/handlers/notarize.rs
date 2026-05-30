//! Notarization endpoints — Proof of Existence service.
//!
//! Clients compute a SHA-256 hash of their document locally (the document
//! never leaves the client) and submit the hash for on-chain timestamping.
//!
//! Endpoints:
//! - POST   /api/v1/notarize          — register a document hash
//! - GET    /api/v1/notarize/verify/{hash} — verify a document hash
//! - GET    /api/v1/notarize/{id}     — get notarization by ID
//! - GET    /api/v1/notarize          — list notarizations

use crate::api::errors::{ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use crate::storage::traits::NotarizationEntry;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct NotarizeRequest {
    /// SHA-256 hash of the document (64 hex chars = 32 bytes).
    pub content_hash: String,
    /// DID or address of the signer.
    pub signer: String,
    /// Ed25519 public key (hex, 64 chars = 32 bytes).
    pub public_key: String,
    /// Ed25519 signature over `"notarize:{content_hash}"`, hex-encoded.
    pub signature: String,
    /// Optional metadata (document name, description, etc.).
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct NotarizeListQuery {
    /// Filter by signer DID/address.
    pub signer: Option<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Register a document hash for on-chain timestamping.
#[post("/notarize")]
pub async fn submit_notarization(
    state: web::Data<AppState>,
    body: web::Json<NotarizeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Validate content_hash: must be 64 hex chars (SHA-256)
    if body.content_hash.len() != 64 || hex::decode(&body.content_hash).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "INVALID_HASH",
                "content_hash must be 64 hex characters (SHA-256)",
            ),
            400,
        )));
    }

    // Validate public_key
    if body.public_key.len() != 64 || hex::decode(&body.public_key).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto(
                "INVALID_PUBLIC_KEY",
                "public_key must be 64 hex characters (32 bytes Ed25519)",
            ),
            400,
        )));
    }

    // Verify signature over "notarize:{content_hash}"
    let sign_msg = format!("notarize:{}", body.content_hash);
    if !verify_ed25519(&body.public_key, sign_msg.as_bytes(), &body.signature) {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            err_dto("INVALID_SIGNATURE", "Ed25519 signature verification failed"),
            401,
        )));
    }

    // Check for duplicate: same content_hash already notarized
    if let Ok(existing) = store.read_notarization_by_hash(&body.content_hash) {
        return Ok(HttpResponse::Conflict().json(ApiResponse::success(
            serde_json::json!({
                "message": "document already notarized",
                "id": existing.id,
                "notarized_at": existing.notarized_at,
                "signer": existing.signer,
            }),
            trace,
        )));
    }

    // Get current block height for anchoring
    let block_height = store.get_latest_height().unwrap_or(0);

    let entry = NotarizationEntry {
        id: uuid::Uuid::new_v4().to_string(),
        content_hash: body.content_hash.clone(),
        signer: body.signer.clone(),
        metadata: body.metadata.clone(),
        notarized_at: now_secs(),
        block_height,
        signature: body.signature.clone(),
        signature_algorithm: Default::default(),
    };

    store
        .write_notarization(&entry)
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Created().json(ApiResponse::success(
        serde_json::json!({
            "id": entry.id,
            "content_hash": entry.content_hash,
            "signer": entry.signer,
            "notarized_at": entry.notarized_at,
            "block_height": entry.block_height,
        }),
        trace,
    )))
}

/// Verify a document hash — returns the notarization record if it exists.
#[get("/notarize/verify/{hash}")]
pub async fn verify_notarization(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let content_hash = path.into_inner();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    if content_hash.len() != 64 || hex::decode(&content_hash).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("INVALID_HASH", "hash must be 64 hex characters (SHA-256)"),
            400,
        )));
    }

    match store.read_notarization_by_hash(&content_hash) {
        Ok(entry) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            serde_json::json!({
                "verified": true,
                "id": entry.id,
                "content_hash": entry.content_hash,
                "signer": entry.signer,
                "notarized_at": entry.notarized_at,
                "block_height": entry.block_height,
                "metadata": entry.metadata,
                "signature": entry.signature,
                "signature_algorithm": entry.signature_algorithm,
            }),
            trace,
        ))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("NOT_FOUND", "no notarization found for this document hash"),
            404,
        ))),
    }
}

/// Get a notarization by ID.
#[get("/notarize/{id}")]
pub async fn get_notarization(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let id = path.into_inner();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    match store.read_notarization(&id) {
        Ok(entry) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            serde_json::json!({
                "id": entry.id,
                "content_hash": entry.content_hash,
                "signer": entry.signer,
                "notarized_at": entry.notarized_at,
                "block_height": entry.block_height,
                "metadata": entry.metadata,
                "signature": entry.signature,
                "signature_algorithm": entry.signature_algorithm,
            }),
            trace,
        ))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("NOT_FOUND", "notarization not found"),
            404,
        ))),
    }
}

/// List notarizations, optionally filtered by signer.
#[get("/notarize")]
pub async fn list_notarizations(
    state: web::Data<AppState>,
    query: web::Query<NotarizeListQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let entries = store
        .list_notarizations(query.signer.as_deref())
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "count": entries.len(),
            "notarizations": entries,
        }),
        trace,
    )))
}
