# polar-bear-biochip

**Bio-chip intelligence framework for Polar Bear Systems**
Multi-sensor EEG + motion fusion · rig-core LLM cognitive inference · ECDSA-signed provenance

[![Crates.io](https://img.shields.io/crates/v/polar-bear-biochip.svg)](https://crates.io/crates/polar-bear-biochip)
[![Docs.rs](https://docs.rs/polar-bear-biochip/badge.svg)](https://docs.rs/polar-bear-biochip)
[![CI](https://github.com/murtazaai/polar-bear-biochip/actions/workflows/ci.yml/badge.svg)](https://github.com/murtazaai/polar-bear-biochip/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85.0+-orange)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/Edition-2024-blue)](https://doc.rust-lang.org/edition-guide/)
[![rig-core](https://img.shields.io/badge/rig--core-0.37-purple)](https://rig.rs)
[![License: PBS](https://img.shields.io/badge/license-PBS-blue)](LICENSE-PBS)

> Built by **[Murtaza Ali Imtiaz](https://github.com/murtazaai)** · Technology Lead · **Polar Bear Systems** · July 2019 – Present

---

## Overview

Core component of the Polar Bear Systems **superpower bio-chip intelligence platform**: bridges
neurotechnology with decentralised AI infrastructure.

Fuses real-time EEG brainwave data (δ θ α β γ) with 3-axis MEMS accelerometer readings into
higher-order cognitive features, then routes them through a **rig-core (ARC)** LLM agent for
classification. Every inference result is **ECDSA-signed** on secp256k1 for blockchain-grade,
tamper-evident data provenance.

> **Demo mode**: all operations run without an API key by default.
> Set `ANTHROPIC_API_KEY` (and optionally `--features ai-agent`) for live inference.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      polar-bear-biochip                             │
├──────────────────────────────────┬──────────────────────────────────┤
│         sensors layer            │        provenance layer          │
│                                  │                                  │
│  bci.rs       δ θ α β γ bands   │  ecdsa_signer.rs  secp256k1     │
│               attention + med    │  EcdsaSigner      sign_result() │
│  accelerometer.rs  3-axis MEMS  │  EcdsaVerifier    from_hex()    │
│               activity state     │  SHA-256 payload hashing        │
│  fusion.rs    SensorFusion       │  64-byte r‖s compact signature  │
│               cognitive_load     │  → SignedOutput JSON on disk    │
│               emotional_valence  │                                  │
│               arousal_level      │                                  │
├──────────────────────────────────┴──────────────────────────────────┤
│         agent layer  (feature = ai-agent)                           │
│                                                                     │
│  biochip_agent.rs   BioChipAgent                                   │
│  ├── ai-agent feature ──► rig-core 0.37 · claude-sonnet-4-6       │
│  └── fallback        ──► curl subprocess · same JSON payload       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## EEG Signal Processing

### Frequency bands (Berger / Niedermeyer taxonomy)

| Band  | Range (Hz) | Cognitive correlate |
|-------|-----------|---------------------|
| Delta | 0.5–4     | Deep sleep, unconscious processing |
| Theta | 4–8       | Drowsiness, creativity, memory encoding |
| Alpha | 8–12      | Relaxed alertness, idle visual cortex, flow states |
| Beta  | 12–30     | Active thinking, focus, problem-solving |
| Gamma | 30–100    | High-level cognition, cross-cortical binding |

### Derived cognitive features

| Feature            | Formula | Range |
|--------------------|---------|-------|
| Attention index    | β / (α + θ + ε) × 0.6 | \[0, 1\] |
| Meditation index   | α / (β + γ + ε) × 4.0 | \[0, 1\] |
| Cognitive load     | β / (α + θ + ε) × 0.5 + activity_boost | \[0, 1\] |
| Emotional valence  | (α − 0.6·β) / total_power | \[−1, +1\] |
| Arousal level      | (β + γ) / total_power | \[0, 1\] |

See [`docs/bci_math.md`](docs/bci_math.md) for full derivations and ICA pipeline notes.

---

## ECDSA Provenance

### Signing (FIPS 186-5 / SEC 1 v2.0 - secp256k1)

| Step | Operation |
|------|-----------|
| 1 | `canonical_json` = `serde_json::to_string(InferenceResult)` |
| 2 | `e` = SHA-256(`canonical_json`) |
| 3 | Sample ephemeral k from CSPRNG |
| 4 | R = k·G;  r = R.x mod n |
| 5 | s = k⁻¹(e + r·d) mod n |
| 6 | Signature = r‖s, 64 bytes → hex-encoded in `SignedOutput.signature_hex` |

### Offline verification

```rust
// No private key required - public key is embedded in SignedOutput.
let valid = EcdsaSigner::verify_signed(&signed_output)?;

// Or use the standalone verifier:
let verifier = EcdsaVerifier::from_hex(&signed_output.public_key_hex)?;
verifier.verify(&sha256_hash, &signed_output.signature_hex)?;
```

---

## Build & Run

### Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Rust stable | ≥ 1.85.0 (MSRV) | `rustup update stable` |
| `curl` | any | for non-`ai-agent` live inference |
| `ANTHROPIC_API_KEY` | - | optional; demo mode if absent |

### Quick start

```bash
git clone https://github.com/murtazaai/polar-bear-biochip
cd polar-bear-biochip
cp .env.example .env          # fill in ANTHROPIC_API_KEY for live inference

# Demo mode - no API key required
cargo run -- run --demo --cycles 5

# Live inference (curl backend)
cargo run --release -- run --cycles 10

# Live inference with rig-core backend
cargo run --release --features ai-agent -- run --cycles 10

# Verify a signed output file offline
cargo run -- verify signed_outputs/cycle_001.json
```

### All CLI options

```
USAGE:
  polar-bear-biochip run    [OPTIONS]
  polar-bear-biochip verify <FILE>

run options:
  -c, --cycles <N>        Inference cycles (0 = infinite)  [default: 5]
  -d, --demo              Force demo mode (no API call)
  -o, --output-dir <DIR>  Signed JSON output directory     [default: signed_outputs]
  -m, --model <MODEL>     Anthropic model                  [default: claude-sonnet-4-6]
```

### Examples

```bash
cargo run --example sensors_demo                           # sensor fusion table (no key)
cargo run --example provenance_demo                        # ECDSA sign/verify/tamper (no key)
cargo run --example agent_demo                             # agent demo (demo fallback)
cargo run --example agent_demo --features ai-agent         # live rig-core inference
```

---

## Sample output

```
  ╔═══════════════════════════════════════════════════╗
  ║   Polar Bear  ·  Bio-Chip Intelligence Framework  ║
  ║   Sensor Fusion  ·  rig-core  ·  ECDSA Provenance ║
  ╚═══════════════════════════════════════════════════╝

INFO  Generating ECDSA keypair (secp256k1)...
INFO  Public Key : 04d23257ac0b...261c583a6ad7

  ──────────────────────────────────────────────────────
  CYCLE 001/005  |  2026-05-19T10:30:00.114Z
  ──────────────────────────────────────────────────────
  [SENSORS] BCI     α=11.5  β=19.2  θ=6.2  δ=2.3  γ=43.9 Hz
  [SENSORS] Accel   x=-0.05  y=+0.23  z=9.69 m/s²  |  Stationary
  [SENSORS] Fused   cogLoad=0.51  valence=-0.00  arousal=0.75  attn=0.62
  [AGENT]   Querying rig-core LLM agent...
  [RESULT]  Alert         : ✅ Normal
  [RESULT]  Cognitive State: Balanced beta-alpha profile - focused, productive engagement
  [RESULT]  Recommendations:
              • Readings within optimal range - maintain current activity
              • Beta dominance confirms active problem-solving mode
              • Schedule a 5-min micro-break within 45 minutes
  [PROV]    Hash      : 1d588fa83a0d8d1a9ffb...
  [PROV]    Signature : a304cd96500e3c4c4a5a...
  [PROV]    ✓  ECDSA signature verified inline (secp256k1 / SHA-256)
  [PROV]    ✓  Signed output → signed_outputs/cycle_001.json
```

---

## Test inventory

### Unit tests (inline in source modules)

| Module | Tests | Coverage |
|---|---|---|
| `sensors/bci.rs` | 3 | Band ranges, derived indices, Default |
| `sensors/accelerometer.rs` | 4 | Gravity axis, magnitude, activity states, Default |
| `sensors/fusion.rs` | 3 | Feature bounds, sequence_id, Default |
| `agent/biochip_agent.rs` | 3 | Demo scenarios, Default, prompt builder |
| `provenance/ecdsa_signer.rs` | 7 | Sign/verify, tamper, from_hex, verifier, geometry |

### Integration tests (`tests/`)

| File | Tests | Coverage |
|---|---|---|
| `tests/sensor_tests.rs` | 12 | BCI bands, accel geometry, fusion boundaries, timestamps |
| `tests/provenance_tests.rs` | 14 | ECDSA geometry, sign/verify, tamper detection, key ops |

**Total: 46 tests**

```bash
cargo test                  # run all 46 tests
cargo test sensors          # sensor layer only
cargo test provenance       # provenance layer only
cargo test -- --nocapture   # verbose output
```

---

## Signed output format

```json
{
  "inference_result": {
    "timestamp": "2026-05-19T10:30:00.117Z",
    "sequence_id": 1,
    "fused_reading": {
      "bci": { "alpha_hz": 11.5, "beta_hz": 19.2, "attention_index": 0.62, "..." : "..." },
      "accelerometer": { "x": -0.05, "z": 9.69, "activity_state": "Stationary", "...": "..." },
      "cognitive_load": 0.51,
      "emotional_valence": -0.0,
      "arousal_level": 0.75
    },
    "cognitive_state": "Balanced beta-alpha profile - focused engagement",
    "alert_level": "Normal",
    "recommendations": [ "...", "...", "..." ]
  },
  "payload_hash_hex": "1d588fa83a0d8d1a9ffb...",
  "signature_hex":    "a304cd96500e3c4c4a5a...",
  "public_key_hex":   "04d23257ac0b...",
  "signed_at":        "2026-05-19T10:30:00.117Z"
}
```

Tamper any field → SHA-256 hash changes → ECDSA signature invalidated.

---

## rig-core integration

```rust
// Without --features ai-agent (curl subprocess - same JSON payload as rig-core):
let agent = BioChipAgent::new("claude-sonnet-4-6", demo);
let result = agent.infer(fused).await?;

// With --features ai-agent (rig-core 0.37 backend - drop-in swap):
// src/agent/biochip_agent.rs rig_inference():
let client = anthropic::Client::from_env()?;
let agent  = client.agent(model).preamble(PREAMBLE).max_tokens(512).build();
agent.prompt(build_prompt(reading)).await
```

Both paths produce identical `InferenceResult` structures.

---

## Zed editor configuration (`.zed/`)

The `.zed/` directory contains a complete, pre-tuned workspace configuration
for the [Zed editor](https://zed.dev). No manual setup required - open the
project root in Zed and everything is ready.

| File | Purpose |
|---|---|
| `settings.json` | Workspace settings: rust-analyzer LSP, format-on-save, inlay hints, clippy diagnostics, semantic highlighting, file exclusions |
| `tasks.json` | Task runner: check, build, run, test, clippy, fmt, doc - all variants including `ai-agent` feature |
| `debug.json` | CodeLLDB launch configurations: debug/release builds, demo and live run modes, verify subcommand, all three example binaries |

### Quick reference - tasks (`Cmd+Shift+R` / `Ctrl+Shift+R`)

| Category | Key tasks |
|---|---|
| Build | `check`, `build (debug)`, `build (release)`, `build (ai-agent feature, *)` |
| Run | `run: demo (5/10 cycles)`, `run: live (* cycles, curl/rig-core backend)` |
| Verify | `verify: signed_outputs/cycle_001.json` |
| Test | `test (all, 46 tests)`, `test: sensor layer only`, `test: provenance layer only`, `test (all, verbose output)` |
| Examples | `example: sensors_demo`, `example: provenance_demo`, `example: agent_demo (*)` |
| Lint | `clippy (all targets)`, `clippy (ai-agent feature, all targets)` |
| Format | `fmt: check` (CI-safe), `fmt: apply` |
| Docs | `doc (open, no-deps)`, `doc (all features, open)` |
| Misc | `clean` |

### Quick reference - debug launches (`Cmd+Shift+D` / `Ctrl+Shift+D`)

| Configuration | Description |
|---|---|
| `Build & Debug (debug)` | Standard debug build, full breakpoint support |
| `Build & Debug (release)` | Optimised build - mirrors production performance |
| `Build & Debug (ai-agent feature)` | Debug build with rig-core backend compiled in |
| `Debug: run --demo --cycles 5/10` | Demo mode, no API key needed |
| `Debug: run live --cycles 10 (*)` | Live inference via curl or rig-core backend |
| `Debug: verify signed_outputs/cycle_001.json` | Offline ECDSA verification |
| `Debug: example sensors_demo` | Sensor fusion walkthrough |
| `Debug: example provenance_demo` | ECDSA sign/verify/tamper walkthrough |
| `Debug: example agent_demo (*)` | Agent demo in demo or live mode |

> **Debugging tests**: Rust test binaries are written to `target/debug/deps/<crate>-<hash>`,
> where the hash is non-deterministic per build. Run `cargo test --no-run` to build and
> discover the binary path, then use Zed's **Attach to process** debug action.

---

## License

Proprietary - © 2026 Murtaza Ali Imtiaz / Polar Bear Systems
See [LICENSE-PBS](LICENSE-PBS) for permitted use.

Licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

---

## Author

**Murtaza Ali Imtiaz** · Technology Lead · **Polar Bear Systems** · (July 2019 – Present)

- GitHub: [@murtazaai](https://github.com/murtazaai)
- LinkedIn: [linkedin.com/in/murtazai](https://linkedin.com/in/murtazai)
- Portfolio: [murtazai.com](https://murtazai.com)
