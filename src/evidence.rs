//! Common experimental evidence vocabulary for NTT resilience.
//!
//! These fields deliberately describe distinct observations. In particular:
//!
//! - requesting a fault does not prove that an injection executed;
//! - an executed injection need not become workload-visible;
//! - detection does not imply correction;
//! - correction does not by itself establish end-to-end correctness;
//! - masking does not by itself establish resilience.
//!
//! This module initially provides vocabulary and classification only. Existing
//! NTT, mitigation, CKKS, CLI, and campaign accounting remain authoritative
//! until later migration milestones explicitly connect them to this model.

/// Evidence collected for one fault-resilience experiment.
///
/// R1d-a is intentionally descriptive rather than prescriptive: it records
/// observations already represented elsewhere in the system without changing
/// their historical semantics.
// Transitional R1d-a allowance: this vocabulary is intentionally introduced
// before production producers/consumers migrate to it. Remove in R1d-b.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExecutionEvidence {
    /// Whether the experiment requested fault injection.
    pub fault_requested: bool,

    /// Number of fault injections that actually executed.
    ///
    /// This remains a count rather than a Boolean because one experiment may
    /// eventually contain multiple injection events.
    pub fault_injections: u64,

    /// Whether a resilience mechanism detected corruption.
    pub fault_detected: bool,

    /// Whether a resilience mechanism reports successful correction/recovery.
    ///
    /// This is not equivalent to end-to-end cryptographic correctness.
    pub fault_corrected: bool,

    /// Whether the injected corruption remained observable at the workload
    /// observation boundary.
    pub outcome_observable: bool,

    /// Whether the experiment itself is admissible for interpretation.
    ///
    /// In particular, requesting a fault that never actually executes makes
    /// the experiment invalid rather than "masked."
    pub execution_valid: bool,
}

// Transitional R1d-a allowance: production use begins in R1d-b.
#[allow(dead_code)]
impl ExecutionEvidence {
    /// Construct evidence from independently observed experiment facts.
    ///
    /// Experiment validity follows the historical CKKS validation rule:
    /// a requested fault must correspond to at least one actual injection.
    /// A no-fault execution is valid only when no injection occurs.
    pub fn classify(
        fault_requested: bool,
        fault_injections: u64,
        fault_detected: bool,
        fault_corrected: bool,
        outcome_observable: bool,
    ) -> Self {
        let execution_valid = if fault_requested {
            fault_injections > 0
        } else {
            fault_injections == 0
        };

        Self {
            fault_requested,
            fault_injections,
            fault_detected,
            fault_corrected,
            outcome_observable,
            execution_valid,
        }
    }

    /// True when at least one requested fault executed but did not remain
    /// observable at the workload boundary.
    ///
    /// This is an observation classification only. It is not a claim of
    /// resilience, security, detection, or correction.
    pub fn injected_but_not_observable(&self) -> bool {
        self.execution_valid
            && self.fault_requested
            && self.fault_injections > 0
            && !self.outcome_observable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_fault_golden_execution_is_valid() {
        let evidence = ExecutionEvidence::classify(false, 0, false, false, false);

        assert!(evidence.execution_valid);
        assert!(!evidence.fault_requested);
        assert_eq!(evidence.fault_injections, 0);
        assert!(!evidence.injected_but_not_observable());
    }

    #[test]
    fn requested_but_unmatched_fault_is_invalid() {
        let evidence = ExecutionEvidence::classify(true, 0, false, false, false);

        assert!(!evidence.execution_valid);
        assert!(evidence.fault_requested);
        assert_eq!(evidence.fault_injections, 0);
        assert!(!evidence.injected_but_not_observable());
    }

    #[test]
    fn injected_but_not_observable_is_valid_experiment() {
        let evidence = ExecutionEvidence::classify(true, 1, false, false, false);

        assert!(evidence.execution_valid);
        assert_eq!(evidence.fault_injections, 1);
        assert!(evidence.injected_but_not_observable());
    }

    #[test]
    fn injected_and_observable_is_valid_experiment() {
        let evidence = ExecutionEvidence::classify(true, 1, false, false, true);

        assert!(evidence.execution_valid);
        assert!(evidence.outcome_observable);
        assert!(!evidence.injected_but_not_observable());
    }

    #[test]
    fn detection_does_not_imply_correction() {
        let evidence = ExecutionEvidence::classify(true, 1, true, false, true);

        assert!(evidence.execution_valid);
        assert!(evidence.fault_detected);
        assert!(!evidence.fault_corrected);
    }

    #[test]
    fn correction_is_recorded_separately_from_observability() {
        let evidence = ExecutionEvidence::classify(true, 1, true, true, false);

        assert!(evidence.execution_valid);
        assert!(evidence.fault_detected);
        assert!(evidence.fault_corrected);
        assert!(!evidence.outcome_observable);
    }

    #[test]
    fn unexpected_injection_in_no_fault_run_is_invalid() {
        let evidence = ExecutionEvidence::classify(false, 1, false, false, true);

        assert!(!evidence.execution_valid);
        assert!(!evidence.fault_requested);
        assert_eq!(evidence.fault_injections, 1);
    }

    #[test]
    fn multiple_actual_injections_remain_counted() {
        let evidence = ExecutionEvidence::classify(true, 3, true, false, true);

        assert!(evidence.execution_valid);
        assert_eq!(evidence.fault_injections, 3);
    }
}
