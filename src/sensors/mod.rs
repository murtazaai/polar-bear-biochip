//! Sensor layer: BCI (EEG), accelerometer, and sensor fusion.

pub mod accelerometer;
pub mod bci;
pub mod fusion;

pub use fusion::SensorFusion;
