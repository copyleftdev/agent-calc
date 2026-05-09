use agent_calc::{StatsRequest, StatsResponse};

fn corr(resp: StatsResponse) -> (f64, usize) {
    match resp {
        StatsResponse::Correlation { pearson_r, n, .. } => (pearson_r, n),
        other => panic!("expected Correlation, got {other:?}"),
    }
}

// ── Spearman ─────────────────────────────────────────────────────────────────

#[test]
fn spearman_perfect_positive() {
    let (rho, _) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![2.0, 4.0, 6.0, 8.0, 10.0],
        }
        .evaluate(),
    );
    assert!((rho - 1.0).abs() < 1e-12);
}

#[test]
fn spearman_perfect_negative() {
    let (rho, _) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![10.0, 8.0, 6.0, 4.0, 2.0],
        }
        .evaluate(),
    );
    assert!((rho + 1.0).abs() < 1e-12);
}

#[test]
fn spearman_known_value_0_8() {
    // ranks_x=[1,2,3,4,5], ranks_y=[1,3,2,5,4]; num=8, denom=10 → ρ=0.8
    let (rho, _) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![1.0, 3.0, 2.0, 5.0, 4.0],
        }
        .evaluate(),
    );
    assert!((rho - 0.8).abs() < 1e-12);
}

#[test]
fn spearman_n_reported_correctly() {
    let (_, n) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0, 3.0],
            y: vec![3.0, 1.0, 2.0],
        }
        .evaluate(),
    );
    assert_eq!(n, 3);
}

#[test]
fn spearman_with_ties_uses_average_ranks() {
    // x=[1,2,2,4]: ranks_x=[1,2.5,2.5,4]; y=[1,2,3,4]: ranks_y=[1,2,3,4]
    // mean_rx=2.5, mean_ry=2.5
    // num = (-1.5)(-1.5)+(0)(-0.5)+(0)(0.5)+(1.5)(1.5) = 4.5
    // denom = sqrt(4.5 * 5.0) = sqrt(22.5); ρ = 4.5/sqrt(22.5)
    let (rho, _) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0, 2.0, 4.0],
            y: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    let expected = 4.5 / 22.5_f64.sqrt();
    assert!((rho - expected).abs() < 1e-12);
}

#[test]
fn spearman_result_is_in_minus_one_to_one() {
    let (rho, _) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![3.0, 1.0, 4.0, 1.0, 5.0],
            y: vec![9.0, 2.0, 6.0, 5.0, 3.0],
        }
        .evaluate(),
    );
    assert!((-1.0..=1.0).contains(&rho));
}

#[test]
fn spearman_two_elements_succeeds() {
    // < → == mutation rejects n=2; < → <= also rejects n=2
    let (rho, n) = corr(
        StatsRequest::SpearmanCorrelation {
            x: vec![1.0, 2.0],
            y: vec![2.0, 4.0],
        }
        .evaluate(),
    );
    assert_eq!(n, 2);
    assert!((rho - 1.0).abs() < 1e-12);
}

#[test]
fn spearman_rejects_mismatched_lengths() {
    match (StatsRequest::SpearmanCorrelation {
        x: vec![1.0, 2.0],
        y: vec![1.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn spearman_rejects_single_pair() {
    match (StatsRequest::SpearmanCorrelation {
        x: vec![1.0],
        y: vec![2.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn spearman_rejects_nonfinite_x() {
    match (StatsRequest::SpearmanCorrelation {
        x: vec![f64::NAN, 2.0],
        y: vec![1.0, 2.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn spearman_rejects_nonfinite_y() {
    // ||→&& mutation: only y is nonfinite
    match (StatsRequest::SpearmanCorrelation {
        x: vec![1.0, 2.0],
        y: vec![1.0, f64::INFINITY],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn spearman_via_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"spearman_correlation","x":[1,2,3],"y":[3,2,1]}"#)
            .unwrap();
    let (rho, _) = corr(req.evaluate());
    assert!((rho + 1.0).abs() < 1e-12);
}

// ── Kendall τ-b ───────────────────────────────────────────────────────────────

#[test]
fn kendall_perfect_positive() {
    // C=10, D=0, T_x=0, T_y=0 → τ=1.0
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![2.0, 4.0, 6.0, 8.0, 10.0],
        }
        .evaluate(),
    );
    assert!((tau - 1.0).abs() < 1e-12);
}

#[test]
fn kendall_perfect_negative() {
    // C=0, D=10, T_x=0, T_y=0 → τ=-1.0
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![10.0, 8.0, 6.0, 4.0, 2.0],
        }
        .evaluate(),
    );
    assert!((tau + 1.0).abs() < 1e-12);
}

#[test]
fn kendall_known_value_0_6() {
    // x=[1,2,3,4,5], y=[1,3,2,5,4]; C=8, D=2, T_x=0, T_y=0
    // τ = (8-2)/sqrt(10*10) = 6/10 = 0.6
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![1.0, 3.0, 2.0, 5.0, 4.0],
        }
        .evaluate(),
    );
    assert!((tau - 0.6).abs() < 1e-12);
}

#[test]
fn kendall_concordant_minus_discordant_not_sum() {
    // C=8, D=2; if - → + mutation: (8+2)/10=1.0 ≠ 0.6
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![1.0, 3.0, 2.0, 5.0, 4.0],
        }
        .evaluate(),
    );
    assert!(tau < 0.9, "expected 0.6, not 1.0 (C+D mutation)");
    assert!(tau > 0.3, "expected 0.6, not lower");
}

#[test]
fn kendall_with_x_tie() {
    // x=[1,2,2,3], y=[1,2,3,4]: C=5, D=0, T_x=1, T_y=0
    // τ = 5/sqrt((5+0+1)*(5+0+0)) = 5/sqrt(30)
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 2.0, 3.0],
            y: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    let expected = 5.0 / 30.0_f64.sqrt();
    assert!((tau - expected).abs() < 1e-12);
}

#[test]
fn kendall_with_y_tie() {
    // x=[1,2,3,4], y=[1,2,2,3]: C=5, D=0, T_x=0, T_y=1
    // τ = 5/sqrt((5+0+0)*(5+0+1)) = 5/sqrt(30) (symmetric to x-tie case)
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0],
            y: vec![1.0, 2.0, 2.0, 3.0],
        }
        .evaluate(),
    );
    let expected = 5.0 / 30.0_f64.sqrt();
    assert!((tau - expected).abs() < 1e-12);
}

#[test]
fn kendall_with_both_x_and_y_ties_exact() {
    // x=[1,2,2,3,3], y=[1,2,3,3,4]
    // C=7, D=0, T_x=2, T_y=1
    // τ = 7/sqrt((7+0+2)*(7+0+1)) = 7/sqrt(72)
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 2.0, 3.0, 3.0],
            y: vec![1.0, 2.0, 3.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    let expected = 7.0 / 72.0_f64.sqrt();
    assert!((tau - expected).abs() < 1e-12);
}

#[test]
fn kendall_tie_x_count_matters_in_denominator() {
    // tau for x-tie case = 5/sqrt(30) ≈ 0.9129
    // if T_x ignored: 5/sqrt(25) = 1.0; if T_x doubled: 5/sqrt(35) ≈ 0.845
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 2.0, 3.0],
            y: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    assert!(tau < 0.95, "T_x must be included (not zero)");
    assert!(tau > 0.85, "T_x must not be over-counted");
}

#[test]
fn kendall_n_reported_correctly() {
    let (_, n) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0],
            y: vec![3.0, 2.0, 1.0],
        }
        .evaluate(),
    );
    assert_eq!(n, 3);
}

#[test]
fn kendall_result_in_minus_one_to_one() {
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![3.0, 1.0, 4.0, 1.0, 5.0],
            y: vec![9.0, 2.0, 6.0, 5.0, 3.0],
        }
        .evaluate(),
    );
    assert!((-1.0..=1.0).contains(&tau));
}

#[test]
fn kendall_signum_direction_correct() {
    // concordant-heavy → τ > 0; discordant-heavy → τ < 0
    let (tau_pos, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![1.0, 3.0, 2.0, 5.0, 4.0],
        }
        .evaluate(),
    );
    let (tau_neg, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y: vec![5.0, 3.0, 4.0, 1.0, 2.0],
        }
        .evaluate(),
    );
    assert!(tau_pos > 0.0);
    assert!(tau_neg < 0.0);
}

#[test]
fn kendall_two_elements_succeeds() {
    // < → == mutation rejects n=2; < → <= also rejects n=2
    let (tau, n) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0],
            y: vec![3.0, 4.0],
        }
        .evaluate(),
    );
    assert_eq!(n, 2);
    assert!((tau - 1.0).abs() < 1e-12);
}

#[test]
fn kendall_rejects_mismatched_lengths() {
    match (StatsRequest::KendallTau {
        x: vec![1.0, 2.0],
        y: vec![1.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kendall_rejects_single_pair() {
    match (StatsRequest::KendallTau {
        x: vec![1.0],
        y: vec![2.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kendall_rejects_nonfinite_x() {
    match (StatsRequest::KendallTau {
        x: vec![f64::NAN, 2.0],
        y: vec![1.0, 2.0],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kendall_rejects_nonfinite_y() {
    // ||→&& mutation: only y is nonfinite
    match (StatsRequest::KendallTau {
        x: vec![1.0, 2.0],
        y: vec![1.0, f64::INFINITY],
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kendall_via_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"kendall_tau","x":[1,2,3],"y":[3,2,1]}"#).unwrap();
    let (tau, _) = corr(req.evaluate());
    assert!((tau + 1.0).abs() < 1e-12);
}

#[test]
fn kendall_denom_uses_both_tie_counts() {
    // x=[1,2,2,3,3], y=[1,2,3,3,4]: C=7, D=0, T_x=2, T_y=1
    // denom = sqrt(9*8) = sqrt(72)
    // Mutation: T_x ignored → sqrt(7*8)=sqrt(56); T_y ignored → sqrt(9*7)=sqrt(63)
    let (tau, _) = corr(
        StatsRequest::KendallTau {
            x: vec![1.0, 2.0, 2.0, 3.0, 3.0],
            y: vec![1.0, 2.0, 3.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    let expected = 7.0 / 72.0_f64.sqrt();
    assert!((tau - expected).abs() < 1e-10);
    // tau ≈ 0.8248; sqrt(56) gives ≈0.936; sqrt(63) gives ≈0.882 — all distinct
    assert!(tau > 0.80 && tau < 0.85);
}
