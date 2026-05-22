//! Vault endpoints — encrypted wallet backup storage.
//!
//! The node stores encrypted wallet blobs keyed by DID. The wallet content
//! is opaque — the node never parses or decrypts it.
//!
//! Recovery: when `VAULT_RECOVERY_SECRET` is set, clients can supply a
//! `recovery_key` (client-derived via KDF) during store. The node computes
//! a blind index via HMAC-SHA3-256 (NIST SP 800-185) and stores the mapping
//! `blind_index → DID`. Recovery looks up the DID and returns the encrypted wallet.

use crate::api::errors::{ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::{channel_id_from_req, get_channel_store};
use crate::app_state::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

fn err_dto(msg: &str) -> ErrorDto {
    ErrorDto {
        code: "VAULT_ERROR".to_string(),
        message: msg.to_string(),
        field: None,
    }
}

/// Compute the blind index for a recovery key using HMAC-SHA3-256.
fn blind_index(secret: &[u8], recovery_key: &[u8]) -> Result<String, String> {
    let hash = pqc_crypto_module::api::hmac_sha3_256(secret, recovery_key)
        .map_err(|e| format!("crypto error: {e}"))?;
    Ok(hash.to_hex())
}

#[derive(Deserialize)]
pub struct VaultStoreRequest {
    pub did: String,
    pub encrypted_wallet: serde_json::Value,
    /// Client-derived recovery key (hex, 64 chars = 32 bytes). Optional.
    pub recovery_key: Option<String>,
}

#[derive(Deserialize)]
pub struct VaultRecoverRequest {
    /// Client-derived recovery key (hex, 64 chars = 32 bytes).
    pub recovery_key: String,
}

/// POST /api/v1/vault/store — store an encrypted wallet backup
#[post("/vault/store")]
pub async fn vault_store(
    state: web::Data<AppState>,
    body: web::Json<VaultStoreRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    if body.did.is_empty() {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error(err_dto("did is required"), 400)));
    }

    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    store
        .write_vault(&body.did, &body.encrypted_wallet)
        .map_err(|e| crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        })?;

    // If recovery_key is provided and VAULT_RECOVERY_SECRET is configured,
    // store the blind index → DID mapping.
    if let Some(ref recovery_key_hex) = body.recovery_key {
        if let Some(ref secret) = state.vault_recovery_secret {
            // Validate: must be exactly 64 hex chars (32 bytes)
            if recovery_key_hex.len() != 64 || hex::decode(recovery_key_hex).is_err() {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    err_dto("recovery_key must be exactly 64 hex characters (32 bytes)"),
                    400,
                )));
            }

            let recovery_key_bytes = hex::decode(recovery_key_hex).unwrap_or_default();
            let idx = blind_index(secret, &recovery_key_bytes).map_err(|e| {
                crate::api::errors::ApiError::StorageError {
                    reason: e.to_string(),
                }
            })?;

            store.write_vault_recovery(&idx, &body.did).map_err(|e| {
                crate::api::errors::ApiError::StorageError {
                    reason: e.to_string(),
                }
            })?;
        }
        // If VAULT_RECOVERY_SECRET is not set, silently ignore recovery_key
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "did": body.did }),
        trace,
    )))
}

/// GET /api/v1/vault/{did} — retrieve an encrypted wallet backup
#[get("/vault/{did}")]
pub async fn vault_get(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let did_raw = path.into_inner();
    let did = urlencoding::decode(&did_raw)
        .unwrap_or_default()
        .to_string();

    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    match store.read_vault(&did) {
        Ok(encrypted_wallet) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            serde_json::json!({
                "did": did,
                "encrypted_wallet": encrypted_wallet,
            }),
            trace,
        ))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto(&format!("vault entry not found: {did}")),
            404,
        ))),
    }
}

/// POST /api/v1/vault/recover — recover wallet by client-derived recovery key
#[post("/vault/recover")]
pub async fn vault_recover(
    state: web::Data<AppState>,
    body: web::Json<VaultRecoverRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    // Require VAULT_RECOVERY_SECRET
    let secret = match state.vault_recovery_secret {
        Some(ref s) => s,
        None => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("vault recovery is not enabled on this node"),
                404,
            )));
        }
    };

    // Validate recovery_key format: exactly 64 hex chars (32 bytes)
    if body.recovery_key.len() != 64 || hex::decode(&body.recovery_key).is_err() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            err_dto("recovery_key must be exactly 64 hex characters (32 bytes)"),
            400,
        )));
    }

    let recovery_key_bytes = hex::decode(&body.recovery_key).unwrap_or_default();
    let idx = blind_index(secret, &recovery_key_bytes).map_err(|e| {
        crate::api::errors::ApiError::StorageError {
            reason: e.to_string(),
        }
    })?;

    let channel = channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    // Look up DID from blind index
    let did = match store.read_vault_by_recovery(&idx) {
        Ok(did) => did,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto("no vault entry found for this recovery key"),
                404,
            )));
        }
    };

    // Fetch the encrypted wallet by DID
    match store.read_vault(&did) {
        Ok(encrypted_wallet) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            serde_json::json!({
                "did": did,
                "encrypted_wallet": encrypted_wallet,
            }),
            trace,
        ))),
        Err(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            err_dto("vault entry not found for recovered DID"),
            404,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blind_index_is_deterministic() {
        let a = blind_index(b"secret", b"recovery").unwrap();
        let b = blind_index(b"secret", b"recovery").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // 32 bytes hex
    }

    #[test]
    fn blind_index_different_secrets_differ() {
        let a = blind_index(b"secret1", b"recovery").unwrap();
        let b = blind_index(b"secret2", b"recovery").unwrap();
        assert_ne!(a, b);
    }
}
