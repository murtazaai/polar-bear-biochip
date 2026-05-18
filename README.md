# polar-bear-biochip

**Bio-Chip Intelligence Framework** — multi-sensor fusion + rig-core LLM orchestration + ECDSA-signed data provenance.

> Built by **[Murtaza Ali Imtiaz](https://github.com/murtazaai)** · Technology Lead · Polar Bear Systems · July 2019–Present

Built at **Polar Bear Systems** as part of the superpower bio-chip intelligence initiative: bridging neurotechnology with decentralised AI infrastructure.

---

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                   polar-bear-biochip pipeline                  │
│                                                                │
│   ┌─────────────┐   ┌─────────────────┐                       │
│   │  BCI Sensor │   │  Accelerometer  │   (tokio async tasks) │
│   │  EEG bands  │   │  3-axis MEMS    │                       │
│   │  δ θ α β γ  │   │  x / y / z m/s²│                       │
│   └──────┬──────┘   └────────┬────────┘                       │
│          │                   │                                 │
│          └─────────┬─────────┘                                 │
│                    ▼                                           │
│           ┌─────────────────┐                                  │
│           │  SensorFusion   │  FusedReading                   │
│           │  cognitive_load │  emotional_valence               │
│           │  arousal_level  │  attention_index                 │
│           └────────┬────────┘                                  │
│                    ▼                                           │
│           ┌─────────────────┐                                  │
│           │  BioChipAgent   │  rig-core / Anthropic API        │
│           │  LLM inference  │  SYSTEM_PROMPT + sensor payload  │
│           │  JSON response  │  cognitive_state + alert_level   │
│           └────────┬────────┘                                  │
│                    ▼                                           │
│           ┌─────────────────┐                                  │
│           │  EcdsaSigner    │  secp256k1 / SHA-256             │
│           │  sign_result()  │  payload → hash → signature      │
│           │  verify_signed()│  offline tamper detection        │
│           └────────┬────────┘                                  │
│                    ▼                                           │
│           signed_outputs/cycle_NNN.json                        │
│           { inference_result, payload_hash_hex,                │
│             signature_hex, public_key_hex, signed_at }         │
└────────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2021) |
| Async runtime | tokio |
| LLM agent framework | rig-core (0xPlaygrounds / ARC) |
| Cryptography | k256 secp256k1 ECDSA + SHA-256 |
| LLM provider | Anthropic Claude (via `/v1/messages`) |
| Serialisation | serde / serde_json |
| CLI | clap 4 |
| Logging | tracing + tracing-subscriber |

---

## Build & Run

### Prerequisites

- Rust ≥ 1.75 (stable)
- `curl` on PATH (for live Anthropic API calls)
- `ANTHROPIC_API_KEY` environment variable (optional — `--demo` works without it)

### Build

```bash
git clone https://github.com/murtazaai/polar-bear-biochip
cd polar-bear-biochip
cargo build --release
```

Expected output:

```
   Compiling polar-bear-biochip v0.1.0
    Finished release [optimized] target(s)
```

### Run — demo mode (no API key required)

```bash
cargo run -- --demo --cycles 5
```

### Run — live Anthropic inference

```bash
export ANTHROPIC_API_KEY=sk-ant-...
cargo run --release
```

### Verify a signed output

```bash
cargo run -- --verify signed_outputs/cycle_001.json
```

### All CLI options

```
polar-bear-biochip [OPTIONS]

Options:
  -c, --cycles <N>        Inference cycles (0 = infinite) [default: 5]
  -d, --demo              Demo mode — no live API call
  -o, --output-dir <DIR>  Signed JSON output directory [default: signed_outputs]
  -v, --verify <FILE>     Verify a signed output file and exit
  -m, --model <MODEL>     Anthropic model [default: claude-3-5-haiku-20241022]
  -h, --help              Print help
  -V, --version           Print version
```

---

## Sample Output

```
  ╔═══════════════════════════════════════════════════╗
  ║   Polar Bear  ·  Bio-Chip Intelligence Framework  ║
  ║   Sensor Fusion  ·  rig-core  ·  ECDSA Provenance ║
  ╚═══════════════════════════════════════════════════╝

INFO  Generating ECDSA keypair (secp256k1)...
INFO  Public Key : 04ba35e2f5fb...311a3f31eb1a
INFO  Initialising sensor fusion (BCI + Accelerometer)...
INFO  Initialising rig-core LLM agent [model=claude-3-5-haiku-20241022]...

  ──────────────────────────────────────────────────────
  CYCLE 001/003  |  2026-05-18T14:07:27.114Z
  ──────────────────────────────────────────────────────
  [SENSORS] BCI     α=11.5  β=19.2  θ=6.2  δ=2.3  γ=43.9 Hz
  [SENSORS] Accel   x=-0.05  y=+0.23  z=9.69 m/s²  |  Stationary
  [SENSORS] Fused   cogLoad=0.51  valence=-0.00  arousal=0.75  attn=0.62
  [AGENT]   Querying rig-core LLM agent...
  [RESULT]  Alert         : ✅ Normal
  [RESULT]  Cognitive State: Balanced beta-alpha profile — focused, productive engagement
  [RESULT]  Recommendations:
              • All readings within optimal range — maintain current activity
              • Beta dominance confirms active problem-solving mode is engaged
              • Schedule a 5-min micro-break within 45 minutes
  [PROV]    Hash      : 1d588fa83a0d8d1a9ffb...
  [PROV]    Signature : a304cd96500e3c4c4a5a...
  [PROV]    ✓  ECDSA signature verified inline (secp256k1 / SHA-256)
  [PROV]    ✓  Signed output → signed_outputs/cycle_001.json
```

### Verify output

```
  ✅  ECDSA signature VALID

  Sequence ID  : 1
  Signed at    : 2026-05-18 14:07:27 UTC
  Cognitive    : Balanced beta-alpha profile — focused, productive engagement
  Alert level  : Normal
  Public Key   : 04ba35e2f5fb...311a3f31eb1a
  Payload Hash : 1d588fa83a0d8d1a9ffb...
  Signature    : a304cd96500e3c4c4a5a...
```

---

## Repository Structure

```
polar-bear-biochip/
├── Cargo.toml
├── README.md
├── .github/workflows/ci.yml       ← CI: build + clippy + smoke test
├── signed_outputs/                ← ECDSA-signed inference JSONs
└── src/
    ├── main.rs                    ← CLI, orchestration loop, verify command
    ├── types.rs                   ← Shared structs (BciReading, FusedReading,
    │                                 InferenceResult, SignedOutput, AlertLevel)
    ├── sensors/
    │   ├── mod.rs
    │   ├── bci.rs                 ← EEG mock sensor (δ θ α β γ + derived indices)
    │   ├── accelerometer.rs       ← 3-axis MEMS mock sensor
    │   └── fusion.rs              ← Sensor fusion → cognitive_load / valence / arousal
    ├── agent/
    │   ├── mod.rs
    │   └── biochip_agent.rs       ← rig-core–compatible LLM agent + demo mode
    └── provenance/
        ├── mod.rs
        └── ecdsa_signer.rs        ← secp256k1 ECDSA sign + offline verify
```

---

## Signed Output Format

Each inference cycle produces a tamper-evident JSON file:

```json
{
  "inference_result": {
    "timestamp": "2026-05-18T14:07:27.117Z",
    "sequence_id": 1,
    "fused_reading": {
      "bci": { "alpha_hz": 11.5, "beta_hz": 19.2, "attention_index": 0.62, ... },
      "accelerometer": { "x": -0.05, "z": 9.69, "activity_state": "Stationary", ... },
      "cognitive_load": 0.51,
      "emotional_valence": -0.00,
      "arousal_level": 0.75
    },
    "cognitive_state": "Balanced beta-alpha profile...",
    "alert_level": "Normal",
    "recommendations": [ "...", "...", "..." ]
  },
  "payload_hash_hex": "1d588fa83a0d8d1a9ffb...",
  "signature_hex":    "a304cd96500e3c4c4a5a...",
  "public_key_hex":   "04ba35e2f5fb...",
  "signed_at":        "2026-05-18T14:07:27.117Z"
}
```

The signature binds the `inference_result` blob to the keypair. Tamper any field → verification fails.

---

## rig-core Integration Note

This repo implements the Anthropic LLM call directly (via `curl` subprocess) to remain buildable on Rust < 1.85. The `BioChipAgent` interface is identical to a rig-core agent:

```rust
// Current implementation (curl-based, compiles on Rust ≥1.75)
let agent = BioChipAgent::new(model, demo);
let result = agent.infer(fused).await?;

// rig-core drop-in (swap agent/biochip_agent.rs on Rust ≥1.85)
let client = rig::providers::anthropic::Client::from_env();
let agent  = client.agent(model).preamble(SYSTEM_PROMPT).max_tokens(512).build();
let result = agent.prompt(&build_prompt(&fused)).await?;
```

See [`src/agent/biochip_agent.rs`](src/agent/biochip_agent.rs) for the exact swap comment.

---

## Story Line

### Situation
Polar Bear Systems needed a bio-chip intelligence platform that combines real-time EEG/motion sensor data with LLM-powered cognitive inference and tamper-evident data provenance — bridging neurotechnology with decentralised AI infrastructure.

### Task
Design and build a production-ready Rust system that: (1) fuses multi-sensor streams asynchronously using tokio, (2) feeds fused readings to a rig-core LLM agent for cognitive state classification, and (3) ECDSA-signs every inference output on the secp256k1 curve for blockchain-grade provenance.

### Action
- Built a three-layer Rust architecture: sensor fusion → rig-core agent → ECDSA provenance.
- Implemented mock BCI sensor (EEG: δ θ α β γ bands with ICA-inspired signal smoothing) and 3-axis accelerometer with gait simulation, fused into `cognitive_load`, `emotional_valence`, and `arousal_level`.
- Wrote a rig-core-compatible `BioChipAgent` with a structured SYSTEM_PROMPT that classifies cognitive states into `Normal / Elevated / Critical` alert levels with actionable recommendations.
- Implemented ECDSA secp256k1 signing (`k256` crate) with SHA-256 payload hashing; every `InferenceResult` is serialised to canonical JSON, hashed, signed, and written to disk as a verifiable `SignedOutput`.
- Added `--verify` CLI command for offline signature verification — demonstrates tamper detection by modifying any field.

### Result
- Complete working Rust binary: `cargo build --release` → zero errors, zero warnings.
- Demo mode (`--demo`) runs without credentials — full pipeline from sensor → LLM → ECDSA proof.
- Live mode (`ANTHROPIC_API_KEY` set) hits the Anthropic Claude API with the same interface rig-core exposes.
- ECDSA verification round-trips correctly on all signed outputs.
- CI pipeline (GitHub Actions) runs on every push: build + check + smoke test.

---

## License

PBS License: [PBS License](./LICENSE-PBS)

---

## Author

**Murtaza Ali Imtiaz**

- LinkedIn: [LinkedIn](https://linkedin.com/in/murtazai)
- GitHub: [@murtazaai](https://github.com/murtazaai)
- Portfolio: [murtazai.com](https://murtazai.com)
