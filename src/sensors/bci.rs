//! # BCI Sensor — EEG brainwave simulation
//!
//! Simulates an Emotiv EPOC-compatible EEG device producing the five standard
//! clinical frequency bands.
//!
//! ## Frequency bands
//!
//! | Band  | Range (Hz) | Cognitive correlate |
//! |-------|-----------|---------------------|
//! | Delta | 0.5–4     | Deep sleep, unconscious processing |
//! | Theta | 4–8       | Drowsiness, creativity, memory encoding |
//! | Alpha | 8–12      | Relaxed alertness, idle visual cortex |
//! | Beta  | 12–30     | Active thinking, focus, problem-solving |
//! | Gamma | 30–100    | High-level cognition, cross-cortex binding |
//!
//! In production this module is replaced by the Emotiv SDK FFI bindings and
//! the ICA (Independent Component Analysis) artifact-removal pipeline.

use chrono::Utc;
use rand::Rng;

use crate::types::BciReading;

/// Mock EEG BCI sensor with temporal smoothing.
///
/// Values evolve as a bounded random walk, ensuring realistic temporal
/// correlation between successive samples.
pub struct BciSensor {
    rng: rand::rngs::ThreadRng,
    prev_alpha: f64,
    prev_beta: f64,
    prev_theta: f64,
    prev_delta: f64,
    prev_gamma: f64,
}

impl BciSensor {
    /// Initialise with randomised starting values in the awake resting range.
    #[must_use]
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            prev_alpha: 10.0 + rng.gen::<f64>() * 2.0,
            prev_beta:  18.0 + rng.gen::<f64>() * 4.0,
            prev_theta:  5.5 + rng.gen::<f64>() * 1.0,
            prev_delta:  2.0 + rng.gen::<f64>() * 0.5,
            prev_gamma: 42.0 + rng.gen::<f64>() * 8.0,
            rng,
        }
    }

    /// Sample the sensor once.
    ///
    /// Each call advances the random walk by at most ±15 % of the band range,
    /// clamped to physiologically plausible bounds for an awake resting adult.
    pub fn sample(&mut self) -> BciReading {
        let alpha = smooth(self.prev_alpha,  8.0,  12.0, &mut self.rng);
        let beta  = smooth(self.prev_beta,  12.0,  30.0, &mut self.rng);
        let theta = smooth(self.prev_theta,  4.0,   8.0, &mut self.rng);
        let delta = smooth(self.prev_delta,  0.5,   4.0, &mut self.rng);
        let gamma = smooth(self.prev_gamma, 30.0,  70.0, &mut self.rng);

        self.prev_alpha = alpha;
        self.prev_beta  = beta;
        self.prev_theta = theta;
        self.prev_delta = delta;
        self.prev_gamma = gamma;

        // Attention:  high beta relative to slow waves → focused.
        let attention  = ((beta  / (alpha + theta + 1.0)) * 0.6).clamp(0.0, 1.0);
        // Meditation: high alpha relative to fast waves → relaxed.
        let meditation = ((alpha / (beta  + gamma  + 1.0)) * 4.0).clamp(0.0, 1.0);

        BciReading {
            timestamp:        Utc::now(),
            delta_hz:         round2(delta),
            theta_hz:         round2(theta),
            alpha_hz:         round2(alpha),
            beta_hz:          round2(beta),
            gamma_hz:         round2(gamma),
            attention_index:  round2(attention),
            meditation_index: round2(meditation),
        }
    }
}

impl Default for BciSensor {
    fn default() -> Self {
        Self::new()
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Advance `prev` by a bounded random step and clamp to `[min, max]`.
fn smooth(prev: f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng) -> f64 {
    let noise: f64 = (rng.gen::<f64>() - 0.5) * (max - min) * 0.15;
    (prev + noise).clamp(min, max)
}

/// Round to 2 decimal places (avoids noisy float tails in JSON output).
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_bands_within_clinical_range() {
        let mut sensor = BciSensor::new();
        for _ in 0..50 {
            let r = sensor.sample();
            assert!(
                (0.5..=4.0).contains(&r.delta_hz),
                "delta out of range: {}",
                r.delta_hz
            );
            assert!(
                (4.0..=8.0).contains(&r.theta_hz),
                "theta out of range: {}",
                r.theta_hz
            );
            assert!(
                (8.0..=12.0).contains(&r.alpha_hz),
                "alpha out of range: {}",
                r.alpha_hz
            );
            assert!(
                (12.0..=30.0).contains(&r.beta_hz),
                "beta out of range: {}",
                r.beta_hz
            );
            assert!(
                (30.0..=70.0).contains(&r.gamma_hz),
                "gamma out of range: {}",
                r.gamma_hz
            );
        }
    }

    #[test]
    fn derived_indices_within_unit_range() {
        let mut sensor = BciSensor::new();
        for _ in 0..50 {
            let r = sensor.sample();
            assert!(
                (0.0..=1.0).contains(&r.attention_index),
                "attention out of [0,1]: {}",
                r.attention_index
            );
            assert!(
                (0.0..=1.0).contains(&r.meditation_index),
                "meditation out of [0,1]: {}",
                r.meditation_index
            );
        }
    }

    #[test]
    fn default_constructs_without_panic() {
        let mut sensor = BciSensor::default();
        let r = sensor.sample();
        assert!(r.alpha_hz > 0.0);
    }
}
