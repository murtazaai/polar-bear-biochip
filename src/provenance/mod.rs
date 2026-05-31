//! Provenance layer: ECDSA secp256k1 signing and offline verification.

/// Provides ECDSA signing and verification functionality using secp256k1.
pub mod ecdsa_signer;

/// Re-exports the [`EcdsaSigner`] and [`EcdsaVerifier`] types.
pub use ecdsa_signer::{EcdsaSigner, EcdsaVerifier};
