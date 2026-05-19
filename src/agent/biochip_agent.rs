//! # Bio-Chip LLM Agent
//!
//! A rig-core agent (when compiled with `--features ai-agent`) that analyses
//! fused sensor readings and returns a structured cognitive state decision.
//!
//! ## Feature flag
//!
//! | Build | Agent backend |
//! |-------|--------------|
//! | `cargo build` | `curl` subprocess → Anthropic `/v1/messages` |
//! | `cargo build --features ai-agent` | `rig-core` → `claude-sonnet-4-6` |
//!
//! Both backends share the same system prompt, JSON response schema, and
//! public `BioChipAgent` interface.  The `ai-agent` feature is simply a
//! compile-time transport switch.
//!
//! ## Architecture
//!
//! ```text
//! User sensor reading
//!     │
//!     ▼
//! BioChipAgent::infer(FusedReading)
//!     │  system prompt: cognitive classification schema
//!     │  user prompt:   sensor payload (bands + derived features)
//!     │
//!     ├── ai-agent feature ──► rig-core anthropic::Client
//!     │                              │
//!     └── fallback ────────► curl /v1/messages subprocess
//!                                    │
//!                                    ▼
//!                           InferenceResult (JSON parsed)
//!                                    │
//!                                    ▼
//!                           EcdsaSigner::sign_result()
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::types::{AlertLevel, FusedReading, InferenceResult};

#[cfg(feature = "ai-agent")]
use rig_core::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

// ── System prompt ─────────────────────────────────────────────────────────────

const PREAMBLE: &str = "\
You are the inference core of a bio-chip intelligence system at Polar Bear Systems. \
You receive fused readings from an EEG sensor and a 3-axis MEMS accelerometer. \
Respond ONLY in this exact JSON format - no markdown, no preamble, no trailing text:\n\
{\"cognitive_state\":\"<one-sentence summary>\",\
\"alert_level\":\"Normal\",\
\"recommendations\":[\"<rec 1>\",\"<rec 2>\",\"<rec 3>\"]}\n\
alert_level must be exactly: Normal | Elevated | Critical.\n\
Normal   = healthy operating range.\n\
Elevated = cognitive or physical stress - attention warranted.\n\
Critical = anomaly requiring immediate intervention.\n\
Interpretation guide:\n\
- Delta/theta dominance → fatigue or drowsiness risk.\n\
- High beta + low alpha  → elevated cognitive load.\n\
- emotional_valence < -0.30 → stress or anxiety marker.\n\
- Running + high beta    → fight-or-flight state.\n\
- Alpha coherence 0.7–0.9 + low load → optimal flow state.";

// ── Anthropic API wire types (used by curl fallback only) ─────────────────────

#[derive(Serialize)]
struct ApiRequest<'a> {
    model:      &'a str,
    max_tokens: u32,
    system:     &'a str,
    messages:   Vec<ApiMessage<'a>>,
}

#[derive(Serialize)]
struct ApiMessage<'a> {
    role:    &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ApiContent>,
}

#[derive(Deserialize)]
struct ApiContent {
    text: Option<String>,
}

// ── Agent ─────────────────────────────────────────────────────────────────────

/// Rig (ARC) Bio-Chip LLM agent.
///
/// With `--features ai-agent` the backend is `rig-core` → `claude-sonnet-4-6`.
/// Without the feature, inference falls back to a `curl` subprocess calling
/// the Anthropic REST API directly - identical JSON payload, same schema.
pub struct BioChipAgent {
    /// Anthropic model identifier.
    model: String,
    /// When `true`, returns deterministic demo responses without any API call.
    demo:  bool,
}

impl BioChipAgent {
    /// Construct a new agent.
    ///
    /// Pass `demo = true` to skip all live API calls (no key required).
    #[must_use]
    pub fn new(model: &str, demo: bool) -> Self {
        #[cfg(feature = "ai-agent")]
        let _ = dotenvy::dotenv();

        Self { model: model.to_string(), demo }
    }

    /// Run one full inference cycle.
    ///
    /// # Errors
    /// Returns an error if the API call fails or the response cannot be parsed.
    pub async fn infer(&self, reading: FusedReading) -> Result<InferenceResult> {
        let raw = if self.demo {
            self.demo_response(&reading)
        } else {
            self.live_inference(&reading).await?
        };
        self.parse_response(reading, raw)
    }

    // ── Live inference ────────────────────────────────────────────────────────

    async fn live_inference(&self, reading: &FusedReading) -> Result<String> {
        #[cfg(feature = "ai-agent")]
        {
            return self.rig_inference(reading).await;
        }
        #[cfg(not(feature = "ai-agent"))]
        {
            self.curl_inference(reading)
        }
    }

    // ── rig-core backend (feature = ai-agent) ─────────────────────────────────

    #[cfg(feature = "ai-agent")]
    async fn rig_inference(&self, reading: &FusedReading) -> Result<String> {
        let client = anthropic::Client::from_env()
            .context("ANTHROPIC_API_KEY not set - pass --demo for offline mode")?;
        let agent  = client
            .agent(self.model.as_str())
            .preamble(PREAMBLE)
            .max_tokens(512)
            .build();

        agent
            .prompt(build_prompt(reading))
            .await
            .map_err(|e| anyhow::anyhow!("rig-core inference error: {e}"))
    }

    // ── curl fallback (no ai-agent feature) ───────────────────────────────────
    // Mirrors the exact JSON payload that rig-core serialises internally.
    // To switch to rig-core: `cargo build --features ai-agent`.

    #[cfg(not(feature = "ai-agent"))]
    fn curl_inference(&self, reading: &FusedReading) -> Result<String> {
        use std::process::Command;

        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY not set - pass --demo for offline demo mode")?;

        let body = serde_json::to_string(&ApiRequest {
            model:      self.model.as_str(),
            max_tokens: 512,
            system:     PREAMBLE,
            messages:   vec![ApiMessage { role: "user", content: &build_prompt(reading) }],
        })
        .context("failed to serialise API request")?;

        let output = Command::new("curl")
            .args([
                "--silent", "--fail",
                "https://api.anthropic.com/v1/messages",
                "--header", "Content-Type: application/json",
                "--header", &format!("x-api-key: {api_key}"),
                "--header", "anthropic-version: 2023-06-01",
                "--data",   body.as_str(),
            ])
            .output()
            .context("curl subprocess failed - install curl or use --demo")?;

        if !output.status.success() {
            anyhow::bail!(
                "Anthropic API error ({})\nstderr: {}\nbody: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout),
            );
        }

        let resp: ApiResponse = serde_json::from_slice(&output.stdout)
            .context("failed to parse Anthropic response")?;

        resp.content
            .into_iter()
            .find_map(|c| c.text)
            .context("empty content array in Anthropic response")
    }

    // ── Demo responses ────────────────────────────────────────────────────────

    fn demo_response(&self, r: &FusedReading) -> String {
        if r.bci.delta_hz > 3.2 || r.bci.theta_hz > 7.0 {
            r#"{"cognitive_state":"Excessive slow-wave activity indicating acute fatigue - microsleep risk detected","alert_level":"Critical","recommendations":["IMMEDIATE: discontinue any safety-critical or high-risk activity","Initiate a 20-minute NREM power-nap protocol to restore prefrontal cortex function","Re-schedule all cognitively demanding tasks to the post-recovery window"]}"#
        } else if r.cognitive_load > 0.72 || r.emotional_valence < -0.30 {
            r#"{"cognitive_state":"Elevated cognitive load with acute mental stress markers in beta-band dominance","alert_level":"Elevated","recommendations":["Decompose the current task into atomic sub-tasks to reduce working-memory pressure","Engage in 2 minutes of slow diaphragmatic breathing to attenuate beta dominance","Schedule a 10-minute active recovery block before resuming deep-focus work"]}"#
        } else if r.bci.meditation_index > 0.58 && r.cognitive_load < 0.38 {
            r#"{"cognitive_state":"Deep alpha-dominant meditative state - optimal window for creative and divergent thinking","alert_level":"Normal","recommendations":["Leverage this flow window for insight-driven or creative work - interruptions are costly","Maintain ambient temperature and hydration to sustain alpha coherence","Log this session: alpha coherence of this quality is a trainable biometric target"]}"#
        } else {
            r#"{"cognitive_state":"Balanced beta-alpha profile consistent with focused, productive cognitive engagement","alert_level":"Normal","recommendations":["All readings within optimal operating range - maintain current activity and environment","Beta dominance confirms active problem-solving mode is fully engaged","Schedule a 5-minute micro-break within 45 minutes to prevent fatigue accumulation"]}"#
        }
        .to_string()
    }

    // ── Response parser ───────────────────────────────────────────────────────

    fn parse_response(&self, reading: FusedReading, raw: String) -> Result<InferenceResult> {
        let clean = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let v: serde_json::Value = serde_json::from_str(clean)
            .with_context(|| format!("LLM JSON parse error - raw response:\n{raw}"))?;

        let cognitive_state = v["cognitive_state"]
            .as_str()
            .unwrap_or("Cognitive state undetermined")
            .to_string();

        let alert_level = match v["alert_level"].as_str().unwrap_or("Normal") {
            "Elevated" => AlertLevel::Elevated,
            "Critical" => AlertLevel::Critical,
            _          => AlertLevel::Normal,
        };

        let recommendations = v["recommendations"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|x| x.as_str()).map(String::from).collect())
            .unwrap_or_default();

        Ok(InferenceResult {
            timestamp:        Utc::now(),
            sequence_id:      reading.sequence_id,
            fused_reading:    reading,
            cognitive_state,
            recommendations,
            alert_level,
            raw_llm_response: raw,
        })
    }
}

impl Default for BioChipAgent {
    fn default() -> Self {
        Self::new("claude-sonnet-4-6", true)
    }
}

// ── Prompt builder ────────────────────────────────────────────────────────────

fn build_prompt(r: &FusedReading) -> String {
    format!(
        "Reading #{seq} @ {ts}\n\
         EEG bands (Hz): delta={d:.2} theta={t:.2} alpha={a:.2} beta={b:.2} gamma={g:.2}\n\
         Indices: attention={att:.2} meditation={med:.2}\n\
         Fused: cognitive_load={cl:.2} emotional_valence={ev:+.2} arousal={ar:.2}\n\
         Accel (m/s²): x={x:+.2} y={y:+.2} z={z:.2} mag={m:.2} state={state:?}",
        seq   = r.sequence_id,
        ts    = r.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        d     = r.bci.delta_hz,  t  = r.bci.theta_hz,
        a     = r.bci.alpha_hz,  b  = r.bci.beta_hz,  g  = r.bci.gamma_hz,
        att   = r.bci.attention_index, med = r.bci.meditation_index,
        cl    = r.cognitive_load, ev = r.emotional_valence, ar = r.arousal_level,
        x     = r.accelerometer.x,    y = r.accelerometer.y,
        z     = r.accelerometer.z,    m = r.accelerometer.magnitude,
        state = r.accelerometer.activity_state,
    )
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::fusion::SensorFusion;

    #[test]
    fn demo_mode_parses_all_scenarios() {
        let agent = BioChipAgent::new("claude-sonnet-4-6", true);
        let mut fusion = SensorFusion::new();

        // Run enough cycles to exercise multiple demo branches.
        for id in 1..=20_u64 {
            let reading = fusion.sample(id);
            // tokio::runtime not needed - demo_response is sync; we call infer via a runtime.
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(agent.infer(reading)).unwrap();
            assert!(!result.cognitive_state.is_empty());
            assert!(!result.recommendations.is_empty());
        }
    }

    #[test]
    fn default_agent_is_in_demo_mode() {
        let agent = BioChipAgent::default();
        assert!(agent.demo, "default agent must be in demo mode (no API key available in tests)");
    }

    #[test]
    fn build_prompt_contains_sequence_id() {
        let mut fusion = SensorFusion::new();
        let reading = fusion.sample(42);
        let prompt = build_prompt(&reading);
        assert!(prompt.contains("#42"), "prompt must embed sequence_id");
    }
}
