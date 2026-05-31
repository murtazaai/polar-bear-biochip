# Changelog

All notable changes to `polar-bear-biochip` are documented here.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.2.0] - 2026-05-19

### Added
- `rustfmt.toml` - code-style rules (100 cols, Rust 2024 edition, crate-level imports)
- `.clippy.toml` - Clippy config with MSRV 1.85.0 and complexity thresholds
- `.env.example` - template for `ANTHROPIC_API_KEY`, model, cycle, and output tuning
- `CHANGELOG.md` - this file
- `CONTRIBUTING.md` - contribution guide with full dev workflow
- `FILE_STRUCTURE.md` - annotated repository map
- `BUG-FIXES.md` - root-cause analysis of resolved issues
- `docs/architecture.md` - full system architecture with ASCII layer diagram and data flow
- `docs/bci_math.md` - EEG signal processing math: band taxonomy, derived indices,
  sensor fusion formulas, activity classification, random walk model, ICA notes
- `examples/sensors_demo.rs` - standalone sensor fusion table demo (no API key)
- `examples/provenance_demo.rs` - ECDSA sign/verify/tamper demo (no API key)
- `examples/agent_demo.rs` - Rig AI agent demo (`--features ai-agent` or demo fallback)
- `tests/sensor_tests.rs` - 12 integration tests covering BCI, accelerometer, fusion
- `tests/provenance_tests.rs` - 14 integration tests covering ECDSA sign/verify/tamper
- `src/lib.rs` - crate root with architecture diagram and public module re-exports
- `src/error.rs` - typed `BiochipError` hierarchy using `thiserror`
- `src/provenance/ecdsa_signer.rs` - `EcdsaVerifier` standalone verifier struct;
  `from_bytes()` and `from_hex()` constructors; `private_key_bytes()` / `verifying_key_hex()`
- `.zed/tasks.json` / `debug.json` - Zed IDE task and debug launch config

### Changed
- `Cargo.toml` - upgraded to **Rust 2024 edition**; added `rust-version = "1.85.0"` (MSRV);
  `[package.metadata.docs.rs]`, `[lints.rust]`, and `[lints.clippy]` tables;
  `[features]` with `ai-agent` optional flag gating `rig-core ^0.37` + `dotenvy`;
  relaxed `=` exact pins to `^` semver-compatible; added `thiserror ^2` and `dotenvy ^0.15`;
  added `[dev-dependencies]` (`pretty_assertions`, `tokio-test`);
  release profile: `panic = "abort"`, `strip = "debuginfo"`
- `src/main.rs` - restructured to `run` / `verify` subcommands (clap `Subcommand`);
  `&PathBuf` parameters replaced with `&Path`; `AlertLevel::Display` used instead of
  inline match; inline verification now uses standalone `EcdsaVerifier`
- `src/types.rs` - added `fmt::Display` for `BciReading`, `AccelerometerReading`,
  `FusedReading`, and `AlertLevel`; added `Eq` derive on `ActivityState` and `AlertLevel`
- `src/sensors/bci.rs` - added `Default` impl; 3 inline unit tests
- `src/sensors/accelerometer.rs` - added `Default` impl; 4 inline unit tests
- `src/sensors/fusion.rs` - added `Default` impl; 3 inline unit tests with
  boundary and monotonicity assertions
- `src/agent/biochip_agent.rs` - restructured with `ai-agent` feature flag;
  `rig-core` used when feature is enabled; `curl` subprocess as fallback;
  added `Default` impl (`demo = true`); 3 inline unit tests
- `src/provenance/ecdsa_signer.rs` - 7 inline unit tests; consistent with
  `polar-bear-hft-crypto` ECDSA module style
- `.github/workflows/ci.yml` - added: MSRV check (1.85.0), `cargo doc` validation,
  `ai-agent` feature build and test steps, smoke-test job with `verify` command

### Fixed
- **Fix 1** - `src/main.rs`: `run_cycle` parameter changed from `&PathBuf` to `&Path`.
  `&PathBuf` is a double-indirection anti-pattern; `&Path` is the idiomatic Rust type
  for borrowed path references (clippy `ptr_arg` lint).
- **Fix 2** - `src/agent/biochip_agent.rs`: removed `#[cfg(not(feature = "demo_only"))]`
  dead feature flag attribute that referenced a non-existent feature.
- **Fix 3** - `src/main.rs`: replaced inline `AlertLevel` match arms with
  `result.alert_level.Display` impl, eliminating the duplicated match pattern.

---

## [0.1.0] - 2026-05-18

Initial release:

- Multi-sensor BCI + accelerometer fusion pipeline
- EEG band simulation (δ θ α β γ) with derived attention and meditation indices
- Sensor fusion: `cognitive_load`, `emotional_valence`, `arousal_level`
- rig-core–compatible `BioChipAgent` with demo mode (no API key required)
- ECDSA secp256k1 signing of every `InferenceResult` → tamper-evident `SignedOutput` JSON
- `--verify` CLI flag for offline signature verification
- GitHub Actions CI: build + smoke test

## [0.2.1] — 2026-05-31

### Fixed
- **LICENSE**: Replaced proprietary `LicensePBS` with `MIT OR Apache-2.0` dual licence; added `LICENSE-MIT` and `LICENSE-APACHE` files. `cargo publish` now passes SPDX validation.
- **Cargo.toml**: Added explicit `[lib]` target so `polar-bear-biochip` is usable as a library dependency as well as a binary.
- **Security**: Removed `.env` (API key file) from repository; `.gitignore` already excluded it but the file was present in the archive.
