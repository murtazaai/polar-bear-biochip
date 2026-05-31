//! Integration tests - ECDSA secp256k1 provenance layer

use polar_bear_biochip::{
    provenance::{EcdsaSigner, EcdsaVerifier},
    sensors::fusion::SensorFusion,
    types::{AlertLevel, InferenceResult},
};
use sha2::Digest;

/// Helper function to create a sample [`InferenceResult`] for testing.
///
/// Returns an [`InferenceResult`] with a sample fused reading and sequence ID.
fn make_result(seq: u64) -> InferenceResult {
    let mut fusion = SensorFusion::new();
    InferenceResult {
        timestamp: chrono::Utc::now(),
        sequence_id: seq,
        fused_reading: fusion.sample(seq),
        cognitive_state: format!("test state {seq}"),
        recommendations: vec!["rec a".to_string(), "rec b".to_string()],
        alert_level: AlertLevel::Normal,
        raw_llm_response:
            r#"{"cognitive_state":"test","alert_level":"Normal","recommendations":[]}"#.to_string(),
    }
}

/// Verifies that the public key is 130 hex characters in length and starts with `04`
/// (uncompressed).
///
/// The public key is uncompressed, so it should start with `04` and be 130 hex characters in
/// length.
///
/// This test ensures that the public key is correctly formatted and can be used for ECDSA
/// operations.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
#[test]
fn public_key_is_130_hex_chars_uncompressed() {
    let signer = EcdsaSigner::generate();
    // Uncompressed SEC 1: 04 || x (32 B) || y (32 B) = 65 bytes → 130 hex chars.
    assert_eq!(signer.public_key_hex().len(), 130);
    assert!(
        signer.public_key_hex().starts_with("04"),
        "uncompressed public key must start with 04"
    );
}

/// Verifies that the verifying key is 66 hex characters in length and starts with `02` or `03`
/// (compressed).
///
/// The verifying key is compressed, so it should start with `02` or `03` and be 66 hex
/// characters in length.
///
/// This test ensures that the verifying key is correctly formatted and can be used for ECDSA
/// operations.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
#[test]
fn verifying_key_is_66_hex_chars_compressed() {
    let signer = EcdsaSigner::generate();
    // Compressed SEC 1: 02/03 || x (32 B) = 33 bytes → 66 hex chars.
    let hex = signer.verifying_key_hex();
    assert_eq!(hex.len(), 66);
    assert!(
        hex.starts_with("02") || hex.starts_with("03"),
        "compressed public key must start with 02 or 03"
    );
}

/// Verifies that the signature and payload hash are correctly formatted and can be used for ECDSA
/// operations.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the signature and payload hash are correctly formatted and can be used
/// for ECDSA operations.
#[test]
#[allow(clippy::similar_names)]
fn sign_verify_roundtrip_is_valid() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(1)).unwrap();
    assert!(EcdsaSigner::verify_signed(&signed).unwrap());
}

/// Verifies that the signature and payload hash are correctly formatted and can be used for ECDSA
/// operations.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the signature and payload hash are correctly formatted and can be used
/// for ECDSA operations.
#[test]
#[allow(clippy::similar_names)]
fn signature_hex_is_128_chars() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(2)).unwrap();
    // Compact r‖s: 64 bytes = 128 hex chars.
    assert_eq!(signed.signature_hex.len(), 128);
}

/// Verifies that the payload hash is correctly formatted and can be used for ECDSA operations.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the payload hash is correctly formatted and can be used for ECDSA
/// operations.
#[test]
#[allow(clippy::similar_names)]
fn payload_hash_hex_is_64_chars() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(3)).unwrap();
    // SHA-256: 32 bytes = 64 hex chars.
    assert_eq!(signed.payload_hash_hex.len(), 64);
}

/// Verifies that tampering with the cognitive state or alert level fails verification.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that tampering with the cognitive state or alert level fails verification.
#[test]
#[allow(clippy::similar_names)]
fn modified_cognitive_state_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(4)).unwrap();
    signed.inference_result.cognitive_state = "tampered!".to_string();
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

/// Verifies that tampering with the alert level fails verification.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that tampering with the alert level fails verification.
#[test]
#[allow(clippy::similar_names)]
fn modified_alert_level_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(5)).unwrap();
    signed.inference_result.alert_level = AlertLevel::Critical;
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

/// Verifies that tampering with the sequence ID fails verification.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that tampering with the sequence ID fails verification.
#[test]
#[allow(clippy::similar_names)]
fn modified_sequence_id_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(6)).unwrap();
    signed.inference_result.sequence_id = 9_999;
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

/// Verifies that the `from_hex` roundtrip preserves the public key.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the `from_hex` roundtrip preserves the public key.
#[test]
fn from_hex_roundtrip_preserves_public_key() {
    let original = EcdsaSigner::generate();
    let restored = EcdsaSigner::from_hex(&original.private_key_hex()).unwrap();
    assert_eq!(original.public_key_hex(), restored.public_key_hex());
}

/// Verifies that invalid hex input returns an error.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that invalid hex input returns an error.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
#[test]
fn from_hex_invalid_input_returns_error() {
    assert!(EcdsaSigner::from_hex("not-valid-hex").is_err());
    assert!(EcdsaSigner::from_hex("deadbeef").is_err()); // too short
}

/// Verifies that the standalone [`EcdsaVerifier`] accepts a valid signed output.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the standalone [`EcdsaVerifier`] accepts a valid signed output.
#[test]
#[allow(clippy::similar_names)]
fn standalone_verifier_accepts_valid_signed_output() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(7)).unwrap();

    let verifier = EcdsaVerifier::from_hex(&signed.public_key_hex).unwrap();
    let hash = sha2::Digest::finalize(sha2::Sha256::new_with_prefix(
        serde_json::to_string(&signed.inference_result)
            .unwrap()
            .as_bytes(),
    ));
    assert!(verifier.verify(&hash, &signed.signature_hex).unwrap());
}

/// Verifies that the standalone [`EcdsaVerifier`] rejects a signed output with the wrong key.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the standalone [`EcdsaVerifier`] rejects a signed output with the wrong
/// key.
#[test]
#[allow(clippy::similar_names)]
fn standalone_verifier_rejects_wrong_key() {
    let signer_a = EcdsaSigner::generate();
    let signer_b = EcdsaSigner::generate();
    let signed = signer_a.sign_result(&make_result(8)).unwrap();

    let verifier_b = EcdsaVerifier::from_hex(&signer_b.public_key_hex()).unwrap();
    let hash = sha2::Digest::finalize(sha2::Sha256::new_with_prefix(
        serde_json::to_string(&signed.inference_result)
            .unwrap()
            .as_bytes(),
    ));
    assert!(!verifier_b.verify(&hash, &signed.signature_hex).unwrap());
}

/// Verifies that the [`EcdsaVerifier`] rejects an invalid hex key.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that the [`EcdsaVerifier`] rejects an invalid hex key.
#[test]
fn verifier_from_hex_invalid_returns_error() {
    assert!(EcdsaVerifier::from_hex("not-a-key").is_err());
}

/// Verifies that multiple cycles produce distinct signatures.
///
/// This test is part of the [`EcdsaSigner`] struct's public key generation and verification.
///
/// This test ensures that multiple cycles produce distinct signatures.
#[test]
fn distinct_results_produce_distinct_signatures() {
    let signer = EcdsaSigner::generate();
    let sig1 = signer.sign_result(&make_result(9)).unwrap().signature_hex;
    let sig2 = signer.sign_result(&make_result(10)).unwrap().signature_hex;
    // Different payloads (different sequence_id + timestamps) must produce different sigs.
    assert_ne!(
        sig1, sig2,
        "distinct payloads must produce distinct signatures"
    );
}
