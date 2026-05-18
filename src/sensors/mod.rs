//! Sensor layer: BCI, accelerometer, and fusion.

pub mod accelerometer;
pub mod bci;
pub mod fusion;

pub use fusion::SensorFusion;
