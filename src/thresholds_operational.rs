use crate::metrics::{f_beta, threshold_sweep, ThresholdPoint};
use anyhow::Result;
use rusqlite::Connection;

#[allow(dead_code)]
pub struct OperationalThreshold {
    pub rule: String,
    pub cutoff: f64,
    pub sensitivity: f64,
    pub specificity: f64,
    pub precision: f64,
    pub f1: f64,
    pub f1_5: f64,
    pub f2: f64,
    pub f3: f64,
    pub fn_count: usize,
    pub fp_count: usize,
}

fn point_to_threshold(p: &ThresholdPoint, rule: &str) -> OperationalThreshold {
    OperationalThreshold {
        rule: rule.into(),
        cutoff: p.cutoff,
        sensitivity: p.sensitivity,
        specificity: p.specificity,
        precision: p.precision,
        f1: p.f1,
        f1_5: p.f1_5,
        f2: p.f2,
        f3: p.f3,
        fn_count: p.fn_count,
        fp_count: p.fp_count,
    }
}

/// Select operational thresholds under the standard selection rules.
///
/// Rules:
/// - max_F1, max_F1_5, max_F2, max_F3: maximise the F-beta score for beta in
///   {1, 1.5, 2, 3} (F-beta is an a priori recall-weighting choice, not an
///   optimised cost ratio);
/// - sens_ge_0.90: maximise specificity subject to sensitivity >= 0.90
///   (fallback: best sensitivity available);
/// - youden_index: maximise sensitivity + specificity - 1.
pub fn select_operational_thresholds(y_true: &[i32], y_prob: &[f64]) -> Vec<OperationalThreshold> {
    let sweep = threshold_sweep(y_true, y_prob);
    let mut result = Vec::new();

    for (beta, rule) in [
        (1.0, "max_F1"),
        (1.5, "max_F1_5"),
        (2.0, "max_F2"),
        (3.0, "max_F3"),
    ] {
        let score = |p: &ThresholdPoint| f_beta(p.precision, p.sensitivity, beta);
        if let Some(best) = sweep
            .iter()
            .max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap())
        {
            result.push(point_to_threshold(best, rule));
        }
    }

    // Sensitivity ≥ 0.90 with best specificity
    if let Some(best) = sweep
        .iter()
        .filter(|p| p.sensitivity >= 0.90)
        .max_by(|a, b| a.specificity.partial_cmp(&b.specificity).unwrap())
    {
        result.push(point_to_threshold(best, "sens_ge_0.90"));
    } else {
        // Fallback: best sensitivity achieved
        if let Some(best) = sweep
            .iter()
            .max_by(|a, b| a.sensitivity.partial_cmp(&b.sensitivity).unwrap())
        {
            result.push(point_to_threshold(best, "best_sensitivity_available"));
        }
    }

    // Youden Index (sensitivity + specificity - 1 maximised)
    if let Some(best) = sweep.iter().max_by(|a, b| {
        let ya = a.sensitivity + a.specificity - 1.0;
        let yb = b.sensitivity + b.specificity - 1.0;
        ya.partial_cmp(&yb).unwrap()
    }) {
        result.push(point_to_threshold(best, "youden_index"));
    }

    result
}

/// Select the cutoff that maximises specificity subject to sensitivity >= 0.90,
/// falling back to the best available sensitivity. Used for the fully nested
/// operational-threshold design (cutoff chosen on inner-CV validation
/// predictions only, then frozen and applied to outer held-out test data).
pub fn select_nested_cutoff(y_inner: &[i32], p_inner: &[f64]) -> (String, f64) {
    let sweep = threshold_sweep(y_inner, p_inner);
    if let Some(best) = sweep
        .iter()
        .filter(|p| p.sensitivity >= 0.90)
        .max_by(|a, b| a.specificity.partial_cmp(&b.specificity).unwrap())
    {
        ("sens_ge_0.90".to_string(), best.cutoff)
    } else if let Some(best) = sweep
        .iter()
        .max_by(|a, b| a.sensitivity.partial_cmp(&b.sensitivity).unwrap())
    {
        ("best_sensitivity_available".to_string(), best.cutoff)
    } else {
        ("none".to_string(), 0.5)
    }
}

pub fn store_operational_thresholds(
    conn: &Connection,
    run_id: &str,
    cv_mode: &str,
    thresholds: &[OperationalThreshold],
) -> Result<()> {
    for t in thresholds {
        conn.execute(
            "INSERT INTO operational_thresholds
             (run_id, cv_mode, threshold_selection_rule, optimized_probability_cutoff,
              resulting_sensitivity, resulting_specificity, false_negatives_count, false_positives_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                run_id,
                cv_mode,
                t.rule,
                t.cutoff,
                t.sensitivity,
                t.specificity,
                t.fn_count as i64,
                t.fp_count as i64,
            ],
        )?;
    }
    Ok(())
}

pub fn store_threshold_sweep(
    conn: &Connection,
    run_id: &str,
    cv_mode: &str,
    y_true: &[i32],
    y_prob: &[f64],
) -> Result<()> {
    let sweep = threshold_sweep(y_true, y_prob);
    for p in &sweep {
        conn.execute(
            "INSERT INTO threshold_sensitivity
             (run_id, cv_mode, probability_cutoff, sensitivity, specificity,
              precision, f1_score, f1_5_score, f2_score, f3_score,
              false_negatives_count, false_positives_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                run_id,
                cv_mode,
                p.cutoff,
                p.sensitivity,
                p.specificity,
                p.precision,
                p.f1,
                p.f1_5,
                p.f2,
                p.f3,
                p.fn_count as i64,
                p.fp_count as i64,
            ],
        )?;
    }
    Ok(())
}

/// Store the fully nested per-fold operating result: cutoff selected on
/// inner-CV validation predictions, metrics computed on the outer test fold.
#[allow(clippy::too_many_arguments)]
pub fn store_nested_threshold(
    conn: &Connection,
    run_id: &str,
    cv_mode: &str,
    repeat: usize,
    fold: usize,
    rule: &str,
    cutoff: f64,
    y_test: &[i32],
    p_test: &[f64],
) -> Result<()> {
    let n_pos = y_test.iter().filter(|&&v| v == 1).count();
    let n_neg = y_test.len() - n_pos;
    let sweep = threshold_sweep(y_test, p_test);
    // Operating point at the frozen cutoff (nearest grid point below or equal).
    let op = sweep
        .iter()
        .filter(|p| p.cutoff <= cutoff + 1e-9)
        .next_back()
        .or_else(|| sweep.first());
    let (sens, spec, prec, f1, f1_5, f2, f3, fn_, fp_) = match op {
        Some(p) => (
            p.sensitivity,
            p.specificity,
            p.precision,
            p.f1,
            p.f1_5,
            p.f2,
            p.f3,
            p.fn_count,
            p.fp_count,
        ),
        None => (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0usize, 0usize),
    };
    conn.execute(
        "INSERT INTO nested_operational_thresholds
         (run_id, cv_mode, repeat_index, fold_index, threshold_selection_rule,
          cutoff_selection_source, optimized_probability_cutoff,
          sensitivity, specificity, precision, f1_score, f1_5_score, f2_score, f3_score,
          false_negatives_count, false_positives_count, n_test_pos, n_test_neg)
         VALUES (?1, ?2, ?3, ?4, ?5, 'inner_cv_validation', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            run_id,
            cv_mode,
            repeat as i64,
            fold as i64,
            rule,
            cutoff,
            sens,
            spec,
            prec,
            f1,
            f1_5,
            f2,
            f3,
            fn_ as i64,
            fp_ as i64,
            n_pos as i64,
            n_neg as i64,
        ],
    )?;
    Ok(())
}

/// Compute screening priority class from predicted probability.
///
/// Pre-specified probability bands (reproducible, documented in the
/// manuscript); they express screening priority for confirmatory testing,
/// never definitive contamination status:
///   Low      p < 0.25
///   Moderate 0.25 <= p < 0.50
///   High     0.50 <= p < 0.75
///   Very_High p >= 0.75
pub fn screening_priority_class(prob: f64) -> &'static str {
    if prob >= 0.75 {
        "Very_High"
    } else if prob >= 0.50 {
        "High"
    } else if prob >= 0.25 {
        "Moderate"
    } else {
        "Low"
    }
}

pub fn screening_priority_reason(class: &str) -> &'static str {
    match class {
        "Very_High" => "Urgent confirmatory laboratory testing recommended",
        "High" => "Confirmatory testing recommended",
        "Moderate" => "Monitor or retest seasonally",
        _ => "Lower immediate priority; routine monitoring",
    }
}
