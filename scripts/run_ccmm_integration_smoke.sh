#!/usr/bin/env bash
set -euo pipefail

RES="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$#" -gt 1 ]; then
    echo "usage: $0 [path-to-ccmm-rs]" >&2
    exit 2
fi

if [ "$#" -eq 1 ]; then
    CCMM="$(cd "$1" && pwd)"
elif [ -d "$RES/../ccmm-rs" ]; then
    CCMM="$(cd "$RES/../ccmm-rs" && pwd)"
else
    echo "ERROR: ccmm-rs not found." >&2
    echo "usage: $0 /path/to/ccmm-rs" >&2
    exit 2
fi

SMOKE="$(mktemp -d "${TMPDIR:-/tmp}/ntt-resilience-ccmm-smoke.XXXXXX")"
trap 'rm -rf "$SMOKE"' EXIT

printf '=== CROSS-REPOSITORY NTT SMOKE TEST ===\n'
printf 'NTT_RESILIENCE=%s\n' "$RES"
printf 'NTT_RESILIENCE_HEAD=%s\n' "$(git -C "$RES" rev-parse HEAD)"
printf 'CCMM_RS=%s\n' "$CCMM"
printf 'CCMM_RS_HEAD=%s\n' "$(git -C "$CCMM" rev-parse HEAD)"

RES_STATUS_BEFORE="$(
    git -C "$RES" status --porcelain |
    grep -vE '^A  scripts/run_ccmm_integration_smoke\.sh$' ||
    true
)"
CCMM_STATUS_BEFORE="$(git -C "$CCMM" status --porcelain)"

if [ -n "$RES_STATUS_BEFORE" ]; then
    echo "ERROR: ntt-resilience has unrelated changes before the smoke test" >&2
    git -C "$RES" status --short
    exit 1
fi

if [ -n "$CCMM_STATUS_BEFORE" ]; then
    echo "ERROR: ccmm-rs must be clean before the smoke test" >&2
    git -C "$CCMM" status --short
    exit 1
fi

mkdir -p "$SMOKE/src"

cat > "$SMOKE/Cargo.toml" <<EOF
[package]
name = "ntt-resilience-ccmm-smoke"
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
    execute_ntt, MitigationMetrics, MitigationOptions, NttDirection, NttExecutionConfig,
    NttImplementation, RingParams,
};

fn mul_mod(a: u64, b: u64, q: u64) -> u64 {
    ((a as u128 * b as u128) % q as u128) as u64
}

fn pow_mod(mut base: u64, mut exp: u64, q: u64) -> u64 {
    let mut result = 1_u64;

    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod(result, base, q);
        }

        base = mul_mod(base, base, q);
        exp >>= 1;
    }

    result
}

fn main() -> Result<(), String> {
    let params = RingParams::new(16, 24)?;

    let n = params.n;
    let q = params.modulus;
    let psi = params.primitive_2n_root;

    let modulus = Modulus::new(q);
    let ccmm_plan = NttPlan::new(modulus, n, psi);

    assert_eq!(ccmm_plan.degree(), n);
    assert_eq!(ccmm_plan.modulus().value(), q);
    assert_eq!(ccmm_plan.psi(), psi);

    let input: Vec<u64> = (0..n)
        .map(|i| ((17 * i + 3) as u64) % q)
        .collect();

    let polynomial = Polynomial::new(modulus, input.clone());

    /*
     * CCMM's public radix-2 path is negacyclic:
     *
     *     coefficient polynomial
     *       -> psi^j twist
     *       -> cyclic NTT using omega = psi^2
     *
     * ntt-resilience exposes the cyclic NTT execution primitive, so the
     * compatibility adapter below applies the same twist explicitly.
     */
    let ccmm_forward = ccmm_plan.forward_radix2(&polynomial);

    let mut twisted = input.clone();
    let mut twist = 1_u64;

    for value in &mut twisted {
        *value = mul_mod(*value, twist, q);
        twist = mul_mod(twist, psi, q);
    }

    let forward_config = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::default(),
        fault: None,
    };

    let mut forward_metrics = MitigationMetrics::default();

    let (resilience_forward, _) = execute_ntt(
        &twisted,
        &params,
        &forward_config,
        None,
        &mut forward_metrics,
    )?;

    let forward_match = ccmm_forward == resilience_forward;

    let ccmm_inverse = ccmm_plan.inverse_radix2(&ccmm_forward);

    let inverse_config = NttExecutionConfig {
        direction: NttDirection::Inverse,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::default(),
        fault: None,
    };

    let mut inverse_metrics = MitigationMetrics::default();

    let (mut resilience_inverse, _) = execute_ntt(
        &resilience_forward,
        &params,
        &inverse_config,
        None,
        &mut inverse_metrics,
    )?;

    let psi_inverse = pow_mod(psi, q - 2, q);
    let mut untwist = 1_u64;

    for value in &mut resilience_inverse {
        *value = mul_mod(*value, untwist, q);
        untwist = mul_mod(untwist, psi_inverse, q);
    }

    let ccmm_roundtrip = ccmm_inverse.coefficients() == input.as_slice();
    let resilience_roundtrip = resilience_inverse == input;
    let inverse_match = ccmm_inverse.coefficients() == resilience_inverse.as_slice();

    let fault_injections =
        forward_metrics.fault_injections + inverse_metrics.fault_injections;

    println!("N={n}");
    println!("Q={q}");
    println!("PSI={psi}");
    println!("OMEGA={}", params.primitive_n_root);
    println!(
        "CCMM_FORWARD_MATCH={}",
        if forward_match { "PASS" } else { "FAIL" }
    );
    println!(
        "CCMM_INVERSE_MATCH={}",
        if inverse_match { "PASS" } else { "FAIL" }
    );
    println!(
        "ROUNDTRIP_NATIVE={}",
        if ccmm_roundtrip { "PASS" } else { "FAIL" }
    );
    println!(
        "ROUNDTRIP_RESILIENT={}",
        if resilience_roundtrip { "PASS" } else { "FAIL" }
    );
    println!("FAULT_INJECTIONS={fault_injections}");

    let pass = forward_match
        && inverse_match
        && ccmm_roundtrip
        && resilience_roundtrip
        && fault_injections == 0;

    println!("SMOKE_TEST={}", if pass { "PASS" } else { "FAIL" });

    if !pass {
        return Err("cross-repository NTT smoke test failed".into());
    }

    Ok(())
}
RUST

printf '\n=== FORMAT DISPOSABLE HARNESS ===\n'
cargo fmt --manifest-path "$SMOKE/Cargo.toml"
cargo fmt --manifest-path "$SMOKE/Cargo.toml" --check

printf '\n=== BUILD / LINT DISPOSABLE HARNESS ===\n'
cargo build --quiet --manifest-path "$SMOKE/Cargo.toml"
cargo clippy \
    --quiet \
    --manifest-path "$SMOKE/Cargo.toml" \
    --all-targets \
    -- -D warnings

printf '\n=== RUN CROSS-REPOSITORY SMOKE TEST ===\n'
cargo run --quiet --manifest-path "$SMOKE/Cargo.toml"

printf '\n=== SOURCE REPOSITORY INTEGRITY ===\n'

RES_STATUS_AFTER="$(
    git -C "$RES" status --porcelain |
    grep -vE '^A  scripts/run_ccmm_integration_smoke\.sh$' ||
    true
)"
CCMM_STATUS_AFTER="$(git -C "$CCMM" status --porcelain)"

test "$RES_STATUS_AFTER" = "$RES_STATUS_BEFORE" || {
    echo "ERROR: ntt-resilience changed during smoke test" >&2
    git -C "$RES" status --short
    exit 1
}

test "$CCMM_STATUS_AFTER" = "$CCMM_STATUS_BEFORE" || {
    echo "ERROR: ccmm-rs changed during smoke test" >&2
    git -C "$CCMM" status --short
    exit 1
}

echo "PASS: both source repositories untouched"
