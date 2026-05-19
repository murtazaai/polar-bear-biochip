//! LLM agent layer: bio-chip cognitive state inference.
//!
//! Compile with `--features ai-agent` to enable rig-core as the inference
//! backend (`claude-sonnet-4-6`). Without the feature the agent calls the
//! Anthropic REST API directly via `curl`.

pub mod biochip_agent;

pub use biochip_agent::BioChipAgent;
