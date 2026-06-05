//! # polar-bear-biochip
//!
//! Bio-chip intelligence framework for Polar Bear (🍨).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    polar-bear-biochip                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  sensors::bci           EEG (δ θ α β γ bands + indices)    │
//! │  sensors::accelerometer 3-axis MEMS (m/s²)                  │
//! │  sensors::fusion        SensorFusion → FusedReading         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  agent::biochip_agent   rig-core LLM agent (ai-agent feat) │
//! │                         curl fallback (no feature flag)     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  provenance::ecdsa_signer  secp256k1 ECDSA sign + verify    │
//! │                            SHA-256 payload hashing          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  types   BciReading · AccelerometerReading · FusedReading   │
//! │          InferenceResult · SignedOutput · AlertLevel        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use polar_bear_biochip::{
//!     sensors::fusion::SensorFusion,
//!     provenance::ecdsa_signer::EcdsaSigner,
//! };
//!
//! let mut fusion = SensorFusion::new();
//! let reading = fusion.sample(1);
//! println!("Cognitive load: {:.2}", reading.cognitive_load);
//!
//! let signer = EcdsaSigner::generate();
//! println!("Public key: {}", signer.public_key_hex());
//! ```

/// Re-exports error types for use by callers.
pub mod agent;
/// Re-exports error types for use by callers.
pub mod error;
/// Re-exports provenance types for use by callers.
pub mod provenance;
/// Re-exports sensor types for use by callers.
pub mod sensors;
/// Re-exports type definitions for use by callers.
pub mod types;
