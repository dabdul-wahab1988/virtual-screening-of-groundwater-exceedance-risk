use crate::io::ThresholdsConfig;
use anyhow::{anyhow, Result};
use rusqlite::Connection;

pub fn store_threshold_definitions(conn: &Connection, cfg: &ThresholdsConfig) -> Result<()> {
    for (target, thr) in &cfg.targets {
        conn.execute(
            "INSERT OR REPLACE INTO target_definitions
             (target_contaminant, source_column, threshold_value, threshold_unit,
              threshold_source, exceedance_direction, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                target,
                thr.column,
                thr.threshold_value,
                thr.unit,
                thr.threshold_source,
                thr.exceedance_direction,
                thr.notes,
            ],
        )?;
    }
    Ok(())
}

pub fn generate_and_store_labels(conn: &Connection, cfg: &ThresholdsConfig) -> Result<()> {
    // Collect all sample_ids
    let mut stmt0 =
        conn.prepare("SELECT sample_id FROM raw_samples ORDER BY original_row_index")?;
    let sample_ids: Vec<String> = stmt0
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;

    for (target, thr) in &cfg.targets {
        let threshold = match thr.threshold_value {
            Some(v) => v,
            None => {
                // Store all rows with label_status = 'no_threshold'
                for sid in &sample_ids {
                    conn.execute(
                        "INSERT OR REPLACE INTO target_labels
                         (target_contaminant, sample_id, measured_value, threshold_value, true_label, label_status)
                         VALUES (?1, ?2, NULL, NULL, NULL, 'no_threshold')",
                        rusqlite::params![target, sid],
                    )?;
                }
                continue;
            }
        };

        for sid in &sample_ids {
            // Fetch cleaned value
            let result: rusqlite::Result<Option<f64>> = conn.query_row(
                "SELECT cleaned_value_real FROM cleaned_measurements
                 WHERE sample_id = ?1 AND variable_name = ?2",
                rusqlite::params![sid, thr.column],
                |row| row.get(0),
            );

            let (measured_val, label, status) = match result {
                Ok(Some(v)) => {
                    let exceeds = match thr.exceedance_direction.as_str() {
                        "greater_than" => v > threshold,
                        "less_than" => v < threshold,
                        _ => v > threshold,
                    };
                    (Some(v), Some(if exceeds { 1i64 } else { 0i64 }), "labelled")
                }
                Ok(None) => (None, None, "missing_value"),
                Err(_) => (None, None, "missing_column"),
            };

            conn.execute(
                "INSERT OR REPLACE INTO target_labels
                 (target_contaminant, sample_id, measured_value, threshold_value, true_label, label_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![target, sid, measured_val, threshold, label, status],
            )?;
        }
    }
    Ok(())
}

pub fn compute_and_store_eligibility(conn: &Connection, min_positive: usize) -> Result<()> {
    let mut stmt_tgt =
        conn.prepare("SELECT DISTINCT target_contaminant FROM target_definitions")?;
    let targets: Vec<String> = stmt_tgt
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;

    for target in &targets {
        let n_total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM target_labels WHERE target_contaminant = ?1 AND true_label IS NOT NULL",
            rusqlite::params![target],
            |r| r.get(0),
        )?;
        let n_pos: i64 = conn.query_row(
            "SELECT COUNT(*) FROM target_labels WHERE target_contaminant = ?1 AND true_label = 1",
            rusqlite::params![target],
            |r| r.get(0),
        )?;
        let n_neg = n_total - n_pos;
        let prevalence = if n_total > 0 {
            n_pos as f64 / n_total as f64
        } else {
            0.0
        };

        let cv_feasible = n_pos >= min_positive as i64 && n_neg >= min_positive as i64;

        let (ml_status, reporting_level, note) = if n_pos < 3 {
            (
                "descriptive_only",
                "Supplement",
                format!("Only {} positive cases; insufficient for any ML", n_pos),
            )
        } else if n_pos < min_positive as i64 {
            (
                "exploratory_only",
                "Supplement",
                format!(
                    "{} positive cases < minimum {} for standard CV",
                    n_pos, min_positive
                ),
            )
        } else if !cv_feasible {
            (
                "exploratory_only",
                "Supplement",
                format!(
                    "Class distribution too extreme for reliable CV (n_pos={}, n_neg={})",
                    n_pos, n_neg
                ),
            )
        } else {
            (
                "modelled",
                "Main text",
                format!("{} positive, {} negative — ML feasible", n_pos, n_neg),
            )
        };

        // Upsert the no_threshold targets
        let has_threshold: i64 = conn.query_row(
            "SELECT COUNT(*) FROM target_definitions WHERE target_contaminant = ?1 AND threshold_value IS NOT NULL",
            rusqlite::params![target],
            |r| r.get(0),
        )?;
        let (final_status, final_note) = if has_threshold == 0 {
            (
                "no_threshold",
                "Threshold not configured — cannot generate labels".to_string(),
            )
        } else {
            (ml_status, note)
        };

        conn.execute(
            "INSERT OR REPLACE INTO target_eligibility
             (target_contaminant, n_samples, n_positive, n_negative, prevalence,
              cv_feasible, ml_status, reporting_level, eligibility_note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                target,
                n_total,
                n_pos,
                n_neg,
                prevalence,
                cv_feasible as i64,
                final_status,
                reporting_level,
                final_note,
            ],
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn get_target_labels(conn: &Connection, target: &str) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT sample_id, true_label FROM target_labels
         WHERE target_contaminant = ?1 AND true_label IS NOT NULL
         ORDER BY sample_id",
    )?;
    let pairs = stmt
        .query_map(rusqlite::params![target], |row| {
            let sid: String = row.get(0)?;
            let lbl: i32 = row.get(1)?;
            Ok((sid, lbl))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if pairs.is_empty() {
        return Err(anyhow!("No labels found for target '{}'", target));
    }
    Ok(pairs)
}

pub fn is_target_eligible(conn: &Connection, target: &str) -> Result<bool> {
    let status: String = conn.query_row(
        "SELECT ml_status FROM target_eligibility WHERE target_contaminant = ?1",
        rusqlite::params![target],
        |r| r.get(0),
    )?;
    Ok(status == "modelled")
}
