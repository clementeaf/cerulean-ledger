# rust-bc vs IOTA Rebased — Gap Analysis

> Last updated: 2026-04-16 (post Phase 5 + extras)
>
> Objective: Track competitive gaps between rust-bc and IOTA Rebased.

---

## IOTA Rebased Summary

IOTA pivoted in late 2024 from its original Tangle (feeless DAG) and abandoned IOTA 2.0 (7 years of development) after a community governance vote. **IOTA Rebased** launched on mainnet May 6, 2025.

| Component | Detail |
|-----------|--------|
| **VM** | MoveVM (object-oriented, resource-based, shared lineage with Sui) |
| **Consensus** | Mysticeti — BFT with uncertified DAG, Delegated Proof-of-Stake |
| **Throughput** | 50,000+ TPS, sub-second finality |
| **Execution** | Parallel (non-conflicting transactions execute concurrently) |
| **Fees** | Low (~0.005 IOTA/tx), burned (deflationary) |
| **EVM** | Available on L2; L1 integration planned |
| **Interoperability** | 150+ blockchain networks connected |
| **Token supply** | 4.6B initial, no hard cap, ~6% annual inflation, fee burning |
| **Staking** | DPoS, 10-15% APY, 150 max validators, 2M IOTA min stake |
| **Next consensus** | Starfish — reduces latency degradation under Byzantine behavior |

---

## Completed Phases

### Phase 1 — BFT Consensus ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Protocol** | Mysticeti (leaderless DAG BFT) | HotStuff-inspired (leader-based, 3-phase) |
| **Fault tolerance** | f = (n-1)/3 | f = (n-1)/3 |
| **Backend selection** | DPoS only | Raft (CFT) or BFT selectable via `CONSENSUS_MODE` |

**Implemented:** `consensus::bft/` — types, quorum validation (2f+1), vote collector, round state machine, round manager (leader rotation, exponential backoff), P2P message types, `ConsensusEngine.with_bft()`, `DagBlock.commit_qc`.

**Verified:** 147 unit tests + 16 adversarial E2E tests (4/7/10-node networks, equivocation, crash faults, silent leaders, partitions, 100-round stress).

### Phase 2 — Parallel Transaction Execution ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Model** | Object-centric (Sui-style), automatic | Wave scheduling via RW set conflict analysis |
| **Parallelism** | At VM level | At batch level (waves) + intra-wave tokio tasks |

**Implemented:** `transaction::parallel` — conflict detector (RAW/WAW/WAR), dependency graph, wave scheduler. `transaction::executor` — synchronous wave executor + `execute_block_concurrent()` with tokio `spawn_blocking` per tx within each wave. `Gateway.commit_block_parallel()` integrates ordering, execution, persistence, and events.

**Verified:** 15 conflict tests + 15 executor tests (sync + async, 100-tx stress, sync/async result parity) + 4 gateway integration tests.

### Phase 3 — Protocol-Native Tokenomics ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Supply** | 4.6B, no hard cap, ~6% inflation | 100M hard cap, halving issuance (50→25→12...) |
| **Fees** | Burned (deflationary) | 80% burned / 20% proposer, EIP-1559 dynamic base fee |
| **Storage** | Storage deposits | Storage deposits (proportional to data size) |

**Implemented:** `tokenomics::economics` — `MAX_SUPPLY`, halving curve, capped rewards, fee split, dynamic base fee (adjusts ±12.5% per block by utilization), epoch tracking, `process_block()` state machine. `tokenomics::storage_deposit` — `DepositLedger` (lock/refund/update lifecycle).

**Verified:** 24 economics tests + 20 storage deposit tests (lifecycle, stress 1000 deposits).

### Phase 4 — Cross-Chain Bridges ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Connectivity** | 150+ chains in production | Bridge framework with chain registry, escrow, proof verification |
| **Status** | Production | Infrastructure ready, no live chain connected yet |

**Implemented:** `bridge::types` — chain registry, message envelope, transfer records. `bridge::escrow` — `EscrowVault` (lock/release outbound, mint/burn inbound, multi-chain wrapped tokens). `bridge::verifier` — SHA-256 Merkle tree builder + inclusion proof verification. `bridge::protocol` — `BridgeEngine` (chain registry, transfer initiation, inbound verify+mint, replay protection, confirmation thresholds).

**Verified:** 5 type tests + 16 escrow tests + 11 verifier tests (tamper, 1000-leaf stress) + 11 protocol tests (unknown chain, inactive, confirmations, invalid proof, replay).

### Phase 5 — On-Chain Governance ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Governance** | On-chain voting | Proposals, stake-weighted voting, timelock execution |

**Implemented:** `governance::params` — typed `ParamRegistry` with 10 protocol defaults. `governance::proposals` — `ProposalStore` (submit with deposit → vote → pass/reject → timelock → execute/cancel). `governance::voting` — `VoteStore` (Yes/No/Abstain, quorum against total staked power, pass threshold on yes/(yes+no), abstain counts for quorum only).

**Verified:** 7 param tests + 13 proposal tests + 14 voting tests (including end-to-end governance flow).

### Extra — Concurrent Execution + Light Client ✅

| Aspect | IOTA | rust-bc |
|--------|------|---------|
| **Intra-wave parallelism** | Native (VM-level) | tokio `spawn_blocking` per tx |
| **Light clients** | Supported | `LightClient` with BFT header verification + state proofs |

**Implemented:** `execute_block_concurrent()` — async tokio executor. `light_client::header` — compact `BlockHeader` (~300 bytes), `HeaderChain`. `light_client::client` — `LightClient` with BFT CommitQC verification and Merkle state proof verification.

**Verified:** 6 concurrent executor tests (async parity with sync) + 13 header tests + 11 client tests (BFT verification, state proofs, 100-header sync).

---

## Remaining Gaps (honest assessment)

| # | Gap | Impact | Detail |
|---|-----|--------|--------|
| 1 | **Real-world TPS** | High | 18.7K TPS benchmarked (single-node). Concurrent executor improves this but needs load testing under BFT with 4+ nodes. IOTA claims 50K+. |
| 2 | **Live bridge** | High | Bridge framework is complete but no chain is connected. IOTA has 150+ chains in production. Next step: connect Ethereum or Cosmos testnet. |
| 3 | **DPoS validator selection** | Medium | Staking exists but validator selection is not proportional to stake. IOTA has full DPoS with 150 validators. |
| 4 | **Developer ecosystem** | Medium | Python SDK + JS SDK + Vite explorer. IOTA has Move language, dApp Kit, mature docs. |
| 5 | **Public testnet** | Medium | Docker local only. Need a public testnet for community testing. |
| 6 | **EVM compatibility** | Low | IOTA has EVM on L2. rust-bc has Wasm (multi-language). Different tradeoff, not a direct gap. |

---

## Persistent Advantages (rust-bc over IOTA)

| Area | Detail |
|------|--------|
| **Post-Quantum Crypto** | ML-DSA-65 (FIPS 204) integrated end-to-end. IOTA has no PQC. Regulatory advantage growing in 2026-2027. |
| **Resource footprint** | ~50 MB/node vs 128 GB RAM per IOTA validator. 2500x lighter. Decisive for IoT and edge. |
| **Dual consensus** | Raft (enterprise) or BFT (semi-public) selectable via env var. IOTA is DPoS-only. |
| **Enterprise permissioned** | mTLS, ACL, org registry, endorsement policies, private data, Fabric-compatible pipeline. IOTA is public-only. |
| **Dynamic base fee** | EIP-1559 style fee adjustment by congestion. IOTA has static fees. |
| **Light client** | BFT-verified header chain + Merkle state proofs. Enables IoT/mobile without full node. |
| **Security posture** | Audit 10/10, FIPS KAT self-tests, Wasmtime v36, 2500+ tests, zero clippy warnings. |
| **Wasm chaincode** | Any language (Rust, Go, C, AssemblyScript). Move is Move-only. |

---

## Test Coverage Summary

| Module | Tests |
|--------|-------|
| BFT consensus (unit) | 147 |
| BFT E2E adversarial | 16 |
| Parallel execution | 15 |
| Executor (sync + concurrent) | 15 |
| Gateway | 30 |
| Tokenomics | 44 |
| Bridge | 43 |
| Governance | 34 |
| Light client | 24 |
| Pre-existing tests | ~880 |
| **Total suite** | **~2500+ executions, 0 failures** |

---

## Strategic Positioning

**Enterprise/permissioned:** rust-bc wins. PQC + 50 MB footprint + Fabric compat + dual consensus + private data. No competitor matches this combination.

**IoT/edge computing:** rust-bc has a natural advantage. 50 MB nodes, PQC for long-lived devices, light client for constrained devices. IOTA's 128 GB validator requirement is prohibitive.

**Public/semi-public:** rust-bc competes structurally (BFT, governance, tokenomics, bridges). Gap is in ecosystem maturity and live network effects. Next steps: public testnet, connect a live bridge, DPoS validator selection.

**Unique differentiator:** No other L1 combines post-quantum signatures + sub-100 MB nodes + Fabric compatibility + dynamic fees + light clients. This positions rust-bc for regulated industries (finance, government, defense) and resource-constrained deployments.
