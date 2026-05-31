//! # polar-bear-biochip CLI
//!
//! ```text
//! USAGE:
//!   polar-bear-biochip [OPTIONS]
//!   polar-bear-biochip verify <FILE>
//!
//! COMMANDS:
//!   run       Run the inference pipeline (default)
//!   verify    Verify a signed output file offline
//! ```

/// The main CLI entrypoint.
use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};
use polar_bear_biochip::{
    agent::BioChipAgent,
    provenance::{EcdsaSigner, EcdsaVerifier},
    sensors::SensorFusion,
    types::{AlertLevel, SignedOutput},
};
use sha2::Digest;
use tracing::{info, warn};

/// The main CLI entrypoint.
///
/// This is the entrypoint for the `polar-bear-biochip` CLI tool.
#[derive(Parser)]
#[command(
    name = "polar-bear-biochip",
    version,
    about = "Bio-chip intelligence: sensor fusion → rig-core LLM → ECDSA provenance",
    long_about = "Fuses EEG + accelerometer data, infers cognitive state via a rig-core LLM\n\
                  agent, and ECDSA-signs every output for blockchain-grade provenance.\n\
                  All operations run in DRY-RUN / DEMO mode by default (no API key needed)."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// The available subcommands for the `polar-bear-biochip` CLI tool.
///
/// Each subcommand corresponds to a different operation or mode of the tool.
#[derive(Subcommand)]
enum Commands {
    /// Run the inference pipeline (default when no subcommand given).
    Run {
        /// Number of inference cycles (0 = infinite).
        #[arg(short, long, default_value_t = 5)]
        cycles: u32,

        /// Force demo mode - simulate LLM responses without calling the API.
        /// Auto-enabled when `ANTHROPIC_API_KEY` is absent.
        #[arg(short, long)]
        demo: bool,

        /// Directory for signed JSON output files.
        #[arg(short, long, default_value = "signed_outputs")]
        output_dir: std::path::PathBuf,

        /// Anthropic model to use for live inference.
        #[arg(short, long, default_value = "claude-sonnet-4-6")]
        model: String,
    },

    /// Verify a previously produced signed output file and exit.
    Verify {
        /// Path to the signed JSON file.
        file: std::path::PathBuf,
    },
}

/// The entrypoint for the `polar-bear-biochip` CLI tool.
///
/// This is the entrypoint for the `polar-bear-biochip` CLI tool.
///
/// It parses the command-line arguments and dispatches the appropriate subcommand.
///
/// Returns `Ok(())` on success, or an error if something went wrong.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("polar_bear_biochip=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run {
        cycles: 5,
        demo: false,
        output_dir: "signed_outputs".into(),
        model: "claude-sonnet-4-6".to_string(),
    }) {
        Commands::Run {
            cycles,
            demo,
            output_dir,
            model,
        } => cmd_run(cycles, demo, &output_dir, &model).await,
        Commands::Verify { file } => cmd_verify(&file),
    }
}

/// Runs the `polar-bear-biochip` CLI tool in run mode.
///
/// This function is responsible for running the `polar-bear-biochip` CLI tool in run mode.
///
/// It parses the command-line arguments and dispatches the appropriate subcommand.
///
/// Returns `Ok(())` on success, or an error if something went wrong.
async fn cmd_run(cycles: u32, demo: bool, output_dir: &Path, model: &str) -> Result<()> {
    println!();
    println!("  ╔═══════════════════════════════════════════════════╗");
    println!("  ║   Polar Bear  ·  Bio-Chip Intelligence Framework  ║");
    println!("  ║   Sensor Fusion  ·  rig-core  ·  ECDSA Provenance ║");
    println!("  ╚═══════════════════════════════════════════════════╝");
    println!();

    let demo = demo || std::env::var("ANTHROPIC_API_KEY").is_err();
    if demo {
        warn!("ANTHROPIC_API_KEY not found or --demo set → running in demo mode.");
        info!("Set ANTHROPIC_API_KEY and remove --demo for live rig-core inference.");
        info!("Or build with: cargo build --features ai-agent");
    } else {
        info!("Live inference mode [model={model}]");
    }

    info!("Generating ECDSA keypair (secp256k1)...");
    let signer = EcdsaSigner::generate();
    info!(
        "Public Key : {}...{}",
        &signer.public_key_hex()[..12],
        &signer.public_key_hex()[118..],
    );

    info!("Initialising sensor fusion (BCI + Accelerometer)...");
    let mut fusion = SensorFusion::new();

    info!("Initialising rig-core LLM agent [model={model}]...");
    let agent = BioChipAgent::new(model, demo);

    std::fs::create_dir_all(output_dir)?;

    let total = if cycles == 0 { u32::MAX } else { cycles };

    for cycle in 1..=total {
        run_cycle(cycle, total, &mut fusion, &agent, &signer, output_dir).await?;
        if cycle < total {
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        }
    }

    println!();
    println!("  ══════════════════════════════════════════════════════");
    println!("  {total} cycle(s) complete.");
    println!("  Signed outputs written to: {}/", output_dir.display());
    println!();
    println!("  Verify offline:");
    println!(
        "    cargo run -- verify {}/cycle_001.json",
        output_dir.display()
    );
    println!("  ══════════════════════════════════════════════════════");

    Ok(())
}

/// Runs a single inference cycle of the `polar-bear-biochip` CLI tool.
///
/// This function is responsible for running a single inference cycle of the `polar-bear-biochip`
/// CLI tool.
///
/// It performs the inference cycle, writes the output to disk, and returns.
///
/// # Arguments
///
/// * `cycle` - The cycle number to run.
/// * `total` - The total number of cycles to run.
/// * `fusion` - The sensor fusion instance to use.
/// * `agent` - The biochip agent instance to use.
/// * `signer` - The ECDSA signer instance to use.
/// * `output_dir` - The directory to write the output to.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if something went wrong.
async fn run_cycle(
    cycle: u32,
    total: u32,
    fusion: &mut SensorFusion,
    agent: &BioChipAgent,
    signer: &EcdsaSigner,
    output_dir: &Path,
) -> Result<()> {
    println!();
    println!("  ──────────────────────────────────────────────────────");
    println!(
        "  CYCLE {:03}/{total:03}  |  {}",
        cycle,
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"),
    );
    println!("  ──────────────────────────────────────────────────────");

    // 1. Fuse sensors
    let fused = fusion.sample(u64::from(cycle));
    println!(
        "  [SENSORS] BCI     α={:.1}  β={:.1}  θ={:.1}  δ={:.1}  γ={:.1} Hz",
        fused.bci.alpha_hz,
        fused.bci.beta_hz,
        fused.bci.theta_hz,
        fused.bci.delta_hz,
        fused.bci.gamma_hz,
    );
    println!(
        "  [SENSORS] Accel   x={:+.2}  y={:+.2}  z={:.2} m/s²  |  {:?}",
        fused.accelerometer.x,
        fused.accelerometer.y,
        fused.accelerometer.z,
        fused.accelerometer.activity_state,
    );
    println!(
        "  [SENSORS] Fused   cogLoad={:.2}  valence={:+.2}  arousal={:.2}  attn={:.2}",
        fused.cognitive_load,
        fused.emotional_valence,
        fused.arousal_level,
        fused.bci.attention_index,
    );

    // 2. rig-core LLM inference
    println!("  [AGENT]   Querying rig-core LLM agent...");
    let result = agent.infer(fused).await?;

    println!("  [RESULT]  Alert         : {}", result.alert_level);
    println!("  [RESULT]  Cognitive State: {}", result.cognitive_state);
    println!("  [RESULT]  Recommendations:");
    for rec in &result.recommendations {
        println!("              • {rec}");
    }

    // 3. ECDSA provenance
    let signed = signer.sign_result(&result)?;
    println!(
        "  [PROV]    Hash      : {}...",
        &signed.payload_hash_hex[..20]
    );
    println!("  [PROV]    Signature : {}...", &signed.signature_hex[..20]);

    // Inline verification - demonstrates round-trip integrity.
    // Uses standalone EcdsaVerifier (no private key required post-sign).
    let verifier = EcdsaVerifier::from_hex(&signed.public_key_hex)?;
    let hash = sha2::Digest::finalize(sha2::Sha256::new_with_prefix(
        serde_json::to_string(&signed.inference_result)?.as_bytes(),
    ));
    if verifier.verify(&hash, &signed.signature_hex)? {
        println!("  [PROV]    ✓  ECDSA signature verified inline (secp256k1 / SHA-256)");
    } else {
        println!("  [PROV]    ✗  ECDSA verification FAILED - investigate immediately");
    }

    // 4. Write signed output to disk
    let out_path = output_dir.join(format!("cycle_{cycle:03}.json"));
    std::fs::write(&out_path, serde_json::to_string_pretty(&signed)?)?;
    println!("  [PROV]    ✓  Signed output → {}", out_path.display());

    Ok(())
}

/// Verifies the ECDSA signature of a signed output file.
///
/// This function is responsible for verifying the ECDSA signature of a signed output file.
///
/// # Arguments
///
/// * `path` - The path to the signed output file to verify.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if something went wrong.
fn cmd_verify(path: &Path) -> Result<()> {
    println!();
    println!("  Verifying: {}", path.display());
    println!();

    let json = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    let signed: SignedOutput = serde_json::from_str(&json)
        .map_err(|e| anyhow::anyhow!("invalid JSON in {}: {e}", path.display()))?;

    let valid = EcdsaSigner::verify_signed(&signed)?;

    if valid {
        println!("  ✅  ECDSA signature VALID");
        println!();
        println!("  Sequence ID  : {}", signed.inference_result.sequence_id);
        println!("  Signed at    : {}", signed.signed_at);
        println!(
            "  Cognitive    : {}",
            signed.inference_result.cognitive_state
        );
        println!("  Alert level  : {}", signed.inference_result.alert_level);
        println!(
            "  Public Key   : {}...{}",
            &signed.public_key_hex[..12],
            &signed.public_key_hex[118..]
        );
        println!("  Payload Hash : {}...", &signed.payload_hash_hex[..20]);
        println!("  Signature    : {}...", &signed.signature_hex[..20]);
    } else {
        eprintln!("  ❌  ECDSA signature INVALID - output may have been tampered");
        std::process::exit(1);
    }

    Ok(())
}
