#!/usr/bin/env bash
set -euo pipefail

RES="/Users/ro/Documents/uci_research/Prelim/ntt-resilience"
CCMM="/Users/ro/Documents/uci_research/Prelim/ccmm-rs"
SMOKE="$(mktemp -d "${TMPDIR:-/tmp}/ntt-fault-smoke.XXXXXX")"

trap 'rm -rf "$SMOKE"' EXIT

printf '=== BASELINES ===\n'
printf 'ntt-resilience: '
git -C "$RES" rev-parse HEAD
printf 'ccmm-rs:        '
git -C "$CCMM" rev-parse HEAD

RES_STATUS_BEFORE="$(
    git -C "$RES" status --porcelain |
    grep -vE '^(\?\?|A ) scripts/run_ccmm_fault_smoke\.sh$' ||
    true
)"

if [ -n "$RES_STATUS_BEFORE" ]; then
    echo "ERROR: ntt-resilience has unrelated changes"
    git -C "$RES" status --short
    exit 1
fi

test -z "$(git -C "$CCMM" status --porcelain)" || {
    echo "ERROR: ccmm-rs is not clean"
    git -C "$CCMM" status --short
    exit 1
}

mkdir -p "$SMOKE/src"

cat > "$SMOKE/Cargo.toml" <<EOF
[package]
name = "ntt-fault-smoke"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
ccmm-rs = { path = "$CCMM" }
ntt_resilience = { path = "$RES" }
EOF

cat > "$SMOKE/src/main.rs" <<'RUST'
use ccmm_rs::ring::{Modulus, NttPlan, Polynomial};

use ntt_resilience::{
    execute_ntt, ChecksumMode, FaultOperand, FaultSite, FaultSpec, MitigationAction,
    MitigationKind, MitigationMetrics, MitigationOptions, NttDirection, NttExecutionConfig,
    NttImplementation, RingParams,
};

fn mul_mod(a: u64, b: u64, q: u64) -> u64 {
    ((a as u128 * b as u128) % q as u128) as u64
}

fn twist(input: &[u64], psi: u64, q: u64) -> Vec<u64> {
    let mut out = input.to_vec();
    let mut w = 1_u64;

    for value in &mut out {
        *value = mul_mod(*value, w, q);
        w = mul_mod(w, psi, q);
    }

    out
}

fn execute(
    input: &[u64],
    params: &RingParams,
    fault: Option<FaultSpec>,
    mitigation: MitigationOptions,
) -> Result<(Vec<u64>, MitigationMetrics), String> {
    let config = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation,
        fault,
    };

    let mut metrics = MitigationMetrics::default();

    let (output, _) =
        execute_ntt(input, params, &config, None, &mut metrics)?;

    Ok((output, metrics))
}

fn main() -> Result<(), String> {
    let params = RingParams::new(16, 24)?;

    let n = params.n;
    let q = params.modulus;
    let psi = params.primitive_2n_root;

    let modulus = Modulus::new(q);
    let plan = NttPlan::new(modulus, n, psi);

    let input: Vec<u64> =
        (0..n).map(|i| ((17 * i + 3) as u64) % q).collect();

    let polynomial = Polynomial::new(modulus, input.clone());

    let oracle = plan.forward_radix2(&polynomial);
    let twisted = twist(&input, psi, q);

    let (clean, clean_metrics) = execute(
        &twisted,
        &params,
        None,
        MitigationOptions::disabled(),
    )?;

    let mut fault = FaultSpec::new(FaultOperand::A, 0, 1, 0);
    fault.site = FaultSite::MulOutput;

    let detect_only = MitigationOptions {
        kind: MitigationKind::ButterflyCheck,
        action: MitigationAction::DetectOnly,
        max_retries: 1,
        checksum_mode: ChecksumMode::Sum,
    };

    let (faulted, detect_metrics) =
        execute(&twisted, &params, Some(fault.clone()), detect_only)?;

    let recompute = MitigationOptions {
        kind: MitigationKind::ButterflyCheck,
        action: MitigationAction::Recompute,
        max_retries: 1,
        checksum_mode: ChecksumMode::Sum,
    };

    let (recovered, recovery_metrics) =
        execute(&twisted, &params, Some(fault), recompute)?;

    let clean_match = clean == oracle;
    let fault_changed = faulted != oracle;
    let recovery_match = recovered == oracle;

    println!("N={n}");
    println!("Q={q}");
    println!("PSI={psi}");

    println!();
    println!("=== CLEAN BASELINE ===");
    println!("CLEAN_MATCH={clean_match}");
    println!(
        "CLEAN_FAULT_INJECTIONS={}",
        clean_metrics.fault_injections
    );

    println!();
    println!("=== INJECT + DETECT ONLY ===");
    println!(
        "FAULT_INJECTIONS={}",
        detect_metrics.fault_injections
    );
    println!(
        "FAULT_DETECTED={}",
        detect_metrics.fault_detected
    );
    println!(
        "FAULT_CORRECTED={}",
        detect_metrics.fault_corrected
    );
    println!("FAULT_CHANGED_OUTPUT={fault_changed}");
    println!(
        "CHECKS_PERFORMED={}",
        detect_metrics.checks_performed
    );
    println!(
        "CHECK_FAILURES={}",
        detect_metrics.check_failures
    );

    println!();
    println!("=== INJECT + RECOMPUTE ===");
    println!(
        "RECOVERY_FAULT_INJECTIONS={}",
        recovery_metrics.fault_injections
    );
    println!(
        "RECOVERY_DETECTED={}",
        recovery_metrics.fault_detected
    );
    println!(
        "RECOVERY_CORRECTED={}",
        recovery_metrics.fault_corrected
    );
    println!(
        "RECOVERY_RECOMPUTATIONS={}",
        recovery_metrics.recomputations
    );
    println!("RECOVERY_MATCH={recovery_match}");

    let pass = clean_match
        && clean_metrics.fault_injections == 0
        && detect_metrics.fault_injections == 1
        && detect_metrics.fault_detected
        && !detect_metrics.fault_corrected
        && fault_changed
        && recovery_metrics.fault_injections == 1
        && recovery_metrics.fault_detected
        && recovery_metrics.fault_corrected
        && recovery_match;

    println!();
    println!(
        "FAULT_SMOKE_TEST={}",
        if pass { "PASS" } else { "FAIL" }
    );

    if !pass {
        return Err("fault smoke test failed".to_string());
    }

    Ok(())
}
RUST

printf '\n=== FORMAT ===\n'
cargo fmt --manifest-path "$SMOKE/Cargo.toml"

printf '\n=== BUILD ===\n'
cargo build --quiet --manifest-path "$SMOKE/Cargo.toml"

printf '\n=== CLIPPY ===\n'
cargo clippy \
    --quiet \
    --manifest-path "$SMOKE/Cargo.toml" \
    --all-targets \
    -- -D warnings

printf '\n=== RUN ===\n'
cargo run --quiet --manifest-path "$SMOKE/Cargo.toml"

printf '\n=== REPOSITORY INTEGRITY ===\n'

RES_STATUS_AFTER="$(
    git -C "$RES" status --porcelain |
    grep -vE '^(\?\?|A ) scripts/run_ccmm_fault_smoke\.sh$' ||
    true
)"

if [ "$RES_STATUS_AFTER" != "$RES_STATUS_BEFORE" ]; then
    echo "ERROR: ntt-resilience changed during fault smoke test"
    git -C "$RES" status --short
    exit 1
fi

test -z "$(git -C "$CCMM" status --porcelain)" || {
    echo "ERROR: ccmm-rs changed during fault smoke test"
    git -C "$CCMM" status --short
    exit 1
}

echo "PASS: both repositories untouched"
