//! In-memory BlockStore implementation for testing and development.

use std::collections::HashMap;
use std::sync::Mutex;

use super::errors::{StorageError, StorageResult};
use super::traits::{
    Acta, AliasEntry, Assembly, Block, BlockStore, ClaimStatus, Credential, IdentityRecord,
    InferenceClaim, Invitation, Scope, Session, Transaction,
};
use crate::registry::compliance::{ComplianceResult, ComplianceRule};
use crate::registry::tokenization::AssetToken;
use crate::registry::types::{Asset, AssetEvent};

/// In-memory store backed by HashMaps; safe for concurrent use via Mutex.
pub struct MemoryStore {
    blocks: Mutex<HashMap<u64, Block>>,
    transactions: Mutex<HashMap<String, Transaction>>,
    identities: Mutex<HashMap<String, IdentityRecord>>,
    credentials: Mutex<HashMap<String, Credential>>,
    latest_height: Mutex<u64>,
    /// Governance proposals: id → Proposal
    proposals: Mutex<HashMap<u64, crate::governance::proposals::Proposal>>,
    /// Governance votes: (proposal_id, voter) → Vote
    votes: Mutex<HashMap<(u64, String), crate::governance::voting::Vote>>,
    /// Vault: DID → encrypted wallet JSON (opaque blob)
    vault: Mutex<HashMap<String, serde_json::Value>>,
    /// Vault recovery: blind_index → DID
    vault_recovery: Mutex<HashMap<String, String>>,
    /// Scopes: id → Scope
    scopes: Mutex<HashMap<String, Scope>>,
    /// Assemblies: id → Assembly
    assemblies: Mutex<HashMap<String, Assembly>>,
    /// Sessions: id → Session
    sessions: Mutex<HashMap<String, Session>>,
    /// Actas: id → Acta
    actas: Mutex<HashMap<String, Acta>>,
    /// Assets: id → Asset
    assets: Mutex<HashMap<String, Asset>>,
    /// Asset events: id → AssetEvent
    asset_events: Mutex<HashMap<String, AssetEvent>>,
    /// Asset tokens: id → AssetToken
    asset_tokens: Mutex<HashMap<String, AssetToken>>,
    /// Compliance rules: id → ComplianceRule
    compliance_rules: Mutex<HashMap<String, ComplianceRule>>,
    /// Compliance results: id → ComplianceResult
    compliance_results: Mutex<HashMap<String, ComplianceResult>>,
    /// Alias registry: commitment → AliasEntry
    aliases: Mutex<HashMap<String, AliasEntry>>,
    /// Inference claims: claim_id → InferenceClaim
    inference_claims: Mutex<HashMap<String, InferenceClaim>>,
    /// Invitations: id → Invitation
    invitations: Mutex<HashMap<String, Invitation>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            blocks: Mutex::new(HashMap::new()),
            transactions: Mutex::new(HashMap::new()),
            identities: Mutex::new(HashMap::new()),
            credentials: Mutex::new(HashMap::new()),
            latest_height: Mutex::new(0),
            proposals: Mutex::new(HashMap::new()),
            votes: Mutex::new(HashMap::new()),
            vault: Mutex::new(HashMap::new()),
            vault_recovery: Mutex::new(HashMap::new()),
            scopes: Mutex::new(HashMap::new()),
            assemblies: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            actas: Mutex::new(HashMap::new()),
            assets: Mutex::new(HashMap::new()),
            asset_events: Mutex::new(HashMap::new()),
            asset_tokens: Mutex::new(HashMap::new()),
            compliance_rules: Mutex::new(HashMap::new()),
            compliance_results: Mutex::new(HashMap::new()),
            aliases: Mutex::new(HashMap::new()),
            inference_claims: Mutex::new(HashMap::new()),
            invitations: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockStore for MemoryStore {
    fn write_block(&self, block: &Block) -> StorageResult<()> {
        let mut blocks = self.blocks.lock().unwrap_or_else(|e| e.into_inner());
        // Reject overwrites of committed blocks — immutability guarantee
        if blocks.contains_key(&block.height) {
            return Err(crate::storage::errors::StorageError::Other(format!(
                "block at height {} already committed",
                block.height
            )));
        }
        let mut latest = self.latest_height.lock().unwrap_or_else(|e| e.into_inner());
        if block.height > *latest {
            *latest = block.height;
        }
        blocks.insert(block.height, block.clone());
        Ok(())
    }

    fn read_block(&self, height: u64) -> StorageResult<Block> {
        self.blocks
            .lock()
            .unwrap()
            .get(&height)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("BLK:{height:012}")))
    }

    fn write_transaction(&self, tx: &Transaction) -> StorageResult<()> {
        self.transactions
            .lock()
            .unwrap()
            .insert(tx.id.clone(), tx.clone());
        Ok(())
    }

    fn read_transaction(&self, tx_id: &str) -> StorageResult<Transaction> {
        self.transactions
            .lock()
            .unwrap()
            .get(tx_id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("TX:{tx_id}")))
    }

    fn write_identity(&self, identity: &IdentityRecord) -> StorageResult<()> {
        self.identities
            .lock()
            .unwrap()
            .insert(identity.did.clone(), identity.clone());
        Ok(())
    }

    fn read_identity(&self, did: &str) -> StorageResult<IdentityRecord> {
        self.identities
            .lock()
            .unwrap()
            .get(did)
            .cloned()
            .ok_or_else(|| StorageError::IdentityNotFound(did.to_string()))
    }

    fn list_identities(&self) -> StorageResult<Vec<IdentityRecord>> {
        Ok(self.identities.lock().unwrap().values().cloned().collect())
    }

    fn write_credential(&self, credential: &Credential) -> StorageResult<()> {
        self.credentials
            .lock()
            .unwrap()
            .insert(credential.id.clone(), credential.clone());
        Ok(())
    }

    fn read_credential(&self, cred_id: &str) -> StorageResult<Credential> {
        self.credentials
            .lock()
            .unwrap()
            .get(cred_id)
            .cloned()
            .ok_or_else(|| StorageError::CredentialNotFound(cred_id.to_string()))
    }

    fn list_credentials(&self) -> StorageResult<Vec<Credential>> {
        Ok(self.credentials.lock().unwrap().values().cloned().collect())
    }

    fn write_batch(&self, blocks: &[Block], txs: &[Transaction]) -> StorageResult<()> {
        if blocks.is_empty() && txs.is_empty() {
            return Err(StorageError::BatchOperationFailed(
                "Empty batch".to_string(),
            ));
        }
        for block in blocks {
            self.write_block(block)?;
        }
        for tx in txs {
            self.write_transaction(tx)?;
        }
        Ok(())
    }

    fn get_latest_height(&self) -> StorageResult<u64> {
        Ok(*self.latest_height.lock().unwrap_or_else(|e| e.into_inner()))
    }

    fn block_exists(&self, height: u64) -> StorageResult<bool> {
        Ok(self
            .blocks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(&height))
    }

    fn transactions_by_block_height(&self, height: u64) -> StorageResult<Vec<Transaction>> {
        let txs = self
            .transactions
            .lock()
            .unwrap()
            .values()
            .filter(|tx| tx.block_height == height)
            .cloned()
            .collect();
        Ok(txs)
    }

    fn credentials_by_subject_did(&self, subject_did: &str) -> StorageResult<Vec<Credential>> {
        let creds = self
            .credentials
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.subject_did == subject_did)
            .cloned()
            .collect();
        Ok(creds)
    }

    fn credentials_by_issuer_did(&self, issuer_did: &str) -> StorageResult<Vec<Credential>> {
        let creds = self
            .credentials
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.issuer_did == issuer_did)
            .cloned()
            .collect();
        Ok(creds)
    }

    // ── Governance persistence ─────────────────────────────────────────────

    fn write_proposal(
        &self,
        proposal: &crate::governance::proposals::Proposal,
    ) -> StorageResult<()> {
        self.proposals
            .lock()
            .unwrap()
            .insert(proposal.id, proposal.clone());
        Ok(())
    }

    fn read_proposal(&self, id: u64) -> StorageResult<crate::governance::proposals::Proposal> {
        self.proposals
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("PROPOSAL:{id:012}")))
    }

    fn list_proposals(&self) -> StorageResult<Vec<crate::governance::proposals::Proposal>> {
        let mut all: Vec<_> = self.proposals.lock().unwrap().values().cloned().collect();
        all.sort_by_key(|p| p.id);
        Ok(all)
    }

    fn write_vote(&self, vote: &crate::governance::voting::Vote) -> StorageResult<()> {
        self.votes
            .lock()
            .unwrap()
            .insert((vote.proposal_id, vote.voter.clone()), vote.clone());
        Ok(())
    }

    fn list_votes(&self, proposal_id: u64) -> StorageResult<Vec<crate::governance::voting::Vote>> {
        let all: Vec<_> = self
            .votes
            .lock()
            .unwrap()
            .values()
            .filter(|v| v.proposal_id == proposal_id)
            .cloned()
            .collect();
        Ok(all)
    }

    fn write_vault(&self, did: &str, encrypted_wallet: &serde_json::Value) -> StorageResult<()> {
        self.vault
            .lock()
            .unwrap()
            .insert(did.to_string(), encrypted_wallet.clone());
        Ok(())
    }

    fn read_vault(&self, did: &str) -> StorageResult<serde_json::Value> {
        self.vault
            .lock()
            .unwrap()
            .get(did)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("vault:{did}")))
    }

    fn write_vault_recovery(&self, blind_index: &str, did: &str) -> StorageResult<()> {
        self.vault_recovery
            .lock()
            .unwrap()
            .insert(blind_index.to_string(), did.to_string());
        Ok(())
    }

    fn read_vault_by_recovery(&self, blind_index: &str) -> StorageResult<String> {
        self.vault_recovery
            .lock()
            .unwrap()
            .get(blind_index)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound("vault recovery entry not found".into()))
    }

    // ── Governance entities ─────────────────────────────────────────────

    fn write_scope(&self, scope: &Scope) -> StorageResult<()> {
        self.scopes
            .lock()
            .unwrap()
            .insert(scope.id.clone(), scope.clone());
        Ok(())
    }
    fn read_scope(&self, id: &str) -> StorageResult<Scope> {
        self.scopes
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("scope:{id}")))
    }
    fn list_scopes(&self) -> StorageResult<Vec<Scope>> {
        Ok(self.scopes.lock().unwrap().values().cloned().collect())
    }
    fn delete_scope(&self, id: &str) -> StorageResult<()> {
        self.scopes.lock().unwrap().remove(id);
        Ok(())
    }

    fn write_assembly(&self, assembly: &Assembly) -> StorageResult<()> {
        self.assemblies
            .lock()
            .unwrap()
            .insert(assembly.id.clone(), assembly.clone());
        Ok(())
    }
    fn read_assembly(&self, id: &str) -> StorageResult<Assembly> {
        self.assemblies
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("assembly:{id}")))
    }
    fn list_assemblies(&self) -> StorageResult<Vec<Assembly>> {
        Ok(self.assemblies.lock().unwrap().values().cloned().collect())
    }
    fn list_assemblies_by_scope(&self, scope_id: &str) -> StorageResult<Vec<Assembly>> {
        Ok(self
            .assemblies
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.scope_id == scope_id)
            .cloned()
            .collect())
    }

    fn delete_assembly(&self, id: &str) -> StorageResult<()> {
        self.assemblies.lock().unwrap().remove(id);
        Ok(())
    }

    fn write_session(&self, session: &Session) -> StorageResult<()> {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session.clone());
        Ok(())
    }
    fn read_session(&self, id: &str) -> StorageResult<Session> {
        self.sessions
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("session:{id}")))
    }
    fn list_sessions_by_assembly(&self, assembly_id: &str) -> StorageResult<Vec<Session>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.assembly_id == assembly_id)
            .cloned()
            .collect())
    }

    fn delete_session(&self, id: &str) -> StorageResult<()> {
        self.sessions.lock().unwrap().remove(id);
        Ok(())
    }

    fn write_acta(&self, acta: &Acta) -> StorageResult<()> {
        self.actas
            .lock()
            .unwrap()
            .insert(acta.id.clone(), acta.clone());
        Ok(())
    }
    fn read_acta(&self, id: &str) -> StorageResult<Acta> {
        self.actas
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("acta:{id}")))
    }
    fn list_actas(&self) -> StorageResult<Vec<Acta>> {
        Ok(self.actas.lock().unwrap().values().cloned().collect())
    }
    fn delete_acta(&self, id: &str) -> StorageResult<()> {
        self.actas.lock().unwrap().remove(id);
        Ok(())
    }

    // ── Asset Registry ──────────────────────────────────────────────────

    fn write_asset(&self, asset: &Asset) -> StorageResult<()> {
        self.assets
            .lock()
            .unwrap()
            .insert(asset.id.clone(), asset.clone());
        Ok(())
    }
    fn read_asset(&self, id: &str) -> StorageResult<Asset> {
        self.assets
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("asset:{id}")))
    }
    fn list_assets(&self) -> StorageResult<Vec<Asset>> {
        Ok(self.assets.lock().unwrap().values().cloned().collect())
    }
    fn delete_asset(&self, id: &str) -> StorageResult<()> {
        self.assets.lock().unwrap().remove(id);
        Ok(())
    }

    fn write_asset_event(&self, event: &AssetEvent) -> StorageResult<()> {
        self.asset_events
            .lock()
            .unwrap()
            .insert(event.id.clone(), event.clone());
        Ok(())
    }
    fn read_asset_event(&self, id: &str) -> StorageResult<AssetEvent> {
        self.asset_events
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("asset_event:{id}")))
    }
    fn list_asset_events(&self, asset_id: &str) -> StorageResult<Vec<AssetEvent>> {
        Ok(self
            .asset_events
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.asset_id == asset_id)
            .cloned()
            .collect())
    }

    // ── RWA Tokenization ────────────────────────────────────────────────

    fn write_asset_token(&self, token: &AssetToken) -> StorageResult<()> {
        self.asset_tokens
            .lock()
            .unwrap()
            .insert(token.id.clone(), token.clone());
        Ok(())
    }
    fn read_asset_token(&self, id: &str) -> StorageResult<AssetToken> {
        self.asset_tokens
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("token:{id}")))
    }
    fn list_asset_tokens(&self) -> StorageResult<Vec<AssetToken>> {
        Ok(self
            .asset_tokens
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    fn delete_asset_token(&self, id: &str) -> StorageResult<()> {
        self.asset_tokens.lock().unwrap().remove(id);
        Ok(())
    }

    // ── Compliance Automation ───────────────────────────────────────────

    fn write_compliance_rule(&self, rule: &ComplianceRule) -> StorageResult<()> {
        self.compliance_rules
            .lock()
            .unwrap()
            .insert(rule.id.clone(), rule.clone());
        Ok(())
    }
    fn read_compliance_rule(&self, id: &str) -> StorageResult<ComplianceRule> {
        self.compliance_rules
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("rule:{id}")))
    }
    fn list_compliance_rules(&self) -> StorageResult<Vec<ComplianceRule>> {
        Ok(self
            .compliance_rules
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    fn delete_compliance_rule(&self, id: &str) -> StorageResult<()> {
        self.compliance_rules.lock().unwrap().remove(id);
        Ok(())
    }

    fn write_compliance_result(&self, result: &ComplianceResult) -> StorageResult<()> {
        self.compliance_results
            .lock()
            .unwrap()
            .insert(result.id.clone(), result.clone());
        Ok(())
    }
    fn list_compliance_results(&self, asset_id: &str) -> StorageResult<Vec<ComplianceResult>> {
        Ok(self
            .compliance_results
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.asset_id == asset_id)
            .cloned()
            .collect())
    }

    // ── Alias Registry ──────────────────────────────────────────────────

    fn write_alias(&self, entry: &AliasEntry) -> StorageResult<()> {
        self.aliases
            .lock()
            .unwrap()
            .insert(entry.commitment.clone(), entry.clone());
        Ok(())
    }

    fn read_alias(&self, commitment: &str) -> StorageResult<AliasEntry> {
        self.aliases
            .lock()
            .unwrap()
            .get(commitment)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("alias:{commitment}")))
    }

    fn read_alias_by_did(&self, did: &str) -> StorageResult<AliasEntry> {
        self.aliases
            .lock()
            .unwrap()
            .values()
            .find(|e| e.did == did && e.status == "active")
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("alias for DID:{did}")))
    }

    fn delete_alias(&self, commitment: &str) -> StorageResult<()> {
        self.aliases.lock().unwrap().remove(commitment);
        Ok(())
    }

    // ── Inference Claims ────────────────────────────────────────────────

    fn write_inference_claim(&self, claim: &InferenceClaim) -> StorageResult<()> {
        self.inference_claims
            .lock()
            .unwrap()
            .insert(claim.id.clone(), claim.clone());
        Ok(())
    }

    fn read_inference_claim(&self, id: &str) -> StorageResult<InferenceClaim> {
        self.inference_claims
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("inference_claim:{id}")))
    }

    fn list_inference_claims(
        &self,
        status: Option<&ClaimStatus>,
        oracle_id: Option<&str>,
        model_hash: Option<&str>,
    ) -> StorageResult<Vec<InferenceClaim>> {
        let claims = self.inference_claims.lock().unwrap();
        let results = claims
            .values()
            .filter(|c| status.is_none_or(|s| &c.status == s))
            .filter(|c| oracle_id.is_none_or(|o| c.oracle_id == o))
            .filter(|c| model_hash.is_none_or(|m| c.model_hash == m))
            .cloned()
            .collect();
        Ok(results)
    }

    // ── Invitations ─────────────────────────────────────────────────────

    fn write_invitation(&self, invitation: &Invitation) -> StorageResult<()> {
        self.invitations
            .lock()
            .unwrap()
            .insert(invitation.id.clone(), invitation.clone());
        Ok(())
    }

    fn read_invitation(&self, id: &str) -> StorageResult<Invitation> {
        self.invitations
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| StorageError::KeyNotFound(format!("invitation:{id}")))
    }

    fn list_invitations_by_commitment(
        &self,
        to_commitment: &str,
    ) -> StorageResult<Vec<Invitation>> {
        Ok(self
            .invitations
            .lock()
            .unwrap()
            .values()
            .filter(|i| i.to_commitment == to_commitment)
            .cloned()
            .collect())
    }

    fn list_invitations_by_sender(&self, from_did: &str) -> StorageResult<Vec<Invitation>> {
        Ok(self
            .invitations
            .lock()
            .unwrap()
            .values()
            .filter(|i| i.from_did == from_did)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_block(height: u64) -> Block {
        Block {
            height,
            timestamp: 1_000 + height,
            parent_hash: [0u8; 32],
            merkle_root: [1u8; 32],
            transactions: vec![format!("tx-{}", height)],
            proposer: "node-1".to_string(),
            signature: vec![2u8; 64],
            signature_algorithm: Default::default(),
            endorsements: vec![],
            secondary_signature: None,
            secondary_signature_algorithm: None,
            hash_algorithm: Default::default(),
            orderer_signature: None,
        }
    }

    fn sample_tx(id: &str, height: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            block_height: height,
            timestamp: 1_000,
            input_did: "did:bc:sender".to_string(),
            output_recipient: "did:bc:receiver".to_string(),
            amount: 42,
            state: "confirmed".to_string(),
        }
    }

    #[test]
    fn write_and_read_block() {
        let store = MemoryStore::new();
        let block = sample_block(1);
        store.write_block(&block).unwrap();
        let fetched = store.read_block(1).unwrap();
        assert_eq!(fetched.height, 1);
        assert_eq!(fetched.proposer, "node-1");
    }

    #[test]
    fn read_missing_block_returns_error() {
        let store = MemoryStore::new();
        assert!(store.read_block(99).is_err());
    }

    #[test]
    fn latest_height_tracks_max() {
        let store = MemoryStore::new();
        assert_eq!(store.get_latest_height().unwrap(), 0);
        store.write_block(&sample_block(3)).unwrap();
        store.write_block(&sample_block(1)).unwrap();
        store.write_block(&sample_block(5)).unwrap();
        assert_eq!(store.get_latest_height().unwrap(), 5);
    }

    #[test]
    fn block_exists_returns_correct_result() {
        let store = MemoryStore::new();
        assert!(!store.block_exists(1).unwrap());
        store.write_block(&sample_block(1)).unwrap();
        assert!(store.block_exists(1).unwrap());
    }

    #[test]
    fn write_and_read_transaction() {
        let store = MemoryStore::new();
        let tx = sample_tx("tx-abc", 1);
        store.write_transaction(&tx).unwrap();
        let fetched = store.read_transaction("tx-abc").unwrap();
        assert_eq!(fetched.amount, 42);
    }

    #[test]
    fn read_missing_transaction_returns_error() {
        let store = MemoryStore::new();
        assert!(store.read_transaction("ghost").is_err());
    }

    #[test]
    fn write_and_read_identity() {
        let store = MemoryStore::new();
        let id = IdentityRecord {
            did: "did:bc:alice".to_string(),
            created_at: 100,
            updated_at: 200,
            status: "active".to_string(),
        };
        store.write_identity(&id).unwrap();
        let fetched = store.read_identity("did:bc:alice").unwrap();
        assert_eq!(fetched.status, "active");
    }

    #[test]
    fn read_missing_identity_returns_error() {
        let store = MemoryStore::new();
        assert!(store.read_identity("did:bc:ghost").is_err());
    }

    #[test]
    fn write_and_read_credential() {
        let store = MemoryStore::new();
        let cred = Credential {
            id: "cred-1".to_string(),
            issuer_did: "did:bc:issuer".to_string(),
            subject_did: "did:bc:subject".to_string(),
            cred_type: "eid".to_string(),
            issued_at: 100,
            expires_at: 999,
            revoked_at: None,
            ..Default::default()
        };
        store.write_credential(&cred).unwrap();
        let fetched = store.read_credential("cred-1").unwrap();
        assert_eq!(fetched.cred_type, "eid");
        assert!(fetched.revoked_at.is_none());
    }

    #[test]
    fn read_missing_credential_returns_error() {
        let store = MemoryStore::new();
        assert!(store.read_credential("ghost").is_err());
    }

    #[test]
    fn write_batch_succeeds() {
        let store = MemoryStore::new();
        let blocks = vec![sample_block(1), sample_block(2)];
        let txs = vec![sample_tx("tx-1", 1), sample_tx("tx-2", 2)];
        store.write_batch(&blocks, &txs).unwrap();
        assert_eq!(store.get_latest_height().unwrap(), 2);
        assert!(store.read_transaction("tx-2").is_ok());
    }

    #[test]
    fn write_batch_empty_returns_error() {
        let store = MemoryStore::new();
        assert!(store.write_batch(&[], &[]).is_err());
    }

    // ── Secondary index: transactions_by_block_height ─────────────────────────

    #[test]
    fn transactions_by_block_height_returns_empty_for_unknown_height() {
        let store = MemoryStore::new();
        assert!(store.transactions_by_block_height(99).unwrap().is_empty());
    }

    #[test]
    fn transactions_by_block_height_filters_correctly() {
        let store = MemoryStore::new();
        store.write_transaction(&sample_tx("tx-1", 3)).unwrap();
        store.write_transaction(&sample_tx("tx-2", 3)).unwrap();
        store.write_transaction(&sample_tx("tx-3", 4)).unwrap();

        let result = store.transactions_by_block_height(3).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.block_height == 3));
    }

    #[test]
    fn transactions_by_block_height_returns_all_for_height() {
        let store = MemoryStore::new();
        for i in 0..5u64 {
            store
                .write_transaction(&sample_tx(&format!("tx-{i}"), 10))
                .unwrap();
        }
        assert_eq!(store.transactions_by_block_height(10).unwrap().len(), 5);
    }

    // ── Secondary index: credentials_by_subject_did ───────────────────────────

    fn sample_cred(id: &str, subject_did: &str) -> Credential {
        Credential {
            id: id.to_string(),
            issuer_did: "did:bc:issuer".to_string(),
            subject_did: subject_did.to_string(),
            cred_type: "eid".to_string(),
            issued_at: 1_000,
            expires_at: 9_999,
            ..Default::default()
        }
    }

    #[test]
    fn credentials_by_subject_did_returns_empty_for_unknown_subject() {
        let store = MemoryStore::new();
        assert!(store
            .credentials_by_subject_did("did:bc:ghost")
            .unwrap()
            .is_empty());
    }

    // ── Governance persistence ────────────────────────────────────────────────

    fn sample_proposal(id: u64) -> crate::governance::proposals::Proposal {
        crate::governance::proposals::Proposal {
            id,
            proposer: "alice".to_string(),
            action: crate::governance::proposals::ProposalAction::TextProposal {
                title: "Test".to_string(),
                description: "A test proposal".to_string(),
            },
            status: crate::governance::proposals::ProposalStatus::Voting,
            deposit: 100,
            submitted_at: 10,
            voting_ends_at: 110,
            timelock_ends_at: None,
            finalized_at: None,
            description: "test".to_string(),
        }
    }

    fn sample_vote(proposal_id: u64, voter: &str) -> crate::governance::voting::Vote {
        crate::governance::voting::Vote {
            voter: voter.to_string(),
            proposal_id,
            option: crate::governance::voting::VoteOption::Yes,
            power: 1000,
            voted_at: 50,
        }
    }

    #[test]
    fn write_and_read_proposal() {
        let store = MemoryStore::new();
        let p = sample_proposal(1);
        store.write_proposal(&p).unwrap();
        let fetched = store.read_proposal(1).unwrap();
        assert_eq!(fetched.proposer, "alice");
    }

    #[test]
    fn read_missing_proposal_returns_error() {
        let store = MemoryStore::new();
        assert!(store.read_proposal(999).is_err());
    }

    #[test]
    fn list_proposals_sorted_by_id() {
        let store = MemoryStore::new();
        store.write_proposal(&sample_proposal(3)).unwrap();
        store.write_proposal(&sample_proposal(1)).unwrap();
        store.write_proposal(&sample_proposal(2)).unwrap();
        let all = store.list_proposals().unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, 1);
        assert_eq!(all[2].id, 3);
    }

    #[test]
    fn write_and_list_votes() {
        let store = MemoryStore::new();
        store.write_vote(&sample_vote(1, "alice")).unwrap();
        store.write_vote(&sample_vote(1, "bob")).unwrap();
        store.write_vote(&sample_vote(2, "alice")).unwrap();
        let votes = store.list_votes(1).unwrap();
        assert_eq!(votes.len(), 2);
        assert!(votes.iter().all(|v| v.proposal_id == 1));
    }

    #[test]
    fn list_votes_empty_for_unknown_proposal() {
        let store = MemoryStore::new();
        assert!(store.list_votes(999).unwrap().is_empty());
    }

    #[test]
    fn proposal_update_overwrites() {
        let store = MemoryStore::new();
        let mut p = sample_proposal(1);
        store.write_proposal(&p).unwrap();
        p.status = crate::governance::proposals::ProposalStatus::Passed;
        store.write_proposal(&p).unwrap();
        let fetched = store.read_proposal(1).unwrap();
        assert_eq!(
            fetched.status,
            crate::governance::proposals::ProposalStatus::Passed
        );
    }

    #[test]
    fn credentials_by_subject_did_filters_correctly() {
        let store = MemoryStore::new();
        store
            .write_credential(&sample_cred("cred-1", "did:bc:alice"))
            .unwrap();
        store
            .write_credential(&sample_cred("cred-2", "did:bc:alice"))
            .unwrap();
        store
            .write_credential(&sample_cred("cred-3", "did:bc:bob"))
            .unwrap();

        let alice = store.credentials_by_subject_did("did:bc:alice").unwrap();
        assert_eq!(alice.len(), 2);
        assert!(alice.iter().all(|c| c.subject_did == "did:bc:alice"));

        let bob = store.credentials_by_subject_did("did:bc:bob").unwrap();
        assert_eq!(bob.len(), 1);
        assert_eq!(bob[0].id, "cred-3");
    }
}
