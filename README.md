# ntt-resilience

**Structure-Aware Software Resilience for Number Theoretic Transforms**

`ntt-resilience` is a research framework for studying software-based detection and recovery of transient arithmetic faults in Number Theoretic Transform (NTT) computations.

The framework exploits algebraic structure intrinsic to radix-2 NTTs rather than relying exclusively on full computational redundancy. It implements fine-grained butterfly consistency invariants, stage-level checksum verification, selective recomputation, configurable fault injection, and end-to-end evaluation through a CKKS-inspired application pipeline.

This repository accompanies the FDTC 2026 paper:

> **Short Paper: Structure-Aware Software Resilience for Number Theoretic Transforms**<br>
> Brittany Liu, Caroline Wang, Jayanta Chowdhury, Igor Nunes, Nahid Farhady, Elif Bilge Kavun, and Ro Cammarota.<br>
> Accepted for presentation at the 2026 Workshop on Fault Diagnosis and Tolerance in Cryptography (FDTC 2026).

## What the Framework Provides

### Structure-Aware Fault Detection

Two complementary software mechanisms exploit intrinsic properties of radix-2 NTT computation:

- **Butterfly consistency invariants** validate individual radix-2 butterflies and provide fine-grained fault localization.
- **Stage-level checksums** use Sum and Sum+Index projections to detect corruption at transform-stage granularity.

The mechanisms expose different trade-offs among protection scope, detection coverage, localization granularity, recovery granularity, and execution overhead.

### Selective Recovery

Detected transient faults can trigger recomputation at the granularity exposed by the detector:

- butterfly-level recomputation for butterfly consistency violations;
- stage-level rollback and recomputation for stage-checksum violations.

Recovery assumes a transient, non-recurring fault model.

### Configurable Fault Injection

The framework supports controlled arithmetic fault injection across representative locations in the NTT datapath, including:

- inputs;
- twiddle-multiplication outputs;
- modular-addition outputs;
- modular-subtraction outputs;
- butterfly outputs; and
- register writes.

Experimental campaigns support single-bit and more complex transient fault models.

### Application-Level Evaluation

A CKKS-inspired encode-compute-decode pipeline is provided to determine whether faults introduced during forward or inverse NTT execution propagate to application-visible output corruption.

The CKKS-inspired pipeline is an **evaluation environment**. The proposed resilience mechanisms operate on the NTT itself and are not specific to homomorphic encryption.

## Repository Structure

~~~text
src/          Rust implementation of NTTs, fault injection,
              mitigation, validation, and evaluation

tests/        Unit and CLI integration tests

scripts/      Automated experimental campaigns

analysis/     Analysis and result-processing utilities

results/      Experimental datasets and generated results

docs/         Additional experimental documentation
~~~

## Building

### Requirements

- Rust stable toolchain
- Cargo
- Python 3.11 or later for experimental automation and analysis

Build the release version:

~~~bash
cargo build --release
~~~

Run the test suite:

~~~bash
cargo test --all
~~~

Check formatting and linting:

~~~bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
~~~

## Quick Start

Run a fault-free CKKS-inspired computation:

~~~bash
cargo run --release -- ckks-demo \
    --n 2048 \
    --bits 54 \
    --validate
~~~

### Inject a Transient Fault

For example, inject a bit corruption into a butterfly output during the forward NTT:

~~~bash
cargo run --release -- ckks-demo \
    --n 2048 \
    --bits 54 \
    --fault \
    --fault-op ntt \
    --fault-site butterfly-output \
    --fault-stage 0 \
    --fault-slot 0 \
    --fault-bit 40 \
    --validate
~~~

### Enable Butterfly Consistency Checking

~~~bash
cargo run --release -- ckks-demo \
    --fault \
    --mitigation butterfly-check \
    --mitigation-action recompute
~~~

### Enable Stage-Level Sum+Index Checking

~~~bash
cargo run --release -- ckks-demo \
    --fault \
    --mitigation stage-checksum \
    --checksum-mode sum-index \
    --mitigation-action detect-only
~~~

## Experimental Campaigns

The `scripts/` directory contains automated campaigns for:

- transient fault propagation and observability;
- butterfly-invariant detection;
- stage-level checksum detection;
- selective recovery;
- practical multi-bit and multi-location fault models; and
- runtime-overhead characterization.

The repository also contains analysis utilities for processing campaign outputs.

Detailed instructions for reproducing the final FDTC 2026 experimental results are provided in [`REPRODUCIBILITY.md`](REPRODUCIBILITY.md).

> **Artifact status:** The repository is currently being synchronized with the FDTC 2026 camera-ready artifact. The final paper-to-artifact mapping and frozen experimental datasets will be identified by the FDTC 2026 release tag.

## Scope

`ntt-resilience` is intended as a research and evaluation framework, not as a production cryptographic library.

The current implementation focuses on radix-2 NTTs and transient arithmetic faults. Persistent faults and active adversarial fault attacks require stronger assumptions and countermeasures and are outside the current recovery model.

Although the FDTC evaluation uses a CKKS-inspired computation to expose application-level effects, the structure-aware mechanisms operate at the NTT level and can be investigated in other NTT-based lattice-cryptographic workloads.

## Citation

If you use this software or build upon the structure-aware resilience mechanisms, please cite:

> Brittany Liu, Caroline Wang, Jayanta Chowdhury, Igor Nunes, Nahid Farhady, Elif Bilge Kavun, and Ro Cammarota.<br>
> **Short Paper: Structure-Aware Software Resilience for Number Theoretic Transforms.**<br>
> FDTC 2026.

Machine-readable citation metadata is available in [`CITATION.cff`](CITATION.cff).

## License

See [`LICENSE`](LICENSE).

## Project

Developed by the **Confidential Intelligence Lab**, University of California, Irvine.

https://github.com/Confidential-Intelligence-Lab
