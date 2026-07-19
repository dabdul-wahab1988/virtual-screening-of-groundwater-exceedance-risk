use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

// ── CSV ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CsvDataset {
    pub headers: Vec<String>,
    pub rows: Vec<HashMap<String, String>>,
    pub n_rows: usize,
    pub n_cols: usize,
}

pub fn read_csv(path: &str) -> Result<CsvDataset> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(false)
        .from_path(path)?;

    let headers: Vec<String> = rdr
        .headers()?
        .iter()
        .map(|s| s.trim().to_string())
        .collect();

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let mut row = HashMap::new();
        for (i, val) in record.iter().enumerate() {
            if let Some(h) = headers.get(i) {
                row.insert(h.clone(), val.trim().to_string());
            }
        }
        rows.push(row);
    }

    let n_rows = rows.len();
    let n_cols = headers.len();
    Ok(CsvDataset {
        headers,
        rows,
        n_rows,
        n_cols,
    })
}

// ── RUNTIME CONFIG ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuntimeConfig {
    pub project: ProjectMeta,
    pub paths: Paths,
    pub cv: CvSettings,
    pub spatial: SpatialSettings,
    pub models: ModelToggles,
    pub outputs: OutputSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Paths {
    pub input_csv: String,
    pub manuscript_outline: String,
    pub thresholds_yaml: String,
    pub leakage_rules_yaml: String,
    pub sqlite_db: String,
    pub logs_dir: String,
    pub gis_exports_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CvSettings {
    pub outer_folds: usize,
    pub spatial_outer_folds: Option<usize>,
    pub inner_folds: usize,
    pub repeats: usize,
    pub random_seed: u64,
    pub minimum_positive_cases_for_ml: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpatialSettings {
    pub latitude_column: String,
    pub longitude_column: String,
    pub clustering_method: String,
    pub number_of_spatial_clusters: Option<usize>,
    pub max_clusters: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelToggles {
    pub run_majority_dummy: bool,
    pub run_stratified_dummy: bool,
    pub run_ec_only_logistic: bool,
    pub run_regularized_logistic: bool,
    pub run_random_forest: bool,
    pub run_gradient_boosted_tree: bool,
    pub run_svm: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputSettings {
    pub write_fold_predictions: bool,
    pub write_shap_values: bool,
    pub write_gis_exports: bool,
}

pub fn read_runtime_config(path: &str) -> Result<RuntimeConfig> {
    let text = fs::read_to_string(path)?;
    let cfg: RuntimeConfig =
        serde_yaml::from_str(&text).map_err(|e| anyhow!("Failed to parse config.yaml: {e}"))?;
    Ok(cfg)
}

// ── THRESHOLDS CONFIG ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThresholdsConfig {
    pub targets: HashMap<String, TargetThreshold>,
    #[serde(default)]
    pub bdl_substitution: BdlSubstitutionConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetThreshold {
    pub column: String,
    pub unit: String,
    pub threshold_value: Option<f64>,
    pub threshold_source: String,
    pub exceedance_direction: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BdlSubstitutionConfig {
    #[serde(default = "default_bdl_method")]
    pub method: String,
    #[serde(default = "default_true")]
    pub use_min_detected_fallback: bool,
    #[serde(default)]
    pub detection_limits: HashMap<String, DetectionLimit>,
}

impl Default for BdlSubstitutionConfig {
    fn default() -> Self {
        Self {
            method: default_bdl_method(),
            use_min_detected_fallback: true,
            detection_limits: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DetectionLimit {
    pub value: Option<f64>,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub source: String,
}

fn default_bdl_method() -> String {
    "limit_div_sqrt2".to_string()
}

fn default_true() -> bool {
    true
}

pub fn read_thresholds_config(path: &str) -> Result<ThresholdsConfig> {
    let text = fs::read_to_string(path)?;
    let cfg: ThresholdsConfig =
        serde_yaml::from_str(&text).map_err(|e| anyhow!("Failed to parse thresholds.yaml: {e}"))?;
    Ok(cfg)
}

// ── LEAKAGE-RULES CONFIG ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeakageRulesConfig {
    pub global_forbidden_patterns: Vec<String>,
    pub targets: HashMap<String, TargetLeakageRule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetLeakageRule {
    #[serde(default)]
    pub forbidden_exact: Vec<String>,
    #[serde(default)]
    pub forbidden_contains: Vec<String>,
    pub special_models: Option<HashMap<String, SpecialModel>>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpecialModel {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub forbidden_exact: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

pub fn read_leakage_rules(path: &str) -> Result<LeakageRulesConfig> {
    let text = fs::read_to_string(path)?;
    let cfg: LeakageRulesConfig = serde_yaml::from_str(&text)
        .map_err(|e| anyhow!("Failed to parse leakage_rules.yaml: {e}"))?;
    Ok(cfg)
}

// ── TEXT FILE ─────────────────────────────────────────────────────────────────

pub fn read_text_file(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
