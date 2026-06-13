//! W3C Interoperability endpoints:
//! - DID Resolution (did-core)
//! - Verifiable Credentials (VC Data Model 2.0)
//! - JSON-LD export for governance results

use crate::api::errors::{ApiResponse, ApiResult, ErrorDto};
use crate::api::handlers::channels::get_channel_store;
use crate::app_state::AppState;
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Serialize;
use serde_json::json;

fn err_dto(msg: &str) -> ErrorDto {
    ErrorDto {
        code: "INTEROP_ERROR".to_string(),
        message: msg.to_string(),
        field: None,
    }
}

// ── W3C DID Resolution (did-core) ───────────────────────────────────────────

#[derive(Serialize)]
struct DidDocument {
    #[serde(rename = "@context")]
    context: Vec<String>,
    id: String,
    authentication: Vec<DidVerificationMethod>,
    #[serde(rename = "verificationMethod")]
    verification_method: Vec<DidVerificationMethod>,
    service: Vec<DidService>,
}

#[derive(Serialize, Clone)]
struct DidVerificationMethod {
    id: String,
    #[serde(rename = "type")]
    method_type: String,
    controller: String,
    #[serde(rename = "publicKeyHex", skip_serializing_if = "Option::is_none")]
    public_key_hex: Option<String>,
}

#[derive(Serialize)]
struct DidService {
    id: String,
    #[serde(rename = "type")]
    service_type: String,
    #[serde(rename = "serviceEndpoint")]
    service_endpoint: String,
}

#[derive(Serialize)]
struct DidResolutionResult {
    #[serde(rename = "@context")]
    context: String,
    #[serde(rename = "didDocument")]
    did_document: DidDocument,
    #[serde(rename = "didDocumentMetadata")]
    did_document_metadata: serde_json::Value,
}

/// GET /api/v1/did/{did} — W3C DID Resolution
#[get("/did/{did}")]
pub async fn resolve_did(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let did_raw = path.into_inner();
    let did = urlencoding::decode(&did_raw)
        .unwrap_or_default()
        .to_string();

    let channel = crate::api::handlers::channels::channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let identity = match store.read_identity(&did) {
        Ok(record) => record,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto(&format!("DID not found: {did}")),
                404,
            )))
        }
    };

    let public_key = identity
        .did
        .split(':')
        .next_back()
        .unwrap_or("")
        .to_string();

    let vm_id = format!("{}#key-1", identity.did);
    let vm = DidVerificationMethod {
        id: vm_id,
        method_type: "Ed25519VerificationKey2020".to_string(),
        controller: identity.did.clone(),
        public_key_hex: Some(public_key),
    };

    let api_port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());

    let doc = DidDocument {
        context: vec![
            "https://www.w3.org/ns/did/v1".to_string(),
            "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
        ],
        id: identity.did.clone(),
        authentication: vec![vm.clone()],
        verification_method: vec![vm],
        service: vec![DidService {
            id: format!("{}#api", identity.did),
            service_type: "CeruleanLedgerAPI".to_string(),
            service_endpoint: format!("http://{bind}:{api_port}/api/v1"),
        }],
    };

    let result = DidResolutionResult {
        context: "https://w3id.org/did-resolution/v1".to_string(),
        did_document: doc,
        did_document_metadata: json!({
            "created": identity.created_at,
            "updated": identity.updated_at,
            "deactivated": identity.status != "active",
        }),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/did+ld+json")
        .json(result))
}

// ── W3C Verifiable Credentials ──────────────────────────────────────────────

/// GET /api/v1/credentials/{id}/vc — Return credential as W3C Verifiable Credential
#[get("/{id}/vc")]
pub async fn get_credential_as_vc(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let id = path.into_inner();

    let channel = crate::api::handlers::channels::channel_id_from_req(&req);
    let store = get_channel_store(&state, channel)?;

    let cred = match store.read_credential(&id) {
        Ok(c) => c,
        Err(_) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto(&format!("credential not found: {id}")),
                404,
            )))
        }
    };

    let vc = json!({
        "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://www.w3.org/ns/credentials/examples/v2"
        ],
        "type": ["VerifiableCredential", &cred.cred_type],
        "issuer": cred.issuer_did,
        "validFrom": cred.issued_at,
        "validUntil": cred.expires_at,
        "credentialSubject": {
            "id": cred.subject_did,
            "credential_id": cred.id,
            "claims": cred.claims,
        },
        "proof": {
            "type": "Ed25519Signature2020",
            "created": cred.issued_at,
            "verificationMethod": format!("{}#key-1", cred.issuer_did),
            "proofPurpose": "assertionMethod",
            "proofValue": &cred.signature,
        }
    });

    Ok(HttpResponse::Ok()
        .content_type("application/vc+ld+json")
        .json(vc))
}

// ── JSON-LD export for governance ───────────────────────────────────────────

/// GET /api/v1/governance/proposals/{id}/export — JSON-LD export of election results
#[get("/governance/proposals/{id}/export")]
pub async fn export_governance_jsonld(
    state: web::Data<AppState>,
    path: web::Path<u64>,
) -> ApiResult<HttpResponse> {
    let id = path.into_inner();

    let proposal_store = match &state.proposal_store {
        Some(s) => s,
        None => {
            return Ok(
                HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(
                    err_dto("governance not configured"),
                    503,
                )),
            )
        }
    };
    let vote_store = match &state.vote_store {
        Some(s) => s,
        None => {
            return Ok(
                HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(
                    err_dto("governance not configured"),
                    503,
                )),
            )
        }
    };

    let proposal = match proposal_store.get(id) {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                err_dto(&format!("proposal {id} not found")),
                404,
            )))
        }
    };

    let registry = match &state.param_registry {
        Some(r) => r,
        None => {
            return Ok(
                HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(
                    err_dto("governance not configured"),
                    503,
                )),
            )
        }
    };
    let quorum = registry.get_u64("quorum_percent", 33);
    let threshold = registry.get_u64("pass_threshold_percent", 67);

    let tally = vote_store.tally(id, 1, quorum, threshold);

    let export = json!({
        "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://schema.org/"
        ],
        "@type": "VoteAction",
        "identifier": format!("cerulean:proposal:{id}"),
        "name": proposal.description,
        "description": format!("{:?}", proposal.action),
        "startTime": proposal.submitted_at,
        "endTime": proposal.voting_ends_at,
        "result": {
            "@type": "VoteResult",
            "yesVotes": tally.yes_power,
            "noVotes": tally.no_power,
            "abstainVotes": tally.abstain_power,
            "totalVotes": tally.total_voted_power,
            "quorumReached": tally.quorum_reached,
            "passed": tally.passed,
            "quorumPercent": quorum,
            "passThresholdPercent": threshold,
        },
        "organizer": {
            "@type": "Organization",
            "identifier": proposal.proposer,
        },
        "instrument": {
            "@type": "Thing",
            "name": "Cerulean Ledger",
            "description": "DLT post-quantum blockchain",
            "url": "https://github.com/clementeaf/cerulean-ledger"
        }
    });

    Ok(HttpResponse::Ok()
        .content_type("application/ld+json")
        .json(export))
}
