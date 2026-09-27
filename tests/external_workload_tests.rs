use ntt_resilience::{
    execute_ntt, execute_ntt_pointwise_mul, ExecutionEvidence, FaultOperand, FaultSpec,
    MitigationMetrics, MitigationOptions, NttDirection, NttExecutionConfig, NttImplementation,
    NttPointwiseMulConfig, NttSystemMetrics, RingParams,
};

#[derive(Debug)]
struct MockWorkloadResult {
    output: Vec<u64>,
    evidence: ExecutionEvidence,
    system_metrics: NttSystemMetrics,
    mitigation_metrics: MitigationMetrics,
}

fn execute_mock_external_workload(
    params: &RingParams,
    input_a: &[u64],
    input_b: &[u64],
    fault: Option<FaultSpec>,
    golden_output: Option<&[u64]>,
) -> Result<MockWorkloadResult, String> {
    let fault_requested = fault.is_some();

    let mut system_metrics = NttSystemMetrics::default();
    let mut aggregate_mitigation = MitigationMetrics::default();

    let forward_a = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault,
    };

    let forward_b = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault: None,
    };

    let inverse = NttExecutionConfig {
        direction: NttDirection::Inverse,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault: None,
    };

    let mut mitigation_a = MitigationMetrics::default();
    let mut mitigation_b = MitigationMetrics::default();
    let mut mitigation_inverse = MitigationMetrics::default();

    let (mut a_hat, _) = execute_ntt(
        input_a,
        params,
        &forward_a,
        Some(&mut system_metrics),
        &mut mitigation_a,
    )?;

    let (mut b_hat, _) = execute_ntt(
        input_b,
        params,
        &forward_b,
        Some(&mut system_metrics),
        &mut mitigation_b,
    )?;

    let pointwise = NttPointwiseMulConfig { fault: None };

    let (product_hat, pointwise_injections) =
        execute_ntt_pointwise_mul(&mut a_hat, &mut b_hat, params, &pointwise)?;

    let (output, _) = execute_ntt(
        &product_hat,
        params,
        &inverse,
        Some(&mut system_metrics),
        &mut mitigation_inverse,
    )?;

    let fault_injections = mitigation_a.fault_injections
        + mitigation_b.fault_injections
        + mitigation_inverse.fault_injections
        + pointwise_injections;

    let fault_detected = mitigation_a.fault_detected
        || mitigation_b.fault_detected
        || mitigation_inverse.fault_detected;

    let fault_corrected = mitigation_a.fault_corrected
        || mitigation_b.fault_corrected
        || mitigation_inverse.fault_corrected;

    aggregate_mitigation.fault_injections = fault_injections;
    aggregate_mitigation.fault_detected = fault_detected;
    aggregate_mitigation.fault_corrected = fault_corrected;

    let outcome_observable = golden_output
        .map(|golden| output.as_slice() != golden)
        .unwrap_or(false);

    let evidence = ExecutionEvidence::classify(
        fault_requested,
        fault_injections,
        fault_detected,
        fault_corrected,
        outcome_observable,
    );

    Ok(MockWorkloadResult {
        output,
        evidence,
        system_metrics,
        mitigation_metrics: aggregate_mitigation,
    })
}

fn test_inputs(params: &RingParams) -> (Vec<u64>, Vec<u64>) {
    let a = (0..params.n)
        .map(|i| ((17 * i + 3) as u64) % params.modulus)
        .collect();

    let b = (0..params.n)
        .map(|i| ((29 * i + 5) as u64) % params.modulus)
        .collect();

    (a, b)
}

#[test]
fn mock_external_workload_executes_through_public_resilience_api() {
    let params = RingParams::new(16, 24).expect("valid ring parameters");
    let (a, b) = test_inputs(&params);

    let result = execute_mock_external_workload(&params, &a, &b, None, None)
        .expect("mock workload should execute");

    assert_eq!(result.output.len(), params.n);

    assert!(!result.evidence.fault_requested);
    assert_eq!(result.evidence.fault_injections, 0);
    assert!(result.evidence.execution_valid);
    assert!(!result.evidence.fault_detected);
    assert!(!result.evidence.fault_corrected);
    assert!(!result.evidence.outcome_observable);

    assert_eq!(result.mitigation_metrics.fault_injections, 0);

    assert!(result.system_metrics.num_mod_muls > 0);
    assert!(result.system_metrics.num_memory_reads > 0);
    assert!(result.system_metrics.num_memory_writes > 0);
}

#[test]
fn mock_external_workload_aggregates_fault_evidence_across_pipeline() {
    let params = RingParams::new(16, 24).expect("valid ring parameters");
    let (a, b) = test_inputs(&params);

    let golden = execute_mock_external_workload(&params, &a, &b, None, None)
        .expect("golden workload should execute");

    let fault = FaultSpec::new(FaultOperand::A, 0, 0, 0);

    let faulted =
        execute_mock_external_workload(&params, &a, &b, Some(fault), Some(&golden.output))
            .expect("faulted workload should execute");

    assert!(faulted.evidence.fault_requested);
    assert!(faulted.evidence.execution_valid);
    assert_eq!(faulted.evidence.fault_injections, 1);

    assert_eq!(
        faulted.evidence.fault_injections,
        faulted.mitigation_metrics.fault_injections
    );

    assert_eq!(
        faulted.evidence.outcome_observable,
        faulted.output != golden.output
    );
}
