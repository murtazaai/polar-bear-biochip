//! Sensor fusion demo - no API key required.
//!
//! ```text
//! cargo run --example sensors_demo
//! ```

/// Demonstrates the sensor fusion process without requiring an API key.
use polar_bear_biochip::sensors::fusion::SensorFusion;

/// Runs the sensor fusion demo, sampling 5 cycles and displaying the results.
fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║         BCI + ACCELEROMETER SENSOR FUSION - DEMO               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    let mut fusion = SensorFusion::new();

    println!("\n  Sampling 5 fusion cycles:\n");
    println!(
        "  {:<6} {:<8} {:<8} {:<8} {:<8} {:<8} | {:<10} {:<10} {:<10}",
        "Cycle", "α Hz", "β Hz", "θ Hz", "δ Hz", "γ Hz", "CogLoad", "Valence", "Arousal"
    );
    println!("  {}", "─".repeat(84));

    for id in 1..=5_u64 {
        let r = fusion.sample(id);
        println!(
            "  {:<6} {:<8.2} {:<8.2} {:<8.2} {:<8.2} {:<8.2} | {:<10.3} {:<+10.3} {:<10.3}",
            id,
            r.bci.alpha_hz,
            r.bci.beta_hz,
            r.bci.theta_hz,
            r.bci.delta_hz,
            r.bci.gamma_hz,
            r.cognitive_load,
            r.emotional_valence,
            r.arousal_level,
        );
    }

    println!();
    println!("  Attention index   : measures β relative to α+θ (focus indicator).");
    println!("  Meditation index  : measures α relative to β+γ (calm indicator).");
    println!("  Cognitive load    : high β + low α → higher load.");
    println!("  Emotional valence : (α − 0.6·β) / total_power  - negative = stress.");
    println!("  Arousal level     : (β + γ) / total_power.");
}
