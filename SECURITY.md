# Security Assumptions & Threat Model

## Overview
This document outlines the security assumptions, threat model, and cryptographic safeguards implemented in the blockchain oracle system.

## Phase 1: Signature Verification (Implemented)

### HMAC-SHA256 Signature Verification
**Purpose**: Ensure authenticity and integrity of oracle price reports

**Implementation**:
- Algorithm: HMAC-SHA256
- Signing Key: `b"oracle-system-hmac-key-v1"` (hardcoded - see production notes)
- Data signed: `oracle_id || price || timestamp` (concatenated as bytes)
- Comparison: Constant-time comparison using `as_slice()` to prevent timing attacks

**Threat Model**:
- **Unauthorized Reports**: An attacker cannot submit a valid price report without knowing the signing key
- **Data Tampering**: Modifying oracle_id, price, or timestamp invalidates the signature
- **Replay Attacks**: Mitigated by timestamp validation (see Phase 2)

**Security Assumptions**:
1. SIGNING_KEY is kept secret and not exposed in code (currently hardcoded - USE SECURE KEY MANAGEMENT IN PRODUCTION)
2. HMAC-SHA256 is not broken (cryptographically sound as of 2024)
3. The digest crate provides correct constant-time comparisons

**Limitations & Production Notes**:
- Current implementation uses hardcoded key for testing
- **PRODUCTION REQUIREMENT**: Use secure key management (e.g., AWS KMS, HashiCorp Vault)
- Key rotation strategy must be implemented before production deployment
- All reports with `timestamp >= 100_000_000` trigger signature verification
- Test mode (timestamps < 100_000_000) skips verification for unit testing

## Phase 2: Timestamp Validation (Implemented)

### Timestamp Drift Constraints
**Purpose**: Prevent oracle attacks using time manipulation and reject stale data

**Implementation**:
- Maximum future drift: 5 minutes (300,000 milliseconds)
- Maximum past drift: 1 hour (3,600,000 milliseconds)
- Validation trigger: Only for production timestamps (>= 100_000_000)
- Time source: `std::time::SystemTime::now()`

**Threat Model**:
- **Future-dated Reports**: An attacker cannot submit reports with timestamps far in the future
- **Stale Data**: Old data is automatically rejected, preventing the use of outdated prices
- **Clock Skew Attacks**: Acceptable clock drift (5 min future) accommodates system clock variations

**Security Assumptions**:
1. System clock is reasonably accurate (within 5 minutes)
2. Validators use synchronized time sources
3. Network latency is accounted for in the 5-minute future window

**Validation Window**:
```
Accept if: current_time - 3600000 <= timestamp <= current_time + 300000
Reject if: timestamp < (current_time - 3600000) OR timestamp > (current_time + 300000)
```

## Phase 3: Reputation-Weighted Voting (Implemented)

### Voting Weight Calculation
**Purpose**: Give more voting power to trusted oracles while maintaining one-person-one-vote baseline

**Implementation**:
- Formula: `weight = (1 + (collateral / 100)).min(10)`
- Collateral is used as reputation proxy
- Non-oracle voters: fixed weight of 1
- Maximum weight multiplier: 10x

**Threat Model**:
- **Sybil Attacks**: Mitigated by requiring collateral/reputation for vote amplification
- **Democracy Preservation**: Even users with zero reputation get at least 1 vote
- **Whale Prevention**: Cap at 10x prevents single oracle from dominating

**Security Assumptions**:
1. Collateral amounts accurately reflect oracle trustworthiness
2. Vote weighting mechanism can't be gamed to accumulate disproportionate power
3. Challenge/resolution process is fair and transparent

## Phase 4: Logging & Observability (Planned)

### Strategic Logging Points
**Planned Implementation**:
- Signature verification attempts (success/failure)
- Timestamp validation failures
- Challenge creation and resolution
- Vote weight calculation
- Slashing events

**Security Benefits**:
- Audit trail for compliance
- Forensic analysis of attack attempts
- Real-time anomaly detection
- Performance monitoring

## Threat Model Summary

### Out of Scope
- Network-level attacks (mitigated by network layer)
- Physical attacks on validator infrastructure
- Quantum computing threats (future consideration)
- Social engineering / key theft (operational security required)

### In Scope
- Cryptographic attacks on HMAC-SHA256
- Timestamp manipulation attempts
- Unauthorized report submission
- Replay attacks
- Vote manipulation

## Deployment Checklist

- [ ] Replace hardcoded SIGNING_KEY with key management system
- [ ] Implement key rotation strategy
- [ ] Configure secure time synchronization (NTP/PTP)
- [ ] Deploy logging infrastructure
- [ ] Set up audit trail storage
- [ ] Conduct security audit by third party
- [ ] Implement rate limiting for report submissions
- [ ] Deploy intrusion detection for signature failures
- [ ] Monitor timestamp validation failure rate
- [ ] Test disaster recovery procedures

## Vulnerability Disclosure

If you discover a security vulnerability in this project, please report it responsibly.

**Contact:** [GitHub Security Advisories](https://github.com/clementeaf/cerulean-ledger/security/advisories/new) (preferred) or open a private issue via repository maintainers

**Process:**
1. Send a description of the vulnerability to the contact above
2. Include steps to reproduce if possible
3. Do not disclose publicly until a fix is available
4. We will acknowledge receipt within 72 hours
5. We will provide a fix timeline within 7 days

We follow coordinated disclosure. Credit is given to reporters in the changelog unless they prefer anonymity.

## Consensus and Network Threat Model

### Raft Ordering

- **Leader manipulation:** Mitigated by Raft protocol — leader is elected by majority vote, log entries require majority replication before commit
- **Split brain:** Raft guarantees at most one leader per term; network partitions result in the minority partition halting (no conflicting blocks)
- **Log truncation:** Persistent Raft log (RocksDB) survives crashes; entries are never deleted, only compacted after snapshotting

### Gossip Protocol

- **Eclipse attacks:** Mitigated by signed alive messages — peers verify Ed25519/ML-DSA-65 signatures on all gossip messages
- **Block withholding:** Anti-entropy protocol detects height gaps between peers and triggers pull-sync to fill missing blocks
- **Sybil on gossip:** Permissioned network — only peers with valid TLS certificates (mTLS) can connect

### Censorship

- **Transaction censorship by orderer:** Mitigated by endorsement pipeline — transactions are endorsed by multiple orgs before reaching the orderer; the orderer cannot modify endorsed transactions without invalidating signatures
- **Block censorship:** All peers verify block contents against endorsement policies; an orderer that omits valid transactions would be detected by peers comparing their pending queues

### Residual Risks

- A compromised Raft majority (>50% of orderers) could halt ordering or reorder transactions
- A compromised CA could issue certificates to unauthorized peers
- Private data dissemination relies on honest member peers — a malicious member could withhold data from other authorized members

## References

- HMAC Specification: RFC 2104
- SHA-256: FIPS 180-4
- Constant-Time Comparison: https://codahale.com/a-lesson-in-timing-attacks/
- Oracle Attack Vectors: Multiple blockchain oracle exploit case studies

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-12-18 | Initial security documentation with Phases 1-3 implementation |
