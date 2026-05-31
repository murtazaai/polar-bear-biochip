//! Sensor layer: BCI (EEG), accelerometer, and sensor fusion.

/// Module for the accelerometer sensor.
pub mod accelerometer;
/// Module for the BCI (EEG) sensor.
pub mod bci;
/// Module for the sensor fusion algorithm.
pub mod fusion;

/// Re-exports the `SensorFusion` struct from the `fusion` module.
pub use fusion::SensorFusion;
