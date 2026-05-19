//! Provenance layer: ECDSA secp256k1 signing and offline verification.

pub mod ecdsa_signer;

pub use ecdsa_signer::{EcdsaSigner, EcdsaVerifier};
