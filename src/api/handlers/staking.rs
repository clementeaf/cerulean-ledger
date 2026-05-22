//! Staking endpoints — validator registration, unstaking, and queries.
//!
//! Endpoints:
//! - POST /staking/stake              — stake tokens to become a validator
//! - POST /staking/unstake            — request unstaking (lock period)
//! - POST /staking/complete-unstake   — complete unstaking after lock period
//! - GET  /staking/validators         — list active validators
//! - GET  /staking/validator/{address} — get validator info
//! - GET  /staking/my-stake/{address}  — get own stake info

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult};
use crate::app_state::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StakeRequest {
    pub address: String,
    pub amount: u64,
}

#[derive(Deserialize)]
pub struct UnstakeRequest {
    pub address: String,
    #[serde(default)]
    pub amount: Option<u64>,
}

#[derive(Deserialize)]
pub struct CompleteUnstakeRequest {
    pub address: String,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// POST /api/v1/staking/stake
#[post("/staking/stake")]
pub async fn stake(
    state: web::Data<AppState>,
    body: web::Json<StakeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    // Check balance from default store
    let balance = state
        .store
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get("default")
        .map(|s| s.calculate_balance(&body.address).unwrap_or(0))
        .unwrap_or(0);

    if balance < body.amount {
        return Err(ApiError::ValidationError {
            field: "amount".to_string(),
            reason: format!(
                "Insufficient balance. Available: {}, required: {}",
                balance, body.amount
            ),
        });
    }

    // Wallet exists if registered in WalletManager OR has balance in BlockStore
    let wallet_exists = {
        let wm = state
            .wallet_manager
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        wm.get_wallet(&body.address).is_some()
    } || balance > 0;

    state
        .staking_manager
        .stake(&body.address, body.amount, wallet_exists)
        .map_err(|e| ApiError::ValidationError {
            field: "staking".to_string(),
            reason: e,
        })?;

    // Record staking transaction in the pool
    let tx = crate::storage::traits::Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        block_height: 0,
        timestamp: now_secs(),
        input_did: body.address.clone(),
        output_recipient: "STAKING".to_string(),
        amount: body.amount,
        state: "pending".to_string(),
    };
    {
        let mut pool = state.tx_pool.lock().unwrap_or_else(|e| e.into_inner());
        let _ = pool.add(tx);
    }

    crate::audit::emit_if_present(
        &state.audit_store,
        crate::audit::AuditAction::TokenStaked,
        &body.address,
        Some(format!("amount={}", body.amount)),
    );

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "address": body.address,
            "staked": body.amount,
        }),
        trace,
    )))
}

/// POST /api/v1/staking/unstake
#[post("/staking/unstake")]
pub async fn request_unstake(
    state: web::Data<AppState>,
    body: web::Json<UnstakeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    let amount = state
        .staking_manager
        .request_unstake(&body.address, body.amount)
        .map_err(|e| ApiError::ValidationError {
            field: "staking".to_string(),
            reason: e,
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "address": body.address,
            "unstake_amount": amount,
            "status": "pending",
        }),
        trace,
    )))
}

/// POST /api/v1/staking/complete-unstake
#[post("/staking/complete-unstake")]
pub async fn complete_unstake(
    state: web::Data<AppState>,
    body: web::Json<CompleteUnstakeRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    let amount = state
        .staking_manager
        .complete_unstake(&body.address)
        .map_err(|e| ApiError::ValidationError {
            field: "staking".to_string(),
            reason: e,
        })?;

    // Record unstaking transaction in the pool
    let tx = crate::storage::traits::Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        block_height: 0,
        timestamp: now_secs(),
        input_did: "STAKING".to_string(),
        output_recipient: body.address.clone(),
        amount,
        state: "pending".to_string(),
    };
    {
        let mut pool = state.tx_pool.lock().unwrap_or_else(|e| e.into_inner());
        let _ = pool.add(tx);
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "address": body.address,
            "released": amount,
            "status": "completed",
        }),
        trace,
    )))
}

/// GET /api/v1/staking/validators
#[get("/staking/validators")]
pub async fn get_validators(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let validators = state.staking_manager.get_active_validators();
    Ok(HttpResponse::Ok().json(ApiResponse::success(validators, trace)))
}

/// GET /api/v1/staking/validator/{address}
#[get("/staking/validator/{address}")]
pub async fn get_validator(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    match state.staking_manager.get_validator(&address) {
        Some(validator) => Ok(HttpResponse::Ok().json(ApiResponse::success(validator, trace))),
        None => Err(ApiError::NotFound {
            resource: format!("validator:{address}"),
        }),
    }
}

/// GET /api/v1/staking/my-stake/{address}
#[get("/staking/my-stake/{address}")]
pub async fn get_my_stake(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    match state.staking_manager.get_validator(&address) {
        Some(validator) => Ok(HttpResponse::Ok().json(ApiResponse::success(validator, trace))),
        None => Err(ApiError::NotFound {
            resource: format!("stake:{address}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_secs_returns_reasonable_value() {
        assert!(now_secs() > 1_700_000_000);
    }
}
