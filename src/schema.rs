use anyhow::Result;
use rusqlite::Connection;

pub fn create_all_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)?;
    Ok(())
}

const DDL: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- =========================================================
-- INPUT & AUDIT TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS project_manifest (
    manifest_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    project_name     TEXT NOT NULL,
    project_version  TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    software_version TEXT NOT NULL,
    db_schema_version TEXT NOT NULL,
    notes            TEXT
);

CREATE TABLE IF NOT EXISTS input_files (
    file_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    file_role     TEXT NOT NULL UNIQUE,
    file_path     TEXT NOT NULL,
    sha256_hash   TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    imported_at   TEXT NOT NULL,
    notes         TEXT
);

CREATE TABLE IF NOT EXISTS manuscript_outline (
    outline_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id       INTEGER REFERENCES input_files(file_id),
    title_detected TEXT,
    full_text     TEXT NOT NULL,
    imported_at   TEXT NOT NULL,
    sha256_hash   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS raw_samples (
    sample_id         TEXT PRIMARY KEY,
    original_row_index INTEGER NOT NULL,
    raw_json          TEXT NOT NULL,
    imported_at       TEXT NOT NULL,
    input_file_hash   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS column_dictionary (
    column_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    column_name        TEXT NOT NULL UNIQUE,
    inferred_type      TEXT NOT NULL,
    scientific_role    TEXT,
    unit               TEXT,
    is_coordinate      INTEGER NOT NULL DEFAULT 0,
    is_target_candidate INTEGER NOT NULL DEFAULT 0,
    is_field_variable  INTEGER NOT NULL DEFAULT 0,
    is_lab_variable    INTEGER NOT NULL DEFAULT 0,
    is_derived_variable INTEGER NOT NULL DEFAULT 0,
    notes              TEXT
);

CREATE TABLE IF NOT EXISTS cleaned_measurements (
    measurement_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    sample_id          TEXT NOT NULL REFERENCES raw_samples(sample_id),
    variable_name      TEXT NOT NULL,
    raw_value_text     TEXT,
    cleaned_value_real REAL,
    unit               TEXT,
    bdl_flag           INTEGER NOT NULL DEFAULT 0,
    bdl_rule           TEXT,
    missing_flag       INTEGER NOT NULL DEFAULT 0,
    cleaning_note      TEXT,
    UNIQUE(sample_id, variable_name)
);

CREATE TABLE IF NOT EXISTS data_audit (
    audit_id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_timestamp           TEXT NOT NULL,
    n_samples                 INTEGER NOT NULL,
    n_columns                 INTEGER NOT NULL,
    missing_coordinate_count  INTEGER NOT NULL,
    duplicate_sample_count    INTEGER NOT NULL,
    bdl_value_count           INTEGER NOT NULL,
    failed_numeric_parse_count INTEGER NOT NULL,
    audit_status              TEXT NOT NULL,
    audit_message             TEXT
);

-- =========================================================
-- TARGET & FEATURE TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS target_definitions (
    target_id            INTEGER PRIMARY KEY AUTOINCREMENT,
    target_contaminant   TEXT NOT NULL UNIQUE,
    source_column        TEXT NOT NULL,
    threshold_value      REAL,
    threshold_unit       TEXT,
    threshold_source     TEXT,
    exceedance_direction TEXT NOT NULL DEFAULT 'greater_than',
    notes                TEXT
);

CREATE TABLE IF NOT EXISTS target_labels (
    label_id            INTEGER PRIMARY KEY AUTOINCREMENT,
    target_contaminant  TEXT NOT NULL,
    sample_id           TEXT NOT NULL REFERENCES raw_samples(sample_id),
    measured_value      REAL,
    threshold_value     REAL,
    true_label          INTEGER,
    label_status        TEXT NOT NULL,
    UNIQUE(target_contaminant, sample_id)
);

CREATE TABLE IF NOT EXISTS target_eligibility (
    target_contaminant     TEXT PRIMARY KEY,
    n_samples              INTEGER NOT NULL,
    n_positive             INTEGER NOT NULL,
    n_negative             INTEGER NOT NULL,
    prevalence             REAL NOT NULL,
    cv_feasible            INTEGER NOT NULL,
    ml_status              TEXT NOT NULL,
    reporting_level        TEXT NOT NULL,
    eligibility_note       TEXT
);

CREATE TABLE IF NOT EXISTS predictor_tiers (
    tier_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tier_name        TEXT NOT NULL UNIQUE,
    tier_description TEXT,
    intended_use     TEXT
);

CREATE TABLE IF NOT EXISTS candidate_features (
    feature_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_name     TEXT NOT NULL,
    source_variable  TEXT,
    tier_name        TEXT NOT NULL REFERENCES predictor_tiers(tier_name),
    feature_type     TEXT NOT NULL,
    formula_if_derived TEXT,
    unit             TEXT,
    enabled          INTEGER NOT NULL DEFAULT 1,
    UNIQUE(feature_name, tier_name)
);

CREATE TABLE IF NOT EXISTS leakage_rules_applied (
    rule_id             INTEGER PRIMARY KEY AUTOINCREMENT,
    target_contaminant  TEXT NOT NULL,
    tier_name           TEXT NOT NULL,
    feature_name        TEXT NOT NULL,
    action              TEXT NOT NULL,
    reason              TEXT,
    rule_source         TEXT,
    UNIQUE(target_contaminant, tier_name, feature_name)
);

-- =========================================================
-- SPATIAL TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS spatial_cluster_metadata (
    spatial_cluster_id              INTEGER PRIMARY KEY,
    centroid_latitude               REAL NOT NULL,
    centroid_longitude              REAL NOT NULL,
    number_of_wells                 INTEGER NOT NULL,
    max_intra_cluster_distance_km   REAL
);

CREATE TABLE IF NOT EXISTS sample_spatial_assignment (
    sample_id           TEXT PRIMARY KEY REFERENCES raw_samples(sample_id),
    latitude            REAL NOT NULL,
    longitude           REAL NOT NULL,
    spatial_cluster_id  INTEGER NOT NULL REFERENCES spatial_cluster_metadata(spatial_cluster_id)
);

-- =========================================================
-- CROSS-VALIDATION TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS fold_assignments (
    assignment_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    cv_mode            TEXT NOT NULL,
    repeat_index       INTEGER NOT NULL,
    fold_index         INTEGER NOT NULL,
    sample_id          TEXT NOT NULL REFERENCES raw_samples(sample_id),
    split_role         TEXT NOT NULL,
    target_contaminant TEXT NOT NULL,
    tier_name          TEXT NOT NULL,
    random_seed        INTEGER NOT NULL,
    UNIQUE(cv_mode, repeat_index, fold_index, sample_id, target_contaminant, tier_name)
);

-- =========================================================
-- MODEL TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS model_runs (
    run_id               TEXT PRIMARY KEY,
    timestamp            TEXT NOT NULL,
    target_contaminant   TEXT NOT NULL,
    predictor_tier       TEXT NOT NULL,
    algorithm            TEXT NOT NULL,
    cv_mode              TEXT NOT NULL,
    threshold_value      REAL,
    threshold_source     TEXT,
    random_seed          INTEGER NOT NULL,
    git_commit_hash      TEXT,
    input_data_hash      TEXT,
    outline_hash         TEXT,
    leakage_rules_hash   TEXT,
    config_hash          TEXT,
    n_features           INTEGER,
    n_train_samples      INTEGER,
    n_outer_folds        INTEGER,
    n_repeats            INTEGER,
    cpu_threads_used     INTEGER,
    total_execution_time_ms INTEGER,
    run_status           TEXT NOT NULL DEFAULT 'completed',
    skip_reason          TEXT
);

CREATE TABLE IF NOT EXISTS model_hyperparameters (
    hp_id               INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id              TEXT NOT NULL REFERENCES model_runs(run_id),
    repeat_index        INTEGER NOT NULL,
    fold_index          INTEGER NOT NULL,
    hyperparameter_name TEXT NOT NULL,
    hyperparameter_value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fold_base_values (
    bv_id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id                 TEXT NOT NULL REFERENCES model_runs(run_id),
    cv_mode                TEXT NOT NULL,
    repeat_index           INTEGER NOT NULL,
    fold_index             INTEGER NOT NULL,
    shap_base_value_logit  REAL,
    shap_base_value_prob   REAL,
    train_fold_prevalence  REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS fold_metrics (
    metric_id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id                 TEXT NOT NULL REFERENCES model_runs(run_id),
    cv_mode                TEXT NOT NULL,
    repeat_index           INTEGER NOT NULL,
    fold_index             INTEGER NOT NULL,
    roc_auc                REAL,
    pr_auc                 REAL,
    balanced_accuracy      REAL,
    recall_sensitivity     REAL,
    specificity            REAL,
    f1_score               REAL,
    f2_score               REAL,
    brier_score            REAL,
    calibration_slope      REAL,
    calibration_intercept  REAL,
    n_test_pos             INTEGER,
    n_test_neg             INTEGER,
    train_time_ms          INTEGER,
    shap_computation_time_ms INTEGER
);

CREATE TABLE IF NOT EXISTS well_predictions (
    pred_id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id                        TEXT NOT NULL REFERENCES model_runs(run_id),
    sample_id                     TEXT NOT NULL REFERENCES raw_samples(sample_id),
    target_contaminant            TEXT NOT NULL,
    predictor_tier                TEXT NOT NULL,
    algorithm                     TEXT NOT NULL,
    cv_mode                       TEXT NOT NULL,
    repeat_index                  INTEGER NOT NULL,
    fold_index                    INTEGER NOT NULL,
    true_label                    INTEGER,
    predicted_probability         REAL NOT NULL,
    predicted_label_default_0_5   INTEGER NOT NULL,
    spatial_cluster_id            INTEGER
);

CREATE TABLE IF NOT EXISTS fold_feature_tracking (
    fft_id             INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id             TEXT NOT NULL REFERENCES model_runs(run_id),
    cv_mode            TEXT NOT NULL,
    repeat_index       INTEGER NOT NULL,
    fold_index         INTEGER NOT NULL,
    feature_name       TEXT NOT NULL,
    is_selected        INTEGER NOT NULL DEFAULT 1,
    mean_absolute_shap REAL,
    selection_reason   TEXT
);

CREATE TABLE IF NOT EXISTS shap_values (
    shap_id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id               TEXT NOT NULL REFERENCES model_runs(run_id),
    sample_id            TEXT NOT NULL REFERENCES raw_samples(sample_id),
    cv_mode              TEXT NOT NULL,
    repeat_index         INTEGER NOT NULL,
    fold_index           INTEGER NOT NULL,
    feature_name         TEXT NOT NULL,
    shap_value           REAL NOT NULL,
    feature_raw_value    REAL,
    feature_cleaned_value REAL
);

-- =========================================================
-- THRESHOLD & EXPORT TABLES
-- =========================================================

CREATE TABLE IF NOT EXISTS operational_thresholds (
    ot_id                         INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id                        TEXT NOT NULL REFERENCES model_runs(run_id),
    cv_mode                       TEXT NOT NULL,
    threshold_selection_rule      TEXT NOT NULL,
    optimized_probability_cutoff  REAL NOT NULL,
    resulting_sensitivity         REAL NOT NULL,
    resulting_specificity         REAL NOT NULL,
    false_negatives_count         INTEGER NOT NULL,
    false_positives_count         INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS threshold_sensitivity (
    ts_id                INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id               TEXT NOT NULL REFERENCES model_runs(run_id),
    cv_mode              TEXT NOT NULL,
    probability_cutoff   REAL NOT NULL,
    sensitivity          REAL NOT NULL,
    specificity          REAL NOT NULL,
    precision            REAL,
    f1_score             REAL,
    f2_score             REAL,
    false_negatives_count INTEGER NOT NULL,
    false_positives_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS screening_priority (
    sp_id                        INTEGER PRIMARY KEY AUTOINCREMENT,
    sample_id                    TEXT NOT NULL REFERENCES raw_samples(sample_id),
    target_contaminant           TEXT NOT NULL,
    best_validated_run_id        TEXT REFERENCES model_runs(run_id),
    predicted_probability_median REAL,
    predicted_probability_lower_ci REAL,
    predicted_probability_upper_ci REAL,
    screening_priority_class     TEXT,
    priority_reason              TEXT,
    UNIQUE(sample_id, target_contaminant)
);

CREATE TABLE IF NOT EXISTS gis_exports_manifest (
    export_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    export_file   TEXT NOT NULL,
    export_type   TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    source_query  TEXT,
    row_count     INTEGER,
    sha256_hash   TEXT
);

-- =========================================================
-- INDEXES FOR FAST R QUERYING
-- =========================================================

CREATE INDEX IF NOT EXISTS idx_cleaned_sample    ON cleaned_measurements(sample_id);
CREATE INDEX IF NOT EXISTS idx_cleaned_variable  ON cleaned_measurements(variable_name);
CREATE INDEX IF NOT EXISTS idx_labels_target     ON target_labels(target_contaminant);
CREATE INDEX IF NOT EXISTS idx_labels_sample     ON target_labels(sample_id);
CREATE INDEX IF NOT EXISTS idx_fold_assign_run   ON fold_assignments(target_contaminant, tier_name, cv_mode);
CREATE INDEX IF NOT EXISTS idx_fold_metrics_run  ON fold_metrics(run_id);
CREATE INDEX IF NOT EXISTS idx_well_pred_run     ON well_predictions(run_id);
CREATE INDEX IF NOT EXISTS idx_well_pred_sample  ON well_predictions(sample_id);
CREATE INDEX IF NOT EXISTS idx_shap_run          ON shap_values(run_id);
CREATE INDEX IF NOT EXISTS idx_shap_feature      ON shap_values(feature_name);
CREATE INDEX IF NOT EXISTS idx_model_runs_target ON model_runs(target_contaminant, predictor_tier, algorithm);
CREATE INDEX IF NOT EXISTS idx_screening_target  ON screening_priority(target_contaminant);
"#;
