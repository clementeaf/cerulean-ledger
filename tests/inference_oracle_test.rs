//! Integration tests for the Optimistic ML Oracle (Phase 1).
//!
//! Tests cover:
//! - Storage layer (MemoryStore): write, read, list with filters
//! - HTTP endpoints: submit, finalize, list, get, models
//! - Validation: bad hashes, unregistered oracle, unstaked oracle, early finalize
//! - Lifecycle: Pending → Finalized

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use actix_web::{test, web, App};
use rust_bc::{
    api::{errors::ApiResponse, routes::ApiRoutes},
    storage::{
        traits::{ClaimStatus, InferenceClaim},
        BlockStore, MemoryStore,
    },
    AppState,
};

// ── Setup ────────────────────────────────────────────────────────────────────

/// Set ACL_MODE=permissive so enforce_acl doesn't block test requests.
fn setup_env() {
    std::env::set_var("ACL_MODE", "permissive");
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn sample_claim(id: &str, oracle: &str, _model: &str, status: ClaimStatus) -> InferenceClaim {
    InferenceClaim {
        id: id.to_string(),
        oracle_id: oracle.to_string(),
        model_hash: "a".repeat(64),
        model_version: "v1.0".to_string(),
        input_hash: "b".repeat(64),
        input_uri: None,
        output: r#"{"result": 42}"#.to_string(),
        output_hash: "c".repeat(64),
        timestamp: now_secs(),
        signature: "d".repeat(128),
        status,
        dispute_deadline: now_secs() + 86400,
        finalized_at: None,
    }
}

fn make_state(store: Arc<MemoryStore>) -> AppState {
    let mut state = AppState::test_default();
    let mut m = HashMap::new();
    m.insert("default".to_string(), store as Arc<dyn BlockStore>);
    state.store = Arc::new(RwLock::new(m));
    state
}

// ── Storage Layer Tests ──────────────────────────────────────────────────────

#[actix_web::test]
async fn storage_write_and_read_claim() {
    let store = MemoryStore::new();
    let claim = sample_claim("claim-1", "oracle-a", "model-x", ClaimStatus::Pending);
    store.write_inference_claim(&claim).unwrap();

    let loaded = store.read_inference_claim("claim-1").unwrap();
    assert_eq!(loaded.id, "claim-1");
    assert_eq!(loaded.oracle_id, "oracle-a");
    assert_eq!(loaded.status, ClaimStatus::Pending);
}

#[actix_web::test]
async fn storage_read_nonexistent_returns_error() {
    let store = MemoryStore::new();
    let result = store.read_inference_claim("nope");
    assert!(result.is_err());
}

#[actix_web::test]
async fn storage_list_all_claims() {
    let store = MemoryStore::new();
    store
        .write_inference_claim(&sample_claim("c1", "o1", "m1", ClaimStatus::Pending))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c2", "o2", "m2", ClaimStatus::Finalized))
        .unwrap();

    let all = store.list_inference_claims(None, None, None).unwrap();
    assert_eq!(all.len(), 2);
}

#[actix_web::test]
async fn storage_list_filter_by_status() {
    let store = MemoryStore::new();
    store
        .write_inference_claim(&sample_claim("c1", "o1", "m1", ClaimStatus::Pending))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c2", "o1", "m1", ClaimStatus::Finalized))
        .unwrap();

    let pending = store
        .list_inference_claims(Some(&ClaimStatus::Pending), None, None)
        .unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "c1");

    let finalized = store
        .list_inference_claims(Some(&ClaimStatus::Finalized), None, None)
        .unwrap();
    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].id, "c2");
}

#[actix_web::test]
async fn storage_list_filter_by_oracle() {
    let store = MemoryStore::new();
    store
        .write_inference_claim(&sample_claim("c1", "oracle-a", "m1", ClaimStatus::Pending))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c2", "oracle-b", "m1", ClaimStatus::Pending))
        .unwrap();

    let filtered = store
        .list_inference_claims(None, Some("oracle-a"), None)
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].oracle_id, "oracle-a");
}

#[actix_web::test]
async fn storage_list_filter_by_model() {
    let store = MemoryStore::new();
    let mut c1 = sample_claim("c1", "o1", "m1", ClaimStatus::Pending);
    c1.model_hash = "a".repeat(64);
    let mut c2 = sample_claim("c2", "o1", "m2", ClaimStatus::Pending);
    c2.model_hash = "f".repeat(64);

    store.write_inference_claim(&c1).unwrap();
    store.write_inference_claim(&c2).unwrap();

    let filtered = store
        .list_inference_claims(None, None, Some(&"a".repeat(64)))
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "c1");
}

#[actix_web::test]
async fn storage_overwrite_claim_updates_status() {
    let store = MemoryStore::new();
    let mut claim = sample_claim("c1", "o1", "m1", ClaimStatus::Pending);
    store.write_inference_claim(&claim).unwrap();

    claim.status = ClaimStatus::Finalized;
    claim.finalized_at = Some(now_secs());
    store.write_inference_claim(&claim).unwrap();

    let loaded = store.read_inference_claim("c1").unwrap();
    assert_eq!(loaded.status, ClaimStatus::Finalized);
    assert!(loaded.finalized_at.is_some());
}

#[actix_web::test]
async fn storage_combined_filters() {
    let store = MemoryStore::new();
    store
        .write_inference_claim(&sample_claim("c1", "o1", "m1", ClaimStatus::Pending))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c2", "o1", "m1", ClaimStatus::Finalized))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c3", "o2", "m1", ClaimStatus::Pending))
        .unwrap();

    // Filter: Pending + oracle o1
    let filtered = store
        .list_inference_claims(Some(&ClaimStatus::Pending), Some("o1"), None)
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "c1");
}

// ── HTTP Endpoint Tests ──────────────────────────────────────────────────────

#[actix_web::test]
async fn http_list_claims_empty() {
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/inference/claims")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ApiResponse<Vec<InferenceClaim>> = test::read_body_json(resp).await;
    assert!(body.status == "ok" || body.status == "Success");
    assert!(body.data.unwrap().is_empty());
}

#[actix_web::test]
async fn http_get_claim_not_found() {
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/inference/claims/nonexistent")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn http_submit_bad_hash_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = serde_json::json!({
        "oracle_id": "test-oracle",
        "model_hash": "too-short",
        "model_version": "v1",
        "input_hash": "b".repeat(64),
        "output": "{}",
        "output_hash": "c".repeat(64),
        "signature": "d".repeat(128),
        "public_key": "e".repeat(64),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn http_submit_unregistered_oracle_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = serde_json::json!({
        "oracle_id": "unregistered-oracle",
        "model_hash": "a".repeat(64),
        "model_version": "v1",
        "input_hash": "b".repeat(64),
        "output": "{}",
        "output_hash": "c".repeat(64),
        "signature": "d".repeat(128),
        "public_key": "e".repeat(64),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn http_finalize_nonexistent_claim() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/finalize/nonexistent")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn http_finalize_before_deadline_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Pre-populate a pending claim with future deadline
    let claim = sample_claim("test-claim", "o1", "m1", ClaimStatus::Pending);
    store.write_inference_claim(&claim).unwrap();

    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/finalize/test-claim")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn http_finalize_after_deadline_succeeds() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Claim with deadline already passed
    let mut claim = sample_claim("test-claim", "o1", "m1", ClaimStatus::Pending);
    claim.dispute_deadline = now_secs() - 1; // Already expired
    store.write_inference_claim(&claim).unwrap();

    let state = make_state(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/finalize/test-claim")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify it's finalized in storage
    let loaded = store.read_inference_claim("test-claim").unwrap();
    assert_eq!(loaded.status, ClaimStatus::Finalized);
    assert!(loaded.finalized_at.is_some());
}

#[actix_web::test]
async fn http_finalize_already_finalized_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let mut claim = sample_claim("test-claim", "o1", "m1", ClaimStatus::Finalized);
    claim.finalized_at = Some(now_secs());
    store.write_inference_claim(&claim).unwrap();

    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/finalize/test-claim")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

#[actix_web::test]
async fn http_list_claims_with_filters() {
    let store = Arc::new(MemoryStore::new());
    store
        .write_inference_claim(&sample_claim("c1", "o1", "m1", ClaimStatus::Pending))
        .unwrap();
    store
        .write_inference_claim(&sample_claim("c2", "o2", "m1", ClaimStatus::Finalized))
        .unwrap();

    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Filter by status=pending
    let req = test::TestRequest::get()
        .uri("/api/v1/inference/claims?status=pending")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: ApiResponse<Vec<InferenceClaim>> = test::read_body_json(resp).await;
    let claims = body.data.unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].id, "c1");

    // Filter by oracle_id=o2
    let req = test::TestRequest::get()
        .uri("/api/v1/inference/claims?oracle_id=o2")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: ApiResponse<Vec<InferenceClaim>> = test::read_body_json(resp).await;
    let claims = body.data.unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].oracle_id, "o2");
}

#[actix_web::test]
async fn http_models_endpoint() {
    let store = Arc::new(MemoryStore::new());
    // Two claims for same model, one for different model
    let mut c1 = sample_claim("c1", "o1", "m1", ClaimStatus::Finalized);
    c1.model_hash = "a".repeat(64);
    let mut c2 = sample_claim("c2", "o1", "m1", ClaimStatus::Pending);
    c2.model_hash = "a".repeat(64);
    let mut c3 = sample_claim("c3", "o2", "m2", ClaimStatus::Pending);
    c3.model_hash = "f".repeat(64);

    store.write_inference_claim(&c1).unwrap();
    store.write_inference_claim(&c2).unwrap();
    store.write_inference_claim(&c3).unwrap();

    let state = make_state(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/inference/models")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: ApiResponse<Vec<serde_json::Value>> = test::read_body_json(resp).await;
    let models = body.data.unwrap();
    assert_eq!(models.len(), 2);
}

// ── Claim Status Serde Tests ─────────────────────────────────────────────────

#[actix_web::test]
async fn claim_status_default_is_pending() {
    assert_eq!(ClaimStatus::default(), ClaimStatus::Pending);
}

#[actix_web::test]
async fn inference_claim_serde_roundtrip() {
    let claim = sample_claim("rt-1", "oracle-x", "model-y", ClaimStatus::Pending);
    let json = serde_json::to_string(&claim).unwrap();
    let decoded: InferenceClaim = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "rt-1");
    assert_eq!(decoded.status, ClaimStatus::Pending);
    assert!(decoded.finalized_at.is_none());
}

#[actix_web::test]
async fn claim_status_serde_all_variants() {
    for status in [
        ClaimStatus::Pending,
        ClaimStatus::Finalized,
        ClaimStatus::Disputed,
        ClaimStatus::Slashed,
        ClaimStatus::Rejected,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let decoded: ClaimStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, status);
    }
}
