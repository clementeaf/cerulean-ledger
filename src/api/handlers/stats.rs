//! Stats endpoint — comprehensive system statistics.
//!
//! Endpoints:
//! - GET /stats — system statistics (blockchain, mempool, network)
//!
//! Note: Prometheus metrics are served at `/metrics` (outside `/api/v1`)
//! by `utilities::get_metrics`.

use crate::api::errors::{ApiResponse, ApiResult};
use crate::app_state::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct StatsResponse {
    blockchain: BlockchainStats,
    mempool: MempoolStats,
    network: NetworkStats,
}

#[derive(Serialize)]
struct BlockchainStats {
    block_count: usize,
    total_transactions: usize,
    difficulty: u8,
    latest_block_hash: String,
    latest_block_index: u64,
    total_coinbase: u64,
    unique_addresses: usize,
    avg_block_time_seconds: f64,
    target_block_time: u64,
    max_transactions_per_block: usize,
    max_block_size_bytes: usize,
}

#[derive(Serialize)]
struct MempoolStats {
    pending_transactions: usize,
    total_fees_pending: u64,
}

#[derive(Serialize)]
struct NetworkStats {
    connected_peers: usize,
}

/// GET /api/v1/stats
#[get("/stats")]
pub async fn get_stats(state: web::Data<AppState>) -> ApiResult<HttpResponse> {
    let trace = uuid::Uuid::new_v4().to_string();

    let (
        block_count,
        difficulty,
        latest_block_hash,
        latest_block_index,
        total_transactions,
        total_coinbase,
        unique_addresses_count,
        avg_block_time,
        target_block_time,
        max_transactions_per_block,
        max_block_size_bytes,
    ) = {
        let store = state
            .store
            .read()
            .ok()
            .and_then(|m| m.get("default").cloned());
        if let Some(store) = store {
            let latest_height = store.get_latest_height().unwrap_or(0);
            let block_count = if store.block_exists(0).unwrap_or(false) {
                (latest_height + 1) as usize
            } else {
                0
            };

            let latest_block = store.read_block(latest_height).ok();
            let latest_block_hash = latest_block
                .as_ref()
                .map(|b| format!("{:x?}", &b.parent_hash[..4]))
                .unwrap_or_default();

            let mut total_tx = 0usize;
            let mut total_coinbase_amount = 0u64;
            let mut unique = std::collections::HashSet::new();
            let mut block_times = Vec::new();
            let mut prev_timestamp = 0u64;

            for h in 0..block_count as u64 {
                if let Ok(block) = store.read_block(h) {
                    total_tx += block.transactions.len();
                    if h > 0 && block.timestamp > prev_timestamp {
                        block_times.push(block.timestamp.saturating_sub(prev_timestamp));
                    }
                    prev_timestamp = block.timestamp;

                    if let Ok(txs) = store.transactions_by_block_height(h) {
                        for tx in &txs {
                            if tx.input_did == "coinbase" {
                                total_coinbase_amount += tx.amount;
                            }
                            if !tx.input_did.is_empty() && tx.input_did != "coinbase" {
                                unique.insert(tx.input_did.clone());
                            }
                            if !tx.output_recipient.is_empty() {
                                unique.insert(tx.output_recipient.clone());
                            }
                        }
                    }
                }
            }

            let avg_bt = if !block_times.is_empty() {
                block_times.iter().sum::<u64>() as f64 / block_times.len() as f64
            } else {
                0.0
            };

            (
                block_count,
                1u8,
                latest_block_hash,
                latest_height,
                total_tx,
                total_coinbase_amount,
                unique.len(),
                avg_bt,
                60u64,
                1000usize,
                1_000_000usize,
            )
        } else {
            (0, 1, String::new(), 0, 0, 0, 0, 0.0, 60, 1000, 1_000_000)
        }
    };

    let (mempool_size, total_fees) = {
        let pool = state.tx_pool.lock().unwrap_or_else(|e| e.into_inner());
        (pool.len(), 0u64)
    };

    let peers_count = if let Some(node) = &state.node {
        let peers = node.peers.lock().unwrap_or_else(|e| e.into_inner());
        peers.len()
    } else {
        0
    };

    let response_data = StatsResponse {
        blockchain: BlockchainStats {
            block_count,
            total_transactions,
            difficulty,
            latest_block_hash,
            latest_block_index,
            total_coinbase,
            unique_addresses: unique_addresses_count,
            avg_block_time_seconds: avg_block_time,
            target_block_time,
            max_transactions_per_block,
            max_block_size_bytes,
        },
        mempool: MempoolStats {
            pending_transactions: mempool_size,
            total_fees_pending: total_fees,
        },
        network: NetworkStats {
            connected_peers: peers_count,
        },
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response_data, trace)))
}
