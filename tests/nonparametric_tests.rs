use agent_calc::{StatsRequest, StatsResponse, Tail};

fn hypothesis(resp: StatsResponse) -> (f64, f64, f64, f64, f64, bool, String) {
    match resp {
        StatsResponse::HypothesisTest {
            statistic,
            p_value,
            dof,
            critical_value,
            alpha,
            reject_h0,
            conclusion,
            ..
        } => (
            statistic,
            p_value,
            dof,
            critical_value,
            alpha,
            reject_h0,
            conclusion,
        ),
        other => panic!("expected HypothesisTest, got {other:?}"),
    }
}

// ── Mann-Whitney U ────────────────────────────────────────────────────────────

#[test]
fn mwu_u1_zero_when_sample1_all_smaller() {
    // sample1=[1,2,3] < sample2=[4,5,6]; all pairs have x1<x2, so U1=0
    let (u1, _, _, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(u1, 0.0);
}

#[test]
fn mwu_u1_n1n2_when_sample1_all_larger() {
    // sample1=[4,5,6] > sample2=[1,2,3]; every pair has x1>x2, so U1=n1*n2=9
    let (u1, _, _, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![4.0, 5.0, 6.0],
            sample2: vec![1.0, 2.0, 3.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(u1, 9.0);
}

#[test]
fn mwu_u1_half_n1n2_for_identical_samples() {
    // identical samples → U1 = n1*n2/2 = 4.5, z=0
    let (u1, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![1.0, 2.0, 3.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(u1, 4.5);
    assert!((p_value - 1.0).abs() < 1e-6);
    assert!(!reject_h0);
}

#[test]
fn mwu_z_approx_for_separated_samples() {
    // z = (0 - 4.5) / sqrt(9*7/12) = -4.5/sqrt(5.25) ≈ -1.9640
    // p ≈ 2*(1-Φ(1.9640)) ≈ 0.04961; sigma_u mutation (+→*) gives p≈0.034
    let (_, p_value, _, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert!(p_value > 0.045 && p_value < 0.055);
}

#[test]
fn mwu_dof_is_zero() {
    let (_, _, dof, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0],
            sample2: vec![3.0, 4.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(dof, 0.0);
}

#[test]
fn mwu_reject_h0_strict_lt() {
    // Force p=1.0 by using identical samples, alpha=1.0 → p < alpha is false
    // but p <= alpha would be true; confirms strict <
    let (_, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![1.0, 2.0, 3.0],
            alpha: 1.0,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(p_value, 1.0);
    assert!(!reject_h0);
}

#[test]
fn mwu_conclusion_contains_reject_or_fail() {
    let (_, _, _, _, _, reject_h0, conclusion) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    if reject_h0 {
        assert!(conclusion.contains("Reject"));
    } else {
        assert!(conclusion.contains("Fail"));
    }
}

#[test]
fn mwu_critical_value_not_unit_two_tailed() {
    // z_{0.975} ≈ 1.96, not 1.0 or -1.0
    let (_, _, _, critical_value, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0],
            sample2: vec![3.0, 4.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert!((critical_value - 1.96).abs() < 0.01);
}

#[test]
fn mwu_default_alpha_via_json() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"mann_whitney_u","sample1":[1,2,3],"sample2":[4,5,6]}"#)
            .unwrap();
    let (_, _, _, _, alpha, _, _) = hypothesis(req.evaluate());
    assert!((alpha - 0.05).abs() < 1e-12);
}

#[test]
fn mwu_asymmetric_sizes_u1_exact() {
    // sample1=[1,2], sample2=[3,4,5]
    // pool [1,2,3,4,5] → ranks [1,2,3,4,5]
    // R1=1+2=3, U1=3-2*3/2=3-3=0
    let (u1, _, _, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0],
            sample2: vec![3.0, 4.0, 5.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(u1, 0.0);
}

#[test]
fn mwu_rejects_sample1_too_small() {
    match (StatsRequest::MannWhitneyU {
        sample1: vec![1.0],
        sample2: vec![2.0, 3.0],
        alpha: 0.05,
        tail: Tail::Two,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

// ── Mann-Whitney U (directional tails / critical value) ──────────────────────

#[test]
fn mwu_right_tail_p_small_when_sample1_larger() {
    // Tail::Right: sample1=[4,5,6] all larger than sample2=[1,2,3]; U1=9
    // z=(9-4.5)/sqrt(5.25)≈+1.964; p=1-Φ(1.964)≈0.0248
    // Mutation 1.0-n.cdf(z) → 1.0+n.cdf(z) gives p≈1.975; → 1.0/n.cdf(z) gives p≈1.026
    let (_, p_value, _, _, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![4.0, 5.0, 6.0],
            sample2: vec![1.0, 2.0, 3.0],
            alpha: 0.05,
            tail: Tail::Right,
        }
        .evaluate(),
    );
    assert!(p_value > 0.01 && p_value < 0.05);
}

#[test]
fn mwu_left_tail_critical_value_is_negative_and_finite() {
    // Tail::Left, alpha=0.05: cv = -Φ⁻¹(0.95) ≈ -1.645
    // Mutation "delete -" gives +1.645; mutation "- → +" gives Φ⁻¹(1.05)=+inf
    let (_, _, _, critical_value, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Left,
        }
        .evaluate(),
    );
    assert!(critical_value < 0.0);
    assert!(critical_value.is_finite());
    assert!((critical_value + 1.645).abs() < 0.01);
}

#[test]
fn mwu_right_tail_critical_value_positive_and_reasonable() {
    // Tail::Right, alpha=0.05: cv = Φ⁻¹(0.95) ≈ +1.645
    // Mutation "- → +" gives Φ⁻¹(1.05)=+inf; "- → /" gives Φ⁻¹(20)=+inf
    let (_, _, _, critical_value, _, _, _) = hypothesis(
        StatsRequest::MannWhitneyU {
            sample1: vec![1.0, 2.0, 3.0],
            sample2: vec![4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Right,
        }
        .evaluate(),
    );
    assert!(critical_value > 0.0);
    assert!(critical_value.is_finite());
    assert!((critical_value - 1.645).abs() < 0.01);
}

// ── Wilcoxon Signed-Rank ──────────────────────────────────────────────────────

#[test]
fn wilcoxon_w_plus_exact_all_positive_diffs() {
    // before=[1,2,3,4,5], after=[2,3,4,5,6], diffs=[1,1,1,1,1]
    // all tied; avg rank = 3.0; W+ = 5*3 = 15
    let (w_plus, _, _, _, _, _, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            after: vec![2.0, 3.0, 4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(w_plus, 15.0);
}

#[test]
fn wilcoxon_w_plus_zero_all_negative_diffs() {
    // all after < before → W+ = 0
    let (w_plus, _, _, _, _, _, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![2.0, 3.0, 4.0, 5.0, 6.0],
            after: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(w_plus, 0.0);
}

#[test]
fn wilcoxon_w_plus_half_n_nz_times_n_nz_plus_one_symmetric() {
    // mixed equal-magnitude diffs → W+ = W- = n*(n+1)/4
    // before=[1,3], after=[3,1] → diffs=[2,-2], |diffs|=[2,2]
    // ranks=[1.5,1.5], W+=1.5, W-=1.5
    let (w_plus, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 3.0],
            after: vec![3.0, 1.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(w_plus, 1.5);
    assert!((p_value - 1.0).abs() < 1e-6);
    assert!(!reject_h0);
}

#[test]
fn wilcoxon_p_value_from_z_is_reasonable() {
    // before=[1,2,3,4,5], after=[2,3,4,5,6]: W+=15, mu=7.5, sigma=sqrt(330/24)≈3.708
    // z=7.5/3.708≈2.023, p≈0.0431; sigma formula mutations give p in 0.025..0.077
    let (_, p_value, _, _, _, _, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            after: vec![2.0, 3.0, 4.0, 5.0, 6.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert!(p_value > 0.038 && p_value < 0.050);
}

#[test]
fn wilcoxon_dof_is_zero() {
    let (_, _, dof, _, _, _, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 2.0],
            after: vec![3.0, 4.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(dof, 0.0);
}

#[test]
fn wilcoxon_reject_h0_strict_lt() {
    // symmetric diffs → p=1.0, alpha=1.0 → p < alpha is false
    let (_, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 3.0],
            after: vec![3.0, 1.0],
            alpha: 1.0,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(p_value, 1.0);
    assert!(!reject_h0);
}

#[test]
fn wilcoxon_rejects_mismatched_lengths() {
    match (StatsRequest::WilcoxonSigned {
        before: vec![1.0, 2.0],
        after: vec![3.0],
        alpha: 0.05,
        tail: Tail::Two,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn wilcoxon_rejects_all_zero_diffs() {
    match (StatsRequest::WilcoxonSigned {
        before: vec![1.0, 2.0],
        after: vec![1.0, 2.0],
        alpha: 0.05,
        tail: Tail::Two,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn wilcoxon_rejects_nonfinite_after_value() {
    // ||→&& mutation: only after has infinity; && would require both to fail
    match (StatsRequest::WilcoxonSigned {
        before: vec![1.0, 2.0],
        after: vec![f64::INFINITY, 2.0],
        alpha: 0.05,
        tail: Tail::Two,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn wilcoxon_default_alpha_via_json() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"wilcoxon_signed","before":[1,2,3],"after":[2,3,4]}"#)
            .unwrap();
    let (_, _, _, _, alpha, _, _) = hypothesis(req.evaluate());
    assert!((alpha - 0.05).abs() < 1e-12);
}

#[test]
fn wilcoxon_asymmetric_diffs_w_plus_exact() {
    // before=[1,2,3], after=[4,3,2] → diffs=[3,1,-1]
    // nonzero all 3; |diffs|=[3,1,1]; sorted=[1,1,3]
    // ranks of [3,1,1]: 3→rank3, 1→rank1.5, 1→rank1.5
    // Original order: [3,1,-1] → abs=[3,1,1] → ranks for those = [3.0, 1.5, 1.5]
    // W+ = rank of diff>0: diffs[0]=3>0→r=3.0, diffs[1]=1>0→r=1.5; W+=3.0+1.5=4.5
    let (w_plus, _, _, _, _, _, _) = hypothesis(
        StatsRequest::WilcoxonSigned {
            before: vec![1.0, 2.0, 3.0],
            after: vec![4.0, 3.0, 2.0],
            alpha: 0.05,
            tail: Tail::Two,
        }
        .evaluate(),
    );
    assert_eq!(w_plus, 4.5);
}

// ── Kruskal-Wallis ────────────────────────────────────────────────────────────

#[test]
fn kw_h_statistic_exact_three_groups() {
    // groups=[[1,2],[3,4],[5,6]], pool [1..6], ranks [1..6]
    // R1=3,R2=7,R3=11; H=(12/42)*(9/2+49/2+121/2)-21=179/7-21=32/7≈4.5714
    let (h, _, _, _, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!((h - 32.0 / 7.0).abs() < 1e-10);
}

#[test]
fn kw_h_near_zero_for_identical_groups() {
    // all groups identical → ranks evenly split → H≈0
    let (h, _, _, _, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![1.0, 2.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!(h.abs() < 1e-6);
}

#[test]
fn kw_dof_is_k_minus_one() {
    // 3 groups → dof=2
    let (_, _, dof, _, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert_eq!(dof, 2.0);
}

#[test]
fn kw_critical_value_chi2_not_unit() {
    // chi2(0.95, 2) ≈ 5.991
    let (_, _, _, critical_value, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!((critical_value - 5.991).abs() < 0.01);
}

#[test]
fn kw_p_value_reasonable_for_separated_groups() {
    // H≈4.57, dof=2; P(chi2>4.57)≈0.10
    let (_, p_value, _, _, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!(p_value > 0.05 && p_value < 0.20);
}

#[test]
fn kw_reject_h0_strict_lt() {
    // H≈0 for identical groups → p≈1.0, alpha=1.0 → strict < → no reject
    let (_, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![1.0, 2.0]],
            alpha: 1.0,
        }
        .evaluate(),
    );
    assert!(p_value > 0.99);
    assert!(!reject_h0);
}

#[test]
fn kw_conclusion_contains_reject_or_fail() {
    let (_, _, _, _, _, reject_h0, conclusion) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    if reject_h0 {
        assert!(conclusion.contains("Reject"));
    } else {
        assert!(conclusion.contains("Fail"));
    }
}

#[test]
fn kw_rejects_single_group() {
    match (StatsRequest::KruskalWallis {
        groups: vec![vec![1.0, 2.0]],
        alpha: 0.05,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kw_rejects_group_with_one_element() {
    match (StatsRequest::KruskalWallis {
        groups: vec![vec![1.0], vec![2.0, 3.0]],
        alpha: 0.05,
    })
    .evaluate()
    {
        StatsResponse::Error { .. } => {}
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn kw_default_alpha_via_json() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"kruskal_wallis","groups":[[1,2],[3,4]]}"#).unwrap();
    let (_, _, _, _, alpha, _, _) = hypothesis(req.evaluate());
    assert!((alpha - 0.05).abs() < 1e-12);
}

#[test]
fn kw_rejects_h0_for_well_separated_groups() {
    // 7 groups of 2, perfectly separated: H=448/35=12.8, dof=6
    // chi2(0.95,6)≈12.592; H>cv so p≈0.046<0.05 → reject_h0=true
    // Mutation < → >: p>alpha=false → reject_h0=false (caught)
    let (h, p_value, _, _, _, reject_h0, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
                vec![5.0, 6.0],
                vec![7.0, 8.0],
                vec![9.0, 10.0],
                vec![11.0, 12.0],
                vec![13.0, 14.0],
            ],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!((h - 448.0 / 35.0).abs() < 1e-10);
    assert!(p_value < 0.05);
    assert!(reject_h0);
}

#[test]
fn kw_h_nonzero_denominator_not_unit() {
    // groups=[[1,3],[5,7],[9,11]]: R1=1+2=3? Let me recompute:
    // pool=[1,3,5,7,9,11] all distinct, ranks [1,2,3,4,5,6]
    // R1=1+2=3, n1=2; R2=3+4=7, n2=2; R3=5+6=11, n3=2
    // h_sum = 9/2 + 49/2 + 121/2 = 179/2
    // H = (12/42)*(179/2) - 21 = 32/7 same as before
    // Use groups of size 3 to get different n:
    // groups=[[1,2,3],[4,5,6]]: R1=1+2+3=6,n1=3; R2=4+5+6=15,n2=3
    // h_sum=36/3+225/3=12+75=87
    // H=(12/42)*87-21=1044/42-21=174/7-21=174/7-147/7=27/7≈3.857
    let (h, _, _, _, _, _, _) = hypothesis(
        StatsRequest::KruskalWallis {
            groups: vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            alpha: 0.05,
        }
        .evaluate(),
    );
    assert!((h - 27.0 / 7.0).abs() < 1e-10);
}
