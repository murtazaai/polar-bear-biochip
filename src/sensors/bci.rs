//! Mock BCI (Brain-Computer Interface) sensor.
//!
//! Simulates an Emotiv EPOC-compatible EEG device producing the five standard
//! frequency bands.  In production this module is replaced by the Emotiv SDK
//! FFI bindings and the ICA artifact-removal pipeline.

use crate::types::BciReading;
use chrono::Utc;
use rand::Rng;

pub struct BciSensor {
    rng: rand::rngs::ThreadRng,
    /// Smoothing factor — adds temporal correlation across samples
    prev_alpha: f64,
    prev_beta: f64,
    prev_theta: f64,
    prev_delta: f64,
    prev_gamma: f64,
}

impl BciSensor {
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

    /// Sample the sensor once.  Values evolve with a random walk bounded by
    /// physiologically plausible ranges for an awake, resting adult.
    pub fn sample(&mut self) -> BciReading {
        let alpha = smooth(self.prev_alpha, 8.0,  12.0, &mut self.rng);
        let beta  = smooth(self.prev_beta,  12.0, 30.0, &mut self.rng);
        let theta = smooth(self.prev_theta,  4.0,  8.0, &mut self.rng);
        let delta = smooth(self.prev_delta,  0.5,  4.0, &mut self.rng);
        let gamma = smooth(self.prev_gamma, 30.0, 70.0, &mut self.rng);

        self.prev_alpha = alpha;
        self.prev_beta  = beta;
        self.prev_theta = theta;
        self.prev_delta = delta;
        self.prev_gamma = gamma;

        // Attention: high beta relative to alpha + theta → focused
        let attention = ((beta / (alpha + theta + 1.0)) * 0.6).clamp(0.0, 1.0);
        // Meditation: high alpha relative to beta + gamma → relaxed
        let meditation = ((alpha / (beta + gamma + 1.0)) * 4.0).clamp(0.0, 1.0);

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

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Random-walk step bounded to [min, max] with 80 % smoothing.
fn smooth(prev: f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng) -> f64 {
    let noise: f64 = (rng.gen::<f64>() - 0.5) * (max - min) * 0.15;
    (prev + noise).clamp(min, max)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
