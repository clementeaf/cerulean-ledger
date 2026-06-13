//! Smart contract (ERC-20) and NFT (ERC-721) handler endpoints.
//!
//! Endpoints:
//! - POST /contracts/debug                              — debug deploy (raw body)
//! - POST /contracts                                    — deploy a new contract
//! - GET  /contracts                                    — list all contracts
//! - GET  /contracts/{address}                          — get contract by address
//! - POST /contracts/{address}/execute                  — execute contract function
//! - GET  /contracts/{address}/balance/{wallet}         — ERC-20 balance of wallet
//! - GET  /contracts/{address}/allowance/{owner}/{spender} — ERC-20 allowance
//! - GET  /contracts/{address}/totalSupply              — ERC-20 total supply
//! - GET  /contracts/{address}/nft/{token_id}/owner     — NFT owner of token
//! - GET  /contracts/{address}/nft/{token_id}/uri       — NFT token URI
//! - GET  /contracts/{address}/nft/{token_id}/approved  — NFT approved address
//! - GET  /contracts/{address}/nft/{token_id}/metadata  — NFT metadata (GET)
//! - POST /contracts/{address}/nft/{token_id}/metadata  — NFT metadata (SET)
//! - GET  /contracts/{address}/nft/balance/{wallet}     — NFT balance of wallet
//! - GET  /contracts/{address}/nft/totalSupply          — NFT total supply
//! - GET  /contracts/{address}/nft/tokens/{owner}       — NFT tokens of owner
//! - GET  /contracts/{address}/nft/index/{index}        — NFT token by index

use std::collections::HashMap;
use std::time::Instant;

use actix_web::web::Bytes;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::api::errors::{enforce_acl, ApiError, ApiResponse, ApiResult};
use crate::app_state::AppState;
use crate::smart_contracts::{ContractFunction, NFTMetadata, SmartContract};

// ── Request structs ─────────────────────────────────────────────────────────

/// Request body for deploying a new smart contract.
#[derive(Deserialize)]
pub struct DeployContractRequest {
    pub owner: String,
    pub contract_type: String,
    pub name: String,
    pub symbol: Option<String>,
    pub total_supply: Option<u64>,
    pub decimals: Option<u8>,
}

/// Request body for executing a contract function.
#[derive(Deserialize)]
pub struct ExecuteContractRequest {
    pub function: String, // "transfer", "mint", "burn", "custom"
    pub params: serde_json::Value,
}

/// Request body for setting NFT metadata.
#[derive(Deserialize)]
pub struct SetNFTMetadataRequest {
    pub metadata: NFTMetadata,
}

// ── Per-caller rate limiting ────────────────────────────────────────────────

/// Tracks request timestamps for per-caller rate limiting.
struct CallerRateLimitInfo {
    requests_per_second: Vec<Instant>,
    requests_per_minute: Vec<Instant>,
}

impl CallerRateLimitInfo {
    fn new() -> Self {
        CallerRateLimitInfo {
            requests_per_second: Vec::new(),
            requests_per_minute: Vec::new(),
        }
    }

    /// Remove timestamps older than 1 second / 1 minute.
    fn cleanup(&mut self, now: Instant) {
        let one_second_ago = now - std::time::Duration::from_secs(1);
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        self.requests_per_second
            .retain(|&time| time > one_second_ago);
        self.requests_per_minute
            .retain(|&time| time > one_minute_ago);
    }
}

lazy_static::lazy_static! {
    static ref ERC20_RATE_LIMIT: std::sync::Mutex<HashMap<String, CallerRateLimitInfo>> =
        std::sync::Mutex::new(HashMap::new());
}

/// Check per-caller rate limit (10 req/s, 100 req/min).
fn check_erc20_rate_limit(caller: &str) -> Result<(), String> {
    const MAX_REQUESTS_PER_SECOND: u32 = 10;
    const MAX_REQUESTS_PER_MINUTE: u32 = 100;

    let mut limits = ERC20_RATE_LIMIT.lock().unwrap_or_else(|e| e.into_inner());
    let now = Instant::now();

    let caller_info = limits
        .entry(caller.to_string())
        .or_insert_with(CallerRateLimitInfo::new);

    caller_info.cleanup(now);

    if caller_info.requests_per_second.len() >= MAX_REQUESTS_PER_SECOND as usize {
        return Err("Rate limit exceeded: too many requests per second".to_string());
    }

    if caller_info.requests_per_minute.len() >= MAX_REQUESTS_PER_MINUTE as usize {
        return Err("Rate limit exceeded: too many requests per minute".to_string());
    }

    caller_info.requests_per_second.push(now);
    caller_info.requests_per_minute.push(now);

    Ok(())
}

// ── Helper: extract a required string param ─────────────────────────────────

fn require_str(params: &serde_json::Value, key: &str) -> Result<String, ApiError> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::ValidationError {
            field: key.to_string(),
            reason: format!("Missing '{key}' parameter"),
        })
}

fn require_u64(params: &serde_json::Value, key: &str) -> Result<u64, ApiError> {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::ValidationError {
            field: key.to_string(),
            reason: format!("Missing '{key}' parameter"),
        })
}

// ── Contract not-found helper ───────────────────────────────────────────────

fn contract_not_found() -> ApiError {
    ApiError::NotFound {
        resource: "Contract not found".to_string(),
    }
}

// ── Endpoints ───────────────────────────────────────────────────────────────

/// Shared deploy logic used by both endpoints.
async fn deploy_contract_inner(
    state: web::Data<AppState>,
    body: Bytes,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    log::debug!("[DEPLOY] Body recibido, length: {}", body.len());

    let deploy_req: DeployContractRequest = serde_json::from_slice(&body).map_err(|e| {
        log::debug!("[DEPLOY] ERROR al parsear JSON: {e}");
        ApiError::ValidationError {
            field: "body".to_string(),
            reason: format!("Invalid JSON: {e}"),
        }
    })?;

    log::debug!(
        "[DEPLOY] Tipo: {}, Owner: {}, Name: {}",
        deploy_req.contract_type,
        deploy_req.owner,
        deploy_req.name
    );

    let contract = SmartContract::new(
        deploy_req.owner.clone(),
        deploy_req.contract_type.clone(),
        deploy_req.name.clone(),
        deploy_req.symbol.clone(),
        deploy_req.total_supply,
        deploy_req.decimals,
    );

    let address = {
        let mut contract_manager = state
            .contract_manager
            .write()
            .unwrap_or_else(|e| e.into_inner());
        contract_manager
            .deploy_contract(contract.clone())
            .map_err(|e| {
                log::debug!("[DEPLOY] ERROR en deploy_contract(): {e}");
                ApiError::ValidationError {
                    field: "contract".to_string(),
                    reason: e,
                }
            })?
    };

    if let Some(node) = &state.node {
        let node_clone = node.clone();
        let contract_clone = contract.clone();
        tokio::spawn(async move {
            node_clone.broadcast_contract(&contract_clone).await;
        });
    }

    log::debug!("[DEPLOY] Deploy completado, address: {address}");
    Ok(HttpResponse::Created().json(ApiResponse::success_with_code(address, 201, trace)))
}

/// POST /api/v1/contracts/debug — debug deploy (raw body inspection)
#[post("/contracts/debug")]
pub async fn deploy_contract_debug(
    state: web::Data<AppState>,
    body: Bytes,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    log::debug!("[DEPLOY DEBUG] Body length: {}", body.len());
    let body_str = String::from_utf8_lossy(&body);
    log::debug!(
        "[DEPLOY DEBUG] Body (first 500 chars): {}",
        &body_str[..body_str.len().min(500)]
    );
    deploy_contract_inner(state, body, req).await
}

/// POST /api/v1/contracts — deploy a new smart contract
#[post("/contracts")]
pub async fn deploy_contract(
    state: web::Data<AppState>,
    body: Bytes,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    deploy_contract_inner(state, body, req).await
}

/// GET /api/v1/contracts — list all deployed contracts
#[get("/contracts")]
pub async fn get_all_contracts(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());
    let contracts: Vec<&SmartContract> = contract_manager.get_all_contracts();
    let contracts_cloned: Vec<SmartContract> = contracts.iter().map(|c| (*c).clone()).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(contracts_cloned, trace)))
}

/// GET /api/v1/contracts/{address} — get contract by address
#[get("/contracts/{address}")]
pub async fn get_contract(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());
    match contract_manager.get_contract(&address) {
        Some(contract) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(contract.clone(), trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// POST /api/v1/contracts/{address}/execute — execute a contract function
#[post("/contracts/{address}/execute")]
pub async fn execute_contract_function(
    state: web::Data<AppState>,
    address: web::Path<String>,
    body: web::Json<ExecuteContractRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    // Extract caller for rate limiting.
    // For ERC-20, the caller must come in params for transfer, approve, transferFrom.
    // For NFT, the caller can come in params or be "from" for transferNFT.
    let caller = body
        .params
        .get("caller")
        .and_then(|v| v.as_str())
        .or_else(|| {
            if body.function == "transfer" || body.function == "transferNFT" {
                body.params.get("from").and_then(|v| v.as_str())
            } else {
                None
            }
        });

    // Per-caller rate limiting for ERC-20 and NFT mutation functions
    if let Some(caller_addr) = caller {
        if matches!(
            body.function.as_str(),
            "transfer"
                | "transferFrom"
                | "approve"
                | "mint"
                | "burn"
                | "mintNFT"
                | "transferNFT"
                | "approveNFT"
                | "transferFromNFT"
                | "burnNFT"
        ) {
            check_erc20_rate_limit(caller_addr).map_err(|e| ApiError::ValidationError {
                field: "rate_limit".to_string(),
                reason: e,
            })?;
        }
    }

    let function = match body.function.as_str() {
        // ERC-20: transfer(to, amount) — caller is the from
        "transfer" => {
            let to = require_str(&body.params, "to")?;
            let amount = require_u64(&body.params, "amount")?;
            ContractFunction::Transfer { to, amount }
        }
        // ERC-20: transferFrom(from, to, amount) — caller is the spender
        "transferFrom" => {
            let from = require_str(&body.params, "from")?;
            let to = require_str(&body.params, "to")?;
            let amount = require_u64(&body.params, "amount")?;
            ContractFunction::TransferFrom { from, to, amount }
        }
        // ERC-20: approve(spender, amount) — caller is the owner
        "approve" => {
            let spender = require_str(&body.params, "spender")?;
            let amount = require_u64(&body.params, "amount")?;
            ContractFunction::Approve { spender, amount }
        }
        "mint" => {
            let to = require_str(&body.params, "to")?;
            let amount = require_u64(&body.params, "amount")?;
            ContractFunction::Mint { to, amount }
        }
        "burn" => {
            let from = require_str(&body.params, "from")?;
            let amount = require_u64(&body.params, "amount")?;
            ContractFunction::Burn { from, amount }
        }
        // NFT: mintNFT(to, token_id, token_uri)
        "mintNFT" => {
            let to = require_str(&body.params, "to")?;
            let token_id = require_u64(&body.params, "token_id")?;
            let token_uri = require_str(&body.params, "token_uri")?;
            ContractFunction::MintNFT {
                to,
                token_id,
                token_uri,
            }
        }
        // NFT: transferNFT(from, to, token_id) — caller is the from or approved
        "transferNFT" => {
            let from = require_str(&body.params, "from")?;
            let to = require_str(&body.params, "to")?;
            let token_id = require_u64(&body.params, "token_id")?;
            ContractFunction::TransferNFT { from, to, token_id }
        }
        // NFT: approveNFT(to, token_id) — caller is the owner
        "approveNFT" => {
            let to = require_str(&body.params, "to")?;
            let token_id = require_u64(&body.params, "token_id")?;
            ContractFunction::ApproveNFT { to, token_id }
        }
        // NFT: transferFromNFT(from, to, token_id) — caller is the spender
        "transferFromNFT" => {
            let from = require_str(&body.params, "from")?;
            let to = require_str(&body.params, "to")?;
            let token_id = require_u64(&body.params, "token_id")?;
            ContractFunction::TransferFromNFT { from, to, token_id }
        }
        // NFT: burnNFT(owner, token_id)
        "burnNFT" => {
            let owner = require_str(&body.params, "owner")?;
            let token_id = require_u64(&body.params, "token_id")?;
            ContractFunction::BurnNFT { owner, token_id }
        }
        "custom" => {
            let name = require_str(&body.params, "name")?;
            let params = body
                .params
                .get("params")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            ContractFunction::Custom { name, params }
        }
        _ => {
            return Err(ApiError::ValidationError {
                field: "function".to_string(),
                reason: format!("Unknown function: {}", body.function),
            });
        }
    };

    // Acquire write lock to execute the function
    let mut contract_manager = state
        .contract_manager
        .write()
        .unwrap_or_else(|e| e.into_inner());

    let execution_result = contract_manager.execute_contract_function(&address, function, caller);

    match execution_result {
        Ok(result) => {
            // Save updated state and broadcast
            let contract_for_broadcast = contract_manager.get_contract(&address).cloned();
            drop(contract_manager); // Release lock before I/O

            if let Some(contract_clone) = contract_for_broadcast {
                if let Some(node) = &state.node {
                    let node_clone = node.clone();
                    let contract_for_broadcast = contract_clone.clone();

                    let peers_count = {
                        match node_clone.peers.lock() {
                            Ok(peers_guard) => peers_guard.len(),
                            Err(e) => {
                                eprintln!("Warning: Error acquiring peers lock: {e}");
                                0
                            }
                        }
                    };

                    if peers_count > 0 {
                        tokio::spawn(async move {
                            node_clone
                                .broadcast_contract_update(&contract_for_broadcast)
                                .await;
                        });
                    }
                }
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(result, trace)))
        }
        Err(e) => {
            let error_msg = if e.contains("Insufficient") {
                e
            } else if e.contains("not found") {
                format!("Contract execution failed: {e}")
            } else {
                format!("Execution error: {e}")
            };

            Err(ApiError::ValidationError {
                field: "execution".to_string(),
                reason: error_msg,
            })
        }
    }
}

/// GET /api/v1/contracts/{address}/balance/{wallet} — ERC-20 balance
#[get("/contracts/{address}/balance/{wallet}")]
pub async fn get_contract_balance(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, wallet_address) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let balance = contract.get_balance(&wallet_address);
            Ok(HttpResponse::Ok().json(ApiResponse::success(balance, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/allowance/{owner}/{spender} — ERC-20 allowance
#[get("/contracts/{address}/allowance/{owner}/{spender}")]
pub async fn get_contract_allowance(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, owner_address, spender_address) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let allowance = contract.allowance(&owner_address, &spender_address);
            Ok(HttpResponse::Ok().json(ApiResponse::success(allowance, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/totalSupply — ERC-20 total supply
#[get("/contracts/{address}/totalSupply")]
pub async fn get_contract_total_supply(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let contract_address = address.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let total_supply = contract.total_supply();
            Ok(HttpResponse::Ok().json(ApiResponse::success(total_supply, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/{token_id}/owner — NFT owner of token
#[get("/contracts/{address}/nft/{token_id}/owner")]
pub async fn get_nft_owner(
    state: web::Data<AppState>,
    path: web::Path<(String, u64)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, token_id) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => match contract.owner_of(token_id) {
            Some(owner) => Ok(HttpResponse::Ok().json(ApiResponse::success(owner, trace))),
            None => Err(ApiError::NotFound {
                resource: format!("Token ID {token_id} does not exist"),
            }),
        },
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/{token_id}/uri — NFT token URI
#[get("/contracts/{address}/nft/{token_id}/uri")]
pub async fn get_nft_token_uri(
    state: web::Data<AppState>,
    path: web::Path<(String, u64)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, token_id) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => match contract.token_uri(token_id) {
            Some(uri) => Ok(HttpResponse::Ok().json(ApiResponse::success(uri, trace))),
            None => Err(ApiError::NotFound {
                resource: format!("Token ID {token_id} does not exist"),
            }),
        },
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/{token_id}/approved — NFT approved address
#[get("/contracts/{address}/nft/{token_id}/approved")]
pub async fn get_nft_approved(
    state: web::Data<AppState>,
    path: web::Path<(String, u64)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, token_id) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            match contract.get_approved(token_id) {
                Some(approved) => {
                    Ok(HttpResponse::Ok().json(ApiResponse::success(approved, trace)))
                }
                None => {
                    // No approval — return empty string (standard ERC-721 behavior)
                    Ok(HttpResponse::Ok().json(ApiResponse::success(String::new(), trace)))
                }
            }
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/{token_id}/metadata — NFT metadata
#[get("/contracts/{address}/nft/{token_id}/metadata")]
pub async fn get_nft_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, u64)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, token_id) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => match contract.get_nft_metadata(token_id) {
            Some(metadata) => {
                Ok(HttpResponse::Ok().json(ApiResponse::success(metadata.clone(), trace)))
            }
            None => Err(ApiError::NotFound {
                resource: format!("Metadata for token ID {token_id} does not exist"),
            }),
        },
        None => Err(contract_not_found()),
    }
}

/// POST /api/v1/contracts/{address}/nft/{token_id}/metadata — set NFT metadata
#[post("/contracts/{address}/nft/{token_id}/metadata")]
pub async fn set_nft_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, u64)>,
    body: web::Json<SetNFTMetadataRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    enforce_acl(
        state.acl_provider.as_deref(),
        state.policy_store.as_deref(),
        "peer/Propose",
        &req,
    )?;
    let trace = uuid::Uuid::new_v4().to_string();

    let (contract_address, token_id) = path.into_inner();
    let mut contract_manager = state
        .contract_manager
        .write()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract_mut(&contract_address) {
        Some(contract) => match contract.set_nft_metadata(token_id, body.metadata.clone()) {
            Ok(()) => {
                drop(contract_manager);

                let msg = format!("Metadata set for token {token_id}");
                Ok(HttpResponse::Ok().json(ApiResponse::success(msg, trace)))
            }
            Err(e) => Err(ApiError::ValidationError {
                field: "metadata".to_string(),
                reason: e,
            }),
        },
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/balance/{wallet} — NFT balance of wallet
#[get("/contracts/{address}/nft/balance/{wallet}")]
pub async fn get_nft_balance(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, wallet_address) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let balance = contract.balance_of_nft(&wallet_address);
            Ok(HttpResponse::Ok().json(ApiResponse::success(balance, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/totalSupply — NFT total minted supply
#[get("/contracts/{address}/nft/totalSupply")]
pub async fn get_nft_total_supply(
    state: web::Data<AppState>,
    address: web::Path<String>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let contract_address = address.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let total_supply = contract.total_supply_nft();
            Ok(HttpResponse::Ok().json(ApiResponse::success(total_supply, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/tokens/{owner} — list NFT tokens of owner
#[get("/contracts/{address}/nft/tokens/{owner}")]
pub async fn get_nft_tokens_of_owner(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, owner_address) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => {
            let tokens = contract.tokens_of_owner(&owner_address);
            Ok(HttpResponse::Ok().json(ApiResponse::success(tokens, trace)))
        }
        None => Err(contract_not_found()),
    }
}

/// GET /api/v1/contracts/{address}/nft/index/{index} — get NFT token by index
#[get("/contracts/{address}/nft/index/{index}")]
pub async fn get_nft_token_by_index(
    state: web::Data<AppState>,
    path: web::Path<(String, usize)>,
) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();
    let (contract_address, index) = path.into_inner();

    let contract_manager = state
        .contract_manager
        .read()
        .unwrap_or_else(|e| e.into_inner());

    match contract_manager.get_contract(&contract_address) {
        Some(contract) => match contract.token_by_index(index) {
            Some(token_id) => Ok(HttpResponse::Ok().json(ApiResponse::success(token_id, trace))),
            None => Err(ApiError::NotFound {
                resource: format!("Token at index {index} does not exist"),
            }),
        },
        None => Err(contract_not_found()),
    }
}
