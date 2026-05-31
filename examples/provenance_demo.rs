//! ECDSA secp256k1 provenance demo - no API key required.
//!
//! Demonstrates:
//! - Key generation and serialisation
//! - Signing an `InferenceResult`
//! - Offline verification via `EcdsaVerifier`
//! - Tamper detection (modified field → failed signature)
//!
//! ```text
//! cargo run --example provenance_demo
//! ```

/// Demonstrates ECDSA secp256k1 provenance using a simulated ARC sensor fusion.
use polar_bear_biochip::{
    provenance::{EcdsaSigner, EcdsaVerifier},
    sensors::fusion::SensorFusion,
    types::{AlertLevel, InferenceResult},
};
use sha2::Digest;

/// Runs the provenance demo, generating a key, signing an inference result, and verifying it.
///
/// This demo simulates an ARC sensor fusion, generates an inference result, and demonstrates
/// the use of ECDSA secp256k1 for provenance verification.
#[allow(clippy::similar_names)]
fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║          ECDSA secp256k1 PROVENANCE - DEMO                      ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // ── 1. Key generation ─────────────────────────────────────────────────────
    println!("\n── Key generation ────────────────────────────────────────────────");
    let signer = EcdsaSigner::generate();
    println!(
        "  Private key (32 B) : {}…",
        &signer.private_key_hex()[..16]
    );
    println!(
        "  Public key (65 B, uncompressed): {}…",
        &signer.public_key_hex()[..20]
    );

    // ── 2. Build a dummy InferenceResult ─────────────────────────────────────
    let mut fusion = SensorFusion::new();
    let result = InferenceResult {
        timestamp: chrono::Utc::now(),
        sequence_id: 1,
        fused_reading: fusion.sample(1),
        cognitive_state: "Balanced beta-alpha profile - focused engagement".to_string(),
        recommendations: vec![
            "Maintain current activity".to_string(),
            "Hydrate within 30 minutes".to_string(),
        ],
        alert_level: AlertLevel::Normal,
        raw_llm_response:
            r#"{"cognitive_state":"...","alert_level":"Normal","recommendations":[]}"#.to_string(),
    };

    // ── 3. Sign ───────────────────────────────────────────────────────────────
    println!("\n── Signing ───────────────────────────────────────────────────────");
    let signed = signer.sign_result(&result)?;
    println!("  Payload hash  : {}…", &signed.payload_hash_hex[..20]);
    println!("  Signature     : {}…", &signed.signature_hex[..20]);
    println!("  Signed at     : {}", signed.signed_at);

    // ── 4. Verify (valid) ─────────────────────────────────────────────────────
    println!("\n── Verification (valid) ──────────────────────────────────────────");
    let valid = EcdsaSigner::verify_signed(&signed)?;
    println!(
        "  EcdsaSigner::verify_signed  → {}",
        if valid { "✅ VALID" } else { "❌ INVALID" }
    );

    let verifier = EcdsaVerifier::from_hex(&signed.public_key_hex)?;
    let hash = sha2::Digest::finalize(sha2::Sha256::new_with_prefix(
        serde_json::to_string(&signed.inference_result)?.as_bytes(),
    ));
    let ok2 = verifier.verify(&hash, &signed.signature_hex)?;
    println!(
        "  EcdsaVerifier::verify       → {}",
        if ok2 { "✅ VALID" } else { "❌ INVALID" }
    );

    // ── 5. Tamper + re-verify (must fail) ─────────────────────────────────────
    println!("\n── Tamper detection ──────────────────────────────────────────────");
    let mut tampered = signed.clone();
    tampered.inference_result.cognitive_state = "TAMPERED STATE".to_string();
    let tampered_valid = EcdsaSigner::verify_signed(&tampered)?;
    println!(
        "  After modifying cognitive_state → {}",
        if tampered_valid {
            "❌ BUG: still valid!"
        } else {
            "✅ INVALID (tamper detected)"
        }
    );

    // ── 6. from_hex round-trip ────────────────────────────────────────────────
    println!("\n── Private key round-trip ────────────────────────────────────────");
    let hex = signer.private_key_hex();
    let restored = EcdsaSigner::from_hex(&hex)?;
    println!(
        "  Restored public key matches: {}",
        if restored.public_key_hex() == signer.public_key_hex() {
            "✅ yes"
        } else {
            "❌ no"
        }
    );

    println!("\n✓  Provenance demo complete.");
    Ok(())
}
