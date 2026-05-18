//! polar-bear-biochip — Bio-Chip Intelligence Framework
//!
//! Pipeline:
//!   tokio multi-sensor stream
//!     → SensorFusion (BCI + Accelerometer → FusedReading)
//!     → BioChipAgent  (rig-core LLM → cognitive state + recommendations)
//!     → EcdsaSigner   (secp256k1 ECDSA → signed JSON)
//!     → signed_outputs/cycle_NNN.json

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, warn};

mod agent;
mod provenance;
mod sensors;
mod types;

use agent::BioChipAgent;
use provenance::EcdsaSigner;
use sensors::SensorFusion;
use types::{AlertLevel, SignedOutput};

// ─── CLI ──────────────────────────────────────────────────────────────────────

/// Polar Bear Bio-Chip Intelligence Framework
///
/// Fuses EEG + accelerometer data, infers cognitive state via a rig-core LLM
/// agent, and ECDSA-signs every output for blockchain-grade provenance.
#[derive(Parser, Debug)]
#[command(
    name    = "polar-bear-biochip",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Bio-chip intelligence: sensor fusion → rig-core LLM → ECDSA provenance"
)]
struct Cli {
    /// Number of inference cycles (0 = run indefinitely)
    #[arg(short, long, default_value_t = 5)]
    cycles: u32,

    /// Demo mode — simulate LLM responses without calling the Anthropic API.
    /// Automatically enabled when ANTHROPIC_API_KEY is not set.
    #[arg(short, long)]
    demo: bool,

    /// Directory for signed JSON output files
    #[arg(short, long, default_value = "signed_outputs")]
    output_dir: PathBuf,

    /// Verify a previously produced signed output file and exit
    #[arg(short, long, value_name = "FILE")]
    verify: Option<PathBuf>,

    /// Anthropic model to use for live inference
    #[arg(short, long, default_value = "claude-3-5-haiku-20241022")]
    model: String,
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("polar_bear_biochip=info".parse()?),
        )
        .compact()
        .init();

    let cli = Cli::parse();

    // ── Verify mode ───────────────────────────────────────────────────────────
    if let Some(path) = cli.verify {
        return cmd_verify(&path);
    }

    // ── Banner ────────────────────────────────────────────────────────────────
    println!();
    println!("  ╔═══════════════════════════════════════════════════╗");
    println!("  ║   Polar Bear  ·  Bio-Chip Intelligence Framework  ║");
    println!("  ║   Sensor Fusion  ·  rig-core  ·  ECDSA Provenance ║");
    println!("  ╚═══════════════════════════════════════════════════╝");
    println!();

    // ── Demo mode detection ───────────────────────────────────────────────────
    let demo = cli.demo || std::env::var("ANTHROPIC_API_KEY").is_err();
    if demo {
        warn!("ANTHROPIC_API_KEY not found or --demo flag set → running in demo mode.");
        info!("Set ANTHROPIC_API_KEY and remove --demo for live rig-core LLM inference.");
    } else {
        info!("Live inference mode: {}", cli.model);
    }

    // ── Initialise subsystems ─────────────────────────────────────────────────
    info!("Generating ECDSA keypair (secp256k1)...");
    let signer = EcdsaSigner::new();
    info!("Public Key : {}...{}", &signer.public_key_hex()[..12], &signer.public_key_hex()[118..]);

    info!("Initialising sensor fusion (BCI + Accelerometer)...");
    let mut fusion = SensorFusion::new();

    info!("Initialising rig-core LLM agent [model={}]...", cli.model);
    let agent = BioChipAgent::new(&cli.model, demo);

    std::fs::create_dir_all(&cli.output_dir)?;

    // ── Main inference loop ───────────────────────────────────────────────────
    let total_cycles = if cli.cycles == 0 { u32::MAX } else { cli.cycles };

    for cycle in 1..=total_cycles {
        run_cycle(cycle, total_cycles, &mut fusion, &agent, &signer, &cli.output_dir).await?;

        if cycle < total_cycles {
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        }
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    println!();
    println!("  ══════════════════════════════════════════════════════");
    println!("  {} cycle(s) complete.", total_cycles);
    println!("  Signed outputs written to: {}/", cli.output_dir.display());
    println!();
    println!("  Verify a signed output:");
    println!("    cargo run -- --verify {}/cycle_001.json", cli.output_dir.display());
    println!("  ══════════════════════════════════════════════════════");

    Ok(())
}

// ─── Single inference cycle ───────────────────────────────────────────────────

async fn run_cycle(
    cycle:      u32,
    total:      u32,
    fusion:     &mut SensorFusion,
    agent:      &BioChipAgent,
    signer:     &EcdsaSigner,
    output_dir: &PathBuf,
) -> Result<()> {
    println!();
    println!("  ──────────────────────────────────────────────────────");
    println!(
        "  CYCLE {:03}/{:03}  |  {}",
        cycle,
        total,
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")
    );
    println!("  ──────────────────────────────────────────────────────");

    // 1. Fuse sensors
    let fused = fusion.sample(cycle as u64);
    println!(
        "  [SENSORS] BCI     α={:.1}  β={:.1}  θ={:.1}  δ={:.1}  γ={:.1} Hz",
        fused.bci.alpha_hz, fused.bci.beta_hz,
        fused.bci.theta_hz, fused.bci.delta_hz, fused.bci.gamma_hz,
    );
    println!(
        "  [SENSORS] Accel   x={:+.2}  y={:+.2}  z={:.2} m/s²  |  {:?}",
        fused.accelerometer.x, fused.accelerometer.y,
        fused.accelerometer.z, fused.accelerometer.activity_state,
    );
    println!(
        "  [SENSORS] Fused   cogLoad={:.2}  valence={:+.2}  arousal={:.2}  attn={:.2}",
        fused.cognitive_load, fused.emotional_valence,
        fused.arousal_level,  fused.bci.attention_index,
    );

    // 2. rig-core LLM inference
    println!("  [AGENT]   Querying rig-core LLM agent...");
    let result = agent.infer(fused).await?;

    let alert_icon = match result.alert_level {
        AlertLevel::Normal   => "✅ Normal",
        AlertLevel::Elevated => "⚠️  Elevated",
        AlertLevel::Critical => "🚨 CRITICAL",
    };
    println!("  [RESULT]  Alert         : {}", alert_icon);
    println!("  [RESULT]  Cognitive State: {}", result.cognitive_state);
    println!("  [RESULT]  Recommendations:");
    for rec in &result.recommendations {
        println!("              • {}", rec);
    }

    // 3. ECDSA provenance
    let signed = signer.sign_result(&result)?;
    println!(
        "  [PROV]    Hash      : {}...",
        &signed.payload_hash_hex[..20]
    );
    println!(
        "  [PROV]    Signature : {}...",
        &signed.signature_hex[..20]
    );

    // Inline verification — demonstrates round-trip integrity
    let valid = EcdsaSigner::verify_signed(&signed)?;
    if valid {
        println!("  [PROV]    ✓  ECDSA signature verified inline (secp256k1 / SHA-256)");
    } else {
        println!("  [PROV]    ✗  ECDSA verification FAILED — investigate immediately");
    }

    // 4. Write signed output to disk
    let out_path = output_dir.join(format!("cycle_{:03}.json", cycle));
    let json = serde_json::to_string_pretty(&signed)?;
    std::fs::write(&out_path, &json)?;
    println!("  [PROV]    ✓  Signed output → {}", out_path.display());

    Ok(())
}

// ─── Verify subcommand ────────────────────────────────────────────────────────

fn cmd_verify(path: &PathBuf) -> Result<()> {
    println!();
    println!("  Verifying: {}", path.display());
    println!();

    let json = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Cannot read file {}: {e}", path.display()))?;

    let signed: SignedOutput = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in {}: {e}", path.display()))?;

    let valid = EcdsaSigner::verify_signed(&signed)?;

    if valid {
        println!("  ✅  ECDSA signature VALID");
        println!();
        println!("  Sequence ID  : {}", signed.inference_result.sequence_id);
        println!("  Signed at    : {}", signed.signed_at);
        println!("  Cognitive    : {}", signed.inference_result.cognitive_state);
        println!("  Alert level  : {:?}", signed.inference_result.alert_level);
        println!("  Public Key   : {}...{}", &signed.public_key_hex[..12], &signed.public_key_hex[118..]);
        println!("  Payload Hash : {}...", &signed.payload_hash_hex[..20]);
        println!("  Signature    : {}...", &signed.signature_hex[..20]);
    } else {
        println!("  ❌  ECDSA signature INVALID — output may have been tampered");
        std::process::exit(1);
    }

    Ok(())
}
