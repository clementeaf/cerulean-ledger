//! Pluggable proof verification for zkML bridges.
//!
//! The `ProofVerifier` trait defines a single `verify()` method that takes a
//! proof blob and the claim's public inputs (model_hash, input_hash, output_hash).
//! Implementations can be swapped at runtime via `AppState.proof_verifier`.
//!
//! Shipped verifiers:
//! - `Sha256CommitmentVerifier`: proof = SHA256(model_hash || input_hash || output_hash).
//!   Not zero-knowledge, but demonstrates the interface and provides integrity guarantees.
//! - Future: Groth16, PLONK (bn254), STARK (FRI) via ezkl or Risc0.

use serde::{Deserialize, Serialize};

/// Supported proof systems.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// SHA256 commitment (baseline, not ZK). Proof = SHA256(model||input||output).
    Sha256Commitment,
    /// Groth16 over BN254 (ezkl, snarkjs).
    Groth16Bn254,
    /// PLONK over BN254.
    PlonkBn254,
    /// STARK with FRI (Risc0, Giza/Cairo).
    StarkFri,
}

/// A proof submitted alongside an inference claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkInferenceProof {
    /// Which proof system was used.
    pub proof_type: ProofType,
    /// The proof bytes, hex-encoded.
    pub proof_data: String,
    /// The verification key, hex-encoded (required for SNARK/STARK, optional for commitment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_key: Option<String>,
}

/// Public inputs to the proof circuit — what the proof attests to.
pub struct ProofPublicInputs<'a> {
    pub model_hash: &'a str,
    pub input_hash: &'a str,
    pub output_hash: &'a str,
}

/// Trait for verifying inference proofs. Implementations are pluggable.
pub trait ProofVerifier: Send + Sync {
    /// Verify a proof against the public inputs.
    ///
    /// Returns `Ok(true)` if valid, `Ok(false)` if invalid,
    /// `Err(reason)` if the proof format is wrong or verifier fails.
    fn verify(&self, proof: &ZkInferenceProof, inputs: &ProofPublicInputs) -> Result<bool, String>;

    /// Which proof types this verifier supports.
    fn supported_types(&self) -> Vec<ProofType>;
}

/// Baseline verifier: proof = hex(SHA256(model_hash || input_hash || output_hash)).
///
/// Not zero-knowledge — anyone can recompute. Provides integrity (the oracle
/// committed to these exact inputs/outputs) but not privacy or computation proof.
/// Useful for testing the pipeline and as a fallback.
pub struct Sha256CommitmentVerifier;

impl ProofVerifier for Sha256CommitmentVerifier {
    fn verify(&self, proof: &ZkInferenceProof, inputs: &ProofPublicInputs) -> Result<bool, String> {
        if proof.proof_type != ProofType::Sha256Commitment {
            return Err(format!(
                "Sha256CommitmentVerifier does not support {:?}",
                proof.proof_type
            ));
        }

        let proof_bytes =
            hex::decode(&proof.proof_data).map_err(|e| format!("invalid proof hex: {e}"))?;

        if proof_bytes.len() != 32 {
            return Err(format!(
                "SHA256 commitment must be 32 bytes, got {}",
                proof_bytes.len()
            ));
        }

        // Recompute: SHA256(model_hash || input_hash || output_hash)
        use pqc_crypto_module::legacy::sha256::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(inputs.model_hash.as_bytes());
        hasher.update(inputs.input_hash.as_bytes());
        hasher.update(inputs.output_hash.as_bytes());
        let expected = hasher.finalize();

        Ok(proof_bytes == expected.as_slice())
    }

    fn supported_types(&self) -> Vec<ProofType> {
        vec![ProofType::Sha256Commitment]
    }
}

/// Dispatching verifier that routes proofs to the appropriate backend.
pub struct MultiVerifier {
    verifiers: Vec<Box<dyn ProofVerifier>>,
}

impl MultiVerifier {
    pub fn new() -> Self {
        Self {
            verifiers: vec![Box::new(Sha256CommitmentVerifier)],
        }
    }

    /// Add a verifier backend (e.g., for Groth16 when ezkl is integrated).
    pub fn add_verifier(&mut self, verifier: Box<dyn ProofVerifier>) {
        self.verifiers.push(verifier);
    }
}

impl Default for MultiVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofVerifier for MultiVerifier {
    fn verify(&self, proof: &ZkInferenceProof, inputs: &ProofPublicInputs) -> Result<bool, String> {
        for v in &self.verifiers {
            if v.supported_types().contains(&proof.proof_type) {
                return v.verify(proof, inputs);
            }
        }
        Err(format!("no verifier registered for {:?}", proof.proof_type))
    }

    fn supported_types(&self) -> Vec<ProofType> {
        self.verifiers
            .iter()
            .flat_map(|v| v.supported_types())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_inputs() -> (String, String, String, String) {
        let model = "a".repeat(64);
        let input = "b".repeat(64);
        let output = "c".repeat(64);

        // Compute expected commitment
        use pqc_crypto_module::legacy::sha256::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(input.as_bytes());
        hasher.update(output.as_bytes());
        let commitment = hex::encode(hasher.finalize());

        (model, input, output, commitment)
    }

    #[test]
    fn sha256_verifier_accepts_valid_proof() {
        let (model, input, output, commitment) = test_inputs();
        let verifier = Sha256CommitmentVerifier;
        let proof = ZkInferenceProof {
            proof_type: ProofType::Sha256Commitment,
            proof_data: commitment,
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: &model,
            input_hash: &input,
            output_hash: &output,
        };
        assert!(verifier.verify(&proof, &inputs).unwrap());
    }

    #[test]
    fn sha256_verifier_rejects_wrong_commitment() {
        let (model, input, output, _) = test_inputs();
        let verifier = Sha256CommitmentVerifier;
        let proof = ZkInferenceProof {
            proof_type: ProofType::Sha256Commitment,
            proof_data: "ff".repeat(32), // wrong
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: &model,
            input_hash: &input,
            output_hash: &output,
        };
        assert!(!verifier.verify(&proof, &inputs).unwrap());
    }

    #[test]
    fn sha256_verifier_rejects_bad_hex() {
        let verifier = Sha256CommitmentVerifier;
        let proof = ZkInferenceProof {
            proof_type: ProofType::Sha256Commitment,
            proof_data: "not_hex".to_string(),
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: "a",
            input_hash: "b",
            output_hash: "c",
        };
        assert!(verifier.verify(&proof, &inputs).is_err());
    }

    #[test]
    fn sha256_verifier_rejects_wrong_type() {
        let verifier = Sha256CommitmentVerifier;
        let proof = ZkInferenceProof {
            proof_type: ProofType::Groth16Bn254,
            proof_data: "ff".repeat(32),
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: "a",
            input_hash: "b",
            output_hash: "c",
        };
        assert!(verifier.verify(&proof, &inputs).is_err());
    }

    #[test]
    fn multi_verifier_dispatches_correctly() {
        let (model, input, output, commitment) = test_inputs();
        let multi = MultiVerifier::new();
        let proof = ZkInferenceProof {
            proof_type: ProofType::Sha256Commitment,
            proof_data: commitment,
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: &model,
            input_hash: &input,
            output_hash: &output,
        };
        assert!(multi.verify(&proof, &inputs).unwrap());
    }

    #[test]
    fn multi_verifier_rejects_unsupported_type() {
        let multi = MultiVerifier::new();
        let proof = ZkInferenceProof {
            proof_type: ProofType::StarkFri,
            proof_data: "ff".repeat(32),
            verification_key: None,
        };
        let inputs = ProofPublicInputs {
            model_hash: "a",
            input_hash: "b",
            output_hash: "c",
        };
        assert!(multi.verify(&proof, &inputs).is_err());
    }

    #[test]
    fn proof_type_serde_roundtrip() {
        for pt in [
            ProofType::Sha256Commitment,
            ProofType::Groth16Bn254,
            ProofType::PlonkBn254,
            ProofType::StarkFri,
        ] {
            let json = serde_json::to_string(&pt).unwrap();
            let decoded: ProofType = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, pt);
        }
    }

    #[test]
    fn zk_inference_proof_serde_roundtrip() {
        let proof = ZkInferenceProof {
            proof_type: ProofType::Groth16Bn254,
            proof_data: "abcdef".to_string(),
            verification_key: Some("vk_hex".to_string()),
        };
        let json = serde_json::to_string(&proof).unwrap();
        let decoded: ZkInferenceProof = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, proof);
    }
}
