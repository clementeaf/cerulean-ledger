use actix_web::{web, Scope};

#[cfg(feature = "evm")]
use crate::api::handlers::evm;
use crate::api::handlers::{
    acl, alias, audit, blocks, chain, chaincode, channels, compliance, compliance_auto, contact,
    credentials, discovery, events, forensic, gateway, governance, governance_entities, identity,
    inference, intelligence, interop, invitations, legal_oracle, msp, notarize, oracle,
    organizations, pentest, pin, private_data, proposals, registry, regulatory, snapshots, staking,
    stress, tokenization, transactions, utilities, vault, zkp,
};

/// API routes configuration
pub struct ApiRoutes;

#[allow(dead_code)]
impl ApiRoutes {
    /// Full configuration: metrics + standalone `/api/v1` scope with scaffold routes.
    /// Used by integration tests that don't load the legacy router.
    pub fn configure(cfg: &mut web::ServiceConfig) {
        Self::configure_metrics(cfg);
        cfg.service(Self::register(web::scope("/api/v1")));
    }

    /// Only register the `/metrics` endpoint (used in production where legacy
    /// router owns the `/api/v1` scope and calls `ApiRoutes::register`).
    pub fn configure_metrics(cfg: &mut web::ServiceConfig) {
        cfg.service(utilities::get_metrics)
            .route("/swagger", web::get().to(utilities::swagger_ui));
    }

    /// Register all scaffold routes into an existing `/api/v1` scope.
    ///
    /// Uses `configure()` closures to break the type chain and avoid
    /// stack overflow from deeply nested Actix generic wrappers.
    pub fn register(scope: Scope) -> Scope {
        scope
            // Sub-scoped (have their own path prefix)
            .service(Self::identity_routes())
            .service(Self::blocks_routes())
            .service(Self::store_blocks_routes())
            .service(Self::chain_routes())
            .service(Self::credentials_routes())
            // Break the chain with configure() to limit generic nesting depth
            .configure(Self::register_store_handlers)
            .configure(Self::register_tx_handlers)
            .configure(Self::register_infra_handlers)
    }

    fn register_store_handlers(cfg: &mut web::ServiceConfig) {
        cfg.service(transactions::store_write_transaction)
            .service(transactions::store_get_transaction)
            .service(identity::store_write_identity)
            .service(identity::store_list_identities)
            .service(identity::store_get_identity)
            .service(credentials::store_write_credential)
            .service(credentials::store_list_credentials)
            .service(credentials::store_get_credential)
            .service(credentials::store_get_credentials_by_subject)
            .service(credentials::store_get_credentials_by_issuer)
            .service(organizations::store_create_organization)
            .service(organizations::store_list_organizations)
            .service(organizations::store_get_organization)
            .service(organizations::store_set_policy)
            .service(organizations::store_get_policy)
            // Governance entities (Cerulean Voto)
            .service(governance_entities::create_scope)
            .service(governance_entities::list_scopes)
            .service(governance_entities::get_scope)
            .service(governance_entities::update_scope)
            .service(governance_entities::remove_scope)
            .service(governance_entities::create_assembly)
            .service(governance_entities::list_assemblies)
            .service(governance_entities::get_assembly)
            .service(governance_entities::update_assembly)
            .service(governance_entities::remove_assembly)
            .service(governance_entities::create_session)
            .service(governance_entities::list_sessions)
            .service(governance_entities::get_session)
            .service(governance_entities::update_session)
            .service(governance_entities::remove_session)
            .service(governance_entities::create_acta)
            .service(governance_entities::list_actas)
            .service(governance_entities::get_acta)
            .service(governance_entities::update_acta)
            .service(governance_entities::remove_acta)
            // Asset Registry
            .service(registry::create_asset)
            .service(registry::list_assets)
            .service(registry::get_asset)
            .service(registry::update_asset)
            .service(registry::remove_asset)
            .service(registry::create_asset_event)
            .service(registry::create_asset_events_batch)
            .service(registry::list_asset_events)
            .service(registry::get_asset_event)
            .service(registry::export_asset)
            // RWA Tokenization
            .service(tokenization::create_token)
            .service(tokenization::list_tokens)
            .service(tokenization::get_token)
            .service(tokenization::update_token)
            .service(tokenization::remove_token)
            // Compliance Automation
            .service(compliance_auto::create_rule)
            .service(compliance_auto::list_rules)
            .service(compliance_auto::get_rule)
            .service(compliance_auto::remove_rule)
            .service(compliance_auto::evaluate_asset)
            .service(compliance_auto::get_results);
    }

    fn register_tx_handlers(cfg: &mut web::ServiceConfig) {
        cfg.service(transactions::create_transaction)
            .service(transactions::get_mempool)
            .service(transactions::get_tx_by_id)
            .service(blocks::explorer_list_blocks)
            .service(blocks::explorer_get_block)
            .service(proposals::submit_proposal)
            .service(proposals::submit_endorsed_transaction)
            .service(channels::create_channel)
            .service(channels::list_channels)
            .service(channels::update_channel_config)
            .service(channels::get_channel_config)
            .service(channels::get_channel_config_history)
            .service(msp::revoke_serial)
            .service(msp::get_msp_info)
            .service(gateway::gateway_submit);
    }

    fn register_infra_handlers(cfg: &mut web::ServiceConfig) {
        cfg.service(private_data::register_collection)
            .service(private_data::put_private_data)
            .service(private_data::get_private_data)
            .service(chaincode::install_chaincode)
            .service(chaincode::approve_chaincode)
            .service(chaincode::commit_chaincode)
            .service(chaincode::simulate_chaincode)
            .service(chaincode::invoke_chaincode)
            .service(chaincode::get_sandbox_report)
            .service(discovery::get_endorsers)
            .service(discovery::get_channel_peers)
            .service(discovery::post_register_peer)
            .service(events::events_blocks)
            .service(events::events_blocks_filtered)
            .service(events::events_blocks_private)
            .service(acl::set_acl)
            .service(acl::list_acls)
            .service(acl::get_acl)
            .service(snapshots::create_snapshot)
            .service(snapshots::list_snapshots)
            .service(snapshots::download_snapshot)
            .service(audit::list_audit_entries)
            .service(audit::export_audit_csv);
        #[cfg(feature = "evm")]
        cfg.service(evm::evm_deploy)
            .service(evm::evm_call)
            .service(evm::evm_static_call)
            .service(evm::evm_list_contracts);
        cfg.service(governance::list_params)
            .service(governance::submit_governance_proposal)
            .service(governance::list_governance_proposals)
            .service(governance::get_governance_proposal)
            .service(governance::cast_governance_vote)
            .service(governance::get_governance_votes)
            .service(governance::get_voter_history)
            .service(governance::tally_governance_votes)
            .service(governance::execute_governance_proposal)
            .service(governance::delegate_vote)
            .service(governance::veto_governance_proposal)
            .service(governance::close_governance_voting)
            .service(pin::generate_pin)
            .service(pin::verify_pin);
        // ZKP identity
        cfg.service(zkp::prove_zkp).service(zkp::verify_zkp);
        // Oracle, forensic, compliance
        cfg.service(oracle::get_oracle_feed)
            .service(oracle::list_oracle_feeds)
            .service(oracle::list_oracle_nodes)
            .service(oracle::oracle_status)
            .service(oracle::register_oracle);
        // Legal oracle
        cfg.service(legal_oracle::query_legal_oracle)
            .service(legal_oracle::list_legal_oracle_records)
            .service(legal_oracle::get_legal_oracle_record)
            .service(forensic::forensic_timeline)
            .service(forensic::forensic_security)
            .service(forensic::forensic_export)
            .service(forensic::forensic_replay)
            .service(forensic::forensic_integrity)
            .service(intelligence::detect_anomalies)
            .service(intelligence::evaluate_risk)
            .service(intelligence::detect_patterns)
            .service(compliance::validate_pacs008)
            .service(compliance::validate_pacs002)
            .service(compliance::validate_pacs004)
            .service(compliance::validate_pain001)
            .service(compliance::validate_pain002)
            .service(compliance::validate_camt053)
            .service(compliance::validate_camt052)
            .service(compliance::list_countries)
            .service(compliance::list_currencies)
            .service(contact::submit_contact)
            .service(contact::list_contacts)
            .service(regulatory::run_checks)
            .service(regulatory::compliance_report)
            .service(stress::stress_report)
            .service(pentest::pentest_report);
        // W3C interoperability (DID Resolution, JSON-LD export)
        // Note: get_credential_as_vc is registered in credentials_routes() scope
        cfg.service(interop::resolve_did)
            .service(interop::export_governance_jsonld);
        // Vault (encrypted wallet backup + recovery)
        cfg.service(vault::vault_store)
            .service(vault::vault_get)
            .service(vault::vault_recover);
        // Inference (Optimistic ML Oracle)
        cfg.service(inference::submit_inference)
            .service(inference::submit_proven)
            .service(inference::challenge_inference)
            .service(inference::finalize_inference)
            .service(inference::list_claims)
            .service(inference::get_claim)
            .service(inference::list_models);
        // Staking (validator registration, unstaking, queries)
        cfg.service(staking::stake)
            .service(staking::request_unstake)
            .service(staking::complete_unstake)
            .service(staking::get_validators)
            .service(staking::get_validator)
            .service(staking::get_my_stake);
        // Alias registry (zero-knowledge alias system)
        cfg.service(alias::alias_register)
            .service(alias::alias_resolve)
            .service(alias::alias_by_did)
            .service(alias::alias_revoke);
        // Notarization (Proof of Existence)
        cfg.service(notarize::submit_notarization)
            .service(notarize::verify_notarization)
            .service(notarize::get_notarization)
            .service(notarize::list_notarizations);
        // Invitations (governance proposal invitations via alias)
        cfg.service(invitations::create_invitation)
            .service(invitations::list_invitations)
            .service(invitations::respond_invitation);
    }

    fn identity_routes() -> Scope {
        web::scope("/identity")
    }

    fn blocks_routes() -> Scope {
        web::scope("/blocks")
            .service(blocks::create_block)
            .service(blocks::list_blocks)
            .service(blocks::get_block_by_index)
            .service(blocks::get_block_by_hash)
    }

    fn store_blocks_routes() -> Scope {
        web::scope("/store/blocks")
            .service(blocks::store_list_blocks)
            .service(blocks::store_latest_height)
            .service(blocks::store_get_block)
            .service(transactions::store_get_transactions_by_block)
    }

    fn store_transactions_routes() -> Scope {
        web::scope("")
            .service(transactions::store_write_transaction)
            .service(transactions::store_get_transaction)
    }

    fn store_identities_routes() -> Scope {
        web::scope("")
            .service(identity::store_write_identity)
            .service(identity::store_list_identities)
            .service(identity::store_get_identity)
    }

    fn store_credentials_routes() -> Scope {
        web::scope("")
            .service(credentials::store_write_credential)
            .service(credentials::store_list_credentials)
            .service(credentials::store_get_credential)
            .service(credentials::store_get_credentials_by_subject)
    }

    fn store_organizations_routes() -> Scope {
        web::scope("")
            .service(organizations::store_create_organization)
            .service(organizations::store_list_organizations)
            .service(organizations::store_get_organization)
    }

    fn store_policies_routes() -> Scope {
        web::scope("")
            .service(organizations::store_set_policy)
            .service(organizations::store_get_policy)
    }

    fn chain_routes() -> Scope {
        web::scope("/chain")
            .service(chain::verify_chain)
            .service(chain::get_blockchain_info)
    }

    fn credentials_routes() -> Scope {
        web::scope("/credentials").service(interop::get_credential_as_vc)
    }

    fn transaction_routes() -> Scope {
        web::scope("")
            .service(transactions::create_transaction)
            .service(transactions::get_mempool)
    }

    fn proposal_routes() -> Scope {
        web::scope("")
            .service(proposals::submit_proposal)
            .service(proposals::submit_endorsed_transaction)
    }

    fn channels_routes() -> Scope {
        web::scope("")
            .service(channels::create_channel)
            .service(channels::list_channels)
            .service(channels::update_channel_config)
            .service(channels::get_channel_config)
            .service(channels::get_channel_config_history)
    }

    fn msp_routes() -> Scope {
        web::scope("")
            .service(msp::revoke_serial)
            .service(msp::get_msp_info)
    }

    fn private_data_routes() -> Scope {
        web::scope("")
            .service(private_data::put_private_data)
            .service(private_data::get_private_data)
    }

    fn chaincode_routes() -> Scope {
        web::scope("")
            .service(chaincode::install_chaincode)
            .service(chaincode::approve_chaincode)
            .service(chaincode::commit_chaincode)
            .service(chaincode::simulate_chaincode)
    }

    fn gateway_routes() -> Scope {
        web::scope("").service(gateway::gateway_submit)
    }

    fn discovery_routes() -> Scope {
        web::scope("")
            .service(discovery::get_endorsers)
            .service(discovery::get_channel_peers)
            .service(discovery::post_register_peer)
    }

    fn events_routes() -> Scope {
        web::scope("")
            .service(events::events_blocks)
            .service(events::events_blocks_filtered)
            .service(events::events_blocks_private)
    }

    fn acl_routes() -> Scope {
        web::scope("")
            .service(acl::set_acl)
            .service(acl::list_acls)
            .service(acl::get_acl)
    }

    fn snapshot_routes() -> Scope {
        web::scope("")
            .service(snapshots::create_snapshot)
            .service(snapshots::list_snapshots)
            .service(snapshots::download_snapshot)
    }

    // utilities (health, version, openapi) registered as .route() in register()
}

#[cfg(test)]
mod tests {
    use super::ApiRoutes;

    #[test]
    fn test_routes_structure() {
        assert_eq!(std::mem::size_of::<ApiRoutes>(), 0);
    }
}
