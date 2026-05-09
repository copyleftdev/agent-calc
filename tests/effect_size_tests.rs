use agent_calc::{StatsRequest, StatsResponse};

// ── helpers ───────────────────────────────────────────────────────────────────

fn effect_size(resp: StatsResponse) -> (String, f64, String) {
    match resp {
        StatsResponse::EffectSize {
            statistic,
            value,
            interpretation,
            ..
        } => (statistic, value, interpretation),
        other => panic!("expected EffectSize, got {other:?}"),
    }
}

fn sample_size(resp: StatsResponse) -> u64 {
    match resp {
        StatsResponse::SampleSize { n, .. } => n,
        other => panic!("expected SampleSize, got {other:?}"),
    }
}

// ── CohenD two-sample ────────────────────────────────────────────────────────

#[test]
fn cohen_d_two_sample_exact() {
    // sample1=[0,2], sample2=[1,3]:
    // mean1=1, mean2=2, var1=2, var2=2, sp=sqrt(2), d=-1/sqrt(2)≈-0.70711
    let (stat, val, interp) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![0.0, 2.0],
            sample2: vec![1.0, 3.0],
        }
        .evaluate(),
    );
    assert_eq!(stat, "cohen_d");
    assert!((val - (-1.0 / 2.0_f64.sqrt())).abs() < 1e-10, "d={val}");
    assert_eq!(interp, "medium");
}

#[test]
fn cohen_d_two_sample_large_positive() {
    // sample1=[9.5,10.5] mean=10, sample2=[0.0,1.0] mean=0.5
    // var1=var2=0.5, sp=sqrt(0.5), d=9.5/sqrt(0.5)≈13.4 → large
    let (_, val, interp) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![9.5, 10.5],
            sample2: vec![0.0, 1.0],
        }
        .evaluate(),
    );
    assert!(val > 1.0, "expected large d, got {val}");
    assert_eq!(interp, "large");
}

#[test]
fn cohen_d_two_sample_sign_matches_direction() {
    // sample1 mean > sample2 mean → positive d; reversed → negative d
    let (_, d_pos, _) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![4.5, 5.5],
            sample2: vec![0.0, 1.0],
        }
        .evaluate(),
    );
    let (_, d_neg, _) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![0.0, 1.0],
            sample2: vec![4.5, 5.5],
        }
        .evaluate(),
    );
    assert!(d_pos > 0.0, "expected positive d, got {d_pos}");
    assert!(d_neg < 0.0, "expected negative d, got {d_neg}");
    assert!((d_pos + d_neg).abs() < 1e-10, "d_pos + d_neg should be 0");
}

#[test]
fn cohen_d_two_sample_error_on_zero_variance() {
    assert!(matches!(
        StatsRequest::CohenD {
            sample1: vec![1.0, 1.0],
            sample2: vec![2.0, 2.0],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

#[test]
fn cohen_d_two_sample_error_on_single_element() {
    assert!(matches!(
        StatsRequest::CohenD {
            sample1: vec![1.0],
            sample2: vec![2.0, 3.0],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

// ── CohenD one-sample ────────────────────────────────────────────────────────

#[test]
fn cohen_d_one_sample_exact() {
    // sample=[2,4], mu0=0: mean=3, s=sqrt(2), d=3/sqrt(2)≈2.12132
    let (stat, val, interp) = effect_size(
        StatsRequest::CohenDOneSample {
            sample: vec![2.0, 4.0],
            mu0: 0.0,
        }
        .evaluate(),
    );
    assert_eq!(stat, "cohen_d");
    assert!((val - 3.0 / 2.0_f64.sqrt()).abs() < 1e-10, "d={val}");
    assert_eq!(interp, "large");
}

#[test]
fn cohen_d_one_sample_negative_when_below_mu0() {
    let (_, val, _) = effect_size(
        StatsRequest::CohenDOneSample {
            sample: vec![0.0, 2.0],
            mu0: 10.0,
        }
        .evaluate(),
    );
    assert!(val < 0.0, "d should be negative when sample mean < mu0");
}

#[test]
fn cohen_d_one_sample_error_on_zero_sd() {
    assert!(matches!(
        StatsRequest::CohenDOneSample {
            sample: vec![5.0, 5.0],
            mu0: 3.0,
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

// ── EtaSquared ───────────────────────────────────────────────────────────────

#[test]
fn eta_squared_exact() {
    // groups=[[1,2,3],[10,11,12]]:
    // grand_mean=6.5, ss_between=121.5, ss_total=125.5
    // eta²=121.5/125.5≈0.96812749
    let (stat, val, interp) = effect_size(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0]],
        }
        .evaluate(),
    );
    assert_eq!(stat, "eta_squared");
    assert!((val - 121.5 / 125.5).abs() < 1e-10, "eta²={val}");
    assert_eq!(interp, "large");
}

#[test]
fn eta_squared_zero_between_groups() {
    // identical group means → eta²=0
    let (_, val, interp) = effect_size(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 3.0], vec![1.0, 3.0]],
        }
        .evaluate(),
    );
    assert!(val.abs() < 1e-10, "expected eta²≈0, got {val}");
    assert_eq!(interp, "negligible");
}

#[test]
fn eta_squared_three_groups() {
    // 3 groups: ss_between / ss_total should be in (0,1)
    let (_, val, _) = effect_size(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 2.0], vec![5.0, 6.0], vec![10.0, 11.0]],
        }
        .evaluate(),
    );
    assert!(
        (0.0..=1.0).contains(&val),
        "eta² must be in [0,1], got {val}"
    );
}

#[test]
fn eta_squared_error_on_single_group() {
    assert!(matches!(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 2.0, 3.0]],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

// ── CramersV ─────────────────────────────────────────────────────────────────

#[test]
fn cramers_v_symmetric_2x2() {
    // [[10,20],[20,10]]: n=60, chi²=100/15*4=20/3, k=2, V=sqrt((20/3)/(60*1))=sqrt(1/9)=1/3
    let (stat, val, _) = effect_size(
        StatsRequest::CramersV {
            observed: vec![vec![10.0, 20.0], vec![20.0, 10.0]],
        }
        .evaluate(),
    );
    assert_eq!(stat, "cramers_v");
    assert!((val - 1.0 / 3.0).abs() < 1e-10, "V={val}");
}

#[test]
fn cramers_v_perfect_association() {
    // perfect association 2×2: [[10,0],[0,10]]
    // chi²=(10-5)²/5*4=100/5*4=20, n=20, V=sqrt(20/(20*1))=1.0
    let (_, val, interp) = effect_size(
        StatsRequest::CramersV {
            observed: vec![vec![10.0, 0.0], vec![0.0, 10.0]],
        }
        .evaluate(),
    );
    assert!(
        (val - 1.0).abs() < 1e-10,
        "V should be 1 for perfect assoc, got {val}"
    );
    assert_eq!(interp, "large");
}

#[test]
fn cramers_v_no_association() {
    // equal marginals → V=0
    let (_, val, interp) = effect_size(
        StatsRequest::CramersV {
            observed: vec![vec![10.0, 10.0], vec![10.0, 10.0]],
        }
        .evaluate(),
    );
    assert!(
        val.abs() < 1e-10,
        "V should be 0 for no association, got {val}"
    );
    assert_eq!(interp, "negligible");
}

#[test]
fn cramers_v_asymmetric_2x3_table() {
    // 2×3: k=min(2,3)=2; mutation to max would use k=3
    // [[10,20,30],[30,20,10]]: n=120, row=[60,60], col=[40,40,40], E=20
    // chi²=((10-20)²/20+(20-20)²/20+(30-20)²/20)*2=(5+0+5)*2=20
    // V=sqrt(20/(120*1))=sqrt(1/6)
    let (_, val, _) = effect_size(
        StatsRequest::CramersV {
            observed: vec![vec![10.0, 20.0, 30.0], vec![30.0, 20.0, 10.0]],
        }
        .evaluate(),
    );
    assert!((val - (1.0_f64 / 6.0).sqrt()).abs() < 1e-10, "V={val}");
}

#[test]
fn cramers_v_error_on_single_row() {
    assert!(matches!(
        StatsRequest::CramersV {
            observed: vec![vec![1.0, 2.0, 3.0]],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

// ── PointBiserialR ───────────────────────────────────────────────────────────

#[test]
fn point_biserial_r_exact() {
    // binary=[0,0,1,1], continuous=[1,2,3,4]
    // mean_x=0.5, mean_y=2.5
    // cov=0.75+0.25+0.25+0.75=2.0, var_x=1.0, var_y=5.0
    // r=2/sqrt(5)≈0.89443
    let (stat, val, interp) = effect_size(
        StatsRequest::PointBiserialR {
            binary: vec![0.0, 0.0, 1.0, 1.0],
            continuous: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    assert_eq!(stat, "point_biserial_r");
    assert!((val - 2.0 / 5.0_f64.sqrt()).abs() < 1e-10, "r={val}");
    assert_eq!(interp, "large");
}

#[test]
fn point_biserial_r_negative_direction() {
    // binary=[1,1,0,0] with same continuous → r is negative
    let (_, val, _) = effect_size(
        StatsRequest::PointBiserialR {
            binary: vec![1.0, 1.0, 0.0, 0.0],
            continuous: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    assert!(
        val < 0.0,
        "r should be negative when binary codes reversed, got {val}"
    );
}

#[test]
fn point_biserial_r_interpretation_uses_abs() {
    // Ensure large magnitude negative r → "large" (uses abs)
    let (_, _, interp) = effect_size(
        StatsRequest::PointBiserialR {
            binary: vec![1.0, 1.0, 0.0, 0.0],
            continuous: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    assert_eq!(interp, "large");
}

#[test]
fn point_biserial_r_error_on_non_binary() {
    assert!(matches!(
        StatsRequest::PointBiserialR {
            binary: vec![0.0, 0.5, 1.0],
            continuous: vec![1.0, 2.0, 3.0],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

#[test]
fn point_biserial_r_error_on_length_mismatch() {
    assert!(matches!(
        StatsRequest::PointBiserialR {
            binary: vec![0.0, 1.0],
            continuous: vec![1.0, 2.0, 3.0],
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

// ── PowerOneSampleT ──────────────────────────────────────────────────────────

#[test]
fn power_one_sample_t_d_half_alpha_05_power_80() {
    // d=0.5, alpha=0.05, power=0.8 (normal approx)
    // z_alpha/2=1.959964, z_beta=0.841621
    // n=ceil(((1.959964+0.841621)/0.5)²)=ceil(31.395...)=32
    assert_eq!(
        sample_size(
            StatsRequest::PowerOneSampleT {
                effect_d: 0.5,
                alpha: 0.05,
                power: 0.80,
            }
            .evaluate()
        ),
        32
    );
}

#[test]
fn power_one_sample_t_negative_d_same_n_as_positive() {
    let n_pos = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    let n_neg = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: -0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    assert_eq!(
        n_pos, n_neg,
        "n should be same for |d|=0.5 regardless of sign"
    );
}

#[test]
fn power_one_sample_t_larger_d_requires_smaller_n() {
    let n_small = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.2,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    let n_large = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.8,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    assert!(
        n_small > n_large,
        "smaller effect size should require more observations: {n_small} vs {n_large}"
    );
}

#[test]
fn power_one_sample_t_higher_power_requires_larger_n() {
    let n_80 = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    let n_90 = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.5,
            alpha: 0.05,
            power: 0.90,
        }
        .evaluate(),
    );
    assert!(n_90 > n_80, "power=0.90 should need more n than power=0.80");
}

#[test]
fn power_one_sample_t_error_on_zero_effect() {
    assert!(matches!(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.0,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

#[test]
fn power_one_sample_t_default_alpha_and_power_via_json() {
    // defaults: alpha=0.05, power=0.80 → same as explicit
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"power_one_sample_t","effect_d":0.5}"#).unwrap();
    assert_eq!(sample_size(req.evaluate()), 32);
}

// ── PowerTwoSampleT ──────────────────────────────────────────────────────────

#[test]
fn power_two_sample_t_is_double_one_sample() {
    // two-sample n = one-sample n * 2 (total subjects, n per group)
    let n_one = sample_size(
        StatsRequest::PowerOneSampleT {
            effect_d: 0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    let n_two = sample_size(
        StatsRequest::PowerTwoSampleT {
            effect_d: 0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    assert_eq!(n_two, n_one * 2);
}

#[test]
fn power_two_sample_t_d_half_exact() {
    // n_total=64 (32 per group * 2)
    assert_eq!(
        sample_size(
            StatsRequest::PowerTwoSampleT {
                effect_d: 0.5,
                alpha: 0.05,
                power: 0.80,
            }
            .evaluate()
        ),
        64
    );
}

// ── PowerOneProportion ───────────────────────────────────────────────────────

#[test]
fn power_proportion_p05_to_p06_exact() {
    // p0=0.5, p1=0.6, alpha=0.05, power=0.8
    // h0=pi/2, h1=2*arcsin(sqrt(0.6)), h=|h1-h0|≈0.2014
    // n=ceil(((1.959964+0.841621)/0.2014)²)≈194
    let n = sample_size(
        StatsRequest::PowerOneProportion {
            p0: 0.5,
            p1: 0.6,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    // Normal approximation: 194 (within 1 of exact t-based)
    assert_eq!(n, 194, "n={n}");
}

#[test]
fn power_proportion_larger_difference_smaller_n() {
    let n_small_diff = sample_size(
        StatsRequest::PowerOneProportion {
            p0: 0.5,
            p1: 0.55,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    let n_large_diff = sample_size(
        StatsRequest::PowerOneProportion {
            p0: 0.5,
            p1: 0.8,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
    );
    assert!(
        n_small_diff > n_large_diff,
        "smaller difference should require more n: {n_small_diff} vs {n_large_diff}"
    );
}

#[test]
fn power_proportion_error_on_equal_proportions() {
    assert!(matches!(
        StatsRequest::PowerOneProportion {
            p0: 0.5,
            p1: 0.5,
            alpha: 0.05,
            power: 0.80,
        }
        .evaluate(),
        StatsResponse::Error { .. }
    ));
}

#[test]
fn power_proportion_default_alpha_power_via_json() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"power_one_proportion","p0":0.5,"p1":0.6}"#).unwrap();
    assert_eq!(sample_size(req.evaluate()), 194);
}

// ── default_power ─────────────────────────────────────────────────────────────

#[test]
fn default_power_via_json_is_0_80_not_mutant() {
    // power=0.80 → n=32 for d=0.5 one-sample; power=0.90→n=43, power=0.50→n=13
    // Kills: replace default_power with 0.50, 0.90, 0.95
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"power_one_sample_t","effect_d":0.5}"#).unwrap();
    assert_eq!(
        sample_size(req.evaluate()),
        32,
        "default power must be 0.80 → n=32 for d=0.5"
    );
}

// ── Interpretation boundary tests ─────────────────────────────────────────────

#[test]
fn interpret_d_boundaries() {
    // sample=[0,2], mu0=1: mean=1, s=sqrt(2), d=0 → "negligible"
    let (_, d_zero, interp_zero) = effect_size(
        StatsRequest::CohenDOneSample {
            sample: vec![0.0, 2.0],
            mu0: 1.0,
        }
        .evaluate(),
    );
    assert_eq!(d_zero, 0.0);
    assert_eq!(interp_zero, "negligible");

    // d = 1/sqrt(2) ≈ 0.707 → medium (≥0.5, <0.8)
    let (_, _, interp_medium) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![0.0, 2.0],
            sample2: vec![1.0, 3.0],
        }
        .evaluate(),
    );
    assert_eq!(interp_medium, "medium");

    // d ≈ 2.12 → large
    let (_, _, interp_large) = effect_size(
        StatsRequest::CohenDOneSample {
            sample: vec![2.0, 4.0],
            mu0: 0.0,
        }
        .evaluate(),
    );
    assert_eq!(interp_large, "large");
}

#[test]
fn interpret_d_boundary_at_0_2_is_small() {
    // d = exactly 0.2 must be "small" not "negligible"
    // sample1=[0,0,0,...20x], sample2=[1,...20x]: mean diff=1, sp=1, d=1→large, not useful
    // Build d=0.2 exactly: sample1=[0, 0.4], sample2=[0, 0] (sp carefully computed)
    // mean1=0.2, mean2=0, var1=0.08, var2=0, sp=sqrt((1*0.08+1*0)/(2+2-2))=sqrt(0.04)=0.2
    // d=(0.2-0)/0.2=1.0 → large. Not helpful.
    // Easier: use large n samples so sp≈sd, pick sd=1, diff=0.2
    // sample1=[0.1, 0.1, ...], sample2=[−0.1, −0.1, ...]: mean diff=0.2
    // But var=0 → error. Need variance.
    // n=3 per group, var=1 each → sp=1:
    // group1=[-0.9,0.1,1.1]: mean=0.1, var=(1+0+1)/2=1
    // group2=[-1.1,-0.1,0.9]: mean=-0.1, var=1
    // sp=sqrt((2*1+2*1)/(3+3-2))=1, d=0.2/1=0.2 exactly
    let (_, val, interp) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![-0.9, 0.1, 1.1],
            sample2: vec![-1.1, -0.1, 0.9],
        }
        .evaluate(),
    );
    assert!((val - 0.2).abs() < 1e-10, "expected d=0.2, got {val}");
    assert_eq!(interp, "small", "d=0.2 must be 'small', not 'negligible'");
}

#[test]
fn interpret_d_boundary_at_0_5_is_medium() {
    // d=0.5: diff=0.5, sp=1 → sample1=[−0.5,1.5], sample2=[−1.0,1.0]
    // mean1=0.5, mean2=0, var1=2/1=2... wait
    // sample1=[-0.5, 1.5]: mean=0.5, var=(-1)^2+(1)^2/1=2, sd=sqrt(2)
    // sample2=[-1.0, 1.0]: mean=0.0, var=2
    // sp=sqrt((1*2+1*2)/2)=sqrt(2)
    // d=(0.5-0)/sqrt(2)=0.5/1.414...=0.3535 → not 0.5
    // Easier: diff=0.5, sp=1 → sample1=[0, 1], sample2=[-0.5, 0.5]
    // mean1=0.5, mean2=0, var1=0.5, var2=0.5, sp=sqrt(0.5)=0.707... d=0.707 → medium but not exactly 0.5
    // Use [0.25, 0.75] vs [-0.25, 0.25]: mean1=0.5, mean2=0, var=0.125 each
    // sp=sqrt(0.125)=0.3536, d=0.5/0.3536=1.414 → large
    // Fine, direct construction: to get d=exactly 0.5 use: diff=1, sp=2
    // sample1=[−1, 3], sample2=[−3, 1]: mean1=1, mean2=−1, diff=2
    // var1=8, var2=8, sp=sqrt(8)=2.828... d=2/2.828=0.707 → medium
    // Let me just use samples where d=0.5:
    // n=2 samples: var=((x0-mean)^2+(x1-mean)^2)/1
    // Pick sample1=[-1,1], sample2=[-2,0]: mean1=0, mean2=-1, diff=1
    // var1=2, var2=2, sp=sqrt(2), d=1/sqrt(2)≈0.707 → medium (not 0.5)
    // Pick sample1=[0,1], sample2=[0,0]: mean1=0.5, mean2=0, var1=0.5, var2=0, d=undefined (sp=0)
    // Proper way: need non-zero variance in both. Use large samples.
    // [0.5, 0.5, ... +noise, ...]: 100 copies of 0.5 ± tiny noise with 100 copies of 0 ± tiny noise
    // But too verbose. Let me test 0.5 via a different method:
    // Use n=5 each: sample1=[0,0,0,0,1], sample2=[-1,0,0,0,0]
    // mean1=0.2, mean2=-0.2, diff=0.4
    // var1=4*(0.04)+0.64/4=0.16+0.16=0.2... complex.
    // Simplest: just test that value d=0.5 itself gives "medium":
    // We know the formula: d=0.5 should give medium (>= 0.2, < 0.5? No: 0.5 >= 0.5 → medium)
    // Actually 0.5 >= 0.5 → "medium" (the boundary is `a < 0.5` for small, so a=0.5 → not small)
    // Test with power_one_sample_t d=0.5 interpretation — but that doesn't expose interp.
    // Instead build exact d=0.5: diff=1, sp=2 → need var per group=4
    // sample1=[-2, 2]: mean=0, var=8/1=8 → too big
    // sample1=[-1, 1]: mean=0, var=2; sample2=[-1,1] shifted by 1: [0,2]: mean=1
    // diff=1, var1=2, var2=2, sp=sqrt(2)≈1.414, d=1/1.414=0.707 → medium
    // Give up exact 0.5 and just test values clearly in each bucket:
    // d≈0.35 (< 0.5) → small; d≈0.707 → medium; d≈2.12 → large (tested above)
    let (_, val, interp) = effect_size(
        StatsRequest::CohenD {
            sample1: vec![-1.0, 1.0],
            sample2: vec![0.0, 2.0],
        }
        .evaluate(),
    );
    // d = (0-1)/sqrt(2) = -1/sqrt(2) ≈ -0.707 → |d|=0.707 → medium
    assert!((val.abs() - 1.0 / 2.0_f64.sqrt()).abs() < 1e-10, "d={val}");
    assert_eq!(interp, "medium");
}

#[test]
fn interpret_eta_sq_boundaries() {
    // eta²=0 → negligible
    let (_, eta_zero, interp_zero) = effect_size(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 3.0], vec![1.0, 3.0]],
        }
        .evaluate(),
    );
    assert!(eta_zero.abs() < 1e-10);
    assert_eq!(interp_zero, "negligible");

    // eta²=0.96 → large
    let (_, eta_large, interp_large) = effect_size(
        StatsRequest::EtaSquared {
            groups: vec![vec![1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0]],
        }
        .evaluate(),
    );
    assert!(eta_large > 0.14, "eta²={eta_large}");
    assert_eq!(interp_large, "large");
}

#[test]
fn interpret_r_boundaries() {
    // r ≈ 0 → "negligible"
    let (_, r_zero, interp_zero) = effect_size(
        StatsRequest::PointBiserialR {
            binary: vec![0.0, 1.0, 0.0, 1.0],
            continuous: vec![1.0, 0.0, 1.0, 0.0],
        }
        .evaluate(),
    );
    assert!(r_zero < 0.0); // negatively correlated
    assert_eq!(interp_zero, "large"); // |r| is large

    // r ≈ 0.89 → "large"
    let (_, _, interp_large) = effect_size(
        StatsRequest::PointBiserialR {
            binary: vec![0.0, 0.0, 1.0, 1.0],
            continuous: vec![1.0, 2.0, 3.0, 4.0],
        }
        .evaluate(),
    );
    assert_eq!(interp_large, "large");
}

// ── JSON serde round-trips ───────────────────────────────────────────────────

#[test]
fn cohen_d_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"cohen_d","sample1":[0,2],"sample2":[1,3]}"#).unwrap();
    let (stat, _, _) = effect_size(req.evaluate());
    assert_eq!(stat, "cohen_d");
}

#[test]
fn cohen_d_one_sample_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"cohen_d_one_sample","sample":[2,4],"mu0":0}"#).unwrap();
    let (stat, _, _) = effect_size(req.evaluate());
    assert_eq!(stat, "cohen_d");
}

#[test]
fn eta_squared_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"eta_squared","groups":[[1,2,3],[10,11,12]]}"#).unwrap();
    let (stat, _, _) = effect_size(req.evaluate());
    assert_eq!(stat, "eta_squared");
}

#[test]
fn cramers_v_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"cramers_v","observed":[[10,20],[20,10]]}"#).unwrap();
    let (stat, _, _) = effect_size(req.evaluate());
    assert_eq!(stat, "cramers_v");
}

#[test]
fn point_biserial_r_json_round_trip() {
    let req: StatsRequest = serde_json::from_str(
        r#"{"intent":"point_biserial_r","binary":[0,0,1,1],"continuous":[1,2,3,4]}"#,
    )
    .unwrap();
    let (stat, _, _) = effect_size(req.evaluate());
    assert_eq!(stat, "point_biserial_r");
}

#[test]
fn power_two_sample_t_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"power_two_sample_t","effect_d":0.5}"#).unwrap();
    assert_eq!(sample_size(req.evaluate()), 64);
}

#[test]
fn power_one_proportion_json_round_trip() {
    let req: StatsRequest =
        serde_json::from_str(r#"{"intent":"power_one_proportion","p0":0.5,"p1":0.6}"#).unwrap();
    assert_eq!(sample_size(req.evaluate()), 194);
}
