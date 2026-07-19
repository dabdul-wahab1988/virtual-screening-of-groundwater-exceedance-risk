/// Ingests raw CSV data into raw_samples and cleaned_measurements tables.
use crate::{
    io::{CsvDataset, ThresholdsConfig},
    utils::{now_iso8601, parse_numeric_bdl, parse_numeric_bdl_with_limit},
};
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

pub fn ingest_raw_samples(
    conn: &Connection,
    dataset: &CsvDataset,
    input_file_hash: &str,
) -> Result<()> {
    for (row_idx, row) in dataset.rows.iter().enumerate() {
        let sample_id = row
            .get("SampleID")
            .cloned()
            .unwrap_or_else(|| format!("ROW_{:04}", row_idx + 1));

        let raw_json = serde_json::to_string(row).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT OR REPLACE INTO raw_samples
             (sample_id, original_row_index, raw_json, imported_at, input_file_hash)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                sample_id,
                row_idx as i64,
                raw_json,
                now_iso8601(),
                input_file_hash,
            ],
        )?;
    }
    log::info!("Ingested {} raw samples", dataset.rows.len());
    Ok(())
}

pub fn ingest_cleaned_measurements(
    conn: &Connection,
    dataset: &CsvDataset,
    thresholds_cfg: &ThresholdsConfig,
) -> Result<(usize, usize, usize)> {
    let skip_cols = ["SampleID", "Location"];
    let mut bdl_count = 0usize;
    let mut missing_count = 0usize;
    let mut parse_error_count = 0usize;
    let configured_limits = configured_detection_limits(thresholds_cfg);
    let inferred_limits = infer_min_detected_limits(dataset);
    let bdl_method = thresholds_cfg.bdl_substitution.method.as_str();
    let use_fallback = thresholds_cfg.bdl_substitution.use_min_detected_fallback;

    for row in &dataset.rows {
        let sample_id = row
            .get("SampleID")
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        for (col, raw_val) in row {
            if skip_cols.contains(&col.as_str()) {
                continue;
            }

            if raw_val.is_empty() {
                missing_count += 1;
                conn.execute(
                    "INSERT OR IGNORE INTO cleaned_measurements
                     (sample_id, variable_name, raw_value_text, cleaned_value_real,
                      bdl_flag, missing_flag, cleaning_note)
                     VALUES (?1, ?2, ?3, NULL, 0, 1, 'empty_value')",
                    rusqlite::params![sample_id, col, raw_val],
                )?;
                continue;
            }

            let (substitution_limit, limit_source) = if bdl_method == "none" {
                (None, "none")
            } else if let Some(limit) = configured_limits.get(col).copied() {
                (Some(limit), "configured_detection_limit")
            } else if use_fallback {
                (inferred_limits.get(col).copied(), "minimum_detected_value")
            } else {
                (None, "none")
            };

            let (cleaned_val, is_bdl, is_err, parsed_bdl_rule) =
                parse_numeric_bdl_with_limit(raw_val, substitution_limit);
            if is_bdl {
                bdl_count += 1;
            }
            if is_err {
                parse_error_count += 1;
            }

            let bdl_rule = if is_bdl {
                parsed_bdl_rule.unwrap_or("bdl_no_rule")
            } else {
                ""
            };

            let note = if is_err {
                "failed_numeric_parse"
            } else if is_bdl && cleaned_val.is_none() {
                "bdl_stored_as_missing"
            } else if is_bdl && raw_val.trim().starts_with('<') {
                "bdl_substituted_using_reported_detection_limit"
            } else if is_bdl && limit_source == "minimum_detected_value" {
                "bdl_substituted_using_min_detected_fallback"
            } else if is_bdl && limit_source == "configured_detection_limit" {
                "bdl_substituted_using_configured_detection_limit"
            } else {
                ""
            };

            let unit = infer_unit_for_col(col);

            conn.execute(
                "INSERT OR REPLACE INTO cleaned_measurements
                 (sample_id, variable_name, raw_value_text, cleaned_value_real,
                  unit, bdl_flag, bdl_rule, missing_flag, cleaning_note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    sample_id,
                    col,
                    raw_val,
                    cleaned_val,
                    unit,
                    is_bdl as i64,
                    bdl_rule,
                    (is_err || cleaned_val.is_none()) as i64,
                    note,
                ],
            )?;
        }
    }

    Ok((bdl_count, missing_count, parse_error_count))
}

fn configured_detection_limits(cfg: &ThresholdsConfig) -> HashMap<String, f64> {
    cfg.bdl_substitution
        .detection_limits
        .iter()
        .filter_map(|(name, limit)| {
            limit
                .value
                .filter(|v| v.is_finite() && *v > 0.0)
                .map(|v| (name.clone(), v))
        })
        .collect()
}

fn infer_min_detected_limits(dataset: &CsvDataset) -> HashMap<String, f64> {
    let mut limits: HashMap<String, f64> = HashMap::new();
    for row in &dataset.rows {
        for (col, raw_val) in row {
            if matches!(col.as_str(), "SampleID" | "Location") {
                continue;
            }
            let (value, is_bdl, is_err) = parse_numeric_bdl(raw_val);
            if is_bdl || is_err {
                continue;
            }
            if let Some(value) = value.filter(|v| v.is_finite() && *v > 0.0) {
                limits
                    .entry(col.clone())
                    .and_modify(|current| {
                        if value < *current {
                            *current = value;
                        }
                    })
                    .or_insert(value);
            }
        }
    }
    limits
}

fn infer_unit_for_col(col: &str) -> &'static str {
    match col {
        "pH" => "dimensionless",
        "Temp." => "°C",
        "EC" => "µS/cm",
        "TDS" | "Na" | "K" | "Mg" | "Ca" | "Cl" | "SO4" | "HCO3" | "CO3" | "NO3" | "F" | "B" => {
            "mg/L"
        }
        "Dx" => "decimal_degrees_lon",
        "Dy" => "decimal_degrees_lat",
        _ => "unknown",
    }
}

pub fn run_data_audit(
    conn: &Connection,
    dataset: &CsvDataset,
    bdl_count: usize,
    parse_error_count: usize,
    lat_col: &str,
    lon_col: &str,
) -> Result<()> {
    let n_samples = dataset.n_rows as i64;
    let n_columns = dataset.n_cols as i64;

    // Count samples missing either latitude OR longitude
    let missing_coords: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT s.sample_id) FROM raw_samples s
         WHERE NOT EXISTS (
             SELECT 1 FROM cleaned_measurements c
             WHERE c.sample_id = s.sample_id AND c.variable_name = ?1
             AND c.cleaned_value_real IS NOT NULL
         )
         OR NOT EXISTS (
             SELECT 1 FROM cleaned_measurements c
             WHERE c.sample_id = s.sample_id AND c.variable_name = ?2
             AND c.cleaned_value_real IS NOT NULL
         )",
            rusqlite::params![lat_col, lon_col],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Count duplicate sample IDs
    let duplicate_ids: i64 = conn
        .query_row(
            "SELECT COUNT(*) - COUNT(DISTINCT sample_id) FROM raw_samples",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let status = if parse_error_count > 5 {
        "WARNING"
    } else {
        "OK"
    };
    let msg = format!(
        "n={}, cols={}, bdl={}, parse_errors={}, missing_coords={}, duplicates={}",
        n_samples, n_columns, bdl_count, parse_error_count, missing_coords, duplicate_ids
    );

    conn.execute(
        "INSERT INTO data_audit
         (audit_timestamp, n_samples, n_columns, missing_coordinate_count,
          duplicate_sample_count, bdl_value_count, failed_numeric_parse_count,
          audit_status, audit_message)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            now_iso8601(),
            n_samples,
            n_columns,
            missing_coords,
            duplicate_ids,
            bdl_count as i64,
            parse_error_count as i64,
            status,
            msg,
        ],
    )?;
    Ok(())
}

/// Load all labelled sample IDs for a target.
pub fn load_labelled_samples(conn: &Connection, target: &str) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT sample_id, true_label
         FROM target_labels
         WHERE target_contaminant = ?1 AND true_label IS NOT NULL
         ORDER BY sample_id",
    )?;
    let pairs = stmt
        .query_map(rusqlite::params![target], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(pairs)
}

/// Get spatial cluster ID for a sample.
pub fn get_spatial_cluster(conn: &Connection, sample_id: &str) -> Option<i64> {
    conn.query_row(
        "SELECT spatial_cluster_id FROM sample_spatial_assignment WHERE sample_id = ?1",
        rusqlite::params![sample_id],
        |r| r.get(0),
    )
    .ok()
}
