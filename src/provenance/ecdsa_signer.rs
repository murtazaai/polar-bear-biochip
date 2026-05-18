//! ECDSA secp256k1 provenance layer.
//!
//! Every `InferenceResult` is canonicalised to JSON, hashed with SHA-256,
//! then signed with a secp256k1 private key.  The resulting `SignedOutput`
//! embeds the signature and the public key so it can be verified offline —
//! forming an immutable, tamper-evident blockchain-style audit trail.
//!
//! Key format:
//!   - Private key  : random ephemeral (regenerated at process start)
//!   - Signature    : compact r||s, 64 bytes, hex-encoded
//!   - Public key   : uncompressed SEC-1 point, 65 bytes, hex-encoded

use anyhow::{Context, Result};
use chrono::Utc;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use k256::EncodedPoint;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::types::{InferenceResult, SignedOutput};

pub struct EcdsaSigner {
    signing_key:   SigningKey,
    verifying_key: VerifyingKey,
}

impl EcdsaSigner {
    /// Generate a fresh ephemeral keypair at process start.
    pub fn new() -> Self {
        let signing_key   = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        Self { signing_key, verifying_key }
    }

    /// Hex-encoded uncompressed public key (65 bytes → 130 hex chars).
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_encoded_point(false).as_bytes())
    }

    /// Sign an `InferenceResult`, returning a `SignedOutput`.
    ///
    /// Steps:
    ///   1. Serialise `InferenceResult` to canonical JSON.
    ///   2. SHA-256 hash the JSON bytes.
    ///   3. ECDSA-sign the hash with the secp256k1 private key.
    ///   4. Embed signature + public key in `SignedOutput`.
    pub fn sign_result(&self, result: &InferenceResult) -> Result<SignedOutput> {
        // 1. Canonical JSON serialisation (deterministic field order via serde)
        let payload_json = serde_json::to_string(result)
            .context("Failed to serialise InferenceResult to JSON")?;

        // 2. SHA-256 hash
        let hash_bytes = Sha256::digest(payload_json.as_bytes());
        let payload_hash_hex = hex::encode(&hash_bytes);

        // 3. ECDSA sign the hash bytes
        //    `SigningKey::sign` accepts raw bytes; k256 applies SHA-256 internally
        //    (ECDSA with SHA-256 = secp256k1/sha256).  We sign the *hash* to ensure
        //    the signature binds to the canonical JSON content.
        let signature: Signature = self.signing_key.sign(&hash_bytes);
        let signature_hex = hex::encode(signature.to_bytes());

        Ok(SignedOutput {
            inference_result: result.clone(),
            payload_hash_hex,
            signature_hex,
            public_key_hex: self.public_key_hex(),
            signed_at: Utc::now(),
        })
    }

    /// Verify a `SignedOutput` entirely from its embedded fields.
    /// Returns `true` if the signature is cryptographically valid.
    ///
    /// This is a pure function — it needs no private key material.
    pub fn verify_signed(signed: &SignedOutput) -> Result<bool> {
        // Reconstruct the verifying key from the embedded hex public key
        let pk_bytes = hex::decode(&signed.public_key_hex)
            .context("Failed to hex-decode public key")?;
        let encoded_point = EncodedPoint::from_bytes(&pk_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid encoded point: {e}"))?;
        let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
            .map_err(|e| anyhow::anyhow!("Failed to construct VerifyingKey: {e}"))?;

        // Reconstruct the payload hash
        let payload_json = serde_json::to_string(&signed.inference_result)
            .context("Failed to re-serialise InferenceResult")?;
        let hash_bytes = Sha256::digest(payload_json.as_bytes());

        // Reconstruct the signature
        let sig_bytes = hex::decode(&signed.signature_hex)
            .context("Failed to hex-decode signature")?;
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid signature bytes: {e}"))?;

        // Verify
        Ok(verifying_key.verify(&hash_bytes, &signature).is_ok())
    }
}
