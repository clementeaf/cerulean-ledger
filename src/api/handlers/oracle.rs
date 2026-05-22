//! Oracle endpoints:
//!   GET  /api/v1/oracle/feeds/{symbol}   — get latest price (with staleness metadata)
//!   GET  /api/v1/oracle/feeds            — list all cached prices (with staleness metadata)
//!   GET  /api/v1/oracle/nodes            — list registered oracle nodes
//!   GET  /api/v1/oracle/status           — oracle subsystem health

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::api::errors::{enforce_acl, ApiResponse, ApiResult, ErrorDto};
use crate::app_state::AppState;

/// Price data enriched with staleness metadata.
#[derive(Debug, Serialize)]
struct PriceFeedResponse {
    symbol: String,
    price: u64,
    timestamp: u64,
    source_count: u64,
    confidence: u8,
    age_ms: u64,
    is_stale: bool,
}

/// Oracle subsystem health summary.
#[derive(Debug, Serialize)]
struct OracleStatus {
    node_count: usize,
    feed_count: usize,
    stale_feeds: usize,
    fresh_feeds: usize,
    pending_reports: usize,
    max_data_age_ms: u64,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// GET /api/v1/oracle/feeds/{symbol}
#[get("/oracle/feeds/{symbol}")]
pub async fn get_oracle_feed(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let symbol = path.into_inner();
    let registry = state.oracle_registry.lock().unwrap();

    match registry.get_price(&symbol) {
        Ok(price) => {
            let now = now_ms();
            let resp = PriceFeedResponse {
                symbol: price.symbol.clone(),
                price: price.price,
                timestamp: price.timestamp,
                source_count: price.source_count,
                confidence: price.confidence,
                age_ms: price.age_ms(now),
                is_stale: !price.is_fresh(now, registry.max_data_age_ms),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(resp, trace)))
        }
        Err(e) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            ErrorDto {
                code: "ORACLE_ERROR".into(),
                message: e,
                field: None,
            },
            404,
        ))),
    }
}

/// GET /api/v1/oracle/feeds
#[get("/oracle/feeds")]
pub async fn list_oracle_feeds(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let registry = state.oracle_registry.lock().unwrap();
    let now = now_ms();

    let feeds: Vec<PriceFeedResponse> = registry
        .price_cache
        .values()
        .map(|p| PriceFeedResponse {
            symbol: p.symbol.clone(),
            price: p.price,
            timestamp: p.timestamp,
            source_count: p.source_count,
            confidence: p.confidence,
            age_ms: p.age_ms(now),
            is_stale: !p.is_fresh(now, registry.max_data_age_ms),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(feeds, trace)))
}

/// GET /api/v1/oracle/nodes
#[get("/oracle/nodes")]
pub async fn list_oracle_nodes(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let registry = state.oracle_registry.lock().unwrap();

    let nodes: Vec<_> = registry.nodes.values().cloned().collect();
    Ok(HttpResponse::Ok().json(ApiResponse::success(nodes, trace)))
}

/// GET /api/v1/oracle/status
#[get("/oracle/status")]
pub async fn oracle_status(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let registry = state.oracle_registry.lock().unwrap();
    let now = now_ms();

    let stale_count = registry
        .price_cache
        .values()
        .filter(|p| !p.is_fresh(now, registry.max_data_age_ms))
        .count();

    let feed_count = registry.price_cache.len();

    let status = OracleStatus {
        node_count: registry.nodes.len(),
        feed_count,
        stale_feeds: stale_count,
        fresh_feeds: feed_count - stale_count,
        pending_reports: registry.pending_reports.len(),
        max_data_age_ms: registry.max_data_age_ms,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(status, trace)))
}

#[derive(Deserialize)]
pub struct RegisterOracleRequest {
    pub address: String,
}

/// POST /api/v1/oracle/register — register a new oracle node.
#[post("/oracle/register")]
pub async fn register_oracle(
    state: web::Data<AppState>,
    body: web::Json<RegisterOracleRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();
    let mut registry = state
        .oracle_registry
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    match registry.register_oracle(body.address.clone()) {
        Ok(()) => Ok(HttpResponse::Created().json(ApiResponse::success(
            serde_json::json!({ "address": body.address, "registered": true }),
            trace,
        ))),
        Err(e) => Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
            ErrorDto {
                code: "ORACLE_EXISTS".into(),
                message: e,
                field: None,
            },
            409,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_feed_response_serializes() {
        let resp = PriceFeedResponse {
            symbol: "BTC/USD".into(),
            price: 10500000,
            timestamp: 1000,
            source_count: 3,
            confidence: 95,
            age_ms: 500,
            is_stale: false,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["is_stale"], false);
        assert_eq!(json["age_ms"], 500);
        assert_eq!(json["symbol"], "BTC/USD");
    }

    #[test]
    fn oracle_status_serializes() {
        let status = OracleStatus {
            node_count: 3,
            feed_count: 2,
            stale_feeds: 1,
            fresh_feeds: 1,
            pending_reports: 0,
            max_data_age_ms: 300_000,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["node_count"], 3);
        assert_eq!(json["stale_feeds"], 1);
    }
}
