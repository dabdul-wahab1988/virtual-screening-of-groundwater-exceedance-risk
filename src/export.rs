use crate::utils::{ensure_dir, now_iso8601, sha256_file};
use anyhow::Result;
use rusqlite::Connection;
use std::{fs::File, io::Write, path::Path};

// ── Generic CSV writer ────────────────────────────────────────────────────────

pub fn write_csv_file(path: &str, headers: &[&str], rows: Vec<Vec<String>>) -> Result<usize> {
    let mut out = File::create(path)?;
    writeln!(out, "{}", headers.join(","))?;
    for row in &rows {
        writeln!(out, "{}", row.join(","))?;
    }
    Ok(rows.len())
}

fn quote_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ── R exports ─────────────────────────────────────────────────────────────────

pub fn export_r_files(conn: &Connection, out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;

    export_target_prevalence_summary(conn, out_dir)?;
    export_leakage_audit_summary(conn, out_dir)?;
    export_model_performance_summary(conn, out_dir)?;
    export_fold_metrics_long(conn, out_dir)?;
    export_out_of_fold_predictions(conn, out_dir)?;
    export_shap_values_long(conn, out_dir)?;
    export_calibration_inputs(conn, out_dir)?;
    export_operational_thresholds(conn, out_dir)?;
    export_threshold_sensitivity(conn, out_dir)?;
    export_screening_priority_table(conn, out_dir)?;

    log::info!("R exports written to {}", out_dir);
    Ok(())
}

fn export_target_prevalence_summary(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/target_prevalence_summary.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT e.target_contaminant, e.n_samples, e.n_positive, e.n_negative,
                e.prevalence, e.cv_feasible, e.ml_status, e.reporting_level, e.eligibility_note,
                d.threshold_value, d.threshold_unit, d.threshold_source
         FROM target_eligibility e
         LEFT JOIN target_definitions d ON d.target_contaminant = e.target_contaminant
         ORDER BY e.target_contaminant",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?.to_string(),
                row.get::<_, i64>(2)?.to_string(),
                row.get::<_, i64>(3)?.to_string(),
                format!("{:.4}", row.get::<_, f64>(4)?),
                row.get::<_, i64>(5)?.to_string(),
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                quote_csv(&row.get::<_, String>(8).unwrap_or_default()),
                row.get::<_, Option<f64>>(9)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                row.get::<_, Option<String>>(11)?.unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "target_contaminant",
            "n_samples",
            "n_positive",
            "n_negative",
            "prevalence",
            "cv_feasible",
            "ml_status",
            "reporting_level",
            "eligibility_note",
            "threshold_value",
            "threshold_unit",
            "threshold_source",
        ],
        rows,
    )?;
    Ok(())
}

fn export_leakage_audit_summary(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/leakage_audit_summary.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT target_contaminant, tier_name, feature_name, action, reason, rule_source
         FROM leakage_rules_applied
         ORDER BY target_contaminant, tier_name, action, feature_name",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                quote_csv(&row.get::<_, String>(4).unwrap_or_default()),
                row.get::<_, String>(5).unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "target_contaminant",
            "tier_name",
            "feature_name",
            "action",
            "reason",
            "rule_source",
        ],
        rows,
    )?;
    Ok(())
}

fn export_model_performance_summary(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/model_performance_summary.csv", out_dir);
    let query = "
        SELECT r.run_id, r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode,
               AVG(m.roc_auc) as mean_roc_auc,
               AVG(m.pr_auc) as mean_pr_auc,
               AVG(m.balanced_accuracy) as mean_bal_acc,
               AVG(m.recall_sensitivity) as mean_recall,
               AVG(m.specificity) as mean_spec,
               AVG(m.f1_score) as mean_f1,
               AVG(m.f2_score) as mean_f2,
               AVG(m.brier_score) as mean_brier,
               COUNT(*) as n_folds,
               MIN(m.pr_auc) as min_pr_auc,
               MAX(m.pr_auc) as max_pr_auc
        FROM model_runs r
        JOIN fold_metrics m ON m.run_id = r.run_id
        GROUP BY r.run_id
        ORDER BY r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode";
    let mut stmt = conn.prepare(query)?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                format!("{:.4}", row.get::<_, f64>(5).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(6).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(7).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(8).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(9).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(10).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(11).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(12).unwrap_or(f64::NAN)),
                row.get::<_, i64>(13)?.to_string(),
                format!("{:.4}", row.get::<_, f64>(14).unwrap_or(f64::NAN)),
                format!("{:.4}", row.get::<_, f64>(15).unwrap_or(f64::NAN)),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "target_contaminant",
            "predictor_tier",
            "algorithm",
            "cv_mode",
            "mean_roc_auc",
            "mean_pr_auc",
            "mean_bal_acc",
            "mean_recall",
            "mean_spec",
            "mean_f1",
            "mean_f2",
            "mean_brier",
            "n_folds",
            "min_pr_auc",
            "max_pr_auc",
        ],
        rows,
    )?;
    Ok(())
}

fn export_fold_metrics_long(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/fold_metrics_long.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT r.run_id, r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode,
                m.repeat_index, m.fold_index, m.roc_auc, m.pr_auc, m.balanced_accuracy,
                m.recall_sensitivity, m.specificity, m.f1_score, m.f2_score, m.brier_score,
                m.calibration_slope, m.calibration_intercept, m.n_test_pos, m.n_test_neg
         FROM fold_metrics m
         JOIN model_runs r ON r.run_id = m.run_id
         ORDER BY r.target_contaminant, r.algorithm, m.repeat_index, m.fold_index",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let fmt = |v: rusqlite::Result<f64>| format!("{:.6}", v.unwrap_or(f64::NAN));
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?.to_string(),
                row.get::<_, i64>(6)?.to_string(),
                fmt(row.get(7)),
                fmt(row.get(8)),
                fmt(row.get(9)),
                fmt(row.get(10)),
                fmt(row.get(11)),
                fmt(row.get(12)),
                fmt(row.get(13)),
                fmt(row.get(14)),
                fmt(row.get(15)),
                fmt(row.get(16)),
                row.get::<_, Option<i64>>(17)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                row.get::<_, Option<i64>>(18)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "target_contaminant",
            "predictor_tier",
            "algorithm",
            "cv_mode",
            "repeat_index",
            "fold_index",
            "roc_auc",
            "pr_auc",
            "balanced_accuracy",
            "recall_sensitivity",
            "specificity",
            "f1_score",
            "f2_score",
            "brier_score",
            "calibration_slope",
            "calibration_intercept",
            "n_test_pos",
            "n_test_neg",
        ],
        rows,
    )?;
    Ok(())
}

fn export_out_of_fold_predictions(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/out_of_fold_predictions.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT p.run_id, p.sample_id, p.target_contaminant, p.predictor_tier,
                p.algorithm, p.cv_mode, p.repeat_index, p.fold_index,
                p.true_label, p.predicted_probability, p.predicted_label_default_0_5,
                p.spatial_cluster_id
         FROM well_predictions p
         ORDER BY p.target_contaminant, p.algorithm, p.sample_id, p.repeat_index, p.fold_index",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?.to_string(),
                row.get::<_, i64>(7)?.to_string(),
                row.get::<_, Option<i64>>(8)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                format!("{:.6}", row.get::<_, f64>(9)?),
                row.get::<_, i64>(10)?.to_string(),
                row.get::<_, Option<i64>>(11)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "sample_id",
            "target_contaminant",
            "predictor_tier",
            "algorithm",
            "cv_mode",
            "repeat_index",
            "fold_index",
            "true_label",
            "predicted_probability",
            "predicted_label_default_0_5",
            "spatial_cluster_id",
        ],
        rows,
    )?;
    Ok(())
}

fn export_shap_values_long(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/shap_values_long.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT s.run_id, s.sample_id, r.target_contaminant, r.predictor_tier, r.algorithm,
                s.cv_mode, s.repeat_index, s.fold_index, s.feature_name,
                s.shap_value, s.feature_cleaned_value
         FROM shap_values s
         JOIN model_runs r ON r.run_id = s.run_id
         ORDER BY r.target_contaminant, r.algorithm, s.sample_id, s.feature_name",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?.to_string(),
                row.get::<_, i64>(7)?.to_string(),
                row.get::<_, String>(8)?,
                format!("{:.8}", row.get::<_, f64>(9)?),
                row.get::<_, Option<f64>>(10)?
                    .map(|v| format!("{:.4}", v))
                    .unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "sample_id",
            "target_contaminant",
            "predictor_tier",
            "algorithm",
            "cv_mode",
            "repeat_index",
            "fold_index",
            "feature_name",
            "shap_value",
            "feature_cleaned_value",
        ],
        rows,
    )?;
    Ok(())
}

fn export_calibration_inputs(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/calibration_inputs.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT p.run_id, r.target_contaminant, r.algorithm, r.cv_mode,
                p.true_label, p.predicted_probability
         FROM well_predictions p
         JOIN model_runs r ON r.run_id = p.run_id
         ORDER BY r.target_contaminant, r.algorithm, p.predicted_probability",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                format!("{:.6}", row.get::<_, f64>(5)?),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "target_contaminant",
            "algorithm",
            "cv_mode",
            "true_label",
            "predicted_probability",
        ],
        rows,
    )?;
    Ok(())
}

fn export_operational_thresholds(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/operational_thresholds.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT ot.run_id, r.target_contaminant, r.predictor_tier, r.algorithm, ot.cv_mode,
                ot.threshold_selection_rule, ot.optimized_probability_cutoff,
                ot.resulting_sensitivity, ot.resulting_specificity,
                ot.false_negatives_count, ot.false_positives_count
         FROM operational_thresholds ot
         JOIN model_runs r ON r.run_id = ot.run_id
         ORDER BY r.target_contaminant, ot.threshold_selection_rule",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                format!("{:.4}", row.get::<_, f64>(6)?),
                format!("{:.4}", row.get::<_, f64>(7)?),
                format!("{:.4}", row.get::<_, f64>(8)?),
                row.get::<_, i64>(9)?.to_string(),
                row.get::<_, i64>(10)?.to_string(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "target_contaminant",
            "predictor_tier",
            "algorithm",
            "cv_mode",
            "threshold_selection_rule",
            "optimized_probability_cutoff",
            "resulting_sensitivity",
            "resulting_specificity",
            "false_negatives_count",
            "false_positives_count",
        ],
        rows,
    )?;
    Ok(())
}

fn export_threshold_sensitivity(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/threshold_sensitivity.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT ts.run_id, r.target_contaminant, r.algorithm, ts.cv_mode,
                ts.probability_cutoff, ts.sensitivity, ts.specificity,
                ts.precision, ts.f1_score, ts.f2_score,
                ts.false_negatives_count, ts.false_positives_count
         FROM threshold_sensitivity ts
         JOIN model_runs r ON r.run_id = ts.run_id
         ORDER BY r.target_contaminant, r.algorithm, ts.probability_cutoff",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let fmt = |v: rusqlite::Result<f64>| format!("{:.4}", v.unwrap_or(f64::NAN));
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                fmt(row.get(4)),
                fmt(row.get(5)),
                fmt(row.get(6)),
                fmt(row.get(7)),
                fmt(row.get(8)),
                fmt(row.get(9)),
                row.get::<_, i64>(10)?.to_string(),
                row.get::<_, i64>(11)?.to_string(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "run_id",
            "target_contaminant",
            "algorithm",
            "cv_mode",
            "probability_cutoff",
            "sensitivity",
            "specificity",
            "precision",
            "f1_score",
            "f2_score",
            "false_negatives_count",
            "false_positives_count",
        ],
        rows,
    )?;
    Ok(())
}

fn export_screening_priority_table(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/screening_priority_table.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT sp.sample_id, sp.target_contaminant,
                sp.predicted_probability_median, sp.predicted_probability_lower_ci,
                sp.predicted_probability_upper_ci, sp.screening_priority_class,
                sp.priority_reason, sp.best_validated_run_id,
                c.latitude, c.longitude
         FROM screening_priority sp
         LEFT JOIN sample_spatial_assignment c ON c.sample_id = sp.sample_id
         ORDER BY sp.target_contaminant, sp.predicted_probability_median DESC",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let fmt_opt = |v: rusqlite::Result<Option<f64>>| {
                v.ok()
                    .flatten()
                    .map(|x| format!("{:.6}", x))
                    .unwrap_or_default()
            };
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                fmt_opt(Ok(row.get::<_, Option<f64>>(2)?)),
                fmt_opt(Ok(row.get::<_, Option<f64>>(3)?)),
                fmt_opt(Ok(row.get::<_, Option<f64>>(4)?)),
                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                quote_csv(&row.get::<_, Option<String>>(6)?.unwrap_or_default()),
                row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                fmt_opt(Ok(row.get::<_, Option<f64>>(8)?)),
                fmt_opt(Ok(row.get::<_, Option<f64>>(9)?)),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "sample_id",
            "target_contaminant",
            "predicted_probability_median",
            "predicted_probability_lower_ci",
            "predicted_probability_upper_ci",
            "screening_priority_class",
            "priority_reason",
            "best_validated_run_id",
            "latitude",
            "longitude",
        ],
        rows,
    )?;
    Ok(())
}

// ── GIS exports ───────────────────────────────────────────────────────────────

pub fn export_gis_files(conn: &Connection, out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;

    // Guard: warn if screening_priority is empty (pipeline not yet run or partial)
    let sp_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM screening_priority", [], |r| r.get(0))
        .unwrap_or(0);
    if sp_count == 0 {
        log::warn!(
            "screening_priority table is empty — GIS exports will have no data rows. \
             Run `run-pipeline` before `export-gis`."
        );
    }

    export_well_screening_priority_points(conn, out_dir)?;
    export_target_specific_probabilities(conn, out_dir)?;
    export_spatial_cluster_metadata(conn, out_dir)?;
    export_gis_manifest(conn, out_dir)?;

    log::info!(
        "GIS exports written to {} ({} screening-priority rows)",
        out_dir,
        sp_count
    );
    Ok(())
}

fn export_well_screening_priority_points(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/well_screening_priority_points.csv", out_dir);
    // One row per well: highest priority class across all targets
    let query = "
        SELECT sp.sample_id, c.longitude, c.latitude,
               GROUP_CONCAT(sp.target_contaminant || ':' || COALESCE(sp.screening_priority_class,'NA'), '|') as targets_classes,
               MAX(sp.predicted_probability_median) as max_probability,
               CASE
                   WHEN MAX(sp.predicted_probability_median) >= 0.75 THEN 'Very_High'
                   WHEN MAX(sp.predicted_probability_median) >= 0.50 THEN 'High'
                   WHEN MAX(sp.predicted_probability_median) >= 0.25 THEN 'Moderate'
                   ELSE 'Low'
               END as overall_priority,
               r.location
        FROM screening_priority sp
        LEFT JOIN sample_spatial_assignment c ON c.sample_id = sp.sample_id
        LEFT JOIN (SELECT sample_id, json_extract(raw_json, '$.Location') as location FROM raw_samples) r ON r.sample_id = sp.sample_id
        GROUP BY sp.sample_id
        ORDER BY max_probability DESC";

    let mut stmt = conn.prepare(query)?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, Option<f64>>(1)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                row.get::<_, Option<f64>>(2)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                quote_csv(&row.get::<_, Option<String>>(3)?.unwrap_or_default()),
                row.get::<_, Option<f64>>(4)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                quote_csv(&row.get::<_, Option<String>>(6)?.unwrap_or_default()),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "sample_id",
            "longitude",
            "latitude",
            "target_priority_classes",
            "max_predicted_probability",
            "overall_priority_class",
            "location",
        ],
        rows,
    )?;
    Ok(())
}

fn export_target_specific_probabilities(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/target_specific_screening_probabilities.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT sp.sample_id, sp.target_contaminant,
                sp.predicted_probability_median, sp.screening_priority_class,
                c.longitude, c.latitude, c.spatial_cluster_id
         FROM screening_priority sp
         LEFT JOIN sample_spatial_assignment c ON c.sample_id = sp.sample_id
         ORDER BY sp.target_contaminant, sp.predicted_probability_median DESC",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<f64>>(2)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                row.get::<_, Option<f64>>(4)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                row.get::<_, Option<f64>>(5)?
                    .map(|v| format!("{:.6}", v))
                    .unwrap_or_default(),
                row.get::<_, Option<i64>>(6)?
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "sample_id",
            "target_contaminant",
            "predicted_probability_median",
            "screening_priority_class",
            "longitude",
            "latitude",
            "spatial_cluster_id",
        ],
        rows,
    )?;
    Ok(())
}

fn export_spatial_cluster_metadata(conn: &Connection, out_dir: &str) -> Result<()> {
    let path = format!("{}/spatial_cluster_metadata.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT spatial_cluster_id, centroid_latitude, centroid_longitude,
                number_of_wells, max_intra_cluster_distance_km
         FROM spatial_cluster_metadata
         ORDER BY spatial_cluster_id",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, i64>(0)?.to_string(),
                format!("{:.6}", row.get::<_, f64>(1)?),
                format!("{:.6}", row.get::<_, f64>(2)?),
                row.get::<_, i64>(3)?.to_string(),
                row.get::<_, Option<f64>>(4)?
                    .map(|v| format!("{:.3}", v))
                    .unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    write_csv_file(
        &path,
        &[
            "spatial_cluster_id",
            "centroid_latitude",
            "centroid_longitude",
            "number_of_wells",
            "max_intra_cluster_distance_km",
        ],
        rows,
    )?;
    Ok(())
}

fn export_gis_manifest(conn: &Connection, out_dir: &str) -> Result<()> {
    let files = [
        "well_screening_priority_points.csv",
        "target_specific_screening_probabilities.csv",
        "spatial_cluster_metadata.csv",
    ];
    for fname in &files {
        let fpath = format!("{}/{}", out_dir, fname);
        let (row_count, hash) = if Path::new(&fpath).exists() {
            let content = std::fs::read_to_string(&fpath).unwrap_or_default();
            let n = content.lines().count().saturating_sub(1);
            let h = sha256_file(&fpath).unwrap_or_default();
            (n as i64, h)
        } else {
            (0, String::new())
        };
        conn.execute(
            "INSERT INTO gis_exports_manifest (export_file, export_type, created_at, row_count, sha256_hash)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![fname, "gis_csv", now_iso8601(), row_count, hash],
        )?;
    }
    // Write manifest CSV
    let manifest_path = format!("{}/gis_exports_manifest.csv", out_dir);
    let mut stmt = conn.prepare(
        "SELECT export_id, export_file, export_type, created_at, row_count, sha256_hash FROM gis_exports_manifest",
    )?;
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            Ok(vec![
                row.get::<_, i64>(0)?.to_string(),
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?.to_string(),
                row.get::<_, String>(5).unwrap_or_default(),
            ])
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    write_csv_file(
        &manifest_path,
        &[
            "export_id",
            "export_file",
            "export_type",
            "created_at",
            "row_count",
            "sha256_hash",
        ],
        rows,
    )?;
    Ok(())
}
