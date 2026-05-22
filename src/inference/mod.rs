//! Optimistic ML Oracle — inference verification subsystem.
//!
//! Provides a pluggable proof verification interface for zkML bridges.
//! Oracles that submit claims with valid ZK proofs bypass the dispute window.

pub mod proof;
