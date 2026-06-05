# Contributing to polar-bear-biochip

> **Polar Bear (🍨)** · Technology Lead: Murtaza Ali Imtiaz
> This repository is published under a restricted proprietary licence for
> portfolio and reference purposes. See [LICENSE-PBS](./LICENSE-PBS) for permitted use.

---

## Development environment

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust stable toolchain | ≥ 1.85.0 (MSRV) | `rustup update stable` |
| `rustfmt` | (with toolchain) | `rustup component add rustfmt` |
| `clippy` | (with toolchain) | `rustup component add clippy` |
| `curl` | any | system package manager (for non-ai-agent inference) |

### Setup

```text
git clone https://github.com/murtazaai/polar-bear-biochip
cd polar-bear-biochip
cp .env.example .env
# Edit .env: set ANTHROPIC_API_KEY=sk-ant-... (only needed for live inference)
```

---

## Workflow

### Build

```text
cargo build                          # debug
cargo build --release                # optimised (use for benchmarks)
cargo build --features ai-agent      # include Rig AI agent module (rig-core 0.37)
```

### Run

```text
# Demo mode - no API key required
cargo run -- run --demo --cycles 5

# Live inference (requires ANTHROPIC_API_KEY)
cargo run --release -- run --cycles 10

# Verify a signed output offline
cargo run -- verify signed_outputs/cycle_001.json
```

### Test

```text
cargo test                           # all unit + integration tests
cargo test --features ai-agent       # include ai-agent feature tests
cargo test sensors                   # filter: sensor layer only
cargo test provenance                # filter: ECDSA provenance only
```

### Examples

```text
cargo run --example sensors_demo     # sensor fusion table (no key)
cargo run --example provenance_demo  # ECDSA sign/verify/tamper (no key)
cargo run --example agent_demo       # Rig agent (--features ai-agent or demo)
```

### Lint and format

```text
cargo fmt                            # auto-format all source
cargo fmt -- --check                 # CI check (no modifications)
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps --open           # build and view docs
```

---

## Code style

All style rules are enforced via [`rustfmt.toml`](./rustfmt.toml) and
[`.clippy.toml`](./.clippy.toml).  The key rules:

- **Width** - 100 columns
- **Edition** - 2024
- **Imports** - `Crate`-level granularity, `StdExternalCrate` grouping
- **Trailing commas** - `Vertical` (multi-line only)
- **Comments** - wrapped at 100 columns, normalised doc attributes
- **Clippy** - `all` + `pedantic`; see `.clippy.toml` for allow-list

---

## CI description

All pushes to `main` or `develop` run:

| Job | What it checks |
|---|---|
| `fmt` | `cargo fmt --check` - zero formatting diff |
| `clippy` | zero warnings on stable + ai-agent feature |
| `build` | release binary + release ai-agent binary |
| `test` | all unit + integration tests |
| `docs` | `cargo doc` compiles without warnings |
| `msrv` | compiles on Rust 1.85.0 (MSRV) |
| `smoke` | binary runs `--demo --cycles 3` + `verify` without error |

---

## Module conventions

- Every public type must have a doc comment.
- Every module file must have a `//!` module-level doc block.
- Every `pub fn` that may fail must return `anyhow::Result<T>`.
- Every non-trivial module must contain an inline `#[cfg(test)]` block.
- New sensors must implement `Default` and the `sample()` pattern.
- Sensor tests must assert band / value boundaries over ≥ 50 samples.
