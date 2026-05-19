# Repository File Structure

```
polar-bear-biochip/
│
│  ── Root tooling & meta ────────────────────────────────────────────
├── Cargo.toml             Rust 2024 edition; all dependencies; lints; features; release profile
├── Cargo.lock             Committed (binary crate); delete + regenerate on dep changes
├── rustfmt.toml           Code-style rules (100 cols, 2024 edition, crate-level imports)
├── .clippy.toml           Clippy config (MSRV 1.85.0, complexity thresholds)
├── .gitignore             Focused Rust-only ignore file; secrets never committed
├── .env.example           Template for ANTHROPIC_API_KEY and pipeline tuning vars
├── LICENSE-PBS            Polar Bear Systems proprietary licence
├── README.md              Project overview, architecture, quick-start, test inventory
├── CHANGELOG.md           Version history (Semantic Versioning)
├── CONTRIBUTING.md        Dev setup, workflow, code-style, CI description
├── BUG-FIXES.md           Root-cause analysis of 5 resolved issues
├── FILE_STRUCTURE.md      This file
│
│  ── GitHub Actions CI ──────────────────────────────────────────────
├── .github/
│   └── workflows/
│       └── ci.yml         fmt → clippy → build → test → docs → MSRV → smoke
│
│  ── Zed IDE config ─────────────────────────────────────────────────
├── .zed/
│   ├── tasks.json         15 cargo build / test / run / example tasks
│   └── debug.json         CodeLLDB debug launch configs
│
│  ── Documentation ──────────────────────────────────────────────────
├── docs/
│   ├── architecture.md    Full layer diagram and data flow
│   └── bci_math.md        EEG taxonomy, derived index formulas, fusion math, ICA notes
│
│  ── Standalone runnable examples ───────────────────────────────────
│  (cargo run --example <name>; no API key needed unless noted)
├── examples/
│   ├── sensors_demo.rs    5-cycle fusion table: bands + cognitive features
│   ├── provenance_demo.rs ECDSA key gen / sign / offline verify / tamper detect
│   └── agent_demo.rs      Bio-chip LLM agent (--features ai-agent or demo fallback)
│
│  ── Integration tests ───────────────────────────────────────────────
├── tests/
│   ├── sensor_tests.rs    12 tests: BCI bands, accelerometer, fusion boundaries
│   └── provenance_tests.rs 14 tests: ECDSA sign/verify/tamper, key ops, geometry
│
│  ── Library + binary source ─────────────────────────────────────────
├── src/
│   ├── lib.rs             Crate root; architecture doc; pub module declarations
│   ├── main.rs            Binary CLI; run / verify subcommands; inference loop
│   ├── types.rs           BciReading · AccelerometerReading · FusedReading
│   │                      InferenceResult · SignedOutput · AlertLevel (with Display)
│   ├── error.rs           BiochipError - thiserror typed error hierarchy
│   │
│   ├── sensors/           Multi-sensor streaming layer
│   │   ├── mod.rs         pub mod declarations; re-exports SensorFusion
│   │   ├── bci.rs         BciSensor - EEG δθαβγ bands, attention + meditation indices
│   │   ├── accelerometer.rs AccelerometerSensor - 3-axis MEMS, gait model, activity classifier
│   │   └── fusion.rs      SensorFusion - cognitive_load / emotional_valence / arousal_level
│   │
│   ├── agent/             Rig (ARC) LLM inference layer
│   │   ├── mod.rs         pub mod; re-exports BioChipAgent
│   │   └── biochip_agent.rs BioChipAgent (rig-core via ai-agent feature; curl fallback)
│   │
│   └── provenance/        secp256k1 ECDSA provenance layer
│       ├── mod.rs         pub mod; re-exports EcdsaSigner + EcdsaVerifier
│       └── ecdsa_signer.rs EcdsaSigner (sign_result, from_hex, verify)
│                           EcdsaVerifier (standalone public-key verifier)
│
│  ── Signed provenance outputs ───────────────────────────────────────
└── signed_outputs/
    ├── .gitkeep           Keeps the directory tracked by git
    └── cycle_001.json     Sample signed output (committed as showcase artefact)
```
