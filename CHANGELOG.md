# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) · Versioning: [SemVer](https://semver.org)

---

## [Unreleased]

### 2026-05-22

**Context-prefixed signing payloads for alias endpoints**

- Register signs `"alias:register:{commitment}"`, revoke signs `"alias:revoke:{commitment}"`
- Prevents cross-endpoint replay (previously both signed the raw commitment)
- **Breaking:** wallets must update their signing payload format

**Fast deploy pipeline**

- `scripts/deploy.sh` — single command: cross-compile → scp → docker rebuild on EC2
- sccache integrated in `Cross.toml` for cached cross-compilation (~6s incremental vs ~12min cold)
- Docker build now uses layer cache (no `--no-cache`), rebuilds in ~1s

**Alias reverse lookup by DID**

- `GET /alias/by-did/{did}` — resolve a DID to its active alias (commitment, encrypted alias, registration date)
- Returns 404 if no alias exists, 410 if revoked
- Uses existing `read_alias_by_did()` storage method (RocksDB `did:` secondary index)

---

### 2026-05-21

**Alias registry — zero-knowledge alias system**

- `POST /alias/register` — register alias commitment (SHA3-256) with Ed25519 signature verification
- `POST /alias/resolve` — resolve commitment to DID + address; returns 410 if revoked
- `POST /alias/revoke` — owner-only revocation with 15-day cooldown before re-registration
- One alias per DID; duplicate commitment returns 409
- `AliasEntry` type: commitment, DID, salt, encrypted alias, status, timestamps
- Storage: `aliases` column family (RocksDB) with `did:` secondary index for reverse lookup

**Invitation system — governance proposal invitations via alias**

- `POST /governance/invitations` — create invitation linked to alias commitment
- `GET /governance/invitations?voter={commitment}` — list pending invitations
- `POST /governance/invitations/respond` — accept or reject with Ed25519 signature
- Rate limit: max 5 invitations per hour per DID, max 20 proposals per invitation
- `Invitation` type with response tracking (responded, accepted)
- Storage: `invitations` column family (RocksDB)

**Deploy pipeline — local cross-compilation**

- `Cross.toml` added for macOS → Linux amd64 cross-compilation
- `reqwest` switched to `rustls-tls` (eliminates dynamic `libssl` dependency)
- Deploy flow: cross-compile locally → scp binary → `Dockerfile.prebuilt` rebuild (~3 min)

**Repo cleanup**

- Removed 7 obsolete scripts (legacy SQLite architecture)
- Removed 12 disabled CI workflow files
- Removed `docs/book/` (empty mdBook stubs), duplicate PDFs, redundant API reference
- Removed `Dockerfile.static`, `docker-compose.yml`, `docker-compose.deploy.yml` (unused)
- Archived `docs/camara/`, `docs/es/`, architecture comparisons, client PDFs → `docs/archive/`
- Moved `ALIAS_API.md`, `ALIAS_DESIGN.md` → `docs/architecture/`

---

### 2026-05-19

**Vault recovery via passphrase-derived key (NIST SP 800-185)**

- `POST /vault/store` now accepts optional `recovery_key` (64 hex chars, client-derived via KDF)
- `POST /vault/recover` — recover wallet by recovery key; returns DID + encrypted wallet
- Server-side blind index via HMAC-SHA3-256 prevents brute-force on DB leak
- `VAULT_RECOVERY_SECRET` env var enables recovery (disabled when absent)
- `hmac_sha3_256()` added to `pqc_crypto_module` public API
- Storage trait: `write_vault_recovery()` / `read_vault_by_recovery()` with MemoryStore and RocksDB implementations
- Recovery key validated as exactly 32 bytes hex; wrong key returns 404

**Release distribution pipeline**

- `scripts/install-cerulean.sh` — one-line installer for Linux servers (systemd service, zero dependencies)
- `scripts/build-release.sh` — disposable EC2 spot build (~$0.03 per release)
- `Dockerfile.prebuilt` — runtime-only image from precompiled binary (no compilation)
- Binaries hosted at `s3://ceruleanledger-releases/`
- Production downsized from t3.medium to t3.small ($15/mo)

---

### 2026-05-17

**Legacy storage removal — API layer fully migrated to BlockStore**

New modules:
- `src/mining.rs` — MiningService backed by BlockStore (replaces Blockchain::mine_block_with_reward)
- `src/transaction/mempool.rs` — TransactionPool using storage::Transaction (replaces models::Mempool)
- `src/storage/compat.rs` — From<> conversions between legacy and new types
- `src/api/handlers/governance_entities.rs` — 14 CRUD endpoints for scopes, assemblies, sessions, actas
- `BlockStore::calculate_balance()`, `transactions_for_address()` — default impls for handler migration

Legacy files removed:
- `state_reconstructor.rs`, `state_snapshot.rs`, `pruning.rs`, `block_storage.rs`
- `fee_validation_test.rs`, `integration_test.rs` (tested legacy Blockchain directly)

Handlers migrated to BlockStore:
- `mine_block` — uses MiningService exclusively
- `get_wallet_balance`, `get_wallet_transactions` — read from BlockStore
- `get_stats` — reads chain data from BlockStore
- `create_transaction` — validates and enqueues via TransactionPool
- `stake`, `unstake`, `claim_airdrop` — create storage::Transaction directly
- `list_blocks`, `get_block_by_index` — read from BlockStore
- `verify_chain`, `get_blockchain_info` — read from BlockStore
- `health_check`, `get_version` — read height from BlockStore

Legacy types (`blockchain.rs`, `models.rs`) are now `pub(crate)` — not exported, only used internally by `network/mod.rs` for P2P sync (not exercised in single-node mode).

RocksDB column families added: `scopes`, `assemblies`, `sessions`, `actas`.

Governance entity CRUD (20 endpoints):
- Scopes, assemblies, sessions, actas: POST, GET, GET/{id}, PUT/{id}, DELETE/{id}

Security fixes:
- Blocks signed with node's SigningProvider (Ed25519/ML-DSA-65)
- `POST /transactions` rejects unsigned non-coinbase transactions
- TransactionPool double-spend prevention via `add_checked()` (validates pending spend vs balance)
- Blind voter ID nonce: `sha256(proposal_id || voter_did || client_nonce)` prevents brute-force deanonymization

Enterprise core modules (`src/registry/`):
- Asset Registry: 10 endpoints — asset CRUD, event history, bulk telemetry ingestion, node-signed certified export
- RWA Tokenization: 5 endpoints — asset tokens with collateral lifecycle (Free/Pledged/Released/Seized)
- Compliance Automation: 6 endpoints — configurable rules, bulk evaluation against asset events, result history

CI workflows disabled (renamed to `.yml.disabled`).

---

### 2026-05-16

**Repo cleanup and v0.1.0 release**

- Extracted frontends and SDKs to dedicated repos
- Removed 60 obsolete scripts, 9 test binaries, 160+ stale docs
- Removed `num_cpus` dependency, dead functions, `tesseract/` prototype
- Slimmed `CLAUDE.md` from 385 to 91 lines
- Rewrote `README.md` with Cerulean Ledger branding
- Tagged `v0.1.0`

---

## [0.1.0] — 2026-05-16

### 2026-05-16

**Monorepo cleanup — extract frontends and SDKs to dedicated repos**

- Removed `block-explorer-vite/`, `cerulean-voto/`, `sdks/` from monorepo
- Each now lives in its own GitHub repo: [cerulean-explorer](https://github.com/clementeaf/cerulean-explorer), [cerulean-voto](https://github.com/clementeaf/cerulean-voto), [cerulean-sdks](https://github.com/clementeaf/cerulean-sdks)
- Added extracted directories to `.gitignore` to prevent residual artifacts
- Updated `CLAUDE.md` to reference dedicated repos instead of local paths

---

### 2026-05-15

**Cerulean Voto — Multi-tenant platform with wallet integration and interoperability**

Vault-backed wallet portability:
- Wallets stored on-chain via `POST /vault/store`, retrieved with `GET /vault/{did}`
- Cross-app: wallet created in Voto is available in Wallet app (and vice versa)
- localStorage is a cache — vault on-chain is the source of truth
- Import existing wallet by DID from any Cerulean app

Chrome extension support:
- Detects `window.cerulean` injected by Cerulean Wallet extension
- One-click connect: gets address + publicKey, registers DID, no passphrase needed
- Setup step 1: three tabs (Extension / Tengo DID / Crear nueva)

Wallet-name separation:
- Wallet = pure crypto (DID + keypair), no name
- Name = padron label, assigned by admin when adding to organization
- DID derived from address (`sha256(pubkey)[0..20]`), not raw public key slice

Dynamic scopes (multi-tenant organizational structure):
- Generic `Scope` model: name, label (user-defined type), parent, channel, members
- No imposed hierarchy — each institution defines its own structure
- Members with roles: admin, voter, observer per scope
- Each scope = own DLT channel with isolated data
- Tree-propagated permissions: founder = root admin, admin role inherits downward

Setup wizard (5-step guided onboarding):
- Step 1: Create wallet or connect existing (extension / DID import / new)
- Step 2: Organization (name, RUT, president, secretary → auto-creates DLT channel)
- Step 3: Inscribe participants by address/DID (admin doesn't create wallets for others)
- Step 4: Organizational structure (optional scopes with DLT channels)
- Step 5: First election (optional)

Governance persistence:
- `write_proposal`/`write_vote` added to BlockStore trait
- MemoryStore and RocksDB implementations
- Migration v2→v3: proposals and votes column families

Vault endpoints:
- `POST /api/v1/vault/store` — store encrypted wallet blob by DID
- `GET /api/v1/vault/{did}` — retrieve encrypted wallet backup
- RocksDB vault column family for persistent storage

Voter history endpoint:
- `GET /api/v1/governance/votes?voter={did}` — all proposals where voter participated
- Searches by raw DID and blind voter ID (for signed votes)

UI/UX overhaul:
- Voters: expandable rows with QR code, DID, address, public key, wallet download
- Vote: receipt with DID, payload SHA-256, Ed25519 signature, 5 animated guarantees
- Actas: green "Anclada" / amber "Pendiente" badge, DID on-chain link
- Admin: interop cards (DID Resolution, VC, JSON-LD, OpenAPI), DLT channel management
- Scopes: tree view with role badges, permission-gated member management
- Layout: active scope indicator in header bar

---

**Cerulean Voto — Real wallet, vote secrecy, acta anchoring, W3C interoperability**

Wallet integration (cerulean-wallet WASM):
- Ed25519 wallet generation via WASM (Argon2id + AES-256-GCM, same crypto as CLI)
- DID derived from public key: `did:cerulean:{sha256(pk)[0..20]}`
- Vote signing with passphrase-protected private key
- Backend verifies Ed25519 signature + DID-to-pubkey binding before accepting vote

Vote secrecy:
- Blind voter ID: `sha256(proposal_id || voter_did)` stored instead of real identity
- Cross-proposal unlinkability, deduplication preserved
- Unsigned votes (legacy/permissive) unchanged

Acta blockchain anchoring:
- SHA-256 hash stored as `did:cerulean:acta:{folio}` on session close
- Acta UI shows trace_id and anchoring status

Voter registry:
- Padron persisted in localStorage with real wallet files
- Vote page validates against registered padron (dropdown, not free text)
- Agenda items linkable to governance proposals

W3C Interoperability:
- `GET /did/{did}` — DID Resolution (did-core), Ed25519VerificationKey2020
- `GET /credentials/{id}/vc` — Verifiable Credential (VC Data Model 2.0)
- `GET /governance/proposals/{id}/export` — JSON-LD (schema.org VoteAction)
- OpenAPI 3.0.3: 66 paths, governance + interop endpoints documented
- Content types: `application/did+ld+json`, `application/vc+ld+json`, `application/ld+json`

Cleanup:
- Removed unused `PageIntro` component, 5 dead types, 2 unused API functions
- 1712 tests pass

---

### 2026-05-14

**Cerulean Voto — Asambleas, Sesiones, Actas, Administracion**

- 4 new modules: Asambleas, Sesiones, Actas, Administracion
- localStorage-backed store (`store.ts`) with correlative numbering
- Sidebar grouped into 3 sections: Votacion, Organizacion, Administracion
- Ley 19.418 Art. 16: convocatoria date/method, deadline validation (5d ordinaria, 3d extraordinaria)
- Ley 19.418 Art. 16: first/second citation quorum, quorum validation with legal warning
- Ley 19.418 Art. 17: actas with folio, org identification (name + RUT), president/secretary signatures
- ISO 15489: actas are permanent records (no delete), SHA-256 integrity hash per acta
- ISO 8601: all dates and timestamps in standard format
- Auto-generated acta on session close with all legally required fields
- Admin panel: org settings, signatory management, quorum config, normativa reference, export/import backup
- Schema migration support: `read()` merges with defaults for backward compatibility

---

### 2026-05-13

**Production deployment — S3/CloudFront + EC2 backend**

- Frontend hosting migrated to S3 + CloudFront (Explorer + Voto)
- Domain `ceruleanledger.com` registered with ACM SSL certificate
- Explorer: `https://ceruleanledger.com`
- Voto: `https://voto.ceruleanledger.com`
- API proxied through CloudFront `/api/*` → EC2 node
- EC2 runs only node + Prometheus + Grafana (t3.medium, us-east-1)

**Deploy infrastructure**

- `docker-compose.deploy.yml`: Caddy TLS, strict ACL, parameterized secrets
- `deploy/Caddyfile`: reverse proxy with security headers
- `.env.deploy.example`: secrets template (JWT, HMAC, Grafana password)
- `.gitignore`: added `.env` files and Caddy data directories
- Seed script accepts `API_URL` env var for remote seeding
- Dockerfile healthcheck supports both HTTP and HTTPS modes

**CI fixes**

- Added `protoc` to performance-guardrails, security-audit, nightly-chaos, pre-lab-audit workflows
- `cargo audit --ignore-yanked` for transitive yanked dependencies (keccak via revm)
- Fixed `signature_algorithm` missing field in `full_benchmark.rs`, `tps_benchmark.rs`, `ordering_throughput.rs`

**Benchmark report** (`docs/architecture/benchmarks/SANDBOX-BENCHMARK-REPORT.md`)

- 10/10 stress modules passed (crypto: 6.6M ops/s, storage: 48K ops/s)
- 40 pentest scenarios: 37 blocked, 3 detected, 0 vulnerable
- Health latency: 20ms avg

---

### 2026-05-12

**Institutional UI overhaul — All modules redesigned for flagship demo**

- Governance: full-width proposal table + slide-in drawer with tally bar, vote buttons, crypto proof
- Credentials: full-width document table + drawer with status banner, content, crypto proof
- Compliance (Audit Trail): category-based filters (Identity, Documents, Governance, Blockchain, Errors), human-readable Spanish labels, domain events by default, sticky thead with internal scroll, drawer detail with ISO 27001 compliance info
- System Health: auto-loading dashboard (replaces empty chaincode lookup) — overview cards, infrastructure checks, regulatory compliance bar, stress performance grid
- Sidebar: "Compliance" → "Cumplimiento", "Audit Trail" → "Registro de Operaciones", "Chaincode Health" → "Salud del Sistema"
- Global smooth transitions: 150ms cubic-bezier on all elements, fade-in animation on panel selection, backdrop fade on drawers
- Backend: `list_credentials()` added to BlockStore trait, MemoryStore, RocksDbBlockStore; `GET /api/v1/store/credentials` endpoint
- API client: `listCredentials()` function

---

**Digital Identity module — Document signing with legal validity**

- `/identity` rewritten as institutional digital identity module: identities list + signed documents panel + document detail drawer
- Documents presented as contracts, titles, certificates — not technical credentials
- Each document shows: signer, date, status (Vigente/Expirado/Revocado), content, and cryptographic proof (ML-DSA-65, SHA-256, blockchain seal)
- Backend: `list_identities()` added to `BlockStore` trait, `MemoryStore`, and `RocksDbBlockStore`; `GET /api/v1/store/identities` endpoint
- `/demo` (RRHH verification) redesigned: horizontal step indicator, single active card, compact layout without scrollbar
- Layout: sidebar groups Red and Tokens hidden for institutional focus
- API client: `listIdentities()` function, `Credential` type extended with `claims` and `status` fields

---

**Identity & Governance Hardening — Zero-panic, bounded, validated**

- Governance handlers: 6 `unwrap()` replaced with proper 503 error responses (zero panic paths)
- Tally arithmetic: `saturating_add`/`saturating_mul` on all vote power accumulation (overflow-safe)
- Voting period: `saturating_add` on `voting_ends_at` calculation
- Identity: hex decode fallback removed — invalid hex returns 400 instead of silent bypass
- Input limits: proposer/voter/delegate max 256B, description max 4KB, title max 256B, param changes max 50 entries
- Empty string rejection: voter, proposer, delegator, delegate, veto caller all validated non-empty
- Semantic param validation: quorum/threshold 1-100, voting_period > 0
- Issuer DID existence check before credential issuance (both `issue_credential` and `store_write_credential`)
- Stress tests: `stress_identity` (DID write+read cycle) and `stress_credential` (credential write+read cycle) added — 10 modules total
- Pentest suite: 33 → 40 scenarios. New: oversized description, empty voter, quorum=0, credential without issuer, vote spam (1000 votes), delegation cycle, signature bypass via invalid hex. All BLOCKED.
- Tests: 1687 passed, 0 critical vulnerabilities

---

**Institutional Integrity Dashboard — Flagship monitoring panel**

- New page `/integridad` — real-time platform integrity dashboard for institutional stakeholders
- 8 horizontal service cards: Security (pentest), Forensic (hash chain), Compliance (21 regulatory checks), Cryptography (PQC/KAT), Storage (RocksDB), Consensus (BFT/validators), Oracles (feed freshness), Intelligence (AML/risk)
- Each card opens a slide-in drawer with full service detail (scenarios, checks, metrics, validators)
- Tabbed right panel: integrity report table + security events timeline
- Stress test performance grid (8 modules: ops/s, p99 latency, status)
- Vertical control cards linking to Credentials, Governance, Finance, Contracts subsystems
- Print CSS for PDF export of integrity reports
- API client: 10 new typed functions (health, version, regulatory, pentest, stress, forensic, oracle status)
- Layout: `h-screen` fixed layout, sidebar sticky with internal scroll, no page-level scrollbar
- Sidebar: new "Integridad" group as primary navigation item
- Zero new dependencies, zero backend changes — all data from existing 44 monitoring endpoints

---

**Production Readiness — Load testing scripts and audit engagement package**

- `tools/k6/load-test.js` — k6 load test suite: 4 scenarios (default, spike, soak, stress), 8 weighted API actions (health, stats, blocks, mempool, audit, identity, wallet+mine), p95 < 500ms and error rate < 1% thresholds, JSON reports
- `docs/SECURITY-AUDIT-ENGAGEMENT.md` — complete engagement package for external auditor: scope (10 components prioritized), architecture summary, crypto inventory (8 algorithms), 6 known risks, internal security work summary, key files for review, environment setup

**Schema Migrations — Versioned RocksDB upgrades on startup**

- `src/storage/migrations.rs` — migration system: schema version in `meta` CF, migration registry with `(version, fn)` entries, runs pending on startup
- Fresh databases stamped at `LATEST_VERSION` (v2) without running migrations
- Existing databases migrate incrementally (v1→v2 verifies new CFs exist)
- Idempotent: crash mid-migration re-runs same step on next startup
- Wired in `main.rs` — migrations run before serving requests, fatal on failure
- 6 unit tests: fresh DB, v1→v2, idempotent, version survives reopen, sorted order

**Graceful Shutdown — Real WAL flush and full task cleanup**

- `RocksDbBlockStore::flush_wal()` — flushes WAL + memtables to SST files before exit
- Shutdown handler now aborts 5 background tasks (was 3): added pull_sync, antientropy
- Per-phase shutdown logging with elapsed time for operator visibility
- 2 unit tests: flush on empty DB, flush persists writes across reopen

**Consolidation — AppState test builder, cross-subsystem tests, legacy deprecation docs**

- `AppState::test_default()` — single constructor with all memory-backed defaults for tests. Replaces ~50-line duplicated constructors across 6 handler test files + unused imports cleaned up.
- `tests/cross_subsystem.rs` — 7 integration tests crossing module boundaries: identity→credential→audit, chaincode→sandbox→audit, legal oracle→audit, credential→ZKP verification, credential revocation→ZKP invalidation, multi-org audit isolation
- Legacy storage (`src/blockchain.rs`) documented as DEPRECATED with migration path in CLAUDE.md. 17 production refs remain in `api_legacy.rs` — both systems operate independently without conflicts.

**Consolidation — Stubs replaced, honest labeling, real HTTP, RocksDB persistence**

- Identity handlers: `create_identity` generates real DID + Ed25519 keypair + persists to store; `get_identity` reads from store; `rotate_key` updates record; `verify_signature` performs real Ed25519 verification
- Credential handlers: `issue_credential` persists to store; `get_credential` reads from store; `verify_credential` checks status + expiry + revocation; `revoke_credential` updates status with audit event
- ZKP module renamed to "commitment-based attribute verification" — docs honestly state verifier sees claim value
- Legal oracle: stub fetch replaced with real `reqwest::blocking` HTTP client (15s timeout, Bearer auth, error handling)
- RocksDB persistence for 3 new stores: `AuditStore` (CF `audit_log`), `SandboxReportStore` (CF `sandbox_reports`), `OracleRecordStore` (CF `oracle_records`). All use existing Column Family pattern in `adapters.rs`. `SandboxReportStore` refactored from concrete struct to trait (`MemorySandboxReportStore` + `RocksDbBlockStore` impl).

**ZKP for Sovereign Identity — Commitment-based attribute verification**

- `src/identity/zkp.rs` — Commitment-based ZKP: SHA-256(value || blinding) proofs without revealing claim data
- 3 predicates: `RangeProof` (numeric >= threshold), `SetMembership` (value in allowed set), `CredentialValidity` (active, not expired/revoked)
- `prove_range()`, `prove_set_membership()`, `prove_credential_validity()` generate `ZkPresentation`
- `verify_presentation()` validates commitment integrity + predicate satisfaction; rejects tampered proofs
- Endpoints: `POST /identity/zkp/prove`, `POST /identity/zkp/verify`
- Audit events on each verification (result only, no claim data revealed)
- 13 unit tests covering all predicates, boundary cases, and tamper detection

**Legal Oracle — Off-chain legal data queries with on-chain records**

- `src/legal_oracle/mod.rs` — `OracleRecord` (source, query, response_hash, timestamp, signature, summary), `OracleRecordStore` trait + `MemoryOracleRecordStore`, SHA-256 response hashing
- `src/legal_oracle/legal.rs` — `LegalOracle` service: configurable sources, injectable fetch function, TTL cache, response hash verification, JSON summary extraction
- Endpoints: `POST /oracle/legal/query`, `GET /oracle/legal/records`, `GET /oracle/legal/records/{id}`
- Audit event emitted on each query
- 15 unit tests (store CRUD, cache hit/miss, hash verification, fetch errors, summary extraction)

**Forensic Dashboard — Compliance UI in block explorer**

- Compliance page (`/compliance`): audit events table with action/org filters, 5 summary indicators (total, blocks mined, DID mutations, chaincode deploys, failed requests), auto-refresh 10s, color-coded action badges
- ChaincodeHealth page (`/chaincode-health`): sandbox report viewer per chaincode version, pass/fail summary card, per-check detail (well-formedness, import whitelist, memory limits)
- API client: `getAuditEvents()`, `getSandboxReport()` with types `AuditEntry`, `SandboxReport`
- Sidebar: new "Compliance" nav group with Audit Trail and Chaincode Health items

**Chaincode Sandbox — Static validation gate before deployment**

- `src/chaincode/sandbox.rs` — Wasm static analysis: well-formedness (wasmparser), import whitelist (6 allowed host functions), memory limits (max 16 pages / 1 MB)
- Auto-detects WAT text vs binary Wasm
- `SandboxReport` struct with per-check results, wasm size, validation duration
- `SandboxReportStore` in-memory for persisting reports per chaincode version
- Gate in `install_chaincode`: rejects Wasm that fails sandbox with 400 + failure details
- `GET /api/v1/chaincode/{id}/sandbox-report?version=...` — query validation report
- Dependencies: `wasmparser = "0.236"`, `wat = "1.246"`
- 10 unit tests (valid, malformed, forbidden imports, oversized memory, boundary, store)

**Audit — Action-level domain events (ISO 27001 compliance)**

- `AuditEntry.action` field with 16 semantic action types (`AuditAction` enum: `BlockMined`, `DidRegistered`, `ChaincodeInstalled`, etc.)
- `AuditStore::query()` now accepts `action` filter alongside existing `org_id` and time range filters
- `emit_domain_event()` / `emit_if_present()` helpers for emitting audit events from business logic without HTTP context
- Domain audit hooks in 8 mutation handlers: mine_block, create_wallet, stake, install_chaincode, store_write_identity, store_write_credential, create_channel, submit_proposal
- `GET /api/v1/audit/requests?action=block_mined` — query by action type
- CSV export includes action column
- `AuditEntry.metadata` optional field for domain-specific context (block height, DID, chaincode hash, etc.)
- 11 unit tests covering action filtering, combined filters, domain event emission, and display formatting
- Integration roadmap: `docs/ROADMAP-INTEGRACION-CERULEAN.md` — 5 phases derived from Abraxas integration guide, adapted to actual architecture

### 2026-05-10

**Enterprise Sandbox**

- Prometheus + Grafana added to sandbox compose (pre-provisioned dashboard at `:3000`)
- Resource governance: memory/CPU limits on all containers (node 512M, frontends 128M)
- Rich seed data: 7 wallets, 8 blocks, wallet transfers, 2 orgs, 2 channels, 7 DIDs, 5 credentials, governance proposals with votes
- Version tracking: `BUILD_VERSION` from git SHA injected at build time
- Nginx error fallback: branded 502/503 page with 5s auto-retry when node is starting
- SSE event stream proxy through nginx (unbuffered, long-lived connections)
- Multi-org demo: org1 (Universidad de Chile) + org2 (Banco Central), channels `academic` + `financial`
- Backup/restore script: `scripts/sandbox-backup.sh` snapshots RocksDB volume as tarball

**Oracle System Hardening**

- Fix: `aggregate_reports` now filters by symbol (was mixing all pending reports)
- Fix: HMAC signature always verified (removed test-mode bypass for timestamps < 100M)
- Fix: constant-time comparison via `Mac::verify_slice` (prevents timing attacks)
- Fix: external connector uses fixed-point pricing (×10^8) for sub-1 tokens (CLP/USD)
- Fix: `retain` only clears processed symbol's reports (other symbols persist)

**Pentest Suite Expanded (20 → 33 scenarios)**

Network layer attacks (PEN-021 to PEN-023):
- Sybil peer flooding: `MembershipTable` now enforces `MAX_PEERS` cap (default 500), rejects excess
- Malformed P2P messages: serde rejects garbage/truncated/oversized payloads without panic
- Eclipse attack: DAG rejects blocks referencing non-existent parents

EVM adversarial tests (PEN-025, PEN-026, PEN-031, PEN-032):
- Reentrancy: revm enforces 1024 call depth + gas limit
- Storage collision: address-scoped storage isolation verified
- Delegatecall abuse: separate contracts cannot modify each other's state remotely
- Gas bomb: infinite loop + memory expansion halts cleanly at gas limit

Consensus attacks (PEN-027, PEN-028):
- Nothing-at-stake: equivocation detector catches dual-fork proposals, slashing applies
- Long-range reorg: BFT commit QC provides finality; without QC, fork choice acknowledged

Economic attacks (PEN-024, PEN-029, PEN-030, PEN-033):
- Front-running sandwich: MVCC conflict detector separates overlapping txs into waves
- Validator grinding: stake-weighted round-robin gives no advantage to address splitting
- Fee suppression: `MIN_BASE_FEE` floor prevents zero-fee exploitation
- Proposer-MEV: deterministic scheduling + multi-org endorsement prevents extraction

Infrastructure:
- `MembershipTable::with_capacity()` constructor for custom peer limits
- `record_alive_full()` now returns `bool` (false = rejected at capacity)

Tests: 1,629+ passed, 0 critical vulnerabilities.

---

**Oracle Maturation**

- External connector wired in `main.rs` — `ORACLE_SOURCES` env var activates real HTTP price feeds
- New env var `ORACLE_CONNECTOR_SYMBOL` (default `BTC/USD`) selects the feed symbol
- API responses now include `age_ms` and `is_stale` fields for every price feed
- New endpoint: `GET /oracle/status` — node count, feed count, stale/fresh breakdown, pending reports

**Pentest Suite Expanded (15 → 20 scenarios)**

Strengthened existing tests (previously descriptive, now exercise real code):
- PEN-003: Replay attack — `MemoryStore` rejects duplicate block height
- PEN-004: Double-spend — `schedule_batch` detects WAW conflict, separates into sequential waves

New attack scenarios:
- PEN-016: Path traversal in channel names (`../`, null bytes) — HashMap keys are literal, no leakage
- PEN-017: Oracle price manipulation (20x outlier) — filtered by median, consensus unaffected
- PEN-018: Governance vote with zero stake — `VoteStore` rejects `ZeroPower`
- PEN-019: Credential forgery with untrusted issuer — detected, distinguishable by DID
- PEN-020: Oversized payload (100KB ISO 20022 message) — no panic, size limits recommended at boundary

**Sandbox Seed Data**

- `scripts/seed-sandbox.sh` — pre-loads 5 wallets, 5 blocks, 5 identities, 3 credentials, 1 governance proposal, 1 contact entry
- Auto-runs after node health check in `scripts/sandbox.sh`
- Idempotent: safe to run multiple times

Tests: 1,604 passed, 0 failures.

### 2026-05-09

**Adversarial Pentest Suite**

15 attack scenarios executed against live code (`src/forensic_pentest.rs`):
- Block tampering, signature forgery, replay, double-spend, equivocation
- ACL bypass, rate limit evasion, channel crossing, identity spoofing
- Hash collision, timestamp manipulation, overflow, null injection, key extraction, state rollback
- 2 real vulnerabilities found and fixed:
  - `SecurityToken::mint` integer overflow → `checked_add`
  - `MemoryStore::write_block` allowed overwrites → duplicate height rejection
- API: `GET /pentest/report`
- 0 critical vulnerabilities remaining

**Forensic, Intelligence, Stress — Full API Exposure**

- Forensic: `GET /forensic/replay`, `GET /forensic/integrity` (block chain verification)
- Intelligence: `POST /intelligence/anomaly`, `POST /intelligence/risk`, `POST /intelligence/patterns`
- Stress: expanded to 8 modules (+ governance, forensic, pattern detection)

**Pre-Launch Modules (5/5 Complete)**

Intelligence layer (`src/intelligence/`):
- Anomaly detection: rolling z-score with configurable threshold and window
- Risk scoring: 6 rules (watchlist, KYC, amount, frequency, identity age, country)
- Pattern recognition: velocity spike, structuring, round-trip, dormant activation

Oracle E2E (`src/oracle_demo.rs`):
- Simulated BTC/USD, ETH/USD, CLP/USD feeds with deterministic pseudo-random walk
- Background poller wired into OracleRegistry via `ORACLE_DEMO=true`
- Queryable via `GET /oracle/feeds`

Regulatory sandbox (`src/regulatory/`):
- 21 compliance checks across 10 categories
- Automated report with SHA-256 content hash
- API: `GET /regulatory/checks`, `GET /regulatory/report`

Forensic audit — replay and integrity:
- `replay_blocks()`: sequential height verification, gap detection
- `verify_chain_integrity()`: hash linkage check, tampering detection

Per-module stress testing (`src/stress.rs`):
- 5 targeted tests: storage, crypto, anomaly, risk, ISO 20022 validation
- Per-module ops/sec, p50/p99 latency, Pass/Degraded/Fail classification
- API: `GET /stress/report?ops=1000`

**Contact Form Backend**

- `POST /api/v1/contact` — stores name, email, org, message
- `GET /api/v1/contact` — admin-only listing
- Landing CTA replaced with inline form posting to own node

**Landing Page Redesign — Non-Technical, Identity-Driven**

Complete rewrite of `block-explorer-vite/src/pages/Landing.tsx`:
- Hero: "La confianza deja de ser una promesa" — original concept, not borrowed
- Live network pulse: fetches `/api/v1/health` every 10s, shows block height + peers
- Three-column thesis: integrity by design, privacy without sacrifice, operational sovereignty
- Four verticals as interactive selector with per-use headline and 3-step mini-flow
- Hard numbers: 18,700 TX/s, 14ms p50, 1,532 tests, 58 components, 193 ISO countries
- Real social proof: Cámara Blockchain Chile audit quote (not fabricated logos)
- Subtle visual identity: `#fafbfc` background, clean typography, generous spacing
- No comparisons, no competitor mentions, no copied patterns

**Oracle, Forensic, Compliance — HTTP API Endpoints**

- Oracle: `GET /oracle/feeds/{symbol}`, `GET /oracle/feeds`, `GET /oracle/nodes`
- Forensic: `GET /forensic/timeline`, `GET /forensic/security`, `POST /forensic/export`
- Compliance: `POST /compliance/validate/{pacs008,pacs002,pacs004,pain001,pain002,camt053,camt052}`, `GET /compliance/countries`, `GET /compliance/currencies`
- `oracle_registry` added to AppState

**ISO Compliance — Full Standard Coverage**

- ISO 20022: 7 message types (pacs.008/002/004, pain.001/002, camt.053/052)
- ISO 3166: expanded to 193 countries
- ISO 4217: expanded to 64 currencies with 3-decimal support
- ISO 8601: date, datetime, duration validators
- ERC-3643: security tokens with identity registry, compliance module, issuer controls

**Auto-Discovery, Pricing**

- Gossip-based peer exchange (`src/discovery/auto.rs`)
- Pricing structure draft (`docs/commercial/PRICING.md`)

**Cerulean Voto — Vote Privacy**

- `POST /vote` returns tally only (no individual votes)
- `GET /votes` restricted to admin role (403 for others)

Tests: 1,532 passed, 0 failures.

### 2026-05-08

**CSIRT Webhook Notifier & Security Events**

- 5 security event variants in `BlockEvent`: `AclDenied`, `EquivocationDetected`, `RateLimitExceeded`, `InvalidSignature`, `ValidatorSlashed`
- `is_security_event()` filter for CSIRT/SIEM forwarding
- `WebhookNotifier` (`src/events/webhook.rs`): subscribes to EventBus, filters security events, POSTs to configurable endpoint with exponential backoff
- Env vars: `CSIRT_WEBHOOK_URL`, `CSIRT_WEBHOOK_SECRET`, `CSIRT_WEBHOOK_TIMEOUT_SECS`
- Wired in `main.rs` — activates only when `CSIRT_WEBHOOK_URL` is set

**Channel Retention Policy**

- `RetentionPolicy` struct: `block_retention_count`, `private_data_ttl_blocks`, `transaction_retention_secs`
- Integrated in `ChannelConfig` with `#[serde(default)]` for backwards compatibility
- `ConfigUpdateType::SetRetention` — configurable via governance config transactions

**Governance Permissive Mode**

- Stake/deposit check skipped in `ACL_MODE=permissive` for proposal submission
- Voter power defaults to 1 in permissive mode (enables sandbox demos without staking)

**Cerulean Voto — UI Overhaul**

- Removed all page titles (sidebar provides context)
- Elections: create form moved to slide-over drawer, history table with internal scroll
- Vote: inline voter bar with per-election buttons, disabled when no name entered
- Vote receipt overlay with animated guarantee checks (signed, immutable, consensus, PQC)
- Voters (Padrón): compact inline registration + verification, no exposed DIDs
- Results: compact cards with tally bars and stats
- All pages: `h-screen` layout with internal scroll, no page-level scrolling
- Borders softened (`border-neutral-100`), shadows removed, padding reduced
- `did:cerulean:` prefix hidden from all user-facing inputs — DIDs generated internally
- "Deposito" / "Garantia" field removed from elections (hardcoded internally)
- "Peso del voto" removed — 1 person = 1 vote

**Sandbox Infrastructure**

- `docker-compose.sandbox.yml` — single-node + explorer + voto compose
- `block-explorer-vite/Dockerfile` + `nginx.conf` — containerized frontend with API proxy
- `cerulean-voto/Dockerfile` + `nginx.conf` — same for voting app
- `scripts/sandbox.sh` — one-command launcher with Cloudflare Quick Tunnels
- `SANDBOX.md` — guide for quick tunnels and custom domain setup

**Commercial Documentation**

- `COMPLIANCE-LEY-21663-CIBERSEGURIDAD.md` + PDF — Ley 21.663 compliance mapping
- `ONE-PAGER-PRODUCTO.md` — non-technical product explanation
- `POLYGON-COMPARISON.md` — Cerulean vs Polygon positioning
- `VERTICAL-HORIZONTAL-MATRIX.md` — how verticals consume platform capabilities
- `GO-TO-MARKET-STRATEGY.md` — 5-step adoption playbook
- `GOVERNANCE-OPERATIONAL.md` — operational governance framework for regulators
- `SLA.md` — 3-tier service level agreement
- `PRODUCT-GAPS-AND-READINESS.md` — 19 gaps identified, 8 closed this session

Tests: 1445 (18 new), 0 failures. All quality gates pass.

### 2026-05-03

**E2E Distributed Demo — PQC ML-DSA-65 Validated**

Connected all subsystems into a functional distributed demo:

- PQC block signing via pluggable `SigningProvider` (Ed25519 or ML-DSA-65)
- Raft ordering: synchronous commit after propose (no async wait)
- Peer replication: `OrderedBlock` broadcast to all connected peers
- TX indexing: gateway writes transactions to store, queryable by `GET /tx/{tx_id}`
- Smart contract execution: `POST /chaincode/{id}/invoke` with shared world state
- Explorer API: `GET /blocks`, `GET /blocks/{height}` (no channel enforcement)
- Strict algorithm validation: unknown `SIGNING_ALGORITHM` values panic (no silent fallback)
- Block signature verification via `verify_block_signature()` dispatching by algorithm
- Gateway and API now share the same store instance (fixes query-after-commit)
- 7 RocksDB test failures fixed (hardcoded `/tmp/` paths → `tempfile::TempDir`)
- Demo compose: `docker-compose.demo.yml` (3 nodes, ARM64 native, PQC, no TLS)
- Scripts: `demo-consistency.sh`, `demo-persistence.sh`
- Contract: `contracts/kv_store.wat` (set/get demo)
- Docs: `DEMO.md` (10-min onboarding), `docs/dev/LOCAL-DLT-TESTING.md`

Tests: 1427 lib (0 failures), all quality gates pass.

### 2026-04-28

**Pre-Lab Findings Closure — All 11 Findings Resolved**

ML-KEM-768 real implementation:
- `pqcrypto-mlkem` v0.1.1 replaces SHA3-based placeholder (F-001 CLOSED)
- Real keygen/encapsulate/decapsulate: pk=1184B, sk=2400B, ct=1088B, ss=32B
- KAT self-test verifies shared secret roundtrip + invalid ciphertext rejection
- 8 unit tests in `mlkem.rs`

ACVP dry-run harness:
- `tools/acvp_dry_run/` rewritten: 6 modules (vectors, sha3, mldsa, mlkem, report, main)
- 3 JSON vector files: SHA3-256 (5), ML-DSA-65 (5), ML-KEM-768 (5)
- CLI: `--algorithm`, `--vectors`, `--all` flags; generates `report.json`
- 15/15 vectors pass; 5 integration tests (F-002 CLOSED)

Security Policy and documentation:
- `SECURITY_POLICY.md` expanded to 16 sections: CO Guide, User Guide, Error Recovery (F-010, F-009 CLOSED)
- `MODULE_SPECIFICATION.md` created: 9 sections with full API surface (F-008 CLOSED)
- SP 800-90B justification added to `ENTROPY_RNG_EVIDENCE.md` (F-003 CLOSED)
- Boundary documentation clarified in `legacy.rs` (F-006 CLOSED)

Code hardening:
- `libc::mlock` on `MldsaPrivateKey`, `MlKemPrivateKey`, `MlKemSharedSecret` (F-004 CLOSED)
- `is_valid_transition()` + 14 exhaustive FSM tests covering all 16 (state, transition) pairs (F-007 CLOSED)
- Reproducible build verified: hash `29af517b` matches across 2 independent builds (F-011 CLOSED)

CI updates:
- `pre-lab-audit.yml` adds `cargo test -p acvp_dry_run` + `cargo run -p acvp_dry_run -- --all`
- ACVP report.json uploaded as artifact

Audit status: 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW open. 1 INFO accepted (F-005).
Traceability: 8 PASS, 4 PARTIAL, 0 FAIL.
Tests: 69 pqc_crypto_module + 5 ACVP + 1427 lib = 1501 total.

**Pre-Lab Mock Audit Package**
- `pre_lab_audit/` — 9 documents: mock audit report, ACVP dry-run plan, IG checklist, clean-room build, entropy/RNG evidence, vendor evidence, traceability matrix, findings register
- Mock audit: 0 CRITICAL, 2 HIGH (ML-KEM placeholder, ACVP vectors), 2 MEDIUM, 2 LOW, 3 INFO
- Traceability matrix: 11/12 requirements PASS, 1 PARTIAL (ML-KEM)
- `tools/acvp_dry_run/` — ACVP test vector harness (SHA3, ML-DSA, ML-KEM dry-run)
- `scripts/clean_room_build.sh` — reproducible build verification
- `.github/workflows/pre-lab-audit.yml` — FIPS pre-lab CI workflow

**CI Reliability Hardening**
- `tests/test_seeds.rs` — shared deterministic seed constants for reproducible chaos/property tests
- `.config/nextest.toml` — cargo-nextest profiles: default (no retry), chaos (1 retry, 120s timeout), ci
- `.github/workflows/coverage.yml` — cargo-llvm-cov coverage report on PR + push main
- `scripts/ci_runtime_report.sh` — PR gate timing budget script (target: ≤15 min)
- PR pipeline runs fast critical gates only (no chaos/fuzz/benchmarks)
- Nightly pipeline runs full adversarial validation + benchmarks
- Fix: `fips_readiness` test correctly targets `pqc_crypto_module` crate

**Self-Auditing CI Pipeline**
- 5 GitHub Actions workflows: `ci.yml` (updated with PQC tests), `security-audit.yml`, `performance-guardrails.yml`, `nightly-chaos.yml`, `fuzz.yml`
- `tests/property_invariants.rs` — 7 proptest invariants (tampering invalidates sigs, equivocation always detected, hash collision resistance, serde roundtrip preserves PQC metadata, consistency catches all mismatches)
- `fuzz/` — 3 libfuzzer targets (block parser, signature parser, gossip message parser)
- `deny.toml` — dependency policy (vulnerable=deny, unmaintained=warn, license allowlist)
- CI env: strict PQC mode on all jobs (`REQUIRE_PQC_SIGNATURES=true`, `HASH_ALGORITHM=sha3-256`)

**CMVP Submission Readiness Package**
- `fips_submission/` — complete CMVP intake package for FIPS 140-3 lab engagement
- `SUBMISSION_CHECKLIST.md` — 15 artifacts tracked (10 ready, 2 needs work, 3 not started)
- `LAB_SELECTION.md` — 5 NVLAP-accredited labs profiled (Acumen, atsec, UL, Leidos, InfoGard)
- `TEST_VECTOR_PLAN.md` — ML-DSA, ML-KEM, SHA3, RNG vector requirements and ACVP gaps
- `BUILD_ENVIRONMENT.md` — Rust toolchain, target platforms, deterministic build instructions
- `GAP_ANALYSIS.md` — 7/10 areas aligned, 2 partial (vectors, RNG), 2 missing (CAVP, lab tooling)
- `VALIDATION_TIMELINE.md` — 5-phase plan, 12-24 month estimate, $80K-$250K range
- `CONTACT_PACKAGE/` — executive summary, module overview, 21 questions for initial lab call

**Pre-CMVP FIPS 140-3 Documentation Package**
- `SECURITY_POLICY.md` — finalized 13-section security policy (module ID, boundary, algorithms, roles, services, FSM, self-tests, zeroization)
- `DESIGN_DOCUMENT.md` — architecture diagram, API entry points, data flow, internal components
- `FINITE_STATE_MODEL.md` — 4-state machine with transitions, forbidden paths, fail-closed behavior
- `KEY_MANAGEMENT.md` — key types, generation, storage (in-memory only), usage, ZeroizeOnDrop destruction
- `SELF_TEST_DOCUMENTATION.md` — 4 KATs (SHA3, ML-DSA, ML-KEM, RNG), failure behavior
- `NON_APPROVED_USAGE.md` — legacy algorithms, runtime guard + feature flag gating
- `OPERATIONAL_GUIDANCE.md` — initialization, configuration, error handling, monitoring
- `build/reproducible_build.md` — Rust toolchain pinning, Cargo.lock, deterministic builds
- `build/module_boundary_definition.md` — files inside/outside boundary, enforcement mechanisms
- `tests/fips_readiness.rs` — 8 tests: pre-init rejection, approved ops, legacy blocking, state transitions, fail-closed, no-panic

**Approved vs Legacy Crypto Separation**
- Runtime guards on all legacy functions: `ensure_not_approved()` blocks Ed25519, SHA-256, HMAC when module is in Approved mode
- Guarded functions: `legacy_ed25519_verify`, `legacy_ed25519_sign`, `legacy_sha256`, `legacy_hmac_sha256`
- `approved-only` Cargo feature flag excludes legacy module at compile time
- 12 tests in `approved_vs_legacy.rs`: legacy blocked in approved, works before init, no fallback, API cleanliness
- `SECURITY_POLICY_DRAFT.md` updated with non-approved algorithm enforcement section

**100% Crypto Boundary Compliance**
- All 28 legacy files migrated from direct crypto imports to `pqc_crypto_module::legacy::*`
- `LEGACY_ALLOWLIST` is now empty — 189/189 files (100%) clean
- `pqc_crypto_module::legacy` re-exports Ed25519, SHA-256, HMAC, rand, ML-DSA raw access as explicitly non-approved APIs
- Boundary test fails if any production file imports raw crypto crates directly
- Ed25519/SHA-256 still available for legacy block verification, but routed through the crypto module boundary

**FIPS-Oriented Crypto Module**
- `crates/pqc_crypto_module/` — standalone crate isolating all PQC cryptography behind a strict boundary
- Approved-mode state machine: `Uninitialized → SelfTesting → Approved → Error`
- All crypto APIs reject calls unless module is in `Approved` state
- ML-DSA-65 sign/verify (FIPS 204), SHA3-256 (FIPS 202), ML-KEM-768 placeholder (FIPS 203)
- Startup KAT self-tests for all algorithms + continuous RNG test
- `ZeroizeOnDrop` on all private key and shared secret types
- No classical algorithm fallback — Ed25519, SHA-256, RSA excluded from module API
- `SECURITY_POLICY_DRAFT.md` aligned with FIPS 140-3 Security Policy structure
- 17 integration tests: API boundary, self-tests, no-fallback, key zeroization
- Workspace configuration: root `Cargo.toml` adds `[workspace]` with `crates/pqc_crypto_module`

**Post-Quantum Readiness — Full Stack Hardening**

Crypto-agility layer:
- `signature_algorithm: SigningAlgorithm` field added to `Block`, `DagBlock`, `Endorsement`, `AliveMessage`, `TransactionProposal` (replaces size-based heuristic detection)
- `hash_algorithm: HashAlgorithm` field added to `Block` (enables migration without breaking old blocks)
- `secondary_signature` + `secondary_signature_algorithm` on `Block` and `DagBlock` for dual-signing migration
- `HashAlgorithm` enum (`Sha256`, `Sha3_256`) with `HASH_ALGORITHM` env var, configurable `hash()` / `hash_with()`, KAT self-tests at startup
- `SigningAlgorithm::is_post_quantum()` helper, `Default` impl (Ed25519 for backwards compat)
- All new fields use `#[serde(default)]` for backwards compatibility with legacy JSON

PQC enforcement:
- `REQUIRE_PQC_SIGNATURES=true` env var rejects Ed25519 in consensus and endorsement validation
- `validate_signature_consistency()` catches tag forgery (declared algorithm vs actual signature size)
- Integrated into `ConsensusEngine::accept_block()` and `validate_endorsements()`

TLS post-quantum handshake:
- `rustls-post-quantum` dependency: X25519+ML-KEM-768 hybrid key exchange
- `TLS_PQC_KEM=true` env var installs PQ `CryptoProvider` at startup
- `install_crypto_provider()` called before any TLS config is built

Dual-signing for migration:
- `dual_sign()` and `verify_dual()` with `Either` / `Both` modes
- `DUAL_SIGN_VERIFY_MODE` env var for transition policy

Equivocation detection:
- `EquivocationDetector` with `ConsensusPosition` key `(height, slot, proposer)`
- `EquivocationProof` with two block hashes + two signatures + algorithm tag
- Gossip deduplication, `receive_proof()`, `is_penalized()` quarantine
- Serde persistence (`to_bytes()` / `from_bytes()`) for restart survival

Slashing economics:
- `PenaltyManager` with `PenaltyRecord`, `PenaltyPolicy`, `PenaltyStatus` (Active/Expired/Permanent)
- Deterministic expiration at `start_height + duration`, permanent mode, escalation on repeat
- Anti-double-slash via `processed_proofs` set, reputation tracking
- Serde persistence for restart survival

New modules:
- `src/crypto/hasher.rs` — configurable SHA-256 / SHA3-256 hash abstraction
- `src/identity/pqc_policy.rs` — PQC enforcement + signature consistency validation
- `src/identity/dual_signing.rs` — dual-signing helpers for crypto migration
- `src/consensus/equivocation.rs` — equivocation detection, proofs, gossip, persistence
- `src/consensus/slashing.rs` — penalty lifecycle, policy, reputation, persistence

New test suites:
- `tests/pqc_security_audit.rs` — 24 adversarial tests (tag forgery, downgrade, dual-sign bypass, TLS, hash migration)
- `tests/chaos_network.rs` — 11 multi-node scenarios (partition, replay, crash, mixed config, flood, convergence)
- `tests/persistent_crash_recovery.rs` — 5 RocksDB crash/restart tests (exact state restoration, tampered storage rejection)
- `tests/crypto_dos_flood.rs` — 6 flood resistance tests (10K invalid flood, duplicate caching, rate limiting, cheap rejection ordering)
- `tests/byzantine_equivocation.rs` — 9 equivocation tests (detection, proof, gossip dedup, penalty, negative cases, stress)
- `tests/equivocation_persistence_partition.rs` — 6 tests (penalty restart survival, cross-partition retroactive detection)
- `tests/slashing_penalty_lifecycle.rs` — 9 tests (active/expired/permanent penalties, restart, escalation, anti-double-slash)
- `tests/performance_guardrails.rs` — 6 threshold tests (cheap rejection 213x faster, 4843 blocks/sec PQC validation, RocksDB 10K reopen 641ms)
- `benches/pqc_performance.rs` — 9 Criterion benchmarks (ML-DSA sign/verify, SHA3, block validation, RocksDB, flood rejection, throughput)

Dependencies: `sha3 = "0.10"`, `rustls-post-quantum = "0.2"`

New env vars: `REQUIRE_PQC_SIGNATURES`, `TLS_PQC_KEM`, `DUAL_SIGN_VERIFY_MODE`, `HASH_ALGORITHM`

Tests: 1503 total (1427 lib + 76 integration), 0 failures

### 2026-04-25

**PIN Module — Generation, Hashing, and DID Association**
- `src/pin/generator.rs` — CSPRNG numeric PIN generator (4-6 digits), Argon2id hashing, verification
- `src/pin/store.rs` — `PinStore` trait with `MemoryPinStore` (DID-to-hashed-PIN mapping)
- `src/api/handlers/pin.rs` — `POST /pin/generate` (generate + hash + associate to DID), `POST /pin/verify` (verify PIN against stored hash)
- `AppState.pin_store` initialized with `MemoryPinStore` at startup
- Dependency: `argon2 = "0.5"`
- 16 unit tests (10 generator + 6 store)

### 2026-04-24

**Cerulean Voto — Electronic voting frontend MVP**
- `cerulean-voto/` — standalone Vite + React + Tailwind app for blockchain-backed elections
- Same stack and patterns as `block-explorer-vite/` (lazy routes, axios unwrap, Tailwind theme, PageIntro)
- Landing page with hero, 3 pillars (immutable, verifiable, post-quantum), dual CTAs
- Dashboard: active/closed election stats, tally bars, navigation to vote
- Elections: create elections via governance API, full history table
- Vote: voter identity (DID + stake), Yes/No/Abstain buttons, live tally bars
- Results: public audit view with percentage bars, quorum/pass indicators
- Voters: register voters via identity API (DID), lookup by DID
- Proxies `/api` to node on port 5174 (independent from block explorer)

**Documentation — E-voting quotation**
- `docs/COTIZACION-VOTO-ELECTRONICO.md` — formal quotation for e-voting system over Cerulean Ledger
- `docs/COTIZACION-VOTO-ELECTRONICO.html` — styled HTML version for PDF export
- `docs/COTIZACION-VOTO-ELECTRONICO.pdf` — print-ready PDF

**Documentation — Presentation materials (Blockchain Chamber Chile)**
- `docs/DEMO-SCRIPT.md` — 5-minute live demo script with timing, commands, checklist, and fallback plan
- `docs/TESSERACT.md` — standalone doc: 4D probability field, 4 physics laws, comparisons, relevance for the Chamber
- `docs/ONE-PAGER-CAMARA.md` — rebranded from "rust-bc" to "Cerulean Ledger", updated year and license
- `docs/PUBLIC-ROADMAP.md` — rebranded, added concrete dates (Q2-Q4 2026, H1-H2 2027), new deliverables and success metrics

**PDF dossier — technical (all-in-one deliverable)**
- `docs/DOSSIER-CAMARA-BLOCKCHAIN-CHILE.pdf` — 44-page consolidated PDF with cover, TOC, and navigable bookmarks
- Individual PDFs for all 7 presentation documents (pandoc + weasyprint)

**Documentation — Commercial dossier (non-technical)**
- `docs/RESUMEN-NO-TECNICO.md` — plain-language overview: what it is, what it solves, for whom
- `docs/QUE-ES-DLT-EMPRESARIAL.md` — DLT concepts, why Rust, FIPS 204, PQC end-to-end, standards (NIST/CNSS/eIDAS/CMF/SII), Fabric comparison
- `docs/CASOS-PRACTICOS.md` — 6 business cases with before/after tables (agro, HR, finance, gov, health, supply chain)
- `docs/POR-QUE-CERULEAN.md` — 5 reasons, value comparison, adoption path, commercial FAQ
- `docs/DOSSIER-COMERCIAL.pdf` — 30-page consolidated PDF with cover, TOC, and navigable bookmarks
- `docs/PRESENTACION.md` — fixed code block overflow in EOV diagram

### 2026-04-23

**Block Explorer — Tesseract page**
- `Tesseract.tsx` — standalone `/tesseract` route explaining the geometric consensus prototype
- Four interactive tabs: Conceptos, Leyes fisicas, Comparativa, Demo
- `FieldDemo.tsx` — interactive 10x10 probability field simulation (seed, crystallize, destroy, self-heal, fake injection)
- "En simples palabras" right-side drawer with accessible analogies
- Bidirectional navigation: Landing ↔ Tesseract via CTA buttons
- Dynamic document title per route, favicon removed

**Governance — HTTP API + Explorer UI**
- `src/api/handlers/governance.rs` — 7 REST endpoints: protocol params, proposal CRUD, voting, tally
- `AppState` fields: `proposal_store`, `vote_store`, `param_registry` initialized at startup
- Routes registered under `/api/v1/governance/`
- `Governance.tsx` — proposals, stake-weighted voting with visual tally bar, protocol parameters table
- 10 governable parameters exposed (block size, fees, quorum, thresholds, etc.)

**Block Explorer — Services routing + Landing refinement**
- `ServicesLayout.tsx` — dedicated layout with compact sidebar listing all services, sticky sidebar (no scroll bleed)
- All service pages mounted under `/services/*` with consistent header and navigation
- `Services.tsx` — card grid (10 services, SVG icons, compact 5-column layout)
- Landing: "Ver servicios" button navigates to `/services`
- `Layout.tsx` — added "Gobernanza" nav group in sidebar
- API client: 7 governance functions + 4 types (Proposal, Vote, TallyResult, ProtocolParam)

**Documentation — Presentation materials for Blockchain Chamber Chile**
- `docs/PRESENTACION.md` — full platform overview tailored for board presentation
- `docs/FAQ.md` — ~40 questions organized by audience (board, enterprise, technical, regulators)
- `docs/PITCH.md` — talking points, one-liners per audience, objection handling, demo flow
- `docs/PQC-TEST-EVIDENCE.md` — concrete PQC test inventory (12 dedicated + 250+ integration)

### 2026-04-17

**Block Explorer — New pages and cleanup**
- Removed legacy Next.js block explorer (`block-explorer/`)
- Added 6 new pages to Vite explorer: Wallets, Transactions, Mining, Staking, Channels, Governance
- New API client functions: `getWallets`, `stakeTokens`, `requestUnstake`, `listChannels`, `createChannel`, `getChannelConfig`
- Updated nav layout with 11 sections (was 6)
- Wallets: list + create wallet
- Transactions: send transactions + live mempool view
- Mining: mine blocks with existing or new wallet
- Staking: stake/unstake tokens + validator table with actions
- Channels: create Fabric-style channels + view config
- Governance: informational page (API endpoints pending backend exposure)
- Demo RRHH: guided 5-step credential verification flow (register issuer → register candidate → issue credential → verify → full profile), highlighted nav button, verification time display
- Wallets page: removed dependency on non-existent `GET /wallets` endpoint; now uses session-based wallet list with lookup by address
- Fixed Vite proxy: `.env` default changed to `http://127.0.0.1:8080` for local development
- Redesigned Layout: flat nav replaced with categorized sidebar (Demos, Red, Tokens, Identidad, Smart Contracts) with descriptions per item, responsive hamburger menu
- Redesigned Home: hub with grouped cards explaining each capability, replaced flat block list

**Documentation**
- `docs/HR-DOCUMENT-VERIFICATION-IMPACT.md` — Impact analysis: blockchain-based document verification for HR hiring processes (DIDs, verifiable credentials, channel privacy, PQC signatures)
- Moved root-level docs to `docs/`: `PUBLIC-ROADMAP.md`, `BENCHMARKS-RESULTS.md`, `HOKTUS-BLOCKCHAIN-IMPACT.md`, `ONE-PAGER-CAMARA.md`
- Cleaned up stale root-level files: `FABRIC-GAP-ANALYSIS.md`, `MULTI-PEER-ENDORSEMENT.md`, `ROADMAP.md`
- Removed tracked Python `__pycache__` files from `sdk-python/`

### 2026-04-16

**Consensus — BFT (Phases 1–3)**
- `consensus::bft::types` — `VoteMessage`, `QuorumCertificate`, `BftPhase` (Prepare→PreCommit→Commit→Decide), phase-aware signing payload for domain separation
- `consensus::bft::quorum` — `QuorumValidator` (2f+1 threshold), `SignatureVerifier` trait, `ensure_bft_viable()` guard (min 4 validators)
- `consensus::bft::vote_collector` — accumulates votes per (phase, round, hash), signals quorum
- `consensus::bft::round` — event-driven state machine per round: AwaitingProposal→Preparing→PreCommitting→Committing→Decided/Failed
- `consensus::bft::round_manager` — orchestrates rounds with round-robin leader rotation, exponential backoff timeouts (3s–30s), highest QC tracking
- `DagBlock.commit_qc` — optional `QuorumCertificate` field for BFT-decided blocks
- `ConsensusEngine.with_bft()` — BFT mode validates CommitQC on non-genesis blocks (phase, hash match, quorum)
- 76 BFT unit tests, 143 total consensus tests

**Consensus — Wire protocol & backend abstraction (Phase 4)**
- `consensus::backend` — `ConsensusBackend` trait (Raft/BFT selection), `ConsensusMode` enum, `CONSENSUS_MODE` env var
- P2P `Message` enum: `BftProposal`, `BftVote`, `BftQuorumCertificate`, `BftViewChange` variants
- 147 total consensus tests

**Consensus — Adversarial E2E tests**
- `tests/bft_e2e.rs` — 16 integration tests simulating multi-node BFT networks
- Scenarios: happy path (4/7/10 nodes), equivocation attacks, crash faults, silent leaders with view change, network partitions (minority/majority), partition healing, alternating partitions, 100-round stress tests, mixed faults across rounds
- Safety assertion: no two honest nodes decide different blocks for the same round
- Liveness assertion: progress with up to f Byzantine faults, stall below threshold

**Parallel Transaction Execution (Phase 2)**
- `transaction::parallel` — conflict detector (RAW/WAW/WAR), dependency graph, wave scheduler with longest-path topological sort
- `transaction::executor` — wave-parallel block executor: MVCC validate + apply writes per wave, deterministic ordering within waves, legacy format adapter
- `Gateway.commit_block_parallel()` — batch commit integrating ordering, parallel execution, block persistence, and event emission; returns `BatchTxResult` with parallelism metrics
- 28 new tests (15 parallel + 9 executor + 4 gateway integration), 30 total gateway tests

**Protocol-Native Tokenomics (Phase 3)**
- `tokenomics::economics` — 100M NOTA supply cap, halving issuance curve (50→25→12...), capped block rewards, 80/20 fee split (burn/proposer), EIP-1559 dynamic base fee, epoch tracking, `process_block()` state machine
- `tokenomics::storage_deposit` — `DepositLedger` for lock/refund lifecycle: proportional to data size, min deposit floor, delta refund on updates, full refund on delete
- 44 new tests (24 economics + 20 storage deposit)

**Cross-Chain Bridge (Phase 4)**
- `bridge::types` — chain registry, message envelope with routing/sequencing, transfer records, inclusion proof structures
- `bridge::escrow` — `EscrowVault` for lock/release (outbound) and mint/burn (inbound) with multi-chain wrapped token balances
- `bridge::verifier` — Merkle tree builder and inclusion proof verification (SHA-256, power-of-two padding, tamper detection)
- `bridge::protocol` — `BridgeEngine` orchestrating chain registry, outbound initiate, inbound verify+mint, replay protection, confirmation threshold checks
- 43 new tests (5 types + 16 escrow + 11 verifier + 11 protocol)

**On-Chain Governance (Phase 5)**
- `governance::params` — typed parameter registry with protocol defaults (block size, fees, slashing, quorum, thresholds)
- `governance::proposals` — proposal lifecycle: submit with deposit → vote → pass/reject → timelock → execute/cancel, with status filtering and ID sequencing
- `governance::voting` — stake-weighted voting (Yes/No/Abstain), quorum check against total staked power, pass threshold on yes/(yes+no), abstain counts for quorum only, full governance integration test
- 34 new tests (7 params + 13 proposals + 14 voting including end-to-end flow)

**Concurrent Execution & Light Client**
- `transaction::executor::execute_block_concurrent()` — async tokio executor: spawns validation tasks per tx within each wave, applies writes deterministically after all validations complete
- `light_client::header` — compact `BlockHeader` (~300 bytes vs ~10 KB full block), `HeaderChain` with hash integrity and parent linkage verification
- `light_client::client` — `LightClient` with BFT header verification (CommitQC validation), state proof verification via Merkle proofs against synced headers, 100-header sync stress test
- 30 new tests (6 concurrent executor + 13 header + 11 client)

**Remaining Gap Closures: DPoS, Bridge E2E, TPS Benchmark**
- `consensus::dpos` — stake-weighted validator selection: committee election (filter, sort, top-N), stake-proportional leader rotation, voting power, 1000-candidate stress test
- `tests/bridge_e2e.rs` — 11 full-lifecycle tests: outbound lock/release, outbound refund, inbound verify/mint, inbound burn/return, multi-chain flows, replay attack, insufficient confirmations, invalid proof, 100-transfer stress (outbound + inbound)
- `tests/tps_benchmark.rs` — 6 throughput benchmarks: independent/contended/mixed workloads, sync vs concurrent parity, measured ~4.5K TPS (debug) for 500 independent txs in 1 wave

**Testnet Infrastructure**
- `testnet::config` — `GenesisConfig` with testnet/devnet/mainnet presets, initial allocations, validator set, DPoS params, validation rules
- `testnet::faucet` — rate-limited token faucet with cooldown, depletion tracking, unlimited mode for devnet
- 18 tests (8 genesis config + 10 faucet)

**EVM Compatibility Layer**
- `evm_compat::abi` — Solidity ABI encoding/decoding (uint256, address, bool, bytes, string), function selectors, DID-to-address derivation
- `evm_compat::precompile` — precompile interface (SHA-256, identity, ecrecover/ripemd160/modexp stubs), gas metering, rust-bc SHA-256 extension at 0x20
- 27 tests (15 ABI + 12 precompile)

**Channel Isolation & Chaincode Upgrade Lifecycle**
- `channel::store` — `ChannelStore` with per-channel world state and block ledger isolation, version independence, key prefixing (Fabric-compatible)
- `chaincode::upgrade` — `UpgradeManager` with multi-org approval lifecycle: propose→approve→commit, progress tracking, unauthorized/duplicate rejection, history
- 24 tests (11 channel store + 13 upgrade lifecycle)

**TPS Benchmark, Bridge Relayer, and Ecosystem Docs**
- `tests/full_benchmark.rs` — release-mode benchmarks: 56K TPS (500 independent), 39K TPS (1000 mixed), 100 BFT rounds/sec, full pipeline (BFT + exec + state)
- `bridge::relayer` — `Relayer` with job queue, batch processing, retry logic, replay protection, status tracking; 7 tests including 100-relay stress
- `docs/book/` — mdBook documentation site: introduction, quickstart, configuration, first dApp guide (Wasm + Python SDK + JS SDK), architecture/API/operations stubs

**Documentation**
- `docs/IOTA-GAP-ANALYSIS.md`: competitive gap analysis vs IOTA Rebased with suggested roadmap

### 2026-04-14

**Node**
- Fix infinite recursion in `Node::p2p_address()` when no announce address is set (fallback is `address`).
- RocksDB open now unions static CFs with `list_cf` on disk so dynamic families (e.g. `private_*` from private data) open without startup errors.

**Tooling**
- `docker-compose.yml`: removed obsolete top-level `version` (Compose v2).
- `scripts/try-it.sh`: local demo without Docker.
- `tests/fuzz_tests.proptest-regressions`: proptest regression seeds.

**Explorer**
- `block-explorer-vite/`: Vite + React UI for the HTTP API; dev server proxies API calls to the node (see `vite.config.ts` / `VITE_API_PROXY_TARGET`). Plain-language flows for identities and credentials.

---

### 2026-04-13 (Debug build stack overflow fix)

- `async_main` refactored into `async_main` + `async_main_inner` with `Box::pin` indirection
- The 1200-line async state machine now lives on the heap instead of the thread stack
- Fixes stack overflow that prevented `cargo run` (debug mode) from starting
- Stack size reduced from 64 MB back to 16 MB (sufficient with heap-allocated future)
- Release mode was unaffected (optimizations already collapsed the state machine)

### 2026-04-13 (E2E test suite compatibility fixes)

- Force HTTP/1.1 in e2e script to avoid HTTP/2 negotiation failures with rustls
- Prefer Homebrew curl (OpenSSL) over macOS system curl (LibreSSL) to fix `bad_record_mac` on POST requests
- E2E result: 104 passed, 0 failed across 26 categories

---

### 2026-04-12 (Security Hardening — P0/P1/P2)

**P0 — ACL enforcement on legacy routes**
- All 12 mutation endpoints in `api_legacy.rs` now call `enforce_acl` (mine, deploy, execute, connect, sync, stake, unstake, airdrop, wallet, nft metadata)
- `mine_block` verifies `miner_address` belongs to a registered wallet
- Debug `eprintln!("[DEPLOY]...")` replaced with `log::debug!`

**P1 — Double-spend and replay prevention**
- `is_double_spend` rewritten: matches by `tx.id` uniqueness across confirmed chain
- New `validate_timestamp` rejects transactions >30s in the future or >10min old
- Rate limiter: `/billing/create-key` no longer exempt; middleware logging via `log::debug!`
- Removed blanket `#![allow(dead_code)]` from `transaction_validation.rs`; per-item allows only

**P2 — Integrity and supply-chain hardening**
- Checkpoint files now include HMAC-SHA256 tag (env `CHECKPOINT_HMAC_SECRET`); tampered/legacy files skipped on load
- Chaincode install computes and logs SHA-256 of Wasm bytes; optional `expected_hash` query param for verification
- `jwt_secret` documented as reserved (not used for auth — mTLS + ACL is active)

**Tests:** 992 passed, 0 failed, 0 clippy warnings

---

### 2026-04-12 (Chaincode Install Fix)

- Input validation middleware now exempts `/chaincode/install` from the JSON-only Content-Type check, allowing `application/octet-stream` for Wasm binary uploads
- E2E test suite: 69 passed, 0 failed (previously 62 passed, 4 failed on chaincode lifecycle)

---

### 2026-04-11 (Audit Hardening)

**Wasmtime upgrade (v21 → v36)**
- Resolves 15 CVEs including sandbox escape, memory leaks, and host panics
- Rust toolchain updated to `nightly-2025-05-01` (1.88.0) for compatibility
- Removed `#![feature(unsigned_is_multiple_of)]` (stable since 1.87)

**Clippy clean pass**
- Zero warnings from `cargo clippy -- -D warnings`
- 199 `uninlined_format_args` auto-fixed for Rust 1.88 lint rules
- Removed crate-level `#![allow(dead_code, unused_imports)]` from `lib.rs` and `main.rs`
- 144 previously hidden warnings resolved: unused imports removed, dead code annotated per-item
- Removed file-level `#![allow(dead_code)]` from `chain_validation.rs`, `transaction_validation.rs`, `network_security.rs`

**Dependency CVE fix**
- `bytes` 1.11.0 → 1.11.1 (RUSTSEC-2026-0007, integer overflow in `BytesMut::reserve`)

---

### 2026-04-10 (Production Readiness — Final Gaps)

**3-node Raft ordering cluster**
- Docker Compose default changed from solo to 3-node Raft (`ORDERING_BACKEND=raft`)
- Orderer1/2/3 with `RAFT_NODE_ID` and `RAFT_PEERS` configured for automatic cluster formation
- Persistent Raft log per orderer (RocksDB at `STORAGE_PATH/raft/`)
- TLS certificates generated for all 3 orderers via `deploy/generate-tls.sh`

**Performance benchmarks published**
- `docs/BENCHMARKS-FULL.md` with Criterion measurements on Apple M-series
- Ordering: 23M tx/s (in-memory), endorsement: 45K/s (Ed25519), RocksDB: 104K blocks/s
- End-to-end pipeline estimate: 5K-15K tx/s on 3-node Raft LAN
- Comparison table with Hyperledger Fabric published TPS

**Chaincode SDK for Rust developers**
- `chaincode-sdk/` — Rust crate that compiles to Wasm for deployment on the blockchain
- API: `state_put`, `state_get`, `state_put_json`, `state_get_json`, `emit_event`, `set_key_policy`, `history_for_key`, `invoke` (cross-chaincode), `set_response`
- Example: `examples/asset_transfer.rs` — complete asset management contract (create, read, transfer, history)
- Compiles to `wasm32-unknown-unknown` target

---

### 2026-04-10 (Certification Readiness — Levels 1-3)

**Level 1 — Enterprise presentation readiness**
- MIT license added
- `JWT_SECRET` required in production (`RUST_BC_ENV=production` panics if missing or default)
- Signing key zeroization: Ed25519 via `ZeroizeOnDrop`, ML-DSA-65 via custom `Drop`
- Integration test fixed for PQC signature migration (`store_blocks_api_test.rs`)

**Level 2 — Third-party audit readiness**
- Property-based tests (proptest): 5 cases for Ed25519 + ML-DSA-65 sign/verify invariants
- Input validation middleware: Content-Type enforcement, max payload size (10 MB), wired at startup
- Vulnerability disclosure policy added to SECURITY.md (72h ack, 7-day fix timeline)
- Consensus threat model added to SECURITY.md (Raft, gossip, censorship attacks + mitigations)
- CI coverage gate: `cargo tarpaulin --fail-under 80`, test steps no longer soft-fail
- Production unwrap audit: single handler unwrap fixed in events.rs
- `docs/ENCRYPTION-AT-REST.md` — LUKS, Docker, cloud encryption guidance

**Level 3 — Formal certification preparation**
- FIPS 140-3 power-up self-tests (KAT): Ed25519, ML-DSA-65, SHA-256 run at startup; node refuses to start on failure
- `docs/FIPS-140-MODULE.md` — cryptographic module boundary, approved algorithms, key management, gap analysis
- `docs/COMPLIANCE-FRAMEWORK.md` — SOC 2 (13 criteria), ISO 27001 (17 Annex A controls), regulatory mapping (Chile CMF, EU eIDAS/GDPR, US FISMA)
- `docs/CERTIFICATION-ROADMAP.md` — three-level roadmap with items, effort, and audience per level

**Dependencies:** `zeroize` 1.7, `proptest` 1.4 (dev)

---

### 2026-04-10 (Fabric Parity Audit + Enterprise Documentation)

**Structural audit against Hyperledger Fabric**
- Verified full Fabric feature parity across 6 critical areas
- Channel ledger isolation confirmed: `StoreMap` (per-channel `HashMap<String, Arc<dyn BlockStore>>`) used by all store handlers via `channel_id_from_req()` + `get_channel_store()`
- Private data dissemination confirmed: selective push to member peers via discovery service, membership validation on receive, `PrivateDataAck` responses
- Chaincode lifecycle confirmed: `Installed → Approved → Committed` state machine with per-org approval tracking and endorsement policy evaluation on commit
- Pull state sync confirmed: `StateRequest`/`StateResponse` messages, anti-entropy gap detection via alive message heights
- WebSocket events confirmed: `actix-ws` upgrade, `EventBus` subscription, channel/chaincode filtering, historical replay, client ack tracking

**Fix: proposals handler channel scoping**
- `POST /api/v1/proposals` now persists transactions to the channel-scoped store (was hardcoded to `"default"`)

**Enterprise documentation**
- `docs/ENTERPRISE.md` — Platform overview for enterprise evaluation (architecture, privacy, consensus, chaincode, endorsement policies, PQC, operations, use cases, Fabric comparison)
- `docs/PQC-ENTERPRISE.md` — Post-quantum cryptography positioning document for the Chamber (NIST FIPS 204 compliance, Fabric comparison, regulatory alignment, deployment model)

---

### 2026-04-10 (Post-Quantum Cryptography — FIPS 204)

**ML-DSA-65 signing provider**
- `MlDsaSigningProvider` implements `SigningProvider` using ML-DSA-65 (FIPS 204, NIST security level 3)
- Keypair generation, signing (3309-byte signatures), and verification via `pqcrypto-mldsa`
- `from_keys(pk, sk)` constructor for restoring providers from persisted key material

**Generalized `SigningProvider` trait**
- Signatures and public keys changed from fixed-size arrays to `Vec<u8>` / `&[u8]`
- New `algorithm()` method returns `SigningAlgorithm` enum (`Ed25519` or `MlDsa65`)
- `SoftwareSigningProvider` (Ed25519) and `HsmSigningProvider` adapted to the new trait

**Variable-length signatures across the stack**
- `Endorsement.signature`: `[u8; 64]` → `Vec<u8>`
- `Block.signature` and `Block.orderer_signature`: `[u8; 64]` → `Vec<u8>`
- `DagBlock.signature`: `[u8; 64]` → `Vec<u8>`
- `TransactionProposal.creator_signature`: `[u8; 64]` → `Vec<u8>`
- `AliveMessage.signature` (gossip): `[u8; 64]` → `Vec<u8>`
- All hex serde helpers updated for variable-length byte vectors

**Runtime algorithm selection**
- `SIGNING_ALGORITHM` env var: `ed25519` (default), `ml-dsa-65` / `mldsa65`
- Logged at startup; unknown values fall back to Ed25519 with a warning

**Legacy transaction verification**
- `Transaction.verify_signature()` auto-detects Ed25519 or ML-DSA-65 by key/signature size

**Dependencies:** `pqcrypto-mldsa` 0.1.2, `pqcrypto-traits` 0.3

---

### 2026-04-07 (Fabric Gap Closure)

**Persistent Raft log (crash-tolerant ordering)**
- `RocksDbRaftStorage` implements `raft::Storage` trait with RocksDB
- Entries, HardState, ConfState, and Snapshots persist to `{STORAGE_PATH}/raft/`
- `RaftNode::new_persistent()` loads state from disk on boot, flushes after each advance
- Each Docker orderer is an independent process with its own persistent Raft DB
- Process crash + restart recovers full Raft state and re-integrates to cluster

**X.509 MSP enforcement**
- `TlsIdentityMiddleware` extracts CN/O from mTLS client certificates via `x509-parser`
- `on_connect` captures DER peer certs from rustls `ServerConnection`
- `enforce_acl` uses TLS identity as authoritative source, headers as fallback
- Role inference from CN: "admin" → Admin, "peer"/"orderer" → Peer, else → Client

---

### 2026-04-07 (Post-MVP — Block 3)

**External chaincode (chaincode-as-a-service)**
- `ChaincodeDefinition.runtime` field: `Wasm` (default) or `External { endpoint, tls }`
- Simulate handler dispatches to `ExternalChaincodeClient` for external runtime
- HTTP POST to `{endpoint}/invoke` with JSON body

**TLS Identity Middleware**
- `TlsIdentityMiddleware` extracts CN/O from `X-TLS-Client-CN`/`X-TLS-Client-O` headers
- Inserts `TlsIdentity` into request extensions for downstream handlers
- Compatible with TLS-terminating proxies

**HSM signing (feature-gated)**
- `#[cfg(feature = "hsm")]` sign/verify paths on `HsmSigningProvider`
- Verify uses `ed25519_dalek` with cached public key
- Sign path documented for PKCS#11 `C_Sign` (requires hardware testing)

**Already complete (preexisting)**
- Hot certificate rotation — SIGHUP + periodic reload already implemented
- Block explorer UI — Next.js app in `block-explorer/`
- CouchDB world state — `WorldState` trait fully implemented in `storage/couchdb.rs`

---

### 2026-04-07 (MVP Readiness)

**Graceful shutdown**
- SIGTERM/SIGINT handler via `tokio::signal` — drains HTTP connections, aborts background tasks, flushes RocksDB

**Persistent service stores**
- 8 of 9 services now persist to RocksDB when `STORAGE_BACKEND=rocksdb`
- New CF impls: `PolicyStore`, `CollectionRegistry`, `ChaincodeDefinitionStore`
- Added serde derives to `PrivateDataCollection`, `ChaincodeDefinition`, `ChaincodeStatus`
- Single shared `Arc<RocksDbBlockStore>` instance for all services
- Explicit failure: node exits if `STORAGE_BACKEND=rocksdb` and DB fails to open (no silent fallback)

**Health check with dependency verification**
- `/api/v1/health` now reports `checks: { storage, peers, ordering }`
- Returns `"degraded"` when storage or ordering is unavailable

**JS/TS SDK — Fabric-style operations**
- New methods: `submitTransaction`, `evaluate`, `registerOrg`, `setPolicy`, `createChannel`, `listChannels`, `putPrivateData`, `getPrivateData`

**Mutex poison recovery**
- Replaced 178 `.lock()/.read()/.write().unwrap()` with `unwrap_or_else(|e| e.into_inner())`
- Prevents cascading panics across threads from poisoned locks

**Documentation**
- `docs/QUICK-START.md` — git clone to first transaction in < 5 minutes
- `docs/API-REFERENCE.md` — all 68 endpoints with curl examples
- `docs/DEPLOYMENT.md` — production config, env vars, security checklist
- `docs/MVP-ROADMAP.md` — task-level breakdown for MVP delivery

---

### 2026-04-07 (CI Stabilization)

**Docker TLS permissions**
- `deploy/generate-tls.sh` now runs `chmod 644` on generated `.pem` files so the non-root container user (`rustbc`, uid 1000) can read them through the read-only `/tls` volume mount

**E2E test resilience**
- Grafana health check skipped when Grafana is not running (CI only starts blockchain nodes)
- Channel membership test asserts "not 403" instead of exact 200, isolating membership enforcement from downstream endorsement errors
- `POST /api/v1/store/transactions` now returns `status_code: 201` in the JSON envelope to match the HTTP 201 Created status

**Flaky Raft test fix**
- `three_nodes_in_process_propose_committed_on_all` routing rounds increased from 30 to 50, accommodating worst-case Raft election timeout randomisation on slow CI runners

**CI status:** all 4 jobs green (Check + Clippy, Build CLI, Unit Tests, E2E Tests)

---

### 2026-04-07 (Production Hardening)

**ACL deny-by-default**
- `enforce_acl()` now denies requests with missing identity, missing ACL infrastructure, or undefined ACL entries
- New env var `ACL_MODE=permissive` restores the old allow-all behavior for local development
- `enforce_channel_membership()` denies requests without `X-Org-Id` on non-default channels (strict mode)

**JWT secret from environment**
- `ApiConfig` reads `JWT_SECRET` env var at startup; falls back to hardcoded default only if unset

**CouchDB async client**
- Replaced `reqwest::blocking::Client` with async `reqwest::Client` in `CouchDbWorldState`
- Sync `WorldState` trait bridged via `block_in_place` + `Handle::block_on` (no runtime deadlock)
- Same fix applied to `ExternalInvoker` in `src/chaincode/invoker.rs`

**Configurable P2P buffer sizes**
- `P2P_RESPONSE_BUFFER_BYTES` — `send_and_wait` responses (default 256 KB, was 64 KB)
- `P2P_HANDLER_BUFFER_BYTES` — per-connection message handler (default 64 KB, was 8 KB)
- `P2P_SYNC_BUFFER_BYTES` — pull-based state sync responses (default 4 MB, was 1 MB)

---

### 2026-04-06 (E2E Tests, Operator Tooling, Full Service Wiring & Gap Analysis)

**All scaffold services wired to startup**
- `org_registry`, `policy_store`, `discovery_service`, `private_data_store`, `collection_registry`, `chaincode_package_store`, `chaincode_definition_store`, `gateway` initialized in `main.rs`
- `POST /api/v1/private-data/collections` endpoint added for collection registration

**Route registration fix**
- `ApiRoutes::register()` uses `.configure()` closures to break the generic type chain and prevent stack overflow from deeply nested Actix wrappers
- `ApiRoutes::configure()` kept for integration tests, `configure_metrics()` for production
- Main thread spawned with 32 MB stack to accommodate release + debug builds

**E2E test suite** (`scripts/e2e-test.sh`) — 42 pass, 0 fail, 0 skip
- Organizations, endorsement policies, channel isolation
- Block mining with multi-node propagation
- Transaction lifecycle (wallet → mempool → mine → block)
- Private data (register collection → write → read authorized → deny unauthorized)
- Discovery (register peers → query endorsers → query channel peers)
- Gateway (endorse → order → commit pipeline)
- Chain integrity, Prometheus metrics, Grafana health
- Store CRUD (identities, credentials)

**Operator CLI** (`scripts/bcctl.sh`)
- 14 commands: `status`, `peers`, `blocks`, `mine`, `wallet create`, `channels`, `channel create`, `orgs`, `logs`, `restart`, `metrics`, `verify`, `consistency`, `env`

**Fabric 2.5 gap analysis** (`FABRIC-GAP-ANALYSIS.md`)
- Detailed comparison: 12 verified E2E categories, 10 implemented-but-untested features, gaps vs Fabric
- Research-backed task backlog with code change requirements, blockers, and exact E2E steps
- Key findings: Raft is in-process only (no network transport), MVCC not wired to gateway, install doesn't create chaincode definition, world_state not initialized for snapshots

---

### 2026-04-05 (Docker & P2P Networking)

**Docker deployment**
- Multi-stage `Dockerfile` (nightly Rust builder + `debian:bookworm-slim` runtime)
- `docker-compose.yml`: 3 peers + 1 orderer + Prometheus + Grafana
- Self-signed TLS via `deploy/generate-tls.sh` (EC P-256, per-node SANs)
- Non-root container user, named volumes for persistence

**Network fixes for containerized nodes**
- `BIND_ADDR` env var for HTTP listen address (default `127.0.0.1`, containers use `0.0.0.0`)
- `P2P_EXTERNAL_ADDRESS` env var for announce address (e.g. `node1:8081`)
- `Node::p2p_address()` helper replaces 8 hardcoded `self.address` formats
- P2P TLS acceptor now configured on the server node (was missing)
- Fixed `TLS_CA_CERT_PATH` env var name in compose (was `TLS_CA_PATH`)

**Route unification**
- Merged legacy and scaffold into a single `/api/v1` scope
- `ApiRoutes::register()` appends scaffold sub-services into the legacy scope
- `ApiRoutes::configure()` retained for integration tests (standalone scope)
- `ApiRoutes::configure_metrics()` used in production (metrics only)
- `health`, `version`, `openapi.json` registered as `.route()` in the legacy scope

**E2E verified**
- 4 nodes healthy, 3 peers each via mutual TLS
- Block mining on node1 propagates to node2/node3 within seconds
- 2020 unit/integration tests passing

---

### 2026-04-04 (Fase 19 — Snapshots + Pagination)

**19.1 — State snapshots**
- `StateSnapshot` metadata struct in `src/storage/snapshot.rs`
- `create_snapshot()`: serializes world state to `{key}\t{version}\t{base64}\n` format with SHA-256 hash
- `restore_snapshot()`: reads `.snap` file, restores world state, verifies hash integrity
- API handlers: `POST /snapshots/{channel_id}`, `GET /snapshots/{channel_id}`, `GET /snapshots/{channel_id}/{id}`
- `AppState.world_state` field added; `base64 = "0.22"` dependency added

**19.2 — State regeneration**
- `regenerate_state()`: replays all blocks from store to rebuild world state

**19.3 — Pagination**
- `PaginationParams` (page/limit/cursor) and `PaginatedResponse<T>` in `src/api/pagination.rs`
- `BlockStore::list_blocks(offset, limit)` with default implementation
- `GET /store/blocks` now accepts `?page=N&limit=M` and returns `PaginatedResponse`

---

### 2026-04-04 (Fase 18 — Delivery Service)

**18.1 — DeliverFiltered**
- `FilteredBlock` and `FilteredTx` structs in `src/events/filtered.rs`
- `to_filtered_block()` strips payload/rwset/endorsements, keeps only tx IDs and validation codes
- `GET /events/blocks/filtered` WebSocket streams `FilteredBlock` summaries

**18.2 — DeliverWithPrivateData**
- `BlockWithPrivateData` struct in `src/events/private_delivery.rs`
- `GET /events/blocks/private` WebSocket with `X-Org-Id` header for collection membership filtering
- `CollectionRegistry::list()` method added for iterating registered collections

**18.3 — Replay and checkpoints**
- `start_block` field in `WsFilter`: replays historical blocks before switching to live
- `ack` + `client_id` checkpoint system: server tracks last acked height per client
- Reconnect with same `client_id` resumes from `last_ack + 1`

---

### 2026-04-04 (Fase 17 — Key History + Chaincode-to-Chaincode)

**17.1 — Key history**
- `HistoryEntry` struct in `storage/traits.rs`
- CF `key_history` in RocksDB with `{key}\x00{version:012}` key schema
- `get_history` method on `WorldState` trait, implemented for Memory and RocksDB
- `put()` and `delete()` auto-append history entries in `MemoryWorldState`
- `get_history_for_key` host function in `WasmExecutor`

**17.2 — Chaincode-to-chaincode invocation**
- `ChaincodeResolver` trait + `StoreBackedResolver` in `src/chaincode/resolver.rs`
- `invoke_chaincode` host function: resolves target, creates child executor, shares `WorldState`
- ACL check via `AclProvider` before cross-chaincode calls (`chaincode/{id}/invoke`)
- `MAX_CHAINCODE_DEPTH=8` recursion limit with depth counter propagation
- `ChaincodeError::NotFound` variant added

---

### 2026-04-04 (Fase 16 — Gossip Protocol Enhancement)

**16.1 — Alive messages**
- `AliveMessage` struct in `src/network/gossip.rs` with Ed25519 signature verification
- `Alive(AliveMessage)` variant in the P2P `Message` enum
- `MembershipTable`: thread-safe peer liveness tracking with suspect sweep
- `start_alive_loop` on `Node`: periodic broadcast + suspect detection
- Refactored `src/network.rs` → `src/network/mod.rs` + `gossip.rs` module

**16.2 — Pull-based state sync**
- `StateRequest { from_height }` and `StateResponse { blocks }` message variants
- `STATE_BATCH_SIZE` (50) caps response payload
- `start_pull_sync_loop` on `Node`: periodic height comparison + block fetch
- Anti-entropy: `latest_height` field on `AliveMessage`, `peers_ahead_of` gap detection

**16.3 — Anchor peers**
- `AnchorPeer` struct with `parse_anchor_peers` from `ANCHOR_PEERS` env var
- `connect_to_anchor_peers` runs before bootstrap for cross-org discovery
- `anchor_peers_from_config` bridges `ChannelConfig.anchor_peers` map to gossip

**16.4 — Leader election per org**
- `LeaderElectionMode` enum (`Static` / `Dynamic`) from `LEADER_ELECTION` env var
- `elect_leader(org_id)`: smallest alive peer address wins; failover on suspect

39 network tests passing.

---

### 2026-04-04 (Fase 15 — Raft Consensus Ordering)

**15.1 — Raft core**
- `RaftNode` in `src/ordering/raft_node.rs`: wrapper over tikv `RawNode<MemStorage>`
- `new`, `tick`, `propose`, `step`, `advance` methods
- Full raft 0.7 ready cycle: handles `messages()` (leader) and `persisted_messages()` (candidate/follower) correctly
- `create_snapshot` / `apply_snapshot` for node catch-up
- 8 tests: init, election, 3-node leader election, propose+commit, 5-entry replication, snapshot transfer

**15.2 — Raft ordering service**
- `RaftOrderingService` in `src/ordering/raft_service.rs`: JSON-serialized TX proposals through raft, committed entry draining with no-op filtering
- `OrderingBackend` trait in `src/ordering/mod.rs` with `submit_tx`, `cut_block`, `pending_count`
- Implemented by both `OrderingService` (solo) and `RaftOrderingService` (raft)
- Backend selection via `ORDERING_BACKEND=raft|solo` env var; `RAFT_NODE_ID`, `RAFT_PEERS` for raft config
- `AppState.ordering_backend: Option<Arc<dyn OrderingBackend>>` added
- 6 tests: 3 raft service + 2 trait object + 1 batch size

**15.3 — Raft network transport**
- `Message::RaftMessage(Vec<u8>)` variant added to P2P `Message` enum
- `src/ordering/raft_transport.rs`: prost encode/decode, `tick_and_collect`, `deliver_raw`
- `prost` dependency aligned to 0.11 (matches raft-proto)
- 3 tests: serde roundtrip, encode/decode roundtrip, 3-node in-process replication through serialized bytes

**15.4 — Orderer block signing**
- `Block.orderer_signature: Option<[u8; 64]>` with `#[serde(default, skip_serializing_if)]`
- `sign_block(block, key)`: `sha256(height || parent_hash || merkle_root)` signed with Ed25519
- `verify_orderer_signature(block, verifying_key)`: `Ok(true)` valid, `Ok(false)` absent, `Err` invalid
- Both backends sign when `with_signing_key(key)` is set
- 4 tests: sign+verify, valid accept, invalid reject, absent accept

---

### 2026-04-04 (Fase 12 — Hardening · §12.3 Benchmarks)

**12.3.1–12.3.3 — Criterion benchmarks** (`benches/ordering_throughput.rs`)
- `ordering_service/submit_and_cut/100` — throughput de ordering: 100 TXs → 1 bloque; reporta TXs/s
- `endorsement_validation/validate_endorsements/{1,3,5,10}` — latencia por endorsement Ed25519 con política `AllOf(N)`
- `event_bus_fanout/publish_1_event/{1,5,10,50}` — costo de `publish()` con N suscriptores activos en canal broadcast
- `criterion = "0.5"` añadido a `[dev-dependencies]`; informes HTML en `target/criterion/`

---

### 2026-04-04 (Fase 7 — Private Data Collections · §7.1.3)

**7.1.3 — Purge de datos expirados**
- `put_private_data_at(collection, key, value, written_at_height, blocks_to_live)` añadido al trait `PrivateDataStore` — default no-op delegando a `put_private_data` para backwards compat
- `purge_expired(current_height)` en el trait con default no-op; `MemoryPrivateDataStore` elimina entradas donde `written_at + blocks_to_live <= current_height`
- Entradas sin TTL (`blocks_to_live = 0`) nunca expiran
- 5 tests: expiración exacta en altura 6, sin expirar antes, purge selectivo (corto vs largo TTL), sin-TTL inmortal, `blocks_to_live=0`

---

### 2026-04-03 (Fase 9 — Fabric Gateway)

**9.1.1 — `Gateway` struct**
- `src/gateway/mod.rs`: campos `org_registry`, `policy_store`, `ordering_service`, `store`
- `mod gateway` declarado en `lib.rs` y `main.rs`
- 3 tests: crear con mocks, store vacío, policy store vacío

**9.1.2 — `Gateway::submit`**
- Pipeline: consulta policy → self-endorse → `ordering_service.submit_tx` → `cut_block` → `store.write_block`
- `TxResult { tx_id, block_height }` como tipo de retorno
- `GatewayError`: `PolicyNotSatisfied`, `Ordering`, `Storage`
- 4 tests: sin policy, `AnyOf` satisfecha, policy no satisfecha, alturas secuenciales

**9.1.3 — `POST /api/v1/gateway/submit`**
- Handler `gateway_submit` en `src/api/handlers/gateway.rs`
- Acepta `{ chaincode_id, transaction: { id, input_did, output_recipient, amount } }`
- Devuelve `{ tx_id, block_height }`; 404 si gateway no configurado; 400 si campos vacíos
- `gateway: Option<Arc<Gateway>>` añadido a `AppState`
- 3 tests HTTP: 200 end-to-end, 404 sin gateway, 400 con campos vacíos

**Total tests: 1470**

---

### 2026-04-03 (Fase 8 — Chaincode Lifecycle · §8.3 Wasm execution)

**8.3.4 — Memory limit**
- `WasmExecutor::with_memory_limit(max_bytes)` builder method
- `StoreLimitsBuilder::memory_size` + `store.limiter()` activan el límite por invocación
- Módulo que pide más páginas de las permitidas falla en instanciación → `ChaincodeError::Execution`
- 2 tests: exceder límite → error, dentro del límite → ok

**8.3.3 — Host functions `put_state` / `get_state`**
- `WasmExecutor::invoke(state, func_name) -> Result<Vec<u8>>`
- ABI: la función Wasm devuelve `i64 = (ptr << 32 | len)`; el host lee `memory[ptr..ptr+len]`
- Imports `env::put_state` y `env::get_state` enlazan la memoria Wasm con `WorldState`
- 2 tests: put→get devuelve `"1"`, estado persistido en `WorldState`

**8.3.2 — `WasmExecutor`**
- `src/chaincode/executor.rs`: `WasmExecutor { engine, module, fuel_limit }`
- Constructor compila Wasm con fuel metering (`Config::consume_fuel(true)`)
- `ChaincodeError::Execution(String)` añadido al enum
- 3 tests: wasm válido ok, fuel_limit guardado, wasm inválido → error

**8.3.1 — Dependencia wasmtime**
- `wasmtime = "21"` añadido a `Cargo.toml`

---

### 2026-04-03 (Fase 7 — Private Data Collections)

**7.2.1 — Access control en handlers de private data**
- `CollectionRegistry` trait + `MemoryCollectionRegistry` en `src/private_data/mod.rs`
- `ApiError::Forbidden` → HTTP 403
- `PUT/GET /api/v1/private-data/{collection}/{key}` en `src/api/handlers/private_data.rs`
- Header `X-Org-Id` obligatorio; `check_membership` verifica org en `member_org_ids` de la collection
- `AppState`: campos `private_data_store` y `collection_registry`
- 6 tests nuevos (member → 200, non-member → 403, sin header → 400, clave ausente → 404)

**7.1.2 — RocksDB private data store**
- `PrivateDataStore` trait + `MemoryPrivateDataStore`; impl para `RocksDbBlockStore` con CF `private_{name}` dinámica
- Helper `sha256` para hash on-chain; DB migrada a `DBWithThreadMode<MultiThreaded>`

**7.1.1 — PrivateDataCollection struct**
- `PrivateDataCollection { name, member_org_ids, required_peer_count, blocks_to_live }` + `is_member()`
- `PrivateDataError`: `InvalidCollection`, `AccessDenied`
- 634 lib + 535 integration tests al cierre de 7.2.1

---

### 2026-04-03 (Fase 3 — Transaction Lifecycle)

**Transaction — Fase 3.1: Read-Write Sets**
- `src/transaction/mod.rs` + `rwset.rs`: `KVRead { key, version }`, `KVWrite { key, value }`, `ReadWriteSet { reads, writes }` con `is_empty()`
- Serde derive en los tres tipos; módulo declarado en `lib.rs` y `main.rs`
- 6 tests nuevos; 531 tests en total

---

### 2026-04-03 (Fase 1–2 — Endorsement + Ordering)

**Endorsement (Fase 1) — completa**
- `src/endorsement/`: `Organization`, `OrgRegistry` trait + `MemoryOrgRegistry`, CF `organizations` en RocksDB
- `EndorsementPolicy` (AnyOf / AllOf / NOutOf / And / Or) + `evaluate()`
- `PolicyStore` trait + `MemoryPolicyStore`
- `Endorsement` struct + `verify_endorsement` + `validate_endorsements`
- `Block.endorsements: Vec<Endorsement>` (serde default)
- `ConsensusEngine::with_policy_store()`: valida endorsements antes de insertar en DAG
- REST: `POST/GET /api/v1/store/organizations`, `GET /api/v1/store/organizations/{id}`, `POST/GET /api/v1/store/policies/{resource_id}`
- `AppState`: `org_registry`, `policy_store`

**Ordering (Fase 2) — completa**
- `src/ordering/`: `NodeRole` enum (Peer / Orderer / PeerAndOrderer) + `FromStr` desde `NODE_ROLE` env
- `OrderingService`: cola `VecDeque<Transaction>`, `submit_tx`, `cut_block` con batch drain
- `run_batch_loop`: tokio task lanzada en `main.rs` si el nodo ordena
- `Node.role: NodeRole`; `Message::SubmitTransaction` y `Message::OrderedBlock`
- `process_message`: orderer ingesta TXs; peer persiste `OrderedBlock` directamente en store
- 525 tests al cierre de Fase 2

---

### 2026-04-03 (Storage)

**Storage — secondary index endpoint**
- `GET /api/v1/store/blocks/{height}/transactions` — queries `transactions_by_block_height` via prefix scan on `tx_by_block` CF

**Storage — secondary index `tx_by_block`**
- New `tx_by_block` CF in RocksDB; key schema `{012-padded-height}:{tx_id}` → empty value
- `write_transaction` and `write_batch` write index entry atomically in the same `WriteBatch`
- `BlockStore::transactions_by_block_height(height)` added to trait; delegated in `Arc<T>` blanket impl
- `MemoryStore`: equivalent linear scan over the HashMap
- 9 new tests (key format, empty result, filtering, no height bleed-over, batch indexing); 463 tests total

**Storage — Fase VI: `MemoryStore` + `Arc<T>` blanket impl**
- `Arc<T: BlockStore>` implements `BlockStore` — lets `Arc<MemoryStore>` be used as `Box<dyn BlockStore>`
- `ConsensusEngine::with_store()` persists accepted blocks into the store

**Storage — Fase V: store-backed REST endpoints**
- `POST/GET /api/v1/store/transactions/{tx_id}`
- `POST/GET /api/v1/store/identities/{did}`
- `POST/GET /api/v1/store/credentials/{cred_id}`
- All handlers return 404 when store is not configured

**Storage — Fase IV: RocksDB Column Families**
- 5 CFs: `blocks`, `transactions`, `identities`, `credentials`, `meta`
- `create_missing_column_families(true)` — compatible with new and existing DBs
- Block keys: zero-padded 12-digit decimal for lexicographic = numeric ordering
- 17 tests: per-type roundtrip, CF isolation, reopen with persisted data

**Storage — Fase III: switcheable backend**
- `STORAGE_BACKEND=rocksdb` → `RocksDbBlockStore` at `STORAGE_PATH`; default → `MemoryStore`
- Fallback to `MemoryStore` if RocksDB fails to open

**Storage — Fase II: RocksDB**
- `RocksDbBlockStore`: JSON serialization, atomic `WriteBatch`, `META:latest_height` tracking
- `rocksdb = "0.22"` added to `Cargo.toml`
- 13 unit tests with `tempfile::TempDir`

**Storage — Fase I: MemoryStore + API**
- `MemoryStore`: `BlockStore` backed by `HashMap` + `Mutex`
- `AppState.store: Option<Arc<dyn BlockStore>>`
- `GET /api/v1/store/blocks/{height}` and `/store/blocks/latest`

**Consensus — Fase H: ConsensusEngine**
- `ConsensusEngine`: wraps `Dag`, `ForkChoice`, and `SlotScheduler`
- `accept_block()` validates and inserts; `canonical_tip()` / `canonical_chain()` query state
- `ConsensusError` typed errors via `thiserror`
- 11 tests

**Consensus — Fase G: Fork Resolution**
- `Dag::subtree_weight()`, `canonical_chain()`, `resolve_fork()`
- `ForkChoiceRule`: `HeaviestSubtree` (default) and `LongestChain`
- 33 tests (22 dag, 11 fork_choice)

**TLS — Fase C: Certificate Pinning**
- `CertPinConfig`: SHA-256 fingerprint allowlist; disabled when empty
- `PinningServerCertVerifier` / `PinningClientCertVerifier`: verify CA first, then fingerprint
- `TLS_PINNED_CERTS` env var (comma-separated); absent = pinning off
- `docs/NETWORK_MEMBERSHIP.md`: pinning section with rotation guide
- 32 TLS tests total

**TLS — Fase B: mTLS**
- `build_server_config_mtls` / `build_client_config_mtls`
- `TLS_MUTUAL=true` + `TLS_CA_CERT_PATH`; explicit error if CA missing
- 2 P2P integration tests (valid handshake, server rejects client without cert)

**TLS — Fase A: TLS básico**
- `src/tls.rs`: PEM loading, `ServerConfig`, `ClientConfig`, `PeerVerification` enum
- `TLS_CERT_PATH`, `TLS_KEY_PATH`, `TLS_VERIFY_PEER`, `TLS_CA_CERT_PATH`
- P2P connections wrapped in `TlsAcceptor` / `TlsConnector`
- Dependencies: `rustls 0.23`, `rustls-pemfile 2`, `tokio-rustls 0.26`, `webpki-roots 0.26`

**CI**
- Added `toolchain: stable` to all GitHub Actions workflows (required by `dtolnay/rust-toolchain@master`)

### Changed
- Docs reorganized: `ANALYSIS/` → `docs/analysis/`, `Documents/` → `docs/archive/`
- Stopped tracking local `blockchain_blocks/` sample data

