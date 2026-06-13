//! Airdrop endpoints — claim, tracking, statistics, eligibility, history, tiers.
//!
//! Endpoints:
//! - POST /airdrop/claim                 — claim airdrop for eligible node
//! - GET  /airdrop/tracking/{address}    — get node tracking info
//! - GET  /airdrop/statistics            — get airdrop statistics
//! - GET  /airdrop/eligible              — list eligible nodes
//! - GET  /airdrop/eligibility/{address} — get eligibility info without claiming
//! - GET  /airdrop/history               — get claim history
//! - GET  /airdrop/tiers                 — get available tiers

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult};
use crate::api::handlers::wallets::store_balance;
use crate::app_state::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClaimAirdropRequest {
    pub node_address: String,
}

/// POST /api/v1/airdrop/claim
#[post("/airdrop/claim")]
pub async fn claim_airdrop(
    state: web::Data<AppState>,
    body: web::Json<ClaimAirdropRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();
    let node_address = body.node_address.clone();

    // Rate limiting: max 10 claims per minute per IP
    let client_ip = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    if !state.airdrop_manager.check_rate_limit(&client_ip, 10) {
        return Err(ApiError::ValidationError {
            field: "rate_limit".to_string(),
            reason: "Rate limit exceeded. Maximum 10 claims per minute.".to_string(),
        });
    }

    if !state.airdrop_manager.is_eligible(&node_address) {
        return Err(ApiError::ValidationError {
            field: "node_address".to_string(),
            reason: "Node is not eligible for airdrop or has already claimed".to_string(),
        });
    }

    let tracking = state
        .airdrop_manager
        .get_node_tracking(&node_address)
        .ok_or(ApiError::NotFound {
            resource: format!("node_tracking:{node_address}"),
        })?;

    let airdrop_amount = state.airdrop_manager.calculate_airdrop_amount(&tracking);
    let airdrop_wallet = state.airdrop_manager.get_airdrop_wallet().to_string();

    let airdrop_wallet_balance = store_balance(&state, &airdrop_wallet);

    if airdrop_wallet_balance < airdrop_amount {
        return Err(ApiError::ValidationError {
            field: "balance".to_string(),
            reason: format!(
                "Insufficient airdrop wallet balance. Required: {airdrop_amount}, Available: {airdrop_wallet_balance}"
            ),
        });
    }

    // Create airdrop transaction
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let transaction_id = uuid::Uuid::new_v4().to_string();
    {
        let store_tx = crate::storage::traits::Transaction {
            id: transaction_id.clone(),
            block_height: 0,
            timestamp: now,
            input_did: airdrop_wallet,
            output_recipient: node_address.clone(),
            amount: airdrop_amount,
            state: "pending".to_string(),
        };
        let mut pool = state.tx_pool.lock().unwrap_or_else(|e| e.into_inner());
        let _ = pool.add(store_tx);
    }

    state
        .airdrop_manager
        .mark_as_claimed(&node_address, transaction_id.clone());
    state
        .airdrop_manager
        .add_pending_claim(&node_address, transaction_id.clone());

    let claim_record = crate::airdrop::ClaimRecord {
        node_address: node_address.clone(),
        claim_timestamp: now,
        airdrop_amount,
        transaction_id: transaction_id.clone(),
        block_index: None,
        tier_id: tracking.eligibility_tier,
        verified: false,
        verification_timestamp: None,
    };
    state.airdrop_manager.add_claim_to_history(claim_record);

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "node_address": node_address,
            "airdrop_amount": airdrop_amount,
            "transaction_id": transaction_id,
            "tier": tracking.eligibility_tier,
            "message": "Airdrop claimed successfully. Transaction added to mempool."
        }),
        trace,
    )))
}

/// GET /api/v1/airdrop/tracking/{address}
#[get("/airdrop/tracking/{address}")]
pub async fn get_node_tracking(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let tracking = state
        .airdrop_manager
        .get_node_tracking(&address)
        .ok_or(ApiError::NotFound {
            resource: format!("node_tracking:{address}"),
        })?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(tracking, trace)))
}

/// GET /api/v1/airdrop/statistics
#[get("/airdrop/statistics")]
pub async fn get_airdrop_statistics(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let stats = state.airdrop_manager.get_statistics();
    Ok(HttpResponse::Ok().json(ApiResponse::success(stats, trace)))
}

/// GET /api/v1/airdrop/eligible
#[get("/airdrop/eligible")]
pub async fn get_eligible_nodes(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let nodes = state.airdrop_manager.get_eligible_nodes();
    Ok(HttpResponse::Ok().json(ApiResponse::success(nodes, trace)))
}

/// GET /api/v1/airdrop/eligibility/{address}
#[get("/airdrop/eligibility/{address}")]
pub async fn get_eligibility_info(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let info = state
        .airdrop_manager
        .get_eligibility_info(&address)
        .ok_or(ApiError::NotFound {
            resource: format!("node_tracking:{address}"),
        })?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(info, trace)))
}

/// GET /api/v1/airdrop/history
#[get("/airdrop/history")]
pub async fn get_claim_history(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let limit = query.get("limit").and_then(|s| s.parse::<u64>().ok());
    let node_address = query.get("node_address").map(|s| s.as_str());
    let history = state.airdrop_manager.get_claim_history(limit, node_address);
    Ok(HttpResponse::Ok().json(ApiResponse::success(history, trace)))
}

/// GET /api/v1/airdrop/tiers
#[get("/airdrop/tiers")]
pub async fn get_airdrop_tiers(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let tiers = state.airdrop_manager.get_tiers();
    Ok(HttpResponse::Ok().json(ApiResponse::success(tiers, trace)))
}
