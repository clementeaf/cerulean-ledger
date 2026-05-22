# Optimistic ML Oracle — Design Document

## Problem

Cerulean's intelligence module (anomaly detection, risk scoring, pattern recognition) runs
computations locally on the node. There is no cryptographic proof that a specific model
produced a specific output. Clients must trust the node blindly.

Protocols like Ora (opML), Bittensor, Modulus Labs, and Giza solve this by making ML
inference verifiable on-chain — either optimistically (dispute-based) or cryptographically
(zkML).

This design adds an **Optimistic ML Oracle** layer to Cerulean, reusing the existing oracle
system, staking, and reputation infrastructure.

## How It Works

```
  Oracle                    Cerulean Node                  Challenger
    |                           |                              |
    |-- submit_inference() ---->|                              |
    |   (model_hash, input,     |                              |
    |    output, signature)     |                              |
    |                           |-- store as "pending" ------->|
    |                           |   (dispute window opens)     |
    |                           |                              |
    |                           |   ... dispute_window ...     |
    |                           |                              |
    |                           |<-- challenge() --------------|
    |                           |   (re-executed output,       |
    |                           |    challenger_stake)          |
    |                           |                              |
    |                           |-- compare outputs ---------->|
    |                           |   match?  → reject challenge |
    |                           |   differ? → slash oracle,    |
    |                           |             reward challenger |
    |                           |                              |
    |                           |-- finalize() (after window)  |
    |                           |   no challenge? → accepted   |
```

## Core Concepts

### Inference Claim

An oracle submits a claim that a model produced a specific output:

```rust
struct InferenceClaim {
    id: String,                  // Unique claim ID
    oracle_id: String,           // Registered oracle (must be staked)
    model_hash: String,          // SHA3-256 of model weights/ONNX file
    model_version: String,       // Human-readable version tag
    input_hash: String,          // SHA3-256 of the input data
    input_uri: Option<String>,   // Optional: where to fetch input for re-execution
    output: String,              // The inference result (JSON-serialized)
    output_hash: String,         // SHA3-256 of output (for quick comparison)
    timestamp: u64,
    signature: String,           // Ed25519 over "inference:{id}:{model_hash}:{output_hash}"
    status: ClaimStatus,         // Pending → Finalized | Disputed → Slashed | Rejected
    dispute_deadline: u64,       // timestamp + dispute_window
}
```

### Claim Lifecycle

```
  Pending ──────────────┬──→ Finalized  (no challenge within window)
                        │
                        └──→ Disputed ──→ Slashed   (challenge succeeded)
                                     └──→ Rejected  (challenge failed, challenger loses bond)
```

### Challenge

Anyone with sufficient stake can challenge a pending claim:

```rust
struct InferenceChallenge {
    claim_id: String,
    challenger_id: String,       // Must be staked
    challenger_output: String,   // Re-executed output
    challenger_output_hash: String,
    bond: u64,                   // Stake locked as anti-spam
    timestamp: u64,
    signature: String,           // Ed25519 over "challenge:{claim_id}:{challenger_output_hash}"
}
```

### Resolution

The node compares `claim.output_hash` vs `challenge.challenger_output_hash`:

- **Match** → Challenge rejected. Challenger loses bond (sent to oracle as reward).
- **Differ** → Oracle slashed. Challenger receives oracle's stake + bond returned.

For deterministic models this is straightforward. For non-deterministic models (temperature > 0),
the design supports a **tolerance threshold** (e.g., cosine similarity > 0.95) or an
**arbiter committee** of N additional oracles.

## Integration with Existing Systems

### Oracle System (`oracle_system.rs`)

- `OracleNode` already has: registration, reputation, fee_balance
- **Extend**: add `InferenceClaim` storage alongside `PriceData`
- **Reuse**: `verify_signature()` pattern (upgrade from HMAC to Ed25519 for claims)
- **Reuse**: reputation updates (`update_reputation()`) on claim finalization/slash

### Staking (`staking.rs`)

- Oracles must be staked to submit claims (minimum stake enforced)
- Challengers must post a bond (fraction of oracle's stake)
- **Reuse**: `StakingManager.stake()`, `detect_and_slash_double_sign()` pattern
- **New**: `slash_inference_fraud()` — slash oracle stake on successful challenge

### Storage (`storage/traits.rs`)

- New column family: `inference_claims` (keyed by claim ID)
- Secondary index: `oracle:{oracle_id}` for listing claims by oracle
- Secondary index: `model:{model_hash}` for listing claims by model
- **Reuse**: existing RocksDB patterns (zero-padded keys, secondary indices)

## API Endpoints

All under `/api/v1/inference/`:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/inference/submit` | Submit an inference claim (oracle, staked) |
| `POST` | `/inference/challenge` | Challenge a pending claim (staked) |
| `GET`  | `/inference/claims` | List claims (filter by status, oracle, model) |
| `GET`  | `/inference/claims/{id}` | Get claim details + challenge history |
| `POST` | `/inference/finalize/{id}` | Finalize a claim after dispute window (anyone) |
| `GET`  | `/inference/models` | List known model hashes with claim counts |

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `INFERENCE_DISPUTE_WINDOW_SECS` | `86400` (24h) | Time window for challenges |
| `INFERENCE_MIN_ORACLE_STAKE` | `5000` | Minimum stake to submit claims |
| `INFERENCE_CHALLENGE_BOND` | `1000` | Bond required to challenge |
| `INFERENCE_TOLERANCE` | `exact` | `exact` or `cosine:0.95` for fuzzy match |

## Signing Payloads

| Action | Payload (UTF-8 bytes) |
|--------|----------------------|
| Submit claim | `inference:{claim_id}:{model_hash}:{output_hash}` |
| Challenge | `challenge:{claim_id}:{challenger_output_hash}` |

Consistent with the `alias:register:{commitment}` pattern already in the codebase.

## Security Considerations

1. **Economic security**: Challenge must be cheaper than fraud profit. Bond prevents spam.
2. **Model pinning**: `model_hash` pins the exact weights. Model updates = new hash.
3. **Input availability**: `input_uri` lets challengers fetch input for re-execution.
   Without it, challenges require out-of-band coordination.
4. **Non-determinism**: Models with temperature > 0 need tolerance thresholds or committee vote.
5. **Griefing**: Challenger loses bond on failed challenge. Prevents frivolous disputes.
6. **Timing**: Claims cannot be challenged after `dispute_deadline`. Finalization is permissionless.

## Comparison with Other Protocols

| Aspect | Cerulean opML | Ora (opML) | Modulus (zkML) | Bittensor |
|--------|--------------|------------|----------------|-----------|
| Proof type | Optimistic (dispute) | Optimistic (dispute) | ZK-SNARK | Consensus |
| Latency | dispute_window | dispute_window | Proof gen (~min) | Block time |
| Cost | Low (only on dispute) | Low | High (prover) | Medium |
| Trust model | Economic | Economic | Mathematical | Economic |
| Model privacy | Hash only on-chain | Hash only | Hash only | Exposed to validators |
| Requires | Staking | Staking | Prover circuit | Validator network |

## Implementation Phases

### Phase 1: Core (MVP)

- `InferenceClaim` and `InferenceChallenge` types
- Storage trait extensions (write/read claims)
- Submit + finalize endpoints
- Integration with staking (min stake check)

### Phase 2: Disputes

- Challenge endpoint
- Resolution logic (exact match)
- Slash/reward mechanics
- Reputation updates

### Phase 3: Non-deterministic Models

- Tolerance threshold configuration
- Arbiter committee (N oracles re-execute, majority wins)
- Cosine similarity comparator for embedding outputs

### Phase 4: zkML Bridge (Future)

- Accept SNARK/STARK proofs as alternative to optimistic verification
- Verify proofs on-chain (no dispute window needed)
- Integration with ezkl or Risc0 for ONNX model circuits
