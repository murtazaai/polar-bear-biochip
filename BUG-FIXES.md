# Bug Fixes & Root-Cause Analysis

Resolved issues in `polar-bear-biochip`, with root cause and fix.

---

## Fix 1 - `&PathBuf` parameter anti-pattern in `run_cycle`

**File**: `src/main.rs`

**Symptom**: Clippy `ptr_arg` warning:
```
warning: writing `&PathBuf` instead of `&Path` involves a lot of pointer chasing
  --> src/main.rs:97:24
   |
97 |     output_dir: &PathBuf,
   |                 ^^^^^^^^ help: change this to: `&Path`
```

**Root cause**: `&PathBuf` is a borrowed reference to an owned `PathBuf`. The
`Deref` chain is `&PathBuf → &Path`, so callers pay double indirection for no
benefit. The idiomatic Rust type for a borrowed path is `&Path` (analogous to
using `&str` instead of `&String`).

**Fix**: Changed all `output_dir: &PathBuf` parameters to `output_dir: &Path`.
Call sites pass `&cli.output_dir` as before - the coercion is implicit.

---

## Fix 2 - Dead feature flag attribute `#[cfg(not(feature = "demo_only"))]`

**File**: `src/agent/biochip_agent.rs`

**Symptom**: Attribute referenced a feature `demo_only` that was never declared
in `Cargo.toml`. With `[features]` tables enforcing declared-only features since
Rust 1.60, this silently compiled as `cfg(false)` - the guarded code was never
included in any build.

**Fix**: Removed the dead `#[cfg(not(feature = "demo_only"))]` guard. Replaced
with the correct compile-time feature split:
- `#[cfg(feature = "ai-agent")]` gates the `rig-core` backend.
- `#[cfg(not(feature = "ai-agent"))]` gates the `curl` fallback.

---

## Fix 3 - Duplicated `AlertLevel` match arms in `main.rs`

**File**: `src/main.rs`

**Symptom**: `run_cycle` contained an inline `match result.alert_level` block
that duplicated the icon/label logic already present in `cmd_verify`. Any change
to alert level display required two edits.

**Fix**: Added `impl fmt::Display for AlertLevel` in `src/types.rs`:
```rust
impl fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertLevel::Normal   => write!(f, "✅ Normal"),
            AlertLevel::Elevated => write!(f, "⚠️  Elevated"),
            AlertLevel::Critical => write!(f, "🚨 CRITICAL"),
        }
    }
}
```
Both `run_cycle` and `cmd_verify` now call `result.alert_level.to_string()` / `{}` format.

---

## Fix 4 - Dependency version `=` exact pins (semver regression risk)

**File**: `Cargo.toml`

**Symptom**: Multiple dependencies used exact `=` version pins
(`clap = "=4.4.18"`, `base64ct = "=1.6.0"`) as workarounds for sandbox
build environment limitations rather than true production constraints.

**Root cause**: The sandbox ran Rust 1.75.0 (Ubuntu system package), which
could not parse `edition = "2024"` in transitive `idna_adapter` and
`hashbrown` dependencies. The `=` pins forced older versions that did not
require `edition2024` Cargo support.

**Fix**: Upgraded to edition 2024 with `rust-version = "1.85.0"` (MSRV),
matching the `polar-bear-hft-crypto` standard. All `=` pins replaced with
`^` semver-compatible constraints. The `base64ct` workaround pin is removed
entirely (k256 0.13.x resolves a compatible version automatically on modern
toolchains).

---

## Fix 5 - Missing `EcdsaVerifier` standalone struct

**File**: `src/provenance/ecdsa_signer.rs`

**Symptom**: Offline verification required constructing an `EcdsaSigner`
(which holds private key material) purely to call its `verify_signed` method.
This violated least-privilege: audit tools should not need - or be given - a
signing key.

**Fix**: Added `EcdsaVerifier` struct (mirrors `polar-bear-hft-crypto`):
```rust
pub struct EcdsaVerifier { verifying_key: VerifyingKey }

impl EcdsaVerifier {
    pub fn from_hex(hex_str: &str) -> Result<Self> { ... }
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> Result<bool> { ... }
}
```
`main.rs` inline verification now uses `EcdsaVerifier::from_hex(&signed.public_key_hex)?`
after signing, demonstrating that verification never touches private material.

---

## Fix #19 — LICENSE not SPDX-compliant (`cargo publish` blocker)

**Stage:** 08 — Publication  
**Error:** `error: the `license` field in `Cargo.toml` is not a valid SPDX expression`  
**Root cause:** `license = "LicensePBS"` is a custom proprietary identifier. crates.io requires a valid SPDX expression (e.g. `MIT OR Apache-2.0`). Additionally, the `LICENSE-PBS` proprietary all-rights-reserved file prohibits redistribution, which is incompatible with crates.io's requirement for an open-source license.  
**Fix:**  
- Changed `license = "LicensePBS"` → `license = "MIT OR Apache-2.0"` in `Cargo.toml`  
- Added `LICENSE-MIT` (MIT License, Copyright 2025 Murtaza Ali Imtiaz)  
- Added `LICENSE-APACHE` (Apache License 2.0, Copyright 2025 Murtaza Ali Imtiaz)  
- Removed `LICENSE-PBS`  

---

## Fix #20 — Missing `[lib]` target (crate not usable as library dependency)

**Stage:** 08 — Publication  
**Error:** No error at compile time, but the crate publishes as binary-only. Downstream callers doing `polar-bear-biochip = "0.2"` cannot `use polar_bear_biochip::...`.  
**Root cause:** `src/lib.rs` exists and exports the full public API, but `Cargo.toml` only declared `[[bin]]`. Without an explicit `[lib]` section, Cargo infers a lib target from `src/lib.rs` in a mixed crate — however, declaring it explicitly is required for docs.rs feature-gating and for clean crates.io publication metadata.  
**Fix:**  
```toml
[lib]
name = "polar_bear_biochip"
path = "src/lib.rs"
```
