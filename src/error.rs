//! Typed error hierarchy for the Bio-Chip Intelligence Framework.
//!
//! All public APIs return `anyhow::Result<T>` for ergonomics at the call site.
//! These typed variants are used internally and surface through `anyhow`'s
//! context chain, so `anyhow::Error::downcast_ref::<BiochipError>()` works
//! wherever a specific variant must be matched.

/// Re-exports the `thiserror::Error` derive macro.
use thiserror::Error;

/// Top-level error type for the `polar-bear-biochip` crate.
///
/// All errors returned by the crate's public APIs are of this type.
///
/// This type is re-exported so callers can match on specific error variants
/// using `anyhow::Error::downcast_ref::<BiochipError>()`.
#[derive(Error, Debug)]
pub enum BiochipError {
    /// A sensor could not produce a valid reading.
    #[error("sensor read failure: {0}")]
    SensorRead(String),

    /// The LLM agent call failed or returned unparseable output.
    #[error("inference error: {0}")]
    Inference(String),

    /// ECDSA key material is invalid or the signing operation failed.
    #[error("ECDSA error: {0}")]
    Ecdsa(String),

    /// A signed output file failed JSON round-trip validation.
    #[error("provenance verification failed: {0}")]
    ProvenanceVerification(String),

    /// JSON serialisation / deserialisation error.
    #[error("serialise error: {source}")]
    Serialise {
        /// The underlying `serde_json` error.
        #[from]
        source: serde_json::Error,
    },

    /// File-system I/O error (reading/writing signed outputs).
    #[error("I/O error: {source}")]
    Io {
        /// The underlying `std::io::Error`.
        #[from]
        source: std::io::Error,
    },

    /// Hex encoding / decoding error.
    #[error("hex error: {source}")]
    Hex {
        /// The underlying `hex::FromHexError`.
        #[from]
        source: hex::FromHexError,
    },
}
