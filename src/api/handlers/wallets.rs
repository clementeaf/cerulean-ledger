//! Wallet endpoints — balance, creation, and transaction history.
//!
//! Endpoints:
//! - GET  /wallets/{address}              — get wallet balance
//! - POST /wallets/create                 — create a new wallet
//! - GET  /wallets/{address}/transactions — get wallet transactions

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult};
use crate::app_state::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse};

/// Calculate balance via BlockStore.
pub(crate) fn store_balance(state: &AppState, address: &str) -> u64 {
    if let Ok(store_map) = state.store.read() {
        if let Some(store) = store_map.get("default") {
            return store.calculate_balance(address).unwrap_or(0);
        }
    }
    0
}

/// GET /api/v1/wallets/{address}
#[get("/wallets/{address}")]
pub async fn get_wallet_balance(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let balance = store_balance(&state, &address);

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "address": address.as_str(),
            "balance": balance,
        }),
        trace,
    )))
}

/// POST /api/v1/wallets/create
#[post("/wallets/create")]
pub async fn create_wallet(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Some(key) = &api_key {
        match state.billing_manager.can_create_wallet(key) {
            Ok(can_create) => {
                if !can_create {
                    return Err(ApiError::ValidationError {
                        field: "billing".to_string(),
                        reason: "Wallet limit reached for your tier".to_string(),
                    });
                }
            }
            Err(e) => {
                return Err(ApiError::UnauthorizedWithMessage { message: e });
            }
        }
    }

    let (address, public_key_hex) = {
        let mut wm = state
            .wallet_manager
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let wallet = wm.create_wallet();
        (wallet.address.clone(), wallet.get_public_key_hex())
    };

    if let Some(key) = &api_key {
        if let Err(e) = state.billing_manager.record_wallet_creation(key) {
            return Err(ApiError::InternalError { reason: e });
        }
    }

    crate::audit::emit_if_present(
        &state.audit_store,
        crate::audit::AuditAction::WalletCreated,
        api_key.as_deref().unwrap_or("unknown"),
        Some(format!("address={address}")),
    );

    Ok(HttpResponse::Created().json(ApiResponse::success(
        serde_json::json!({
            "address": address,
            "public_key": public_key_hex,
        }),
        trace,
    )))
}

/// GET /api/v1/wallets/{address}/transactions
#[get("/wallets/{address}/transactions")]
pub async fn get_wallet_transactions(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let txs = if let Ok(store_map) = state.store.read() {
        if let Some(store) = store_map.get("default") {
            store.transactions_for_address(&address).unwrap_or_default()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(txs, trace)))
}
