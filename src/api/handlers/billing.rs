//! Billing endpoints — API key management and usage tracking.
//!
//! Endpoints:
//! - POST /billing/create-key       — create a new API key
//! - POST /billing/deactivate-key   — deactivate an API key
//! - GET  /billing/usage            — get usage stats for an API key

use crate::api::errors::{ApiError, ApiResponse, ApiResult};
use crate::app_state::AppState;
use crate::billing::BillingTier;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub tier: String,
}

#[derive(Deserialize)]
pub struct DeactivateKeyRequest {
    pub api_key: String,
}

/// POST /api/v1/billing/create-key
#[post("/billing/create-key")]
pub async fn create_api_key(
    state: web::Data<AppState>,
    body: web::Json<CreateApiKeyRequest>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let tier = BillingTier::from_tier_str(&body.tier).ok_or(ApiError::ValidationError {
        field: "tier".to_string(),
        reason: "Invalid tier. Options: free, basic, pro, enterprise".to_string(),
    })?;

    let key = state
        .billing_manager
        .create_api_key(tier)
        .map_err(|e| ApiError::InternalError { reason: e })?;

    Ok(HttpResponse::Created().json(ApiResponse::success(
        serde_json::json!({ "api_key": key }),
        trace,
    )))
}

/// POST /api/v1/billing/deactivate-key
#[post("/billing/deactivate-key")]
pub async fn deactivate_api_key(
    state: web::Data<AppState>,
    body: web::Json<DeactivateKeyRequest>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    state
        .billing_manager
        .deactivate_key(&body.api_key)
        .map_err(|e| ApiError::ValidationError {
            field: "api_key".to_string(),
            reason: e,
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "status": "deactivated" }),
        trace,
    )))
}

/// GET /api/v1/billing/usage
#[get("/billing/usage")]
pub async fn get_billing_usage(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::UnauthorizedWithMessage {
            message: "API key required in X-API-Key header".to_string(),
        })?;

    let usage = state
        .billing_manager
        .get_usage(api_key)
        .map_err(|e| ApiError::UnauthorizedWithMessage { message: e })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(usage, trace)))
}
