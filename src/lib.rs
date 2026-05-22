// Librería para tests de integración
pub(crate) mod blockchain;
pub mod contract_validation;
pub mod governance_contracts;
pub(crate) mod models;
pub mod multisig_contracts;
pub mod network_security;
pub mod oracle_collateral;
pub mod oracle_connector;
pub mod oracle_demo;
pub mod oracle_system;
pub mod plugin_contracts;
pub mod staking_rewards;
pub mod transaction_validation;

// Phase 2 Week 2-3: Storage & Consensus Layers
pub mod consensus;
pub mod crypto;
pub mod identity;
pub mod storage;

// Runtime stack (shared with binary for AppState and scaffold handlers)
pub mod airdrop;
pub mod billing;
pub mod cache;
pub mod checkpoint;
pub mod metrics;
pub mod network;
pub mod smart_contracts;
pub mod staking;

// Phase 3 Week 5: REST API Gateway
pub mod acl;
pub mod api;
pub mod app_state;
pub mod audit;
pub mod block_creation;
pub mod bridge;
pub mod chaincode;
pub mod channel;
pub mod compliance;
pub mod discovery;
pub mod endorsement;
pub mod events;
#[cfg(feature = "evm")]
pub mod evm_compat;
pub mod forensic;
pub mod forensic_pentest;
mod fuzz_tests;
pub mod gateway;
pub mod governance;
pub mod inference;
pub mod intelligence;
pub mod legal_oracle;
pub mod light_client;
pub mod mining;
pub mod msp;
pub mod ordering;
pub mod pin;
pub mod pki;
pub mod private_data;
pub mod registry;
pub mod regulatory;
pub mod stress;
pub mod testnet;
pub mod tls;
pub mod tokenomics;
pub mod transaction;

pub use app_state::AppState;
