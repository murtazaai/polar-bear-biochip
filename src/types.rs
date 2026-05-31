//! Shared data structures for the Bio-Chip Intelligence Framework.
//!
//! ## Data flow
//!
//! ```text
//! BciReading ─┐
//!             ├─► FusedReading ─► InferenceResult ─► SignedOutput
//! AccelReading┘
//! ```

/// Represents a raw sensor reading from the BCI sensor (Emotiv EPOC-compatible).
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Raw sensor readings ───────────────────────────────────────────────────────

/// EEG brainwave reading from the BCI sensor (Emotiv EPOC-compatible).
///
/// Frequency bands follow the standard clinical EEG taxonomy (see `docs/bci_math.md`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BciReading {
    /// UTC timestamp of this sample.
    pub timestamp: DateTime<Utc>,
    /// Delta band  0.5–4 Hz  - deep sleep / unconscious processing.
    pub delta_hz: f64,
    /// Theta band  4–8 Hz   - drowsiness, creativity, memory encoding.
    pub theta_hz: f64,
    /// Alpha band  8–12 Hz  - relaxed alertness, idle visual cortex.
    pub alpha_hz: f64,
    /// Beta band   12–30 Hz - active thinking, focus, problem-solving.
    pub beta_hz: f64,
    /// Gamma band  30–100 Hz - high-level cognition, cross-cortex binding.
    pub gamma_hz: f64,
    /// Derived attention index  \[0.0 – 1.0\]  (β / (α + θ) normalised).
    pub attention_index: f64,
    /// Derived meditation index \[0.0 – 1.0\]  (α / (β + γ) normalised).
    pub meditation_index: f64,
}

/// Represents a raw sensor reading from the BCI sensor (Emotiv EPOC-compatible).
///
/// This struct holds the raw sensor readings from the BCI sensor, including delta, theta, alpha,
/// beta, and gamma band frequencies, as well as derived attention and meditation indices.
impl fmt::Display for BciReading {
    /// Formats this reading as a human-readable string.
    ///
    /// The output includes the delta, theta, alpha, beta, and gamma band frequencies, as well as
    /// the derived attention and meditation indices.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "δ={:.1} θ={:.1} α={:.1} β={:.1} γ={:.1} Hz | attn={:.2} med={:.2}",
            self.delta_hz,
            self.theta_hz,
            self.alpha_hz,
            self.beta_hz,
            self.gamma_hz,
            self.attention_index,
            self.meditation_index,
        )
    }
}

/// 3-axis MEMS accelerometer reading (units: m/s²).
///
/// This struct holds the raw accelerometer readings from the MEMS sensor, including lateral,
/// sagittal, and vertical acceleration, as well as the inferred physical activity state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelerometerReading {
    /// UTC timestamp of this sample.
    pub timestamp: DateTime<Utc>,
    /// Lateral acceleration (m/s²).
    pub x: f64,
    /// Sagittal acceleration (m/s²).
    pub y: f64,
    /// Vertical acceleration including gravity ≈ 9.81 m/s² at rest (m/s²).
    pub z: f64,
    /// Euclidean magnitude √(x²+y²+z²).
    pub magnitude: f64,
    /// Inferred physical activity state.
    pub activity_state: ActivityState,
}

/// Inferred physical activity state derived from accelerometer magnitude and
/// lateral dynamics.
///
/// This enum represents the inferred physical activity state based on the accelerometer readings.
///
/// The `Display` implementation formats the reading as a human-readable string.
impl fmt::Display for AccelerometerReading {
    /// Formats the reading as a human-readable string.
    ///
    /// The string includes the x, y, z acceleration values and the inferred activity state.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "x={:+.2} y={:+.2} z={:.2} m/s² | {:?}",
            self.x, self.y, self.z, self.activity_state,
        )
    }
}

/// Inferred physical activity state derived from accelerometer magnitude and
/// lateral dynamics.
///
/// This enum represents the inferred physical activity state based on the accelerometer readings.
///
/// The `Display` implementation formats the state as a human-readable string.
///
/// The `Debug` implementation formats the state as a human-readable string.
///
/// The `Serialize` and `Deserialize` implementations allow the state to be encoded and decoded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityState {
    /// Net magnitude ≤ 10.8 m/s² - no significant motion.
    Stationary,
    /// Periodic gait oscillation - 10.8–12.5 m/s².
    Walking,
    /// High-magnitude gait - > 12.5 m/s².
    Running,
    /// High lateral jerk - rapid directional change (gesture).
    Gesture,
}

/// Sensor-fused reading combining BCI + accelerometer into higher-order
/// cognitive features. This is the payload forwarded to the LLM agent.
///
/// The `Debug` implementation formats the reading as a human-readable string.
///
/// The `Serialize` and `Deserialize` implementations allow the reading to be encoded and decoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedReading {
    /// UTC timestamp of fusion.
    pub timestamp: DateTime<Utc>,
    /// Monotonically increasing cycle counter.
    pub sequence_id: u64,
    /// Raw BCI sample contributing to this fusion.
    pub bci: BciReading,
    /// Raw accelerometer sample contributing to this fusion.
    pub accelerometer: AccelerometerReading,
    /// Derived cognitive load   \[0.0 – 1.0\] - high β + low α → higher load.
    pub cognitive_load: f64,
    /// Derived emotional valence \[-1.0 – +1.0\] - negative = stress, positive = calm.
    pub emotional_valence: f64,
    /// Derived arousal level    \[0.0 – 1.0\] - γ + β dominance.
    pub arousal_level: f64,
}

/// Formats the reading as a human-readable string.
///
/// The string includes the sequence ID, cognitive load, emotional valence, and arousal level.
///
/// The format is: `#<sequence_id> cogLoad=<cognitive_load> valence=<emotional_valence>
/// arousal=<arousal_level>`
impl fmt::Display for FusedReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} cogLoad={:.2} valence={:+.2} arousal={:.2}",
            self.sequence_id, self.cognitive_load, self.emotional_valence, self.arousal_level,
        )
    }
}

/// Output of the rig-core LLM agent after analysing a [`FusedReading`].
///
/// The `Debug` implementation formats the result as a human-readable string.
///
/// The `Serialize` and `Deserialize` implementations allow the result to be encoded and decoded.
///
/// The `Display` implementation formats the result as a human-readable string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// UTC timestamp of the inference.
    pub timestamp: DateTime<Utc>,
    /// Mirrors [`FusedReading::sequence_id`].
    pub sequence_id: u64,
    /// The fused sensor reading that triggered this inference.
    pub fused_reading: FusedReading,
    /// One-line cognitive state summary generated by the LLM.
    pub cognitive_state: String,
    /// Actionable recommendations (2–4 bullet points).
    pub recommendations: Vec<String>,
    /// Severity classification.
    pub alert_level: AlertLevel,
    /// Raw LLM response preserved verbatim for audit and replay.
    pub raw_llm_response: String,
}

/// Alert severity derived from the LLM's cognitive state classification.
///
/// The `Display` implementation formats the severity as a human-readable string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Readings within expected healthy operating range.
    Normal,
    /// Elevated cognitive/physical stress - attention warranted.
    Elevated,
    /// Critical anomaly detected - immediate intervention required.
    Critical,
}

/// Formats the alert level as a human-readable string.
///
/// The string is formatted as `✅ Normal`, `⚠️  Elevated`, or `🚨 CRITICAL`.
///
/// The `Debug` implementation formats the severity as a human-readable string.
///
/// The `Display` implementation formats the severity as a human-readable string.
impl fmt::Display for AlertLevel {
    /// Formats the alert level as a human-readable string.
    ///
    /// The string is formatted as `✅ Normal`, `⚠️  Elevated`, or `🚨 CRITICAL`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertLevel::Normal => write!(f, "✅ Normal"),
            AlertLevel::Elevated => write!(f, "⚠️  Elevated"),
            AlertLevel::Critical => write!(f, "🚨 CRITICAL"),
        }
    }
}

/// ECDSA-signed wrapper around an [`InferenceResult`].
///
/// Written to disk as pretty-printed JSON and verifiable offline using the
/// embedded uncompressed secp256k1 public key.
///
/// ## Tamper evidence
/// Any modification to `inference_result` invalidates the SHA-256 hash,
/// which in turn invalidates the ECDSA signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOutput {
    /// The inference result being attested.
    pub inference_result: InferenceResult,
    /// Hex-encoded SHA-256 digest of the canonical JSON payload (64 chars).
    pub payload_hash_hex: String,
    /// Hex-encoded compact (r‖s) ECDSA secp256k1 signature (128 chars).
    pub signature_hex: String,
    /// Hex-encoded uncompressed secp256k1 public key 04‖x‖y (130 chars).
    pub public_key_hex: String,
    /// UTC timestamp of the signing operation.
    pub signed_at: DateTime<Utc>,
}
