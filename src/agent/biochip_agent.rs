//! rig-core–compatible LLM agent for cognitive state inference.
//!
//! ## Interface contract
//! This module presents the same public interface that a rig-core agent exposes:
//!
//! ```ignore
//! let agent  = BioChipAgent::new(model, demo_mode);
//! let result = agent.infer(fused_reading).await?;
//! ```
//!
//! ## HTTP transport
//! The Anthropic `/v1/messages` API is called via `std::process::Command` + `curl`
//! — the same JSON payload structure that rig-core serialises internally.
//! To swap in rig-core on a ≥1.85 toolchain, replace `live_inference()` with:
//!
//! ```ignore
//! let client = rig::providers::anthropic::Client::from_env();
//! let agent  = client.agent(&self.model).preamble(SYSTEM_PROMPT).max_tokens(512).build();
//! Ok(agent.prompt(&build_prompt(reading)).await?)
//! ```
//!
//! ## Demo mode
//! When `ANTHROPIC_API_KEY` is absent or `--demo` is passed, the agent returns
//! deterministic simulated responses so the repo is runnable with no credentials.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::types::{AlertLevel, FusedReading, InferenceResult};

// ─── System prompt ── mirrors rig-core `.preamble()` ─────────────────────────

const SYSTEM_PROMPT: &str = "You are the inference core of a bio-chip intelligence system. \
You receive fused readings from an EEG sensor and a 3-axis accelerometer. \
Respond ONLY in this JSON format — no markdown, no preamble, no extra text:\n\
{\"cognitive_state\":\"<one-sentence summary>\",\"alert_level\":\"Normal\",\
\"recommendations\":[\"<rec 1>\",\"<rec 2>\",\"<rec 3>\"]}\n\
alert_level must be exactly: Normal | Elevated | Critical.\n\
Normal=healthy range. Elevated=stress warrants attention. Critical=immediate intervention.\n\
Interpretation: delta/theta dominance=fatigue; high beta+low alpha=cognitive overload; \
emotional_valence<-0.3=stress marker; Running+high beta=fight-or-flight; \
alpha coherence 0.7-0.9+low load=optimal flow state.";

// ─── Anthropic API wire types ─────────────────────────────────────────────────

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

// ─── Agent ────────────────────────────────────────────────────────────────────

pub struct BioChipAgent {
    model: String,
    demo:  bool,
}

impl BioChipAgent {
    pub fn new(model: &str, demo: bool) -> Self {
        Self { model: model.to_string(), demo }
    }

    pub async fn infer(&self, reading: FusedReading) -> Result<InferenceResult> {
        let raw = if self.demo {
            self.demo_response(&reading)
        } else {
            self.live_inference(&reading)?
        };
        self.parse_response(reading, raw)
    }

    // ── Live call to Anthropic /v1/messages ───────────────────────────────────

    fn live_inference(&self, reading: &FusedReading) -> Result<String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY not set — pass --demo for offline demo mode")?;

        let prompt = build_prompt(reading);

        let body = serde_json::to_string(&ApiRequest {
            model:      self.model.as_str(),
            max_tokens: 512,
            system:     SYSTEM_PROMPT,
            messages:   vec![ApiMessage { role: "user", content: prompt.as_str() }],
        })
        .context("Failed to serialise API request")?;

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
            .context("curl subprocess failed — ensure curl is installed")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Anthropic API HTTP error\nstderr: {stderr}\nbody: {stdout}");
        }

        let resp: ApiResponse = serde_json::from_slice(&output.stdout)
            .context("Could not parse Anthropic response JSON")?;

        resp.content
            .into_iter()
            .find_map(|c| c.text)
            .context("Empty content array in Anthropic response")
    }

    // ── Demo responses ────────────────────────────────────────────────────────

    fn demo_response(&self, r: &FusedReading) -> String {
        if r.bci.delta_hz > 3.2 || r.bci.theta_hz > 7.0 {
            r#"{"cognitive_state":"Excessive slow-wave activity indicating acute fatigue — microsleep risk detected","alert_level":"Critical","recommendations":["IMMEDIATE: discontinue any safety-critical or high-risk activity now","Initiate a 20-minute NREM power-nap protocol to restore prefrontal function","Re-schedule all demanding tasks to the post-recovery window"]}"#
        } else if r.cognitive_load > 0.72 || r.emotional_valence < -0.30 {
            r#"{"cognitive_state":"Elevated cognitive load with acute mental stress markers in beta-band dominance","alert_level":"Elevated","recommendations":["Decompose the current task into atomic sub-tasks to reduce working-memory pressure","Engage in 2 minutes of slow diaphragmatic breathing to attenuate beta dominance","Schedule a 10-minute active recovery block before resuming deep-focus work"]}"#
        } else if r.bci.meditation_index > 0.58 && r.cognitive_load < 0.38 {
            r#"{"cognitive_state":"Deep alpha-dominant meditative state — optimal window for creative and divergent thinking","alert_level":"Normal","recommendations":["Leverage this flow window for insight-driven or creative work — interruptions are costly","Maintain ambient temperature and hydration to sustain alpha coherence","Log this session; alpha coherence of this quality is a trainable biometric target"]}"#
        } else {
            r#"{"cognitive_state":"Balanced beta-alpha profile consistent with focused, productive cognitive engagement","alert_level":"Normal","recommendations":["All readings within optimal operating range — maintain current activity and environment","Beta dominance confirms active problem-solving mode is engaged","Schedule a 5-minute micro-break within 45 minutes to prevent fatigue accumulation"]}"#
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
            .map_err(|e| anyhow::anyhow!("LLM JSON parse error: {e}\nRaw:\n{raw}"))?;

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

// ─── Prompt builder ───────────────────────────────────────────────────────────

fn build_prompt(r: &FusedReading) -> String {
    format!(
        "Reading #{seq} @ {ts}\n\
         EEG bands (Hz): delta={d:.2} theta={t:.2} alpha={a:.2} beta={b:.2} gamma={g:.2}\n\
         Indices: attention={att:.2} meditation={med:.2}\n\
         Fused: cognitive_load={cl:.2} emotional_valence={ev:+.2} arousal={ar:.2}\n\
         Accel (m/s²): x={x:+.2} y={y:+.2} z={z:.2} mag={m:.2} state={state:?}",
        seq   = r.sequence_id,
        ts    = r.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        d     = r.bci.delta_hz,  t = r.bci.theta_hz,
        a     = r.bci.alpha_hz,  b = r.bci.beta_hz, g = r.bci.gamma_hz,
        att   = r.bci.attention_index, med = r.bci.meditation_index,
        cl    = r.cognitive_load, ev = r.emotional_valence, ar = r.arousal_level,
        x     = r.accelerometer.x, y = r.accelerometer.y, z = r.accelerometer.z,
        m     = r.accelerometer.magnitude, state = r.accelerometer.activity_state,
    )
}
