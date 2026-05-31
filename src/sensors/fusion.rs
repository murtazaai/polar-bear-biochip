//! # Sensor Fusion
//!
//! Combines the latest BCI (EEG) and accelerometer readings into a single
//! [`FusedReading`] with three higher-order cognitive feature scalars.
//!
//! ## Feature derivation
//!
//! | Feature            | Formula (simplified)                            | Range       |
//! |--------------------|--------------------------------------------------|-------------|
//! | `cognitive_load`   | β / (α + θ) + activity_boost                    | \[0.0, 1.0\] |
//! | `emotional_valence`| (α − 0.6·β) / total_power                       | \[-1.0, 1.0\]|
//! | `arousal_level`    | (β + γ) / total_power                           | \[0.0, 1.0\] |
//!
//! The `activity_boost` adds a small increment to cognitive load when the
//! accelerometer reports Walking (0.05), Gesture (0.08), or Running (0.12),
//! reflecting motor-cortex engagement.

/// Fuses a [`BciSensor`] and an [`AccelerometerSensor`] into a single-call sample.
use chrono::Utc;

use crate::{
    sensors::{accelerometer::AccelerometerSensor, bci::BciSensor},
    types::{ActivityState, FusedReading},
};

/// Combines a [`BciSensor`] and an [`AccelerometerSensor`] into a
/// single-call fused sample.
///
/// The `sequence_id` is typically the inference cycle counter.
///
/// The [`FusedReading`] contains the timestamp, sequence ID, and sensor readings.
pub struct SensorFusion {
    bci: BciSensor,
    accel: AccelerometerSensor,
}

/// Constructs a [`SensorFusion`] with freshly initialised sensors.
///
/// The sensors are initialised with a fresh random number generator.
impl SensorFusion {
    /// Construct with freshly initialised sensors.
    ///
    /// The sensors are initialised with a fresh random number generator.
    ///
    /// # Returns
    ///
    /// A new [`SensorFusion`] instance with freshly initialised sensors.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bci: BciSensor::new(),
            accel: AccelerometerSensor::new(),
        }
    }

    /// Sample both sensors and fuse into one [`FusedReading`].
    ///
    /// `sequence_id` is typically the inference cycle counter.
    ///
    /// # Returns
    ///
    /// A [`FusedReading`] containing the timestamp, sequence ID, and sensor readings.
    pub fn sample(&mut self, sequence_id: u64) -> FusedReading {
        let bci = self.bci.sample();
        let accel = self.accel.sample();

        let total_power = bci.delta_hz + bci.theta_hz + bci.alpha_hz + bci.beta_hz + bci.gamma_hz;

        // ── Cognitive load ────────────────────────────────────────────────────
        let activity_boost = match accel.activity_state {
            ActivityState::Stationary => 0.0,
            ActivityState::Walking => 0.05,
            ActivityState::Running => 0.12,
            ActivityState::Gesture => 0.08,
        };
        let cognitive_load = ((bci.beta_hz / (bci.alpha_hz + bci.theta_hz + 1.0)) * 0.5
            + activity_boost)
            .clamp(0.0, 1.0);

        // ── Emotional valence ─────────────────────────────────────────────────
        // Alpha dominance → positive / calm; beta/gamma → stress / negative.
        let emotional_valence =
            ((bci.alpha_hz - bci.beta_hz * 0.6) / (total_power + 1.0)).clamp(-1.0, 1.0);

        // ── Arousal level ─────────────────────────────────────────────────────
        let arousal_level = ((bci.beta_hz + bci.gamma_hz) / (total_power + 1.0)).clamp(0.0, 1.0);

        FusedReading {
            timestamp: Utc::now(),
            sequence_id,
            bci,
            accelerometer: accel,
            cognitive_load: round2(cognitive_load),
            emotional_valence: round2(emotional_valence),
            arousal_level: round2(arousal_level),
        }
    }
}

/// Default implementation uses [`SensorFusion::new`].
///
/// This allows `SensorFusion` to be used as a [`Default`] type,
/// with freshly initialised sensors.
///
/// ```
/// let fusion = SensorFusion::default();
/// ```
///
/// This is equivalent to `SensorFusion::new()`.
impl Default for SensorFusion {
    fn default() -> Self {
        Self::new()
    }
}

/// Rounds a value to two decimal places.
///
/// ```
/// let rounded = round2(1.2345);
/// assert_eq!(rounded, 1.23);
/// ```
///
/// This is used internally to round sensor readings to two decimal places.
///
/// ```
/// let rounded = round2(1.2345);
/// assert_eq!(rounded, 1.23);
/// ```
///
/// This function is also used to round sensor readings to two decimal places in the `sample`
/// method.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Unit tests for the `SensorFusion` struct.
///
/// These tests verify that the `SensorFusion` struct correctly fuses sensor readings
/// and produces output within the expected bounds.
#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that fused features are within the expected bounds.
    ///
    /// This test samples the `SensorFusion` struct multiple times and verifies that
    /// the fused features remain within the expected range of [0,1] for cognitive load,
    /// [-1,1] for emotional valence, and [0,1] for arousal level.
    ///
    /// This test uses a loop to sample the `SensorFusion` struct 50 times and verifies
    /// that the fused features remain within the expected bounds.
    #[test]
    fn fused_features_within_expected_bounds() {
        let mut fusion = SensorFusion::new();
        for id in 1..=50_u64 {
            let r = fusion.sample(id);
            assert!(
                (0.0..=1.0).contains(&r.cognitive_load),
                "cognitive_load out of [0,1]: {}",
                r.cognitive_load
            );
            assert!(
                (-1.0..=1.0).contains(&r.emotional_valence),
                "emotional_valence out of [-1,1]: {}",
                r.emotional_valence
            );
            assert!(
                (0.0..=1.0).contains(&r.arousal_level),
                "arousal_level out of [0,1]: {}",
                r.arousal_level
            );
        }
    }

    /// Verifies that the sequence ID is preserved across multiple samples.
    ///
    /// This test samples the `SensorFusion` struct with a sequence ID and verifies that
    /// the sequence ID is preserved in the result.
    ///
    /// This test uses a loop to sample the `SensorFusion` struct with a sequence ID and
    /// verifies that the sequence ID is preserved in the result.
    ///
    /// This test verifies that the sequence ID is preserved across multiple samples.
    #[test]
    fn sequence_id_preserved() {
        let mut fusion = SensorFusion::new();
        for id in [1_u64, 42, 1_000, u64::MAX / 2] {
            let r = fusion.sample(id);
            assert_eq!(r.sequence_id, id);
        }
    }

    /// Verifies that the default constructor does not panic.
    ///
    /// This test verifies that the default constructor does not panic.
    #[test]
    fn default_constructs_without_panic() {
        let mut fusion = SensorFusion::default();
        let r = fusion.sample(1);
        assert!(r.cognitive_load >= 0.0);
    }
}
