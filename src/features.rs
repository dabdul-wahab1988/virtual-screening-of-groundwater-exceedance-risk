use crate::{io::LeakageRulesConfig, leakage::apply_leakage_filter};
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

// ── Column metadata ─────────────────────────────────────────────────────────

pub struct ColumnMeta {
    pub name: String,
    pub inferred_type: String,
    pub scientific_role: String,
    pub unit: String,
    pub is_coordinate: bool,
    pub is_target_candidate: bool,
    pub is_field_variable: bool,
    pub is_lab_variable: bool,
    pub is_derived_variable: bool,
}

/// Classify every CSV column into scientific roles.
pub fn build_column_metadata(headers: &[String]) -> Vec<ColumnMeta> {
    let targets = ["Na", "Cl", "TDS", "B", "F", "NO3"];
    let field_vars = ["pH", "Temp.", "EC"];
    let coord_vars = ["Dx", "Dy"];
    let id_vars = ["SampleID", "Location"];

    headers
        .iter()
        .map(|h| {
            let is_coord = coord_vars.contains(&h.as_str());
            let is_target = targets.contains(&h.as_str());
            let is_field = field_vars.contains(&h.as_str());
            let is_id = id_vars.contains(&h.as_str());
            let is_lab = !is_coord && !is_target && !is_field && !is_id;

            let sci_role = if is_id {
                "identifier"
            } else if is_coord {
                "coordinate"
            } else if is_field {
                "field_parameter"
            } else if is_target {
                "target_contaminant"
            } else {
                "laboratory_chemical"
            };

            let unit = infer_unit(h);

            ColumnMeta {
                name: h.clone(),
                inferred_type: "numeric".to_string(),
                scientific_role: sci_role.to_string(),
                unit,
                is_coordinate: is_coord,
                is_target_candidate: is_target,
                is_field_variable: is_field,
                is_lab_variable: is_lab && !is_id,
                is_derived_variable: false,
            }
        })
        .collect()
}

fn infer_unit(col: &str) -> String {
    match col {
        "pH" => "dimensionless".into(),
        "Temp." => "°C".into(),
        "EC" => "µS/cm".into(),
        "TDS" => "mg/L".into(),
        "Dx" => "decimal_degrees_lon".into(),
        "Dy" => "decimal_degrees_lat".into(),
        "SampleID" | "Location" => "text".into(),
        _ => "mg/L".into(),
    }
}

pub fn store_column_dictionary(conn: &Connection, metas: &[ColumnMeta]) -> Result<()> {
    for m in metas {
        conn.execute(
            "INSERT OR IGNORE INTO column_dictionary
             (column_name, inferred_type, scientific_role, unit,
              is_coordinate, is_target_candidate, is_field_variable,
              is_lab_variable, is_derived_variable)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                m.name,
                m.inferred_type,
                m.scientific_role,
                m.unit,
                m.is_coordinate as i64,
                m.is_target_candidate as i64,
                m.is_field_variable as i64,
                m.is_lab_variable as i64,
                m.is_derived_variable as i64,
            ],
        )?;
    }
    Ok(())
}

// ── Tier candidate lists ─────────────────────────────────────────────────────

/// Returns the candidate feature names for a given tier (before leakage filtering).
pub fn tier_candidate_features(tier: &str) -> Vec<String> {
    match tier {
        "Tier1_Field" => vec![
            "pH".into(),
            "Temp.".into(),
            "EC".into(),
            "Dx".into(),
            "Dy".into(),
        ],
        "Tier2_Reduced" => vec![
            "pH".into(),
            "Temp.".into(),
            "EC".into(),
            "Dx".into(),
            "Dy".into(),
            "Na".into(),
            "K".into(),
            "Mg".into(),
            "Ca".into(),
            "Cl".into(),
            "HCO3".into(),
            "SO4".into(),
        ],
        "Tier3_Full" => vec![
            "pH".into(),
            "Temp.".into(),
            "EC".into(),
            "Dx".into(),
            "Dy".into(),
            "Na".into(),
            "K".into(),
            "Mg".into(),
            "Ca".into(),
            "Cl".into(),
            "HCO3".into(),
            "SO4".into(),
            "CO3".into(),
            "NO3".into(),
            "F".into(),
            "B".into(),
            // Derived ratios — only added for Tier 3
            "Na_Cl_ratio".into(),
            "Mg_Ca_ratio".into(),
            "HCO3_Cl_ratio".into(),
            "SAR".into(),
            "Ca_Mg_ratio".into(),
        ],
        _ => vec![],
    }
}

/// Store candidate features for all tiers in SQLite.
pub fn store_candidate_features(conn: &Connection) -> Result<()> {
    let tiers = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"];
    for tier in &tiers {
        for feat in &tier_candidate_features(tier) {
            let (feat_type, formula) = classify_feature(feat);
            conn.execute(
                "INSERT OR IGNORE INTO candidate_features
                 (feature_name, source_variable, tier_name, feature_type, formula_if_derived, enabled)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                rusqlite::params![feat, feat, tier, feat_type, formula],
            )?;
        }
    }
    Ok(())
}

fn classify_feature(feat: &str) -> (&'static str, Option<String>) {
    match feat {
        "Na_Cl_ratio" => ("derived_ratio", Some("Na/Cl".into())),
        "Mg_Ca_ratio" => ("derived_ratio", Some("Mg/Ca".into())),
        "HCO3_Cl_ratio" => ("derived_ratio", Some("HCO3/Cl".into())),
        "SAR" => ("derived_ratio", Some("Na/sqrt(0.5*(Ca+Mg))".into())),
        "Ca_Mg_ratio" => ("derived_ratio", Some("Ca/Mg".into())),
        "Dx" | "Dy" => ("coordinate", None),
        _ => ("measured", None),
    }
}

// ── Feature matrix construction from SQLite ─────────────────────────────────

/// Load cleaned feature values for a set of samples and features.
/// Returns (sample_ids, feature_matrix) — one row per sample.
pub fn load_feature_matrix(
    conn: &Connection,
    sample_ids: &[String],
    feature_names: &[String],
) -> Result<Vec<Vec<f64>>> {
    // Build a lookup map: sample_id -> variable_name -> value
    let mut values: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for sid in sample_ids {
        let mut stmt = conn.prepare_cached(
            "SELECT variable_name, cleaned_value_real
             FROM cleaned_measurements
             WHERE sample_id = ?1 AND cleaned_value_real IS NOT NULL",
        )?;
        let rows = stmt.query_map(rusqlite::params![sid], |row| {
            let var: String = row.get(0)?;
            let val: f64 = row.get(1)?;
            Ok((var, val))
        })?;
        let entry = values.entry(sid.clone()).or_default();
        for row in rows {
            let (var, val) = row?;
            entry.insert(var, val);
        }
    }

    // Build matrix; missing values become NaN (handled by imputer)
    let matrix = sample_ids
        .iter()
        .map(|sid| {
            let row_map = values.get(sid).cloned().unwrap_or_default();
            feature_names
                .iter()
                .map(|f| {
                    // Derived features
                    compute_derived(f, &row_map).unwrap_or(f64::NAN)
                })
                .collect::<Vec<f64>>()
        })
        .collect();

    Ok(matrix)
}

fn compute_derived(feat: &str, vals: &HashMap<String, f64>) -> Option<f64> {
    match feat {
        "Na_Cl_ratio" => {
            let na = vals.get("Na")?;
            let cl = vals.get("Cl").filter(|&&v| v > 0.0)?;
            Some(na / cl)
        }
        "Mg_Ca_ratio" => {
            let mg = vals.get("Mg")?;
            let ca = vals.get("Ca").filter(|&&v| v > 0.0)?;
            Some(mg / ca)
        }
        "HCO3_Cl_ratio" => {
            let h = vals.get("HCO3")?;
            let cl = vals.get("Cl").filter(|&&v| v > 0.0)?;
            Some(h / cl)
        }
        "SAR" => {
            let na = vals.get("Na")?;
            let ca = vals.get("Ca")?;
            let mg = vals.get("Mg")?;
            // SAR requires meq/L: Na/22.99, Ca/(40.08/2)=Ca/20.04, Mg/(24.31/2)=Mg/12.155
            let na_meq = na / 22.99;
            let ca_meq = ca / 20.04;
            let mg_meq = mg / 12.155;
            let denom = (0.5 * (ca_meq + mg_meq)).sqrt();
            if denom == 0.0 {
                return None;
            }
            Some(na_meq / denom)
        }
        "Ca_Mg_ratio" => {
            let ca = vals.get("Ca")?;
            let mg = vals.get("Mg").filter(|&&v| v > 0.0)?;
            Some(ca / mg)
        }
        other => vals.get(other).copied(),
    }
}

/// Compute and store derived ratios for all samples in cleaned_measurements.
pub fn compute_and_store_derived_features(conn: &Connection) -> Result<()> {
    let mut stmt_sids =
        conn.prepare("SELECT sample_id FROM raw_samples ORDER BY original_row_index")?;
    let sample_ids: Vec<String> = stmt_sids
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;

    let derived = [
        ("Na_Cl_ratio", "Na/Cl", "mg/L ratio"),
        ("Mg_Ca_ratio", "Mg/Ca", "mg/L ratio"),
        ("HCO3_Cl_ratio", "HCO3/Cl", "mg/L ratio"),
        ("SAR", "Na/sqrt(0.5*(Ca+Mg))", "(meq/L)^0.5"),
        ("Ca_Mg_ratio", "Ca/Mg", "mg/L ratio"),
    ];

    for sid in &sample_ids {
        // Load current values
        let mut vals: HashMap<String, f64> = HashMap::new();
        let mut stmt = conn.prepare_cached(
            "SELECT variable_name, cleaned_value_real FROM cleaned_measurements
             WHERE sample_id = ?1 AND cleaned_value_real IS NOT NULL",
        )?;
        for row in stmt.query_map(rusqlite::params![sid], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        })? {
            let (k, v) = row?;
            vals.insert(k, v);
        }

        for (feat, formula, unit) in &derived {
            if let Some(val) = compute_derived(feat, &vals) {
                if val.is_finite() {
                    conn.execute(
                        "INSERT OR REPLACE INTO cleaned_measurements
                         (sample_id, variable_name, raw_value_text, cleaned_value_real,
                          unit, bdl_flag, missing_flag, cleaning_note)
                         VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, ?6)",
                        rusqlite::params![
                            sid,
                            feat,
                            formula,
                            val,
                            unit,
                            "Derived ratio computed post-ingestion",
                        ],
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Get all feature names that passed leakage filtering for a target/tier combo.
///
/// `storage_tier` is the name stored in the DB audit table.  For special TDS
/// variants pass the effective tier (e.g. "Tier2_Reduced_TDS_EC_strict") so
/// that the strict variant does not overwrite the standard model's audit row.
pub fn get_filtered_features(
    conn: &Connection,
    target: &str,
    tier: &str,
    storage_tier: &str,
    rules: &LeakageRulesConfig,
    special_variant: Option<&str>,
) -> Result<Vec<String>> {
    let candidates = tier_candidate_features(tier);
    let (included, _excluded) = apply_leakage_filter(
        &candidates,
        target,
        tier,
        storage_tier,
        rules,
        special_variant,
        conn,
    )?;
    Ok(included)
}
