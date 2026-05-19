//! Rig (ARC) bio-chip agent demo.
//!
//! Requires the `ai-agent` feature and a valid `ANTHROPIC_API_KEY`:
//!
//! ```text
//! export ANTHROPIC_API_KEY=sk-ant-...
//! cargo run --example agent_demo --features ai-agent
//! ```
//!
//! Without the feature flag or API key, the agent automatically falls back
//! to demo mode (deterministic responses, no API call):
//!
//! ```text
//! cargo run --example agent_demo
//! ```

use polar_bear_biochip::{agent::BioChipAgent, sensors::fusion::SensorFusion};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       RIG (ARC) BIO-CHIP AGENT DEMO  ·  DRY-RUN MODE           ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    let demo  = std::env::var("ANTHROPIC_API_KEY").is_err();
    let model = std::env::var("BIOCHIP_MODEL")
        .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());

    if demo {
        println!("  ⚠  DEMO MODE: set ANTHROPIC_API_KEY for live inference.");
        #[cfg(feature = "ai-agent")]
        println!("  ⚠  Build with: cargo run --example agent_demo --features ai-agent");
    } else {
        println!("  ✓  Live inference mode — model: {model}");
    }
    println!();

    let agent  = BioChipAgent::new(&model, demo);
    let mut fusion = SensorFusion::new();

    for id in 1..=3_u64 {
        let reading = fusion.sample(id);
        println!("── Cycle {id} ───────────────────────────────────────────────────");
        println!("   {reading}");

        let result = agent.infer(reading).await?;
        println!("   Alert : {}", result.alert_level);
        println!("   State : {}", result.cognitive_state);
        println!("   Recs  :");
        for rec in &result.recommendations {
            println!("     • {rec}");
        }
        println!();
    }

    println!("✓  Agent demo complete.");
    Ok(())
}
