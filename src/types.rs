//! Shared data structures for the Bio-Chip Intelligence Framework.
//!
//! Data flows:
//!   BciReading + AccelerometerReading → FusedReading → InferenceResult → SignedOutput

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Raw sensor readings ──────────────────────────────────────────────────────

/// EEG brainwave reading from the BCI sensor (Emotiv EPOC-compatible).
/// Frequency bands follow the standard clinical EEG taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciReading {
    pub timestamp: DateTime<Utc>,
    /// Delta  0.5–4 Hz  — deep sleep / unconscious processing
    pub delta_hz: f64,
    /// Theta  4–8 Hz    — drowsiness, creativity, memory encoding
    pub theta_hz: f64,
    /// Alpha  8–12 Hz   — relaxed alertness, idle visual cortex
    pub alpha_hz: f64,
    /// Beta   12–30 Hz  — active thinking, focus, problem-solving
    pub beta_hz: f64,
    /// Gamma  30–100 Hz — high-level cognition, cross-cortex binding
    pub gamma_hz: f64,
    /// Derived attention index  [0.0 – 1.0]
    pub attention_index: f64,
    /// Derived meditation index [0.0 – 1.0]
    pub meditation_index: f64,
}

/// 3-axis accelerometer reading (MEMS sensor, units: m/s²).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelerometerReading {
    pub timestamp: DateTime<Utc>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Euclidean magnitude √(x²+y²+z²)
    pub magnitude: f64,
    pub activity_state: ActivityState,
}

/// Inferred physical activity state from accelerometer data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityState {
    Stationary,
    Walking,
    Running,
    Gesture,
}

// ─── Fused reading ────────────────────────────────────────────────────────────

/// Sensor-fused reading combining BCI + accelerometer into higher-order features.
/// This is the payload sent to the rig-core LLM agent for cognitive inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedReading {
    pub timestamp: DateTime<Utc>,
    pub sequence_id: u64,
    pub bci: BciReading,
    pub accelerometer: AccelerometerReading,
    /// Derived cognitive load   [0.0 – 1.0]  (high beta + low alpha → higher load)
    pub cognitive_load: f64,
    /// Derived emotional valence [-1.0 – +1.0] (negative = stress, positive = calm)
    pub emotional_valence: f64,
    /// Derived arousal level    [0.0 – 1.0]  (gamma + beta dominance)
    pub arousal_level: f64,
}

// ─── Inference result ─────────────────────────────────────────────────────────

/// Output of the rig-core LLM agent after analysing a FusedReading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub timestamp: DateTime<Utc>,
    pub sequence_id: u64,
    pub fused_reading: FusedReading,
    /// One-line cognitive state summary produced by the LLM
    pub cognitive_state: String,
    /// Actionable recommendations (2–4 bullet points)
    pub recommendations: Vec<String>,
    pub alert_level: AlertLevel,
    /// Raw LLM response preserved for audit
    pub raw_llm_response: String,
}

/// Alert severity derived from the LLM's cognitive state classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Readings within expected healthy operating range
    Normal,
    /// Elevated cognitive/physical stress — attention warranted
    Elevated,
    /// Critical anomaly detected — immediate intervention required
    Critical,
}

// ─── Signed output (provenance layer) ────────────────────────────────────────

/// ECDSA-signed wrapper around an InferenceResult.
/// Written to disk as JSON; verifiable offline using the embedded public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOutput {
    pub inference_result: InferenceResult,
    /// Hex-encoded SHA-256 digest of the canonical JSON payload
    pub payload_hash_hex: String,
    /// Hex-encoded compact (r||s) ECDSA secp256k1 signature (64 bytes)
    pub signature_hex: String,
    /// Hex-encoded uncompressed secp256k1 public key (65 bytes)
    pub public_key_hex: String,
    pub signed_at: DateTime<Utc>,
}
