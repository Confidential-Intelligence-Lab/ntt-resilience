use ntt_resilience::{
    execute_ntt, execute_ntt_pointwise_mul, MitigationMetrics, MitigationOptions, NttDirection,
    NttExecutionConfig, NttImplementation, NttPointwiseMulConfig, RingParams,
};

#[test]
fn external_consumer_can_execute_ntt_round_trip() {
    let params = RingParams::new(16, 24).expect("valid NTT parameters");

    let input: Vec<u64> = (0..params.n)
        .map(|i| ((17 * i + 5) as u64) % params.modulus)
        .collect();

    let forward = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault: None,
    };

    let mut forward_mitigation = MitigationMetrics::default();

    let (ntt_output, forward_trace) =
        execute_ntt(&input, &params, &forward, None, &mut forward_mitigation)
            .expect("external forward NTT execution should succeed");

    assert_eq!(ntt_output.len(), params.n);
    assert!(forward_trace.is_empty());
    assert_eq!(forward_mitigation.fault_injections, 0);
    assert!(!forward_mitigation.fault_detected);
    assert!(!forward_mitigation.fault_corrected);

    let inverse = NttExecutionConfig {
        direction: NttDirection::Inverse,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault: None,
    };

    let mut inverse_mitigation = MitigationMetrics::default();

    let (recovered, inverse_trace) = execute_ntt(
        &ntt_output,
        &params,
        &inverse,
        None,
        &mut inverse_mitigation,
    )
    .expect("external inverse NTT execution should succeed");

    assert_eq!(recovered, input);
    assert!(inverse_trace.is_empty());
    assert_eq!(inverse_mitigation.fault_injections, 0);
}

#[test]
fn external_consumer_can_execute_ntt_domain_pointwise_multiply() {
    let params = RingParams::new(16, 24).expect("valid NTT parameters");

    let input_a: Vec<u64> = (0..params.n)
        .map(|i| ((19 * i + 3) as u64) % params.modulus)
        .collect();

    let input_b: Vec<u64> = (0..params.n)
        .map(|i| ((23 * i + 7) as u64) % params.modulus)
        .collect();

    let forward = NttExecutionConfig {
        direction: NttDirection::Forward,
        trace: false,
        implementation: NttImplementation::Radix2,
        mitigation: MitigationOptions::disabled(),
        fault: None,
    };

    let mut mitigation_a = MitigationMetrics::default();
    let mut mitigation_b = MitigationMetrics::default();

    let (mut a_hat, _) = execute_ntt(&input_a, &params, &forward, None, &mut mitigation_a)
        .expect("external NTT(A) should succeed");

    let (mut b_hat, _) = execute_ntt(&input_b, &params, &forward, None, &mut mitigation_b)
        .expect("external NTT(B) should succeed");

    let pointwise = NttPointwiseMulConfig { fault: None };

    let (product, injections) =
        execute_ntt_pointwise_mul(&mut a_hat, &mut b_hat, &params, &pointwise)
            .expect("external pointwise multiplication should succeed");

    assert_eq!(product.len(), params.n);
    assert_eq!(injections, 0);
}
