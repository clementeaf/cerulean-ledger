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
        traits::{ClaimStatus, InferenceChallenge, InferenceClaim, OutputTolerance},
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
        tolerance: OutputTolerance::Exact,
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

// ── Phase 2: Challenge Tests ─────────────────────────────────────────────────

fn make_state_with_staking(store: Arc<MemoryStore>) -> AppState {
    let mut state = make_state(store);
    // Register oracle and challenger as staked validators
    state
        .staking_manager
        .stake("oracle-a", 10_000, true)
        .unwrap();
    state
        .staking_manager
        .stake("challenger-1", 5_000, true)
        .unwrap();
    // Register oracle in oracle registry
    {
        let mut registry = state.oracle_registry.lock().unwrap();
        registry.register_oracle("oracle-a".to_string()).unwrap();
    }
    state
}

/// Generate a real Ed25519 signed challenge body.
fn challenge_body(claim_id: &str, output_hash: &str) -> serde_json::Value {
    use pqc_crypto_module::legacy::ed25519::{Signer, SigningKey};
    use rand::rngs::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let public_key = signing_key.verifying_key();
    let msg = format!("challenge:{claim_id}:{output_hash}");
    let signature = signing_key.sign(msg.as_bytes());

    serde_json::json!({
        "claim_id": claim_id,
        "challenger_id": "challenger-1",
        "challenger_output": r#"{"result": 99}"#,
        "challenger_output_hash": output_hash,
        "signature": hex::encode(signature.to_bytes()),
        "public_key": hex::encode(public_key.to_bytes()),
    })
}

// ── Storage: challenge write/read ────────────────────────────────────────────

#[actix_web::test]
async fn storage_write_and_list_challenges() {
    let store = MemoryStore::new();
    let ch = InferenceChallenge {
        id: "ch-1".to_string(),
        claim_id: "claim-1".to_string(),
        challenger_id: "challenger-1".to_string(),
        challenger_output: "{}".to_string(),
        challenger_output_hash: "f".repeat(64),
        bond: 1000,
        timestamp: now_secs(),
        signature: "sig".to_string(),
        succeeded: Some(true),
    };
    store.write_inference_challenge(&ch).unwrap();

    let challenges = store.list_challenges_by_claim("claim-1").unwrap();
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].id, "ch-1");
    assert_eq!(challenges[0].succeeded, Some(true));

    // Different claim returns empty
    let empty = store.list_challenges_by_claim("claim-999").unwrap();
    assert!(empty.is_empty());
}

// ── HTTP: challenge non-pending claim ────────────────────────────────────────

#[actix_web::test]
async fn http_challenge_finalized_claim_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let mut claim = sample_claim("fc-1", "oracle-a", "m1", ClaimStatus::Finalized);
    claim.finalized_at = Some(now_secs());
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = challenge_body("fc-1", &"f".repeat(64));
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

// ── HTTP: challenge after dispute window ─────────────────────────────────────

#[actix_web::test]
async fn http_challenge_after_deadline_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let mut claim = sample_claim("exp-1", "oracle-a", "m1", ClaimStatus::Pending);
    claim.dispute_deadline = now_secs() - 1; // Already expired
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = challenge_body("exp-1", &"f".repeat(64));
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── HTTP: self-challenge blocked ─────────────────────────────────────────────

#[actix_web::test]
async fn http_self_challenge_blocked() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = sample_claim("sc-1", "oracle-a", "m1", ClaimStatus::Pending);
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger = oracle-a (same as claim oracle)
    let body = serde_json::json!({
        "claim_id": "sc-1",
        "challenger_id": "oracle-a",
        "challenger_output": "{}",
        "challenger_output_hash": "f".repeat(64),
        "signature": "d".repeat(128),
        "public_key": "e".repeat(64),
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── HTTP: challenger not staked ──────────────────────────────────────────────

#[actix_web::test]
async fn http_challenge_unstaked_challenger_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = sample_claim("us-1", "oracle-a", "m1", ClaimStatus::Pending);
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = serde_json::json!({
        "claim_id": "us-1",
        "challenger_id": "nobody",
        "challenger_output": "{}",
        "challenger_output_hash": "f".repeat(64),
        "signature": "d".repeat(128),
        "public_key": "e".repeat(64),
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── HTTP: successful challenge (outputs differ → slash) ──────────────────────

#[actix_web::test]
async fn http_challenge_succeeds_slash_oracle() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = sample_claim("sl-1", "oracle-a", "m1", ClaimStatus::Pending);
    // claim.output_hash = "c".repeat(64)
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger submits DIFFERENT output hash → challenge succeeds
    let body = challenge_body("sl-1", &"f".repeat(64));
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], true);
    assert_eq!(data["claim_status"], "Slashed");

    // Verify claim status in storage
    let loaded = store.read_inference_claim("sl-1").unwrap();
    assert_eq!(loaded.status, ClaimStatus::Slashed);

    // Verify challenge was stored
    let challenges = store.list_challenges_by_claim("sl-1").unwrap();
    assert_eq!(challenges.len(), 1);
    assert_eq!(challenges[0].succeeded, Some(true));
}

// ── HTTP: failed challenge (outputs match → reject) ──────────────────────────

#[actix_web::test]
async fn http_challenge_fails_same_output() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = sample_claim("rj-1", "oracle-a", "m1", ClaimStatus::Pending);
    // claim.output_hash = "c".repeat(64)
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger submits SAME output hash → challenge fails
    let body = challenge_body("rj-1", &"c".repeat(64));
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], false);
    assert_eq!(data["claim_status"], "Rejected");

    // Verify claim status
    let loaded = store.read_inference_claim("rj-1").unwrap();
    assert_eq!(loaded.status, ClaimStatus::Rejected);
}

// ── HTTP: challenge nonexistent claim ────────────────────────────────────────

#[actix_web::test]
async fn http_challenge_nonexistent_claim() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let body = challenge_body("nope", &"f".repeat(64));
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

// ── InferenceChallenge serde ─────────────────────────────────────────────────

#[actix_web::test]
async fn inference_challenge_serde_roundtrip() {
    let ch = InferenceChallenge {
        id: "ch-rt".to_string(),
        claim_id: "claim-rt".to_string(),
        challenger_id: "ch-id".to_string(),
        challenger_output: "{}".to_string(),
        challenger_output_hash: "f".repeat(64),
        bond: 1000,
        timestamp: now_secs(),
        signature: "sig".to_string(),
        succeeded: Some(false),
    };
    let json = serde_json::to_string(&ch).unwrap();
    let decoded: InferenceChallenge = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "ch-rt");
    assert_eq!(decoded.succeeded, Some(false));
}

// ── Phase 3: Tolerance Tests ─────────────────────────────────────────────────

// Helper: create a claim with specific tolerance and output
fn claim_with_tolerance(
    id: &str,
    output: &str,
    output_hash: &str,
    tolerance: OutputTolerance,
) -> InferenceClaim {
    InferenceClaim {
        id: id.to_string(),
        oracle_id: "oracle-a".to_string(),
        model_hash: "a".repeat(64),
        model_version: "v1.0".to_string(),
        input_hash: "b".repeat(64),
        input_uri: None,
        output: output.to_string(),
        output_hash: output_hash.to_string(),
        timestamp: now_secs(),
        signature: "d".repeat(128),
        status: ClaimStatus::Pending,
        tolerance,
        dispute_deadline: now_secs() + 86400,
        finalized_at: None,
    }
}

fn challenge_body_with_output(
    claim_id: &str,
    output_hash: &str,
    challenger_output: &str,
) -> serde_json::Value {
    use pqc_crypto_module::legacy::ed25519::{Signer, SigningKey};
    use rand::rngs::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let public_key = signing_key.verifying_key();
    let msg = format!("challenge:{claim_id}:{output_hash}");
    let signature = signing_key.sign(msg.as_bytes());

    serde_json::json!({
        "claim_id": claim_id,
        "challenger_id": "challenger-1",
        "challenger_output": challenger_output,
        "challenger_output_hash": output_hash,
        "signature": hex::encode(signature.to_bytes()),
        "public_key": hex::encode(public_key.to_bytes()),
    })
}

// ── Numeric tolerance: within threshold → no fraud ───────────────────────────

#[actix_web::test]
async fn http_numeric_tolerance_within_threshold_no_fraud() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Oracle output: 42.0, challenger output: 42.5, threshold: 1.0 → match
    let claim = claim_with_tolerance(
        "nt-1",
        r#"{"result": 42.0}"#,
        &"c".repeat(64),
        OutputTolerance::Numeric { threshold: 1.0 },
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger output: 42.5 (within 1.0 threshold) but different hash
    let body = challenge_body_with_output("nt-1", &"f".repeat(64), r#"{"result": 42.5}"#);
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    // Outputs are within tolerance → challenge fails (oracle was correct)
    assert_eq!(data["succeeded"], false);
    assert_eq!(data["claim_status"], "Rejected");
}

// ── Numeric tolerance: outside threshold → fraud ─────────────────────────────

#[actix_web::test]
async fn http_numeric_tolerance_outside_threshold_fraud() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Oracle output: 42.0, threshold: 1.0
    let claim = claim_with_tolerance(
        "nt-2",
        r#"{"result": 42.0}"#,
        &"c".repeat(64),
        OutputTolerance::Numeric { threshold: 1.0 },
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger output: 50.0 (outside 1.0 threshold)
    let body = challenge_body_with_output("nt-2", &"f".repeat(64), r#"{"result": 50.0}"#);
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], true);
    assert_eq!(data["claim_status"], "Slashed");
}

// ── Cosine tolerance: similar vectors → no fraud ─────────────────────────────

#[actix_web::test]
async fn http_cosine_tolerance_similar_vectors_no_fraud() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Oracle embedding: [1.0, 0.0, 0.0]
    let claim = claim_with_tolerance(
        "ct-1",
        "[1.0, 0.0, 0.0]",
        &"c".repeat(64),
        OutputTolerance::Cosine {
            min_similarity: 0.95,
        },
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger embedding: [0.99, 0.1, 0.0] → cosine ~ 0.995 (above 0.95)
    let body = challenge_body_with_output("ct-1", &"f".repeat(64), "[0.99, 0.1, 0.0]");
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], false);
    assert_eq!(data["claim_status"], "Rejected");
}

// ── Cosine tolerance: orthogonal vectors → fraud ─────────────────────────────

#[actix_web::test]
async fn http_cosine_tolerance_different_vectors_fraud() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    // Oracle embedding: [1.0, 0.0, 0.0]
    let claim = claim_with_tolerance(
        "ct-2",
        "[1.0, 0.0, 0.0]",
        &"c".repeat(64),
        OutputTolerance::Cosine {
            min_similarity: 0.95,
        },
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger embedding: [0.0, 1.0, 0.0] → cosine = 0.0 (below 0.95)
    let body = challenge_body_with_output("ct-2", &"f".repeat(64), "[0.0, 1.0, 0.0]");
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], true);
    assert_eq!(data["claim_status"], "Slashed");
}

// ── Exact mode backward compat ───────────────────────────────────────────────

#[actix_web::test]
async fn http_exact_tolerance_same_hash_no_fraud() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = claim_with_tolerance(
        "ex-1",
        "any output",
        &"c".repeat(64),
        OutputTolerance::Exact,
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Same hash → no fraud
    let body = challenge_body_with_output("ex-1", &"c".repeat(64), "same output");
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["succeeded"], false);
    assert_eq!(data["claim_status"], "Rejected");
}

// ── OutputTolerance serde ────────────────────────────────────────────────────

#[actix_web::test]
async fn output_tolerance_serde_roundtrip() {
    for tolerance in [
        OutputTolerance::Exact,
        OutputTolerance::Numeric { threshold: 0.5 },
        OutputTolerance::Cosine {
            min_similarity: 0.95,
        },
    ] {
        let json = serde_json::to_string(&tolerance).unwrap();
        let decoded: OutputTolerance = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, tolerance);
    }
}

#[actix_web::test]
async fn output_tolerance_default_is_exact() {
    assert_eq!(OutputTolerance::default(), OutputTolerance::Exact);
}

// ── Cosine with JSON embedding field ─────────────────────────────────────────

#[actix_web::test]
async fn http_cosine_json_embedding_field() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let claim = claim_with_tolerance(
        "ce-1",
        r#"{"embedding": [1.0, 0.0, 0.0]}"#,
        &"c".repeat(64),
        OutputTolerance::Cosine {
            min_similarity: 0.9,
        },
    );
    store.write_inference_claim(&claim).unwrap();

    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    // Challenger uses "embedding" field too, similar vector
    let body = challenge_body_with_output(
        "ce-1",
        &"f".repeat(64),
        r#"{"embedding": [0.95, 0.1, 0.05]}"#,
    );
    let req = test::TestRequest::post()
        .uri("/api/v1/inference/challenge")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    // cosine(1,0,0 · 0.95,0.1,0.05) ≈ 0.994 > 0.9 → no fraud
    assert_eq!(data["succeeded"], false);
}

// ── Phase 4: zkML Bridge Tests ───────────────────────────────────────────────

/// Generate a valid SHA256 commitment proof for the given hashes.
fn make_sha256_proof(model_hash: &str, input_hash: &str, output_hash: &str) -> serde_json::Value {
    use pqc_crypto_module::legacy::sha256::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(model_hash.as_bytes());
    hasher.update(input_hash.as_bytes());
    hasher.update(output_hash.as_bytes());
    let commitment = hex::encode(hasher.finalize());
    serde_json::json!({
        "proof_type": "Sha256Commitment",
        "proof_data": commitment,
    })
}

fn proven_submit_body(
    model_hash: &str,
    output_hash: &str,
    proof: serde_json::Value,
) -> serde_json::Value {
    use pqc_crypto_module::legacy::ed25519::{Signer, SigningKey};
    use rand::rngs::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let public_key = signing_key.verifying_key();
    let msg = format!("inference:submit:{model_hash}:{output_hash}");
    let signature = signing_key.sign(msg.as_bytes());

    serde_json::json!({
        "oracle_id": "oracle-a",
        "model_hash": model_hash,
        "model_version": "v1.0",
        "input_hash": "b".repeat(64),
        "output": r#"{"result": 42}"#,
        "output_hash": output_hash,
        "signature": hex::encode(signature.to_bytes()),
        "public_key": hex::encode(public_key.to_bytes()),
        "proof": proof,
    })
}

// ── Valid proof → instant Finalized ──────────────────────────────────────────

#[actix_web::test]
async fn http_submit_proven_valid_sha256_instant_finalized() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state_with_staking(store.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let model_hash = "a".repeat(64);
    let output_hash = "c".repeat(64);
    let input_hash = "b".repeat(64);
    let proof = make_sha256_proof(&model_hash, &input_hash, &output_hash);
    let body = proven_submit_body(&model_hash, &output_hash, proof);

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit-proven")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let result: ApiResponse<serde_json::Value> = test::read_body_json(resp).await;
    let data = result.data.unwrap();
    assert_eq!(data["status"], "Finalized");
    assert_eq!(data["proof_type"], "Sha256Commitment");
    assert!(data["finalized_at"].is_number());

    // Verify it's in storage as Finalized
    let claim_id = data["id"].as_str().unwrap();
    let loaded = store.read_inference_claim(claim_id).unwrap();
    assert_eq!(loaded.status, ClaimStatus::Finalized);
    assert!(loaded.finalized_at.is_some());
}

// ── Invalid proof → rejected ─────────────────────────────────────────────────

#[actix_web::test]
async fn http_submit_proven_invalid_sha256_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let model_hash = "a".repeat(64);
    let output_hash = "c".repeat(64);
    // Wrong commitment (doesn't match the actual hashes)
    let bad_proof = serde_json::json!({
        "proof_type": "Sha256Commitment",
        "proof_data": "ff".repeat(32),
    });
    let body = proven_submit_body(&model_hash, &output_hash, bad_proof);

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit-proven")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── Unsupported proof type → error ───────────────────────────────────────────

#[actix_web::test]
async fn http_submit_proven_unsupported_type_error() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let model_hash = "a".repeat(64);
    let output_hash = "c".repeat(64);
    let unsupported_proof = serde_json::json!({
        "proof_type": "Groth16Bn254",
        "proof_data": "ff".repeat(32),
        "verification_key": "vk_hex",
    });
    let body = proven_submit_body(&model_hash, &output_hash, unsupported_proof);

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit-proven")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── Bad proof hex → error ────────────────────────────────────────────────────

#[actix_web::test]
async fn http_submit_proven_bad_proof_hex_error() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state_with_staking(store);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let model_hash = "a".repeat(64);
    let output_hash = "c".repeat(64);
    let bad_hex_proof = serde_json::json!({
        "proof_type": "Sha256Commitment",
        "proof_data": "not_valid_hex",
    });
    let body = proven_submit_body(&model_hash, &output_hash, bad_hex_proof);

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit-proven")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ── Unregistered oracle → rejected (same validation as submit) ───────────────

#[actix_web::test]
async fn http_submit_proven_unregistered_oracle_rejected() {
    setup_env();
    let store = Arc::new(MemoryStore::new());
    let state = make_state(store); // no staking setup
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(ApiRoutes::configure),
    )
    .await;

    let model_hash = "a".repeat(64);
    let output_hash = "c".repeat(64);
    let input_hash = "b".repeat(64);
    let proof = make_sha256_proof(&model_hash, &input_hash, &output_hash);
    let body = proven_submit_body(&model_hash, &output_hash, proof);

    let req = test::TestRequest::post()
        .uri("/api/v1/inference/submit-proven")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
