# Alias System — Technical Design

## Problem

Users need a human-friendly identifier (alias/pseudonym) to invite others and interact socially, without exposing real identity on-chain. All communication must comply with NIST standards end-to-end.

## Principles

1. **Zero-knowledge alias**: on-chain storage never contains plaintext alias. Only cryptographic commitments.
2. **NIST compliance**: all crypto operations use NIST-approved algorithms (SHA-3, AES-256-GCM, ML-DSA-65, Argon2id).
3. **Event-driven communication**: inter-service requests flow through an authenticated event bus — no direct REST calls between services for alias resolution.
4. **Client-side resolution**: alias hashing/encryption happens on the client. The server never sees the plaintext alias.

## Architecture

```
cerulean-wallet (client)          event bus              cerulean-ledger (node)
  |                                  |                        |
  |-- register_alias(hash) --------->|-- ALIAS_REGISTERED --->|-- store hash:DID
  |                                  |                        |
  |-- resolve_alias(hash) ---------->|-- ALIAS_LOOKUP ------->|-- return DID
  |                                  |<-- ALIAS_RESOLVED -----|
  |<-- DID --------------------------                         |
  |                                  |                        |
cerulean-voto (voting app)          |                        |
  |-- invite_by_alias(hash) ------->|-- ALIAS_LOOKUP ------->|
  |                                  |<-- ALIAS_RESOLVED -----|
  |<-- DID + address ----------------|                        |
```

## Alias Registration

### Client (cerulean-wallet)

```
1. User enters alias: "pedro"
2. Normalize: lowercase, trim, NFC unicode normalization
3. Derive alias commitment:
     salt = random(32 bytes)
     commitment = SHA3-256(salt || alias)
4. Encrypt alias for self-recovery:
     encrypted_alias = AES-256-GCM(key=wallet_key, plaintext=alias)
5. Publish event:
     ALIAS_REGISTER {
       did: "did:cerulean:<address>",
       commitment: hex(commitment),
       salt: hex(salt),
       encrypted_alias: hex(encrypted_alias),
       signature: Ed25519_sign(commitment, private_key)
     }
```

### On-chain (cerulean-ledger)

```
alias_registry:
  commitment (SHA3-256)  ->  { did, salt, encrypted_alias, registered_at }
```

- One alias per DID. Re-registration replaces the previous.
- Commitment is indexed for O(1) lookup.
- `encrypted_alias` is opaque to the node — only the wallet owner can decrypt.

## Alias Resolution

### Lookup flow (client-side)

```
1. Inviter types alias: "pedro"
2. Normalize: lowercase, trim, NFC
3. Need salt — two options:
   a) Brute: query all salts (leaks nothing, but expensive)
   b) Salt-hint: hash(alias) used as a secondary index (weaker privacy)
   c) Blind lookup via event bus (recommended, see below)
```

### Recommended: Blind lookup via event bus

```
1. Client computes: blind_hash = SHA3-256("alias_lookup" || alias)
2. Publishes event:
     ALIAS_LOOKUP {
       blind_hash: hex(blind_hash),
       requester_did: "did:cerulean:<address>",
       signature: Ed25519_sign(blind_hash, private_key)
     }
3. Node checks all commitments:
     for each (commitment, salt, did) in registry:
       if SHA3-256(salt || recover_alias_from(blind_hash)) == commitment:
         emit ALIAS_RESOLVED { did, address }
```

**Problem**: node cannot reverse `blind_hash` without knowing the alias.

### Practical approach: deterministic salt

```
1. Salt derived from alias itself:
     salt = SHA3-256("cerulean:alias:salt:" || alias)[0..16]
2. Commitment:
     commitment = SHA3-256(salt || alias)
3. Lookup:
     anyone who knows the alias can compute commitment directly
     query: GET event ALIAS_LOOKUP { commitment } -> { did }
4. Privacy:
     - On-chain: only commitment visible (cannot reverse to alias)
     - Brute-force: attacker needs to guess the alias to compute commitment
     - Common aliases get same protection as passwords (entropy-dependent)
```

This is the simplest NIST-compliant approach. The tradeoff: common aliases (e.g., "pedro") are brute-forceable by someone scanning the chain. Mitigation: require minimum alias length (6+) or append a user-chosen PIN.

## Event Bus

### Why event-driven

- **Decoupling**: cerulean-voto doesn't need direct access to cerulean-ledger API.
- **Audit trail**: every alias operation is a signed event — append-only log.
- **Rate limiting**: bus enforces per-DID rate limits on lookups (anti-enumeration).
- **NIST compliance**: TLS 1.3 between services, signed events, no plaintext in transit.

### Event types

| Event | Producer | Consumer | Payload |
|-------|----------|----------|---------|
| `ALIAS_REGISTER` | wallet | ledger | commitment, salt, encrypted_alias, sig |
| `ALIAS_REGISTERED` | ledger | wallet, voto | did, commitment, timestamp |
| `ALIAS_LOOKUP` | wallet, voto | ledger | commitment, requester_did, sig |
| `ALIAS_RESOLVED` | ledger | requester | did, address (or NOT_FOUND) |
| `ALIAS_DEREGISTER` | wallet | ledger | did, sig |

### Transport options

| Option | Pros | Cons |
|--------|------|------|
| **Redis Streams** | Simple, fast, built-in consumer groups | Single point of failure |
| **NATS** | Lightweight, TLS native, subject-based routing | New dependency |
| **Kafka** | Durable, exactly-once, audit-friendly | Heavy for current scale |
| **In-process channels** | Zero infra, start here | Doesn't scale to multiple nodes |

**Recommendation**: start with **NATS** — lightweight, TLS 1.3 native, supports request/reply pattern for lookups. Migrate to Kafka only if multi-node replication is needed.

## NIST Compliance Checklist

| Requirement | Standard | Implementation |
|-------------|----------|----------------|
| Hash function | FIPS 202 | SHA3-256 for commitments |
| Symmetric encryption | FIPS 197 + SP 800-38D | AES-256-GCM for encrypted_alias |
| Digital signatures | FIPS 186-5 / FIPS 204 | Ed25519 (classical) + ML-DSA-65 (PQC) |
| Key derivation | SP 800-63B | Argon2id (passphrase -> key) |
| TLS | SP 800-52 Rev2 | TLS 1.3 for all inter-service communication |
| Random number generation | SP 800-90A | OS CSPRNG (getrandom/urandom) |
| Key management | SP 800-57 | Keys encrypted at rest, zeroized after use |
| Event authentication | SP 800-177 | Every event signed by producer's Ed25519 key |
| Anti-enumeration | SP 800-63B sec 5.2.3 | Rate-limited lookups, no batch queries |

## Anti-Enumeration Protections

- Lookup events are rate-limited: max 10 per minute per DID.
- No wildcard or batch alias queries.
- `ALIAS_RESOLVED` only returns DID, not alias or salt.
- Failed lookups return same response shape as successful ones (constant-time).
- Optional: require proof-of-work token on lookup events (CPU cost to brute-force).

## Migration Path

### Phase 1 — cerulean-ledger
- Add `alias_registry` storage module.
- Event handlers: `ALIAS_REGISTER`, `ALIAS_LOOKUP`, `ALIAS_DEREGISTER`.
- REST endpoint (temporary, until bus is ready): `POST /api/v1/alias/register`, `POST /api/v1/alias/resolve`.

### Phase 2 — cerulean-wallet
- Alias registration during `/create-hd` flow (optional field).
- Alias resolution in contacts / invite flow.
- Extension: alias display in dashboard (decrypted locally).

### Phase 3 — cerulean-voto
- Invite by alias: input alias -> resolve -> send invitation to DID.
- Display alias (if resolved) instead of raw address in voter lists.

### Phase 4 — Event bus
- Deploy NATS with TLS 1.3.
- Migrate REST alias endpoints to event-driven.
- Add audit log consumer.

## Decisions

### 1. Alias Uniqueness

**Unique globally. Case-sensitive.** "User" and "user" are distinct aliases.

- On registration, node checks if commitment already exists. If so, rejects with `ALIAS_TAKEN`.
- Since commitment = `SHA3-256(salt || alias)` and salt is deterministic from alias, same alias always produces same commitment. Uniqueness check is a simple index lookup.
- No normalization (no lowercasing, no Unicode NFKC folding). The alias is hashed exactly as entered. Users must remember their exact casing.

### 2. Alias Format

**Free-form with security constraints:**

- Min length: 3 characters.
- Max length: **32 characters** (prevents payload injection, keeps commitments bounded).
- Allowed characters: Unicode letters, digits, underscores, hyphens. Regex: `^[\p{L}\p{N}_-]{3,32}$`.
- No whitespace, no control characters, no HTML/JS special chars (`<>'";&`).
- Validation happens client-side before hashing AND server-side before storing.
- The alias is never interpreted, rendered, or interpolated — only hashed. Even so, strict charset prevents any injection vector if a bug ever leaks the plaintext.

### 3. Recovery

**Yes — via `encrypted_alias` decrypted with wallet key.**

- `encrypted_alias` stored on-chain is AES-256-GCM encrypted with the wallet's derived key.
- Only the wallet owner can decrypt (requires passphrase -> Argon2id -> AES key).
- Recovery flow: unlock wallet -> decrypt `encrypted_alias` from alias registry -> display alias.
- If wallet is lost but mnemonic is available: recover wallet -> same key -> decrypt alias.
- NIST compliance: AES-256-GCM (FIPS 197 + SP 800-38D), Argon2id (SP 800-63B).

### 4. Revocation

**Mark as revoked + 15-day cooldown.** Immutable ledger — append-only.

- `ALIAS_DEREGISTER` event sets `status: "revoked"` and `revoked_at` timestamp.
- During cooldown (15 days from `revoked_at`): alias cannot be re-registered by anyone. Lookup returns `ALIAS_REVOKED`.
- After cooldown expires: alias is released. Another user can register it. Lookup returns `NOT_FOUND`.
- Original owner can also re-register during or after cooldown (priority window not needed — first come first served after expiry).
- User can register a new, different alias immediately after revoking (no need to wait).

## Invitation System

### Flow

```
cerulean-voto                    event bus                cerulean-wallet
  |                                 |                         |
  |-- INVITE { from, to, proposals[] } -->                    |
  |                                 |-- push notification --->|
  |                                 |                         |-- user sees:
  |                                 |                         |   "X te invito a N votaciones"
  |                                 |                         |   [Participar] [Rechazar]
  |                                 |                         |
  |                                 |<-- INVITE_RESPONSE -----|
  |<-- accepted/rejected -----------|                         |
```

### User Consent Levels

When a user receives an invitation, the wallet presents four options:

| Option | Behavior | Stored as |
|--------|----------|-----------|
| **Participar esta vez** | Accept this batch only. Future invitations from same person require manual approval. | `consent: "once"` |
| **Permitir futuras** | Accept + whitelist this inviter. Future invitations arrive but still require per-invitation confirmation. | `consent: "whitelist"` |
| **Auto-aceptar** | Accept + auto-confirm all future invitations from this inviter. No manual approval needed. | `consent: "auto"` |
| **Rechazar** | Decline. Inviter is not blocked — can invite again later. | `consent: "rejected"` |

### Consent Storage (cerulean-wallet)

```
invitation_preferences:
  inviter_did  ->  {
    consent: "once" | "whitelist" | "auto",
    alias_encrypted: hex,       // inviter alias (encrypted, for display)
    granted_at: timestamp,
    last_invitation_at: timestamp
  }
```

- Stored in wallet's local storage (localStorage / chrome.storage.local).
- Encrypted at rest — tied to wallet key.
- User can revoke any consent level from wallet settings at any time.
- `auto` consent can be downgraded to `whitelist` or revoked entirely.

### Auto-accept Rules

When consent is `"auto"` for an inviter:

1. Invitation arrives via event bus.
2. Wallet checks `invitation_preferences[inviter_did].consent === "auto"`.
3. If yes: auto-responds `INVITE_ACCEPTED` without user prompt.
4. Notification shown passively: "Fuiste incluido en N votaciones por [alias]" (informational, no action required).
5. User can still abstain or vote freely — auto-accept only confirms participation, not the vote itself.

### Security Constraints

- Consent is **per-inviter**, not global. No "auto-accept all" option.
- `auto` consent has an optional expiry (default: 90 days, configurable).
- Max invitations per batch: 20 (prevents spam even with auto-accept).
- Rate limit: max 5 invitation events per inviter per hour.
- All invitation events are signed by the inviter's Ed25519 key — wallet verifies before presenting.
- Invitation payload never contains executable content — only proposal IDs and metadata.

### 5. PQC Timeline

**Open — requires analysis.** Considerations:

- Ed25519 is currently used for all signatures (wallet, events, votes).
- ML-DSA-65 is available in CLI but not in WASM (pqcrypto-mldsa C bindings don't compile to wasm32).
- Event bus signatures could migrate to ML-DSA-65 server-side (node-to-node) before client-side.
- Suggested approach: **dual-signature** during transition. Events carry both Ed25519 and ML-DSA-65 signatures. Verifiers accept either. Once WASM support exists, deprecate Ed25519-only.
- Timeline depends on: WASM PQC library maturity, NIST final standards ratification, and performance benchmarks on mobile devices.
- **Action item**: benchmark ML-DSA-65 verify on target devices. If < 100ms, viable for client-side. If not, keep PQC server-side only.
