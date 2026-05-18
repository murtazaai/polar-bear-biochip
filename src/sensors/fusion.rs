//! Sensor fusion layer.
//!
//! Combines the latest BCI (EEG) and accelerometer readings into a single
//! `FusedReading` with three derived higher-order cognitive features:
//!
//! | Feature            | Formula (simplified)                              |
//! |--------------------|---------------------------------------------------|
//! | cognitive_load     | β / (α + θ) normalised, boosted by activity       |
//! | emotional_valence  | (α − β) / (α + β), damped by gamma dominance      |
//! | arousal_level      | (β + γ) / total_power                             |

use crate::types::{ActivityState, FusedReading};
use chrono::Utc;

use super::{accelerometer::AccelerometerSensor, bci::BciSensor};

pub struct SensorFusion {
    bci:   BciSensor,
    accel: AccelerometerSensor,
}

impl SensorFusion {
    pub fn new() -> Self {
        Self {
            bci:   BciSensor::new(),
            accel: AccelerometerSensor::new(),
        }
    }

    /// Sample both sensors and fuse into one `FusedReading`.
    pub fn sample(&mut self, sequence_id: u64) -> FusedReading {
        let bci   = self.bci.sample();
        let accel = self.accel.sample();

        let total_power = bci.delta_hz + bci.theta_hz + bci.alpha_hz
                        + bci.beta_hz  + bci.gamma_hz;

        // ── Cognitive load ──────────────────────────────────────────────────
        // High beta relative to slow waves → higher mental effort.
        // Physical activity adds a modest boost (motor cortex engagement).
        let activity_boost = match accel.activity_state {
            ActivityState::Stationary => 0.0,
            ActivityState::Walking    => 0.05,
            ActivityState::Running    => 0.12,
            ActivityState::Gesture    => 0.08,
        };
        let cognitive_load = ((bci.beta_hz / (bci.alpha_hz + bci.theta_hz + 1.0)) * 0.5
            + activity_boost)
            .clamp(0.0, 1.0);

        // ── Emotional valence ───────────────────────────────────────────────
        // Alpha dominance → positive / calm.
        // Beta / gamma dominance → stress / negative.
        let emotional_valence = ((bci.alpha_hz - bci.beta_hz * 0.6) / (total_power + 1.0))
            .clamp(-1.0, 1.0);

        // ── Arousal level ───────────────────────────────────────────────────
        let arousal_level = ((bci.beta_hz + bci.gamma_hz) / (total_power + 1.0))
            .clamp(0.0, 1.0);

        FusedReading {
            timestamp:         Utc::now(),
            sequence_id,
            bci,
            accelerometer:     accel,
            cognitive_load:    round2(cognitive_load),
            emotional_valence: round2(emotional_valence),
            arousal_level:     round2(arousal_level),
        }
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
