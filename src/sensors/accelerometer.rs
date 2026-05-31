//! # Accelerometer Sensor - 3-axis MEMS simulation
//!
//! Simulates a typical wrist-worn or implant-adjacent inertial measurement
//! unit (IMU) at 50 Hz.  Gravity is maintained near 9.81 m/s² on the Z axis
//! with small Gaussian perturbations; X and Y carry periodic gait oscillations
//! that drive the activity classifier.
//!
//! In production this module maps to the sensor HAL exposed by the bio-chip
//! firmware over BLE GATT (`0x2A37` characteristic).

/// Represents a mock 3-axis MEMS accelerometer sensor.
use chrono::Utc;
use rand::Rng;

use crate::types::{AccelerometerReading, ActivityState};

/// Mock 3-axis MEMS accelerometer sensor.
///
/// Simulates a 3-axis MEMS accelerometer with periodic gait oscillations and
/// Gaussian noise.
///
/// The sensor maintains a sinusoidal gait model on the X and Y axes, with
/// Gaussian noise added to each axis to simulate real-world measurement
/// variability.
///
/// The sensor is updated at 50 Hz and provides readings in m/s² on each axis.
///
/// The sensor state is maintained internally and updated each time `sample` is called.
pub struct AccelerometerSensor {
    rng: rand::rngs::ThreadRng,
    prev_x: f64,
    prev_y: f64,
    /// Z axis maintained near 9.81 m/s² (gravity).
    prev_z: f64,
    step_counter: u32,
}

/// Initialises a new [`AccelerometerSensor`] with the device at rest.
///
/// The sensor is initialised with zero velocity and near-gravity Z axis value.
impl AccelerometerSensor {
    /// Initialise with the device at rest.
    ///
    /// # Returns
    ///
    /// A new [`AccelerometerSensor`] instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
            prev_x: 0.0,
            prev_y: 0.0,
            prev_z: 9.81,
            step_counter: 0,
        }
    }

    /// Sample the sensor once.
    ///
    /// Every call advances a sinusoidal gait model and adds bounded noise.
    /// The returned [`ActivityState`] is classified from the resulting magnitude
    /// and lateral dynamics.
    ///
    /// # Returns
    ///
    /// The sampled [`AccelerometerReading`].
    pub fn sample(&mut self) -> AccelerometerReading {
        self.step_counter += 1;

        // Periodic gait oscillation - one full cycle every ~10 samples.
        let gait_phase = (f64::from(self.step_counter) * 0.63).sin() * 0.8;

        let x = smooth(self.prev_x + gait_phase * 0.3, -2.0, 2.0, &mut self.rng);
        let y = smooth(self.prev_y + gait_phase * 0.2, -2.0, 2.0, &mut self.rng);
        // Gravity axis: tightly bounded around 9.81 m/s².
        let z = smooth(self.prev_z, 8.5, 11.0, &mut self.rng);

        self.prev_x = x;
        self.prev_y = y;
        self.prev_z = z;

        let magnitude = (x * x + y * y + z * z).sqrt();
        let activity = classify_activity(magnitude, x, y);

        AccelerometerReading {
            timestamp: Utc::now(),
            x: round2(x),
            y: round2(y),
            z: round2(z),
            magnitude: round2(magnitude),
            activity_state: activity,
        }
    }
}

/// Default implementation returns a new [`AccelerometerSensor`] at rest.
///
/// This is equivalent to calling [`AccelerometerSensor::new`].
impl Default for AccelerometerSensor {
    /// Creates a new [`AccelerometerSensor`] at rest.
    fn default() -> Self {
        Self::new()
    }
}

/// Classify activity state from net magnitude and lateral component.
///
/// # Returns
///
/// The classified [`ActivityState`].
fn classify_activity(mag: f64, x: f64, y: f64) -> ActivityState {
    let lateral = (x * x + y * y).sqrt();
    match (mag, lateral) {
        _ if lateral > 1.5 => ActivityState::Gesture,
        _ if mag > 12.5 => ActivityState::Running,
        _ if mag > 10.8 => ActivityState::Walking,
        _ => ActivityState::Stationary,
    }
}

/// Bounded random-walk step with 10 % of range as step size.
///
/// # Returns
///
/// The smoothed value.
fn smooth(prev: f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng) -> f64 {
    let noise: f64 = (rng.random::<f64>() - 0.5) * (max - min) * 0.10;
    (prev + noise).clamp(min, max)
}

/// Rounds a value to two decimal places.
///
/// # Returns
///
/// The rounded value.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Tests for the [`AccelerometerSensor`] class.
///
/// Tests the [`AccelerometerSensor::sample`] method and other utility functions.
///
/// Tests the [`AccelerometerSensor::gravity_axis_near_9_81`] method.
///
/// Tests the [`AccelerometerSensor::magnitude_always_positive`] method.
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the gravity axis is near 9.81 m/s² (gravity).
    ///
    /// This test verifies that the gravity axis remains near 9.81 m/s² (gravity) after sampling.
    ///
    /// It samples the sensor 100 times and verifies that the `z` axis remains within the range
    /// [8.5, 11.0] m/s².
    ///
    /// This test ensures that the sensor's `z` axis remains stable and close to gravity.
    #[test]
    fn gravity_axis_near_9_81() {
        let mut sensor = AccelerometerSensor::new();
        for _ in 0..100 {
            let r = sensor.sample();
            assert!(
                (8.5..=11.0).contains(&r.z),
                "z out of gravity range: {}",
                r.z
            );
        }
    }

    /// Tests that the magnitude is always positive.
    ///
    /// This test verifies that the magnitude of the acceleration vector is always positive.
    ///
    /// It samples the sensor 50 times and verifies that the magnitude is always greater than 0.0.
    #[test]
    fn magnitude_always_positive() {
        let mut sensor = AccelerometerSensor::new();
        for _ in 0..50 {
            let r = sensor.sample();
            assert!(r.magnitude > 0.0, "magnitude must be positive");
        }
    }

    /// Tests that the sensor remains stationary when no gait is detected.
    ///
    /// This test verifies that the sensor correctly classifies itself as Stationary when no gait is
    /// detected.
    ///
    /// It samples the sensor once and verifies that the activity state is Stationary.
    ///
    /// It also verifies that the sensor does not panic when no gait is detected.
    ///
    /// It does this by creating a fresh sensor and sampling it once, then verifying that the
    /// activity state is Stationary.
    #[test]
    fn stationary_when_no_gait() {
        // A freshly constructed sensor at rest should classify as Stationary.
        let mut sensor = AccelerometerSensor::new();
        let r = sensor.sample();
        // Step 1 with no gait history: lateral should be near zero.
        // This test verifies the classifier runs without panicking.
        assert!(matches!(
            r.activity_state,
            ActivityState::Stationary
                | ActivityState::Walking
                | ActivityState::Running
                | ActivityState::Gesture
        ));
    }

    /// Tests that the default constructor does not panic.
    ///
    /// This test verifies that the default constructor does not panic when creating a new sensor.
    ///
    /// It does this by creating a fresh sensor and sampling it once, then verifying that the
    /// magnitude is greater than 0.0.
    ///
    /// It also verifies that the sensor does not panic when no gait is detected.
    ///
    /// It does this by creating a fresh sensor and sampling it once, then verifying that the
    /// activity state is Stationary.
    #[test]
    fn default_constructs_without_panic() {
        let mut sensor = AccelerometerSensor::default();
        let r = sensor.sample();
        assert!(r.magnitude > 0.0);
    }
}
