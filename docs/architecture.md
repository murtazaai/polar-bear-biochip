# polar-bear-biochip

## SYSTEM ARCHITECTURE

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        polar-bear-biochip                               │
│         Bio-Chip Intelligence Framework · Polar Bear Systems            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
          ┌─────────────────────────▼──────────────────────────┐
          │              CLI Entry Point  (main.rs)             │
          │   [run | verify]                                    │
          └──────────┬──────────────────────────┬──────────────┘
                     │                          │
     ┌───────────────▼───────────┐  ┌───────────▼──────────────────────┐
     │     sensors/              │  │     provenance/                  │
     │                           │  │                                  │
     │  bci.rs                   │  │  ecdsa_signer.rs                 │
     │  BciSensor                │  │  EcdsaSigner  (sign_result)      │
     │  δ θ α β γ bands          │  │  EcdsaVerifier (from_hex)        │
     │  attention + meditation   │  │  secp256k1 / FIPS 186-5          │
     │                           │  │  SHA-256 payload hashing         │
     │  accelerometer.rs         │  │  64-byte compact r‖s signature   │
     │  AccelerometerSensor      │  │  → SignedOutput (JSON on disk)   │
     │  3-axis MEMS  m/s²        │  └──────────────────────────────────┘
     │  ActivityState classifier │
     │                           │                 ▲
     │  fusion.rs                │                 │  sign_result()
     │  SensorFusion             │  ┌──────────────┴──────────────────┐
     │  → FusedReading           │  │     agent/                      │
     │    cognitive_load         │  │                                  │
     │    emotional_valence      │  │  biochip_agent.rs               │
     │    arousal_level          │  │  BioChipAgent                   │
     └───────────────────────────┘  │                                  │
                     │              │  ┌────────────────────────────┐  │
                     │              │  │ ai-agent feature           │  │
                     └──────────────►  │ rig-core 0.37              │  │
                     FusedReading   │  │ claude-sonnet-4-6          │  │
                                    │  └────────────────────────────┘  │
                                    │                                  │
                                    │  ┌────────────────────────────┐  │
                                    │  │ fallback (no feature)      │  │
                                    │  │ curl /v1/messages          │  │
                                    │  │ same JSON payload as rig   │  │
                                    │  └────────────────────────────┘  │
                                    │                                  │
                                    │  → InferenceResult              │
                                    └──────────────────────────────────┘
```

---

## Layer descriptions

### `sensors/` — Multi-sensor streaming layer

Produces fused EEG + motion data at a configurable cycle rate.

| Module | Specification | Output |
|---|---|---|
| `bci.rs` | Clinical EEG taxonomy (Berger 1929 + Niedermeyer 2004) | `BciReading` with 5 bands + derived indices |
| `accelerometer.rs` | 3-axis MEMS at 50 Hz, gait model | `AccelerometerReading` with activity state |
| `fusion.rs` | Feature engineering (cognitive load / valence / arousal) | `FusedReading` |

### `agent/` — Rig (ARC) LLM inference layer

`BioChipAgent::infer(FusedReading)` → `InferenceResult`

Two compile-time backends, identical interface:

| Build | Backend | Latency |
|---|---|---|
| `cargo build` | `curl` subprocess → Anthropic REST | ~400 ms (network) |
| `cargo build --features ai-agent` | `rig-core` → `claude-sonnet-4-6` | ~400 ms (network) |
| `--demo` flag | deterministic in-process | < 1 ms |

### `provenance/` — Secp256k1 ECDSA provenance layer

Every `InferenceResult` is:
1. Serialised to canonical JSON
2. SHA-256 hashed
3. ECDSA-signed with a session keypair (secp256k1)
4. Written as `SignedOutput` JSON to `signed_outputs/cycle_NNN.json`

Offline verification requires only the `public_key_hex` embedded in the file — no private key material.

---

## Data flow

```
BciSensor.sample()
    +
AccelerometerSensor.sample()
    │
    ▼
SensorFusion.sample(seq_id) → FusedReading
    │
    ▼
BioChipAgent.infer(FusedReading) → InferenceResult
    │
    ▼
EcdsaSigner.sign_result(&InferenceResult) → SignedOutput
    │
    ▼
signed_outputs/cycle_NNN.json
    │
    ▼ (offline)
EcdsaSigner::verify_signed(&SignedOutput) → bool
```
