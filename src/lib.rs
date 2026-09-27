//! Reusable NTT resilience execution library.
//!
//! The crate-level API exposes the canonical resilience execution boundary
//! without changing the historical transform, fault-injection, mitigation,
//! or experiment semantics used by the FDTC results.

pub mod evidence;
pub mod fault;
pub mod metrics;
pub mod mitigation;
pub mod modarith;
pub mod ntt;
pub mod params;
pub mod validation;

pub use evidence::{EvidenceAccumulator, ExecutionEvidence};
pub use fault::{FaultOperand, FaultSite, FaultSpec};
pub use mitigation::{
    ChecksumMode, MitigationAction, MitigationKind, MitigationMetrics, MitigationOptions,
};
pub use ntt::{
    execute_ntt, execute_ntt_pointwise_mul, NttDirection, NttExecutionConfig, NttImplementation,
    NttPointwiseMulConfig, NttSystemMetrics, StageTrace,
};
pub use params::RingParams;
