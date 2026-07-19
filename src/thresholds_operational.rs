use crate::metrics::threshold_sweep;
use anyhow::Result;
use rusqlite::Connection;

pub struct OperationalThreshold {
    pub rule: String,
    pub cutoff: f64,
    pub sensitivity: f64,
    pub specificity: f64,
    pub fn_count: usize,
    pub fp_count: usize,
}

pub fn select_operational_thresholds(y_true: &[i32], y_prob: &[f64]) -> Vec<OperationalThreshold> {
    let sweep = threshold_sweep(y_true, y_prob);
    let mut result = Vec::new();

    // 1. Maximum F2 (sensitivity-weighted)
    if let Some(best) = sweep.iter().max_by(|a, b| a.f2.partial_cmp(&b.f2).unwrap()) {
        result.push(OperationalThreshold {
            rule: "max_F2".into(),
            cutoff: best.cutoff,
            sensitivity: best.sensitivity,
            specificity: best.specificity,
            fn_count: best.fn_count,
            fp_count: best.fp_count,
        });
    }

    // 2. Sensitivity ≥ 0.90 with best specificity
    if let Some(best) = sweep
        .iter()
        .filter(|p| p.sensitivity >= 0.90)
        .max_by(|a, b| a.specificity.partial_cmp(&b.specificity).unwrap())
    {
        result.push(OperationalThreshold {
            rule: "sens_ge_0.90".into(),
            cutoff: best.cutoff,
            sensitivity: best.sensitivity,
            specificity: best.specificity,
            fn_count: best.fn_count,
            fp_count: best.fp_count,
        });
    } else {
        // Fallback: best sensitivity achieved
        if let Some(best) = sweep
            .iter()
            .max_by(|a, b| a.sensitivity.partial_cmp(&b.sensitivity).unwrap())
        {
            result.push(OperationalThreshold {
                rule: "best_sensitivity_available".into(),
                cutoff: best.cutoff,
                sensitivity: best.sensitivity,
                specificity: best.specificity,
                fn_count: best.fn_count,
                fp_count: best.fp_count,
            });
        }
    }

    // 3. Youden Index (sensitivity + specificity - 1 maximised)
    if let Some(best) = sweep.iter().max_by(|a, b| {
        let ya = a.sensitivity + a.specificity - 1.0;
        let yb = b.sensitivity + b.specificity - 1.0;
        ya.partial_cmp(&yb).unwrap()
    }) {
        result.push(OperationalThreshold {
            rule: "youden_index".into(),
            cutoff: best.cutoff,
            sensitivity: best.sensitivity,
            specificity: best.specificity,
            fn_count: best.fn_count,
            fp_count: best.fp_count,
        });
    }

    result
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
              precision, f1_score, f2_score, false_negatives_count, false_positives_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                run_id,
                cv_mode,
                p.cutoff,
                p.sensitivity,
                p.specificity,
                p.precision,
                p.f1,
                p.f2,
                p.fn_count as i64,
                p.fp_count as i64,
            ],
        )?;
    }
    Ok(())
}

/// Compute screening priority class from predicted probability.
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
