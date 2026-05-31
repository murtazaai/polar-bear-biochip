//! LLM agent layer: bio-chip cognitive state inference.
//!
//! Compile with `--features ai-agent` to enable rig-core as the inference
//! backend (`claude-sonnet-4-6`). Without the feature the agent calls the
//! Anthropic REST API directly via `curl`.

/// The `BioChipAgent` is responsible for generating prompts and calling the LLM backend to infer
/// cognitive state.
pub mod biochip_agent;

/// Re-exports the `BioChipAgent` struct.
///
/// This allows the `BioChipAgent` to be used as a standalone struct without needing to import
/// the `biochip_agent` module directly.
pub use biochip_agent::BioChipAgent;
