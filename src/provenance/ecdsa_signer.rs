//! # ECDSA secp256k1 Provenance Layer
//!
//! ## Algorithm walkthrough (forward-engineered from FIPS 186-5 / SEC 1 v2.0)
//!
//! ### Domain parameters (secp256k1)
//! - Prime field:  p = 2²⁵⁶ − 2³² − 977
//! - Curve:        y² ≡ x³ + 7  (mod p)   ← Weierstrass form
//! - Base point:   G  (compressed, 33 bytes on-chain)
//! - Order:        n  (256-bit prime - number of points in the group)
//! - Cofactor:     h = 1
//!
//! ### Provenance signing (payload bytes P, private key d)
//! 1. canonical_json = serde_json::to_string(InferenceResult)
//! 2. e = SHA-256(canonical_json)          ← payload hash
//! 3. Ephemeral k sampled from CSPRNG
//! 4. R = k·G;  r = R.x mod n
//! 5. s = k⁻¹ · (e + r·d) mod n
//! 6. Signature = (r, s) - 64 bytes compact r‖s
//!
//! ### Offline verification
//! 1. Recompute e = SHA-256(canonical_json)
//! 2. w = s⁻¹ mod n
//! 3. u₁ = e·w mod n;  u₂ = r·w mod n
//! 4. X = u₁·G + u₂·Q
//! 5. Valid iff X.x mod n == r
//!
//! ### Why secp256k1?
//! - Bitcoin / Ethereum / Solana native curve - same key material, zero extra dependencies
//! - 256-bit security at ~128-bit classical security level
//! - 64-byte compact signatures (low overhead in JSON provenance records)

use anyhow::{Result, anyhow};
use chrono::Utc;
use k256::{
    EncodedPoint,
    ecdsa::{
        Signature, SigningKey, VerifyingKey,
        signature::{Signer, Verifier},
    },
};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::types::{InferenceResult, SignedOutput};

// ── EcdsaSigner ───────────────────────────────────────────────────────────────

/// An ECDSA signer backed by a secp256k1 private key.
///
/// # Security
/// The private key is zeroed on drop via k256's `ZeroizeOnDrop`.
/// Never serialise the private scalar to logs, JSON responses, or env vars.
pub struct EcdsaSigner {
    signing_key: SigningKey,
}

impl EcdsaSigner {
    /// Generate a fresh ephemeral keypair using a cryptographically secure RNG.
    #[must_use]
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::random(&mut OsRng),
        }
    }

    /// Restore a signer from a 32-byte big-endian private scalar.
    ///
    /// # Errors
    /// Returns an error if `bytes` is not a valid scalar in \[1, n-1\].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let signing_key = SigningKey::from_slice(bytes)
            .map_err(|e| anyhow!("invalid ECDSA private key bytes: {e}"))?;
        Ok(Self { signing_key })
    }

    /// Restore a signer from a lowercase hex-encoded private scalar (64 chars).
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|e| anyhow!("hex decode: {e}"))?;
        Self::from_bytes(&bytes)
    }

    // ── Serialisation ─────────────────────────────────────────────────────────

    /// Export the private key as 32 raw bytes.  **Never** log or transmit this.
    #[must_use]
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes().into()
    }

    /// Export the private key as a lowercase hex string (64 chars).
    #[must_use]
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.private_key_bytes())
    }

    /// Export the **uncompressed** 65-byte SEC 1 public key (04‖x‖y).
    ///
    /// Used as the embedded public key in [`SignedOutput`] for offline verification.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        let vk = VerifyingKey::from(&self.signing_key);
        hex::encode(vk.to_encoded_point(false).as_bytes())
    }

    /// Export the **compressed** 33-byte SEC 1 public key.
    #[must_use]
    pub fn verifying_key_hex(&self) -> String {
        let vk = VerifyingKey::from(&self.signing_key);
        hex::encode(vk.to_encoded_point(true).as_bytes())
    }

    // ── Signing ───────────────────────────────────────────────────────────────

    /// Sign an [`InferenceResult`], returning a tamper-evident [`SignedOutput`].
    ///
    /// ## Steps
    /// 1. Serialise `result` to canonical JSON.
    /// 2. SHA-256 hash the bytes.
    /// 3. ECDSA-sign the hash (secp256k1 / SHA-256).
    /// 4. Embed signature + uncompressed public key in [`SignedOutput`].
    ///
    /// # Errors
    /// Returns an error if JSON serialisation fails.
    pub fn sign_result(&self, result: &InferenceResult) -> Result<SignedOutput> {
        let payload_json = serde_json::to_string(result)
            .map_err(|e| anyhow!("JSON serialise: {e}"))?;

        let hash_bytes        = Sha256::digest(payload_json.as_bytes());
        let payload_hash_hex  = hex::encode(&hash_bytes);

        let signature: Signature = self.signing_key.sign(&hash_bytes);
        let signature_hex        = hex::encode(signature.to_bytes());

        Ok(SignedOutput {
            inference_result: result.clone(),
            payload_hash_hex,
            signature_hex,
            public_key_hex: self.public_key_hex(),
            signed_at: Utc::now(),
        })
    }

    /// Verify a `(message, signature_hex)` pair against this signer's public key.
    ///
    /// # Errors
    /// Returns an error if the hex is malformed or the signature bytes are invalid.
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> Result<bool> {
        EcdsaVerifier::from_hex(&self.public_key_hex())?.verify(message, signature_hex)
    }

    // ── Offline verification ──────────────────────────────────────────────────

    /// Verify a [`SignedOutput`] entirely from its embedded fields.
    ///
    /// Returns `true` iff the ECDSA signature is cryptographically valid for
    /// the canonical JSON of `signed.inference_result`.
    ///
    /// This is a **pure function** - no private key material is required.
    ///
    /// # Errors
    /// Returns an error on hex decode failure, invalid key material, or
    /// JSON re-serialisation failure.
    pub fn verify_signed(signed: &SignedOutput) -> Result<bool> {
        let payload_json = serde_json::to_string(&signed.inference_result)
            .map_err(|e| anyhow!("re-serialise: {e}"))?;
        let hash_bytes = Sha256::digest(payload_json.as_bytes());

        EcdsaVerifier::from_hex(&signed.public_key_hex)?.verify(&hash_bytes, &signed.signature_hex)
    }
}

// ── EcdsaVerifier ─────────────────────────────────────────────────────────────

/// Standalone public-key verifier - no private key required.
///
/// Useful for offline audit tools that receive a [`SignedOutput`] JSON and
/// need to validate it without access to the original signing process.
pub struct EcdsaVerifier {
    verifying_key: VerifyingKey,
}

impl EcdsaVerifier {
    /// Construct from a compressed (33-byte / 66-char hex) or
    /// uncompressed (65-byte / 130-char hex) SEC 1 public key.
    ///
    /// # Errors
    /// Returns an error if the hex is malformed or the point is not on secp256k1.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|e| anyhow!("hex decode: {e}"))?;
        let point = EncodedPoint::from_bytes(bytes)
            .map_err(|e| anyhow!("invalid SEC 1 point: {e}"))?;
        let verifying_key = VerifyingKey::from_encoded_point(&point)
            .map_err(|e| anyhow!("point not on curve: {e}"))?;
        Ok(Self { verifying_key })
    }

    /// Verify that `signature_hex` is a valid ECDSA signature over `message`.
    ///
    /// # Errors
    /// Returns an error if the hex or signature bytes are malformed.
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> Result<bool> {
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| anyhow!("sig hex decode: {e}"))?;
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| anyhow!("invalid signature bytes: {e}"))?;
        Ok(self.verifying_key.verify(message, &signature).is_ok())
    }
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::fusion::SensorFusion;
    use crate::types::{AlertLevel, InferenceResult};
    use chrono::Utc;

    fn dummy_result(seq: u64) -> InferenceResult {
        let mut fusion = SensorFusion::new();
        InferenceResult {
            timestamp:         Utc::now(),
            sequence_id:       seq,
            fused_reading:     fusion.sample(seq),
            cognitive_state:   "test state".to_string(),
            recommendations:   vec!["rec a".to_string(), "rec b".to_string()],
            alert_level:       AlertLevel::Normal,
            raw_llm_response:  r#"{"cognitive_state":"test","alert_level":"Normal","recommendations":[]}"#.to_string(),
        }
    }

    #[test]
    fn sign_verify_roundtrip() {
        let signer = EcdsaSigner::generate();
        let result = dummy_result(1);
        let signed = signer.sign_result(&result).unwrap();

        assert!(EcdsaSigner::verify_signed(&signed).unwrap());
    }

    #[test]
    fn tampered_cognitive_state_fails_verification() {
        let signer = EcdsaSigner::generate();
        let mut signed = signer.sign_result(&dummy_result(2)).unwrap();

        signed.inference_result.cognitive_state = "tampered!".to_string();
        assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
    }

    #[test]
    fn tampered_signature_fails_verification() {
        let signer = EcdsaSigner::generate();
        let mut signed = signer.sign_result(&dummy_result(3)).unwrap();

        // Flip the first byte of the signature hex.
        let mut sig = signed.signature_hex.clone();
        let flipped = if sig.starts_with('a') { "b" } else { "a" };
        sig.replace_range(0..1, flipped);
        signed.signature_hex = sig;

        assert!(EcdsaSigner::verify_signed(&signed).is_err() || !EcdsaSigner::verify_signed(&signed).unwrap());
    }

    #[test]
    fn from_hex_roundtrip_preserves_public_key() {
        let original = EcdsaSigner::generate();
        let hex      = original.private_key_hex();
        let restored = EcdsaSigner::from_hex(&hex).unwrap();
        assert_eq!(original.public_key_hex(), restored.public_key_hex());
    }

    #[test]
    fn standalone_verifier_accepts_valid_signature() {
        let signer = EcdsaSigner::generate();
        let result = dummy_result(4);
        let signed = signer.sign_result(&result).unwrap();

        let verifier = EcdsaVerifier::from_hex(&signed.public_key_hex).unwrap();
        let hash     = Sha256::digest(
            serde_json::to_string(&signed.inference_result)
                .unwrap()
                .as_bytes(),
        );
        assert!(verifier.verify(&hash, &signed.signature_hex).unwrap());
    }

    #[test]
    fn cross_key_verification_fails() {
        let signer_a = EcdsaSigner::generate();
        let signer_b = EcdsaSigner::generate();
        let signed   = signer_a.sign_result(&dummy_result(5)).unwrap();

        let verifier_b = EcdsaVerifier::from_hex(&signer_b.public_key_hex()).unwrap();
        let hash       = Sha256::digest(
            serde_json::to_string(&signed.inference_result)
                .unwrap()
                .as_bytes(),
        );
        assert!(!verifier_b.verify(&hash, &signed.signature_hex).unwrap());
    }

    #[test]
    fn public_key_hex_is_130_chars_uncompressed() {
        let signer = EcdsaSigner::generate();
        // Uncompressed SEC 1: 04 || x (32 B) || y (32 B) = 65 bytes = 130 hex chars.
        assert_eq!(signer.public_key_hex().len(), 130);
    }

    #[test]
    fn signature_hex_is_128_chars() {
        let signer = EcdsaSigner::generate();
        let result = dummy_result(6);
        let signed = signer.sign_result(&result).unwrap();
        // Compact r‖s: 64 bytes = 128 hex chars.
        assert_eq!(signed.signature_hex.len(), 128);
    }
}
