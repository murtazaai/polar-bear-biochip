//! Integration tests - ECDSA secp256k1 provenance layer

use polar_bear_biochip::{
    provenance::{EcdsaSigner, EcdsaVerifier},
    sensors::fusion::SensorFusion,
    types::{AlertLevel, InferenceResult},
};

// ── Test helper ───────────────────────────────────────────────────────────────

fn make_result(seq: u64) -> InferenceResult {
    let mut fusion = SensorFusion::new();
    InferenceResult {
        timestamp:        chrono::Utc::now(),
        sequence_id:      seq,
        fused_reading:    fusion.sample(seq),
        cognitive_state:  format!("test state {seq}"),
        recommendations:  vec!["rec a".to_string(), "rec b".to_string()],
        alert_level:      AlertLevel::Normal,
        raw_llm_response: r#"{"cognitive_state":"test","alert_level":"Normal","recommendations":[]}"#
            .to_string(),
    }
}

// ── Public key geometry ───────────────────────────────────────────────────────

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

// ── Sign / verify round-trip ──────────────────────────────────────────────────

#[test]
fn sign_verify_roundtrip_is_valid() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(1)).unwrap();
    assert!(EcdsaSigner::verify_signed(&signed).unwrap());
}

#[test]
fn signature_hex_is_128_chars() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(2)).unwrap();
    // Compact r‖s: 64 bytes = 128 hex chars.
    assert_eq!(signed.signature_hex.len(), 128);
}

#[test]
fn payload_hash_hex_is_64_chars() {
    let signer = EcdsaSigner::generate();
    let signed = signer.sign_result(&make_result(3)).unwrap();
    // SHA-256: 32 bytes = 64 hex chars.
    assert_eq!(signed.payload_hash_hex.len(), 64);
}

// ── Tamper detection ──────────────────────────────────────────────────────────

#[test]
fn modified_cognitive_state_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(4)).unwrap();
    signed.inference_result.cognitive_state = "tampered!".to_string();
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

#[test]
fn modified_alert_level_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(5)).unwrap();
    signed.inference_result.alert_level = AlertLevel::Critical;
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

#[test]
fn modified_sequence_id_fails_verification() {
    let signer = EcdsaSigner::generate();
    let mut signed = signer.sign_result(&make_result(6)).unwrap();
    signed.inference_result.sequence_id = 9_999;
    assert!(!EcdsaSigner::verify_signed(&signed).unwrap());
}

// ── Key operations ────────────────────────────────────────────────────────────

#[test]
fn from_hex_roundtrip_preserves_public_key() {
    let original = EcdsaSigner::generate();
    let restored = EcdsaSigner::from_hex(&original.private_key_hex()).unwrap();
    assert_eq!(original.public_key_hex(), restored.public_key_hex());
}

#[test]
fn from_hex_invalid_input_returns_error() {
    assert!(EcdsaSigner::from_hex("not-valid-hex").is_err());
    assert!(EcdsaSigner::from_hex("deadbeef").is_err()); // too short
}

// ── Standalone EcdsaVerifier ──────────────────────────────────────────────────

#[test]
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

#[test]
fn standalone_verifier_rejects_wrong_key() {
    let signer_a = EcdsaSigner::generate();
    let signer_b = EcdsaSigner::generate();
    let signed   = signer_a.sign_result(&make_result(8)).unwrap();

    let verifier_b = EcdsaVerifier::from_hex(&signer_b.public_key_hex()).unwrap();
    let hash       = sha2::Digest::finalize(sha2::Sha256::new_with_prefix(
        serde_json::to_string(&signed.inference_result)
            .unwrap()
            .as_bytes(),
    ));
    assert!(!verifier_b.verify(&hash, &signed.signature_hex).unwrap());
}

#[test]
fn verifier_from_hex_invalid_returns_error() {
    assert!(EcdsaVerifier::from_hex("not-a-key").is_err());
}

// ── Multiple cycles produce distinct signatures ───────────────────────────────

#[test]
fn distinct_results_produce_distinct_signatures() {
    let signer = EcdsaSigner::generate();
    let sig1   = signer.sign_result(&make_result(9)).unwrap().signature_hex;
    let sig2   = signer.sign_result(&make_result(10)).unwrap().signature_hex;
    // Different payloads (different sequence_id + timestamps) must produce different sigs.
    assert_ne!(sig1, sig2, "distinct payloads must produce distinct signatures");
}
