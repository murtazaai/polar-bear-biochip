//! Mock 3-axis MEMS accelerometer sensor.
//!
//! Simulates a typical wrist-worn or implant-adjacent inertial sensor at
//! 50 Hz with gravity compensation.  In production this maps to the sensor
//! HAL exposed by the bio-chip firmware over BLE GATT.

use crate::types::{AccelerometerReading, ActivityState};
use chrono::Utc;
use rand::Rng;

pub struct AccelerometerSensor {
    rng: rand::rngs::ThreadRng,
    prev_x: f64,
    prev_y: f64,
    /// Gravity component on Z is kept near 9.81 m/s²
    prev_z: f64,
    step_counter: u32,
}

impl AccelerometerSensor {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            prev_x: 0.0,
            prev_y: 0.0,
            prev_z: 9.81,
            step_counter: 0,
        }
    }

    pub fn sample(&mut self) -> AccelerometerReading {
        self.step_counter += 1;

        // Introduce periodic gait oscillation every ~10 cycles → walking simulation
        let gait_phase = (self.step_counter as f64 * 0.63).sin() * 0.8;

        let x = smooth(self.prev_x + gait_phase * 0.3, -2.0, 2.0, &mut self.rng);
        let y = smooth(self.prev_y + gait_phase * 0.2, -2.0, 2.0, &mut self.rng);
        let z = smooth(self.prev_z, 8.5, 11.0, &mut self.rng); // gravity ±noise

        self.prev_x = x;
        self.prev_y = y;
        self.prev_z = z;

        let magnitude = (x * x + y * y + z * z).sqrt();
        let activity  = classify_activity(magnitude, x, y);

        AccelerometerReading {
            timestamp:      Utc::now(),
            x:              round2(x),
            y:              round2(y),
            z:              round2(z),
            magnitude:      round2(magnitude),
            activity_state: activity,
        }
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn classify_activity(mag: f64, x: f64, y: f64) -> ActivityState {
    let lateral = (x * x + y * y).sqrt();
    match (mag, lateral) {
        _ if lateral > 1.5 => ActivityState::Gesture,
        _ if mag > 12.5    => ActivityState::Running,
        _ if mag > 10.8    => ActivityState::Walking,
        _                  => ActivityState::Stationary,
    }
}

fn smooth(prev: f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng) -> f64 {
    let noise: f64 = (rng.gen::<f64>() - 0.5) * (max - min) * 0.10;
    (prev + noise).clamp(min, max)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
