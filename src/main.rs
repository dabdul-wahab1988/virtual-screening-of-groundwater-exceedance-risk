mod calibration;
mod cli;
mod cv;
mod export;
mod features;
mod io;
mod leakage;
mod manifest;
mod metrics;
mod models;
mod outline;
mod preprocessing;
mod schema;
mod shap;
mod spatial;
mod targets;
mod thresholds;
mod thresholds_operational;
mod utils;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::time::Instant;
use utils::now_iso8601;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::ValidateInputs {
            data,
            outline,
            thresholds,
            leakage,
            config,
        } => cmd_validate_inputs(&data, &outline, &thresholds, &leakage, &config),
        Commands::InitDb { db } => cmd_init_db(&db),
        Commands::IngestInputs {
            data,
            outline,
            thresholds,
            leakage,
            config,
            db,
        } => cmd_ingest_inputs(&data, &outline, &thresholds, &leakage, &config, &db),
        Commands::AuditLeakage { db } => cmd_audit_leakage(&db),
        Commands::RunTarget {
            target,
            tier,
            cv_mode,
            db,
        } => cmd_run_target(&target, &tier, &cv_mode, &db),
        Commands::RunPipeline { db, config } => cmd_run_pipeline(&db, &config),
        Commands::ExportR { db, out } => cmd_export_r(&db, &out),
        Commands::ExportGis { db, out } => cmd_export_gis(&db, &out),
    }
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: validate-inputs
// ═══════════════════════════════════════════════════════════════════

fn cmd_validate_inputs(
    data: &str,
    outline: &str,
    thresholds: &str,
    leakage: &str,
    config: &str,
) -> Result<()> {
    log::info!("=== validate-inputs ===");
    let mut all_ok = true;

    for (label, path) in &[
        ("raw CSV", data),
        ("outline", outline),
        ("thresholds", thresholds),
        ("leakage rules", leakage),
        ("config", config),
    ] {
        if utils::file_exists(path) {
            let hash = utils::sha256_file(path)?;
            let size = utils::file_size_bytes(path)?;
            log::info!(
                "  [OK] {} — {} ({} bytes, sha256={}...)",
                label,
                path,
                size,
                &hash[..16]
            );
        } else {
            log::error!("  [MISSING] {} not found: {}", label, path);
            all_ok = false;
        }
    }

    // CSV inspection
    if utils::file_exists(data) {
        let dataset = io::read_csv(data)?;
        log::info!(
            "  CSV: {} rows × {} columns",
            dataset.n_rows,
            dataset.n_cols
        );
        log::info!("  Columns: {}", dataset.headers.join(", "));

        let required_cols = ["SampleID", "Dx", "Dy", "Na", "Cl", "TDS", "F", "NO3"];
        for col in &required_cols {
            if dataset.headers.iter().any(|h| h == col) {
                log::info!("  [OK] Required column present: {}", col);
            } else {
                log::warn!("  [WARN] Required column missing: {}", col);
            }
        }

        // Check B column
        if !dataset.headers.contains(&"B".to_string()) {
            log::warn!("  [WARN] Column 'B' (Boron) not found in CSV");
        }

        // BDL count
        let bdl_count: usize = dataset
            .rows
            .iter()
            .flat_map(|r| r.values())
            .filter(|v| v.trim().eq_ignore_ascii_case("bdl"))
            .count();
        log::info!("  BDL values detected: {}", bdl_count);
    }

    // Thresholds YAML
    if utils::file_exists(thresholds) {
        let cfg = io::read_thresholds_config(thresholds)?;
        for target in &["Na", "Cl", "TDS", "B", "F", "NO3"] {
            if let Some(t) = cfg.targets.get(*target) {
                match t.threshold_value {
                    Some(v) => log::info!("  [OK] Threshold {}: {} {}", target, v, t.unit),
                    None => log::warn!(
                        "  [WARN] Threshold {} has null value — target will be skipped",
                        target
                    ),
                }
            } else {
                log::warn!("  [WARN] Target {} not found in thresholds YAML", target);
            }
        }
    }

    if all_ok {
        log::info!("Validation PASSED — all inputs present");
    } else {
        return Err(anyhow!("Validation FAILED — missing input files"));
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: init-db
// ═══════════════════════════════════════════════════════════════════

fn cmd_init_db(db_path: &str) -> Result<()> {
    log::info!("=== init-db: {} ===", db_path);
    utils::ensure_dir(
        std::path::Path::new(db_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("."),
    )?;
    let conn = Connection::open(db_path)?;
    schema::create_all_tables(&conn).context("Failed to create schema tables")?;
    log::info!("Database initialised: {}", db_path);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: ingest-inputs
// ═══════════════════════════════════════════════════════════════════

fn cmd_ingest_inputs(
    data: &str,
    outline_path: &str,
    thresholds_path: &str,
    leakage_path: &str,
    config_path: &str,
    db_path: &str,
) -> Result<()> {
    log::info!("=== ingest-inputs ===");
    let conn = open_db(db_path)?;

    // ── Register input files ────────────────────────────────────────
    let csv_hash = manifest::register_input_file(
        &conn,
        &manifest::FileEntry {
            role: "raw_groundwater_csv",
            path: data,
            notes: "Primary analytical dataset",
        },
    )?;
    manifest::register_input_file(
        &conn,
        &manifest::FileEntry {
            role: "manuscript_outline",
            path: outline_path,
            notes: "Scientific analysis plan",
        },
    )?;
    manifest::register_input_file(
        &conn,
        &manifest::FileEntry {
            role: "thresholds_yaml",
            path: thresholds_path,
            notes: "Exceedance thresholds",
        },
    )?;
    manifest::register_input_file(
        &conn,
        &manifest::FileEntry {
            role: "leakage_rules_yaml",
            path: leakage_path,
            notes: "Target-wise leakage rules",
        },
    )?;
    manifest::register_input_file(
        &conn,
        &manifest::FileEntry {
            role: "runtime_config_yaml",
            path: config_path,
            notes: "Runtime configuration",
        },
    )?;
    log::info!("Input files registered");

    // ── Project manifest ────────────────────────────────────────────
    let cfg = io::read_runtime_config(config_path)?;
    let thr_cfg = io::read_thresholds_config(thresholds_path)?;
    manifest::register_project_manifest(&conn, &cfg.project.name, &cfg.project.version)?;

    // ── Store outline ───────────────────────────────────────────────
    let outline_text = io::read_text_file(outline_path)?;
    outline::store_outline(&conn, &outline_text)?;
    log::info!("Outline stored ({} chars)", outline_text.len());

    // ── Ingest raw CSV ──────────────────────────────────────────────
    let dataset = io::read_csv(data)?;
    log::info!(
        "CSV loaded: {} samples × {} columns",
        dataset.n_rows,
        dataset.n_cols
    );

    targets::ingest_raw_samples(&conn, &dataset, &csv_hash)?;
    let (bdl_count, _missing, parse_errors) =
        targets::ingest_cleaned_measurements(&conn, &dataset, &thr_cfg)?;
    log::info!(
        "Cleaned measurements ingested (bdl={}, errors={})",
        bdl_count,
        parse_errors
    );

    // ── Column dictionary ───────────────────────────────────────────
    let metas = features::build_column_metadata(&dataset.headers);
    features::store_column_dictionary(&conn, &metas)?;

    // ── Data audit ──────────────────────────────────────────────────
    targets::run_data_audit(
        &conn,
        &dataset,
        bdl_count,
        parse_errors,
        &cfg.spatial.latitude_column,
        &cfg.spatial.longitude_column,
    )?;

    // ── Threshold definitions ───────────────────────────────────────
    thresholds::store_threshold_definitions(&conn, &thr_cfg)?;
    thresholds::generate_and_store_labels(&conn, &thr_cfg)?;
    thresholds::compute_and_store_eligibility(&conn, cfg.cv.minimum_positive_cases_for_ml)?;
    log::info!("Threshold definitions and labels stored");

    // ── Predictor tiers and candidate features ──────────────────────
    leakage::store_predictor_tiers(&conn)?;
    features::store_candidate_features(&conn)?;

    // ── Derived ratio features ──────────────────────────────────────
    features::compute_and_store_derived_features(&conn)?;
    log::info!("Derived ratio features computed");

    // ── Spatial clustering ──────────────────────────────────────────
    spatial::build_and_store_spatial_clusters(
        &conn,
        &cfg.spatial.latitude_column,
        &cfg.spatial.longitude_column,
        cfg.spatial.max_clusters,
        cfg.spatial.number_of_spatial_clusters,
    )?;

    log::info!("Ingestion complete. DB: {}", db_path);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: audit-leakage
// ═══════════════════════════════════════════════════════════════════

fn cmd_audit_leakage(db_path: &str) -> Result<()> {
    log::info!("=== audit-leakage ===");
    let conn = open_db(db_path)?;

    // Load leakage rules from DB file path
    let leakage_path: String = conn.query_row(
        "SELECT file_path FROM input_files WHERE file_role = 'leakage_rules_yaml'",
        [],
        |r| r.get(0),
    )?;
    let rules = io::read_leakage_rules(&leakage_path)?;

    let targets = ["Na", "Cl", "TDS", "B", "F", "NO3"];
    let tiers = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"];

    for target in &targets {
        for tier in &tiers {
            let candidates = features::tier_candidate_features(tier);
            // storage_tier == tier for standard models (no overwrite risk)
            let (included, excluded) = leakage::apply_leakage_filter(
                &candidates,
                target,
                tier,
                tier,
                &rules,
                None,
                &conn,
            )?;
            log::info!(
                "  {} / {}: {} included, {} excluded",
                target,
                tier,
                included.len(),
                excluded.len()
            );
            if included.is_empty() {
                log::warn!(
                    "  WARNING: No features remain for {} / {} after leakage filtering!",
                    target,
                    tier
                );
            }
        }
        // TDS special models — use the full effective tier name for storage
        if *target == "TDS" {
            for variant in &["TDS_EC_inclusive", "TDS_EC_strict"] {
                let storage_tier = format!("Tier2_Reduced_{}", variant);
                let candidates = features::tier_candidate_features("Tier2_Reduced");
                let (inc, exc) = leakage::apply_leakage_filter(
                    &candidates,
                    target,
                    "Tier2_Reduced",
                    &storage_tier,
                    &rules,
                    Some(variant),
                    &conn,
                )?;
                log::info!(
                    "  TDS / {} [{}]: {} included, {} excluded",
                    "Tier2_Reduced",
                    variant,
                    inc.len(),
                    exc.len()
                );
            }
        }
    }
    log::info!("Leakage audit complete");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: run-target
// ═══════════════════════════════════════════════════════════════════

fn cmd_run_target(target: &str, tier: &str, cv_mode: &str, db_path: &str) -> Result<()> {
    log::info!("=== run-target: {} / {} / {} ===", target, tier, cv_mode);
    let conn = open_db(db_path)?;

    // Check eligibility
    if !thresholds::is_target_eligible(&conn, target)? {
        log::warn!("Target '{}' is not eligible for ML — skipping", target);
        return Ok(());
    }

    // Load config
    let config_path: String = conn.query_row(
        "SELECT file_path FROM input_files WHERE file_role = 'runtime_config_yaml'",
        [],
        |r| r.get(0),
    )?;
    let cfg = io::read_runtime_config(&config_path)?;

    let leakage_path: String = conn.query_row(
        "SELECT file_path FROM input_files WHERE file_role = 'leakage_rules_yaml'",
        [],
        |r| r.get(0),
    )?;
    let rules = io::read_leakage_rules(&leakage_path)?;

    run_target_internal(&conn, target, tier, cv_mode, &cfg, &rules)
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: run-pipeline
// ═══════════════════════════════════════════════════════════════════

fn cmd_run_pipeline(db_path: &str, config_path: &str) -> Result<()> {
    log::info!("=== run-pipeline ===");
    let conn = open_db(db_path)?;
    let cfg = io::read_runtime_config(config_path)?;

    let leakage_path: String = conn.query_row(
        "SELECT file_path FROM input_files WHERE file_role = 'leakage_rules_yaml'",
        [],
        |r| r.get(0),
    )?;
    let rules = io::read_leakage_rules(&leakage_path)?;

    let targets = ["Na", "Cl", "TDS", "B", "F", "NO3"];
    let tiers = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"];
    let cv_modes = ["Stratified_Nested_CV", "Spatial_Group_CV"];

    let mut run_count = 0;
    let pipeline_start = Instant::now();

    for target in &targets {
        match thresholds::is_target_eligible(&conn, target) {
            Ok(true) => {}
            Ok(false) => {
                log::info!("Skipping {} — not eligible for ML", target);
                continue;
            }
            Err(e) => {
                log::warn!("Cannot check eligibility for {}: {}", target, e);
                continue;
            }
        }

        for tier in &tiers {
            for cv_mode in &cv_modes {
                log::info!("  Running: {} / {} / {}", target, tier, cv_mode);
                if let Err(e) = run_target_internal(&conn, target, tier, cv_mode, &cfg, &rules) {
                    log::error!("  FAILED {}/{}/{}: {}", target, tier, cv_mode, e);
                } else {
                    run_count += 1;
                }
            }
        }

        // TDS sensitivity model with EC variants
        if *target == "TDS" {
            for variant in &["TDS_EC_inclusive", "TDS_EC_strict"] {
                for cv_mode in &cv_modes {
                    let special_tier = format!("Tier2_Reduced_{}", variant);
                    log::info!("  Running TDS special: {} / {}", special_tier, cv_mode);
                    if let Err(e) = run_target_with_variant(
                        &conn,
                        "TDS",
                        "Tier2_Reduced",
                        cv_mode,
                        Some(variant),
                        &cfg,
                        &rules,
                    ) {
                        log::error!("  FAILED TDS/{}/{}: {}", special_tier, cv_mode, e);
                    } else {
                        run_count += 1;
                    }
                }
            }
        }
    }

    let elapsed_ms = pipeline_start.elapsed().as_millis();
    log::info!(
        "Pipeline complete: {} run groups in {}ms",
        run_count,
        elapsed_ms
    );

    // Write run manifest JSON
    let manifest = serde_json::json!({
        "pipeline_completed_at": now_iso8601(),
        "total_run_groups": run_count,
        "elapsed_ms": elapsed_ms,
        "targets": targets,
        "tiers": tiers,
        "cv_modes": cv_modes,
    });
    let manifest_path = "reports/run_manifest.json";
    utils::ensure_dir("reports")?;
    std::fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    log::info!("Run manifest written to {}", manifest_path);

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: export-r
// ═══════════════════════════════════════════════════════════════════

fn cmd_export_r(db_path: &str, out_dir: &str) -> Result<()> {
    log::info!("=== export-r ===");
    let conn = open_db(db_path)?;
    export::export_r_files(&conn, out_dir)
}

// ═══════════════════════════════════════════════════════════════════
// COMMAND: export-gis
// ═══════════════════════════════════════════════════════════════════

fn cmd_export_gis(db_path: &str, out_dir: &str) -> Result<()> {
    log::info!("=== export-gis ===");
    let conn = open_db(db_path)?;
    export::export_gis_files(&conn, out_dir)
}

// ═══════════════════════════════════════════════════════════════════
// INTERNAL: run_target_internal
// ═══════════════════════════════════════════════════════════════════

fn run_target_internal(
    conn: &Connection,
    target: &str,
    tier: &str,
    cv_mode: &str,
    cfg: &io::RuntimeConfig,
    rules: &io::LeakageRulesConfig,
) -> Result<()> {
    run_target_with_variant(conn, target, tier, cv_mode, None, cfg, rules)
}

fn run_target_with_variant(
    conn: &Connection,
    target: &str,
    tier: &str,
    cv_mode: &str,
    special_variant: Option<&str>,
    cfg: &io::RuntimeConfig,
    rules: &io::LeakageRulesConfig,
) -> Result<()> {
    use cv::{subset_ids, subset_rows, subset_y};
    use models::*;
    use preprocessing::FoldPreprocessor;

    let effective_tier = match special_variant {
        Some(v) => format!("{}_{}", tier, v),
        None => tier.to_string(),
    };

    // ── Load labels ─────────────────────────────────────────────────
    let labelled = targets::load_labelled_samples(conn, target)?;
    let sample_ids: Vec<String> = labelled.iter().map(|(s, _)| s.clone()).collect();
    let y: Vec<i32> = labelled.iter().map(|(_, l)| *l).collect();
    let n = sample_ids.len();
    let outer_folds = if cv_mode == "Spatial_Group_CV" {
        cfg.cv.spatial_outer_folds.unwrap_or(cfg.cv.outer_folds)
    } else {
        cfg.cv.outer_folds
    };

    if n < outer_folds * 2 {
        return Err(anyhow!(
            "Insufficient samples ({}) for {}-fold CV on target '{}'",
            n,
            outer_folds,
            target
        ));
    }

    // ── Feature set ─────────────────────────────────────────────────
    // Pass effective_tier as storage_tier so special variants don't overwrite
    // the standard model's audit record in leakage_rules_applied.
    let feature_names = features::get_filtered_features(
        conn,
        target,
        tier,
        &effective_tier,
        rules,
        special_variant,
    )?;
    if feature_names.is_empty() {
        return Err(anyhow!(
            "No features remain after leakage filtering for {}/{}",
            target,
            tier
        ));
    }

    // ── Build feature matrix ─────────────────────────────────────────
    let x_full = features::load_feature_matrix(conn, &sample_ids, &feature_names)?;
    let n_features = feature_names.len();

    // ── Generate CV folds ────────────────────────────────────────────
    let folds = match cv_mode {
        "Spatial_Group_CV" => {
            let assignments = spatial::load_cluster_assignments(conn)?;
            let folds = cv::spatial_group_kfold(&sample_ids, &assignments, outer_folds)?;
            cv::validate_binary_folds(&folds, &y, outer_folds, cv_mode)?;
            folds
        }
        _ => {
            let folds =
                cv::repeated_stratified_kfold(&y, outer_folds, cfg.cv.repeats, cfg.cv.random_seed)?;
            cv::validate_binary_folds(&folds, &y, outer_folds * cfg.cv.repeats, cv_mode)?;
            folds
        }
    };

    // ── Hashes for provenance ────────────────────────────────────────
    let csv_hash = manifest::get_file_hash(conn, "raw_groundwater_csv")?.unwrap_or_default();
    let outline_hash = manifest::get_file_hash(conn, "manuscript_outline")?.unwrap_or_default();
    let leakage_hash = manifest::get_file_hash(conn, "leakage_rules_yaml")?.unwrap_or_default();
    let config_hash = manifest::get_file_hash(conn, "runtime_config_yaml")?.unwrap_or_default();

    // ── Threshold value ──────────────────────────────────────────────
    let threshold_info: (Option<f64>, Option<String>) = conn.query_row(
        "SELECT threshold_value, threshold_source FROM target_definitions WHERE target_contaminant = ?1",
        rusqlite::params![target],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).unwrap_or((None, None));

    // ── Algorithm list ───────────────────────────────────────────────
    let algorithms: Vec<&str> = {
        let mut v = Vec::new();
        if cfg.models.run_majority_dummy {
            v.push("DummyMajority");
        }
        if cfg.models.run_stratified_dummy {
            v.push("DummyStratified");
        }
        // EC-only only for tiers containing EC and non-circular
        if cfg.models.run_ec_only_logistic && feature_names.contains(&"EC".to_string()) {
            v.push("EcOnlyLogistic");
        }
        if cfg.models.run_regularized_logistic {
            v.push("LogisticRegression");
        }
        if cfg.models.run_random_forest {
            v.push("RandomForest");
        }
        if cfg.models.run_gradient_boosted_tree {
            v.push("GradientBoostedTrees");
        }
        v
    };

    for algorithm in &algorithms {
        let run_start = Instant::now();

        // Generate a stable run_id
        let run_id = format!(
            "{}_{}_{}_{}",
            target,
            effective_tier.replace(' ', "_"),
            algorithm,
            cv_mode
        );
        let run_id = utils::sha256_str(&run_id)[..32].to_string();

        // Clean up any child rows from a prior partial run BEFORE replacing the
        // model_runs row (FK constraints would block INSERT OR REPLACE otherwise).
        for tbl in &[
            "fold_metrics",
            "well_predictions",
            "shap_values",
            "fold_base_values",
            "fold_feature_tracking",
            "model_hyperparameters",
            "operational_thresholds",
            "threshold_sensitivity",
        ] {
            conn.execute(
                &format!("DELETE FROM {} WHERE run_id = ?1", tbl),
                rusqlite::params![run_id],
            )?;
        }

        // Register model run
        conn.execute(
            "INSERT OR REPLACE INTO model_runs
             (run_id, timestamp, target_contaminant, predictor_tier, algorithm,
              cv_mode, threshold_value, threshold_source, random_seed,
              input_data_hash, outline_hash, leakage_rules_hash, config_hash,
              n_features, n_train_samples, n_outer_folds, n_repeats,
              cpu_threads_used, run_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 1, 'running')",
            rusqlite::params![
                run_id, now_iso8601(), target, effective_tier, algorithm, cv_mode,
                threshold_info.0, threshold_info.1, cfg.cv.random_seed as i64,
                csv_hash, outline_hash, leakage_hash, config_hash,
                n_features as i64, n as i64,
                outer_folds as i64, cfg.cv.repeats as i64,
            ],
        )?;

        // ── Store fold assignments ───────────────────────────────────
        conn.execute(
            "DELETE FROM fold_assignments
             WHERE cv_mode = ?1 AND target_contaminant = ?2
               AND tier_name = ?3 AND random_seed = ?4",
            rusqlite::params![cv_mode, target, effective_tier, cfg.cv.random_seed as i64],
        )?;
        for fold in &folds {
            for &idx in &fold.train_idx {
                conn.execute(
                    "INSERT OR IGNORE INTO fold_assignments
                     (cv_mode, repeat_index, fold_index, sample_id, split_role,
                      target_contaminant, tier_name, random_seed)
                     VALUES (?1, ?2, ?3, ?4, 'train', ?5, ?6, ?7)",
                    rusqlite::params![
                        cv_mode,
                        fold.repeat as i64,
                        fold.fold as i64,
                        sample_ids[idx],
                        target,
                        effective_tier,
                        cfg.cv.random_seed as i64
                    ],
                )?;
            }
            for &idx in &fold.test_idx {
                conn.execute(
                    "INSERT OR IGNORE INTO fold_assignments
                     (cv_mode, repeat_index, fold_index, sample_id, split_role,
                      target_contaminant, tier_name, random_seed)
                     VALUES (?1, ?2, ?3, ?4, 'test', ?5, ?6, ?7)",
                    rusqlite::params![
                        cv_mode,
                        fold.repeat as i64,
                        fold.fold as i64,
                        sample_ids[idx],
                        target,
                        effective_tier,
                        cfg.cv.random_seed as i64
                    ],
                )?;
            }
        }

        // ── CV loop ──────────────────────────────────────────────────
        let mut all_true: Vec<i32> = Vec::new();
        let mut all_pred: Vec<f64> = Vec::new();

        for fold in &folds {
            let x_train = subset_rows(&x_full, &fold.train_idx);
            let y_train = subset_y(&y, &fold.train_idx);
            let x_test = subset_rows(&x_full, &fold.test_idx);
            let y_test = subset_y(&y, &fold.test_idx);
            let ids_test = subset_ids(&sample_ids, &fold.test_idx);

            let fold_seed = cfg
                .cv
                .random_seed
                .wrapping_add(fold.repeat as u64 * 31337 + fold.fold as u64);

            // Preprocess inside fold
            let preprocessor = FoldPreprocessor::fit(&x_train);
            let x_train_scaled = preprocessor.transform(&x_train);
            let x_test_scaled = preprocessor.transform(&x_test);

            let class_weight = preprocessing::compute_class_weight(&y_train);

            // Tune and train model
            let train_start = Instant::now();
            let model: Box<dyn Model> = match *algorithm {
                "DummyMajority" => {
                    let mut m = DummyMajority::new();
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    Box::new(m)
                }
                "DummyStratified" => {
                    let mut m = DummyStratified::new();
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    Box::new(m)
                }
                "EcOnlyLogistic" => {
                    let mut m = EcOnlyLogistic::new(&feature_names);
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    Box::new(m)
                }
                "LogisticRegression" => {
                    let best_c = tune_logistic_c(
                        &x_train_scaled,
                        &y_train,
                        class_weight,
                        cfg.cv.inner_folds,
                        fold_seed,
                    );
                    let mut m = LogisticRegression::new(best_c);
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    conn.execute(
                        "INSERT INTO model_hyperparameters (run_id, repeat_index, fold_index, hyperparameter_name, hyperparameter_value)
                         VALUES (?1, ?2, ?3, 'C', ?4)",
                        rusqlite::params![run_id, fold.repeat as i64, fold.fold as i64, best_c.to_string()],
                    )?;
                    Box::new(m)
                }
                "RandomForest" => {
                    let (n_est, depth) = tune_random_forest(
                        &x_train_scaled,
                        &y_train,
                        class_weight,
                        cfg.cv.inner_folds,
                        fold_seed,
                    );
                    let mut m = RandomForest::new(n_est, depth);
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    conn.execute(
                        "INSERT INTO model_hyperparameters (run_id, repeat_index, fold_index, hyperparameter_name, hyperparameter_value)
                         VALUES (?1, ?2, ?3, 'n_estimators_depth', ?4)",
                        rusqlite::params![run_id, fold.repeat as i64, fold.fold as i64, format!("{}_{}", n_est, depth)],
                    )?;
                    Box::new(m)
                }
                "GradientBoostedTrees" => {
                    let (n_est, lr) = tune_gbt(
                        &x_train_scaled,
                        &y_train,
                        class_weight,
                        cfg.cv.inner_folds,
                        fold_seed,
                    );
                    let mut m = GradientBoostedTrees::new(n_est, lr, 3);
                    m.fit(&x_train_scaled, &y_train, class_weight, fold_seed);
                    conn.execute(
                        "INSERT INTO model_hyperparameters (run_id, repeat_index, fold_index, hyperparameter_name, hyperparameter_value)
                         VALUES (?1, ?2, ?3, 'n_estimators_lr', ?4)",
                        rusqlite::params![run_id, fold.repeat as i64, fold.fold as i64, format!("{}_{}", n_est, lr)],
                    )?;
                    Box::new(m)
                }
                _ => continue,
            };
            let train_ms = train_start.elapsed().as_millis() as i64;

            // Predict
            let proba = model.predict_proba(&x_test_scaled);

            // ── Fold metrics ─────────────────────────────────────────
            let metrics_bundle = metrics::compute_all_metrics(&y_test, &proba);
            conn.execute(
                "INSERT INTO fold_metrics
                 (run_id, cv_mode, repeat_index, fold_index,
                  roc_auc, pr_auc, balanced_accuracy, recall_sensitivity,
                  specificity, f1_score, f2_score, brier_score,
                  calibration_slope, calibration_intercept,
                  n_test_pos, n_test_neg, train_time_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                rusqlite::params![
                    run_id, cv_mode, fold.repeat as i64, fold.fold as i64,
                    metrics_bundle.roc_auc, metrics_bundle.pr_auc,
                    metrics_bundle.balanced_accuracy, metrics_bundle.recall_sensitivity,
                    metrics_bundle.specificity, metrics_bundle.f1_score, metrics_bundle.f2_score,
                    metrics_bundle.brier_score,
                    metrics_bundle.calibration_slope, metrics_bundle.calibration_intercept,
                    metrics_bundle.n_pos as i64, metrics_bundle.n_neg as i64,
                    train_ms,
                ],
            )?;

            // ── Predictions ──────────────────────────────────────────
            for (i, sid) in ids_test.iter().enumerate() {
                let prob = proba[i];
                let pred_label = if prob >= 0.5 { 1i64 } else { 0i64 };
                let cluster_id = targets::get_spatial_cluster(conn, sid);
                conn.execute(
                    "INSERT INTO well_predictions
                     (run_id, sample_id, target_contaminant, predictor_tier, algorithm,
                      cv_mode, repeat_index, fold_index, true_label,
                      predicted_probability, predicted_label_default_0_5, spatial_cluster_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    rusqlite::params![
                        run_id,
                        sid,
                        target,
                        effective_tier,
                        algorithm,
                        cv_mode,
                        fold.repeat as i64,
                        fold.fold as i64,
                        y_test[i],
                        prob,
                        pred_label,
                        cluster_id,
                    ],
                )?;
            }

            all_true.extend_from_slice(&y_test);
            all_pred.extend_from_slice(&proba);

            // ── SHAP ─────────────────────────────────────────────────
            let shap_start = Instant::now();
            let shap_result = shap::compute_shap(model.as_ref(), &x_test_scaled, &y_train);
            let train_prev =
                y_train.iter().filter(|&&v| v == 1).count() as f64 / y_train.len() as f64;

            shap::store_fold_base_values(
                conn,
                &run_id,
                cv_mode,
                fold.repeat,
                fold.fold,
                &shap_result,
                train_prev,
            )?;
            shap::store_shap_values(
                conn,
                &run_id,
                &ids_test,
                &feature_names,
                &shap_result,
                &x_test_scaled,
                cv_mode,
                fold.repeat,
                fold.fold,
            )?;
            shap::store_fold_feature_tracking(
                conn,
                &run_id,
                cv_mode,
                fold.repeat,
                fold.fold,
                &feature_names,
                &shap_result,
            )?;

            let shap_ms = shap_start.elapsed().as_millis() as i64;
            conn.execute(
                "UPDATE fold_metrics SET shap_computation_time_ms = ?1
                 WHERE run_id = ?2 AND cv_mode = ?3 AND repeat_index = ?4 AND fold_index = ?5",
                rusqlite::params![
                    shap_ms,
                    run_id,
                    cv_mode,
                    fold.repeat as i64,
                    fold.fold as i64
                ],
            )?;
        }

        // ── Aggregate threshold analysis ─────────────────────────────
        if !all_true.is_empty() && !all_pred.is_empty() {
            let op_thresholds =
                thresholds_operational::select_operational_thresholds(&all_true, &all_pred);
            thresholds_operational::store_operational_thresholds(
                conn,
                &run_id,
                cv_mode,
                &op_thresholds,
            )?;
            thresholds_operational::store_threshold_sweep(
                conn, &run_id, cv_mode, &all_true, &all_pred,
            )?;
        }

        let total_ms = run_start.elapsed().as_millis() as i64;
        conn.execute(
            "UPDATE model_runs SET run_status = 'completed', total_execution_time_ms = ?1 WHERE run_id = ?2",
            rusqlite::params![total_ms, run_id],
        )?;

        if !all_true.is_empty() && !all_pred.is_empty() {
            update_screening_priority(conn, target, &sample_ids)?;
        }

        log::info!(
            "    {} / {} / {} — done in {}ms",
            target,
            algorithm,
            cv_mode,
            total_ms
        );
    }

    Ok(())
}

/// Select the best completed non-dummy run for a target, then upsert the
/// screening_priority table from that run's out-of-fold predictions only.
fn update_screening_priority(conn: &Connection, target: &str, sample_ids: &[String]) -> Result<()> {
    use thresholds_operational::{screening_priority_class, screening_priority_reason};
    use utils::percentile_sorted;

    let best_run_id: Option<String> = conn
        .query_row(
            "SELECT mr.run_id
         FROM model_runs mr
         JOIN fold_metrics fm ON fm.run_id = mr.run_id
         WHERE mr.target_contaminant = ?1
           AND mr.run_status = 'completed'
           AND mr.algorithm NOT IN ('DummyMajority', 'DummyStratified')
         GROUP BY mr.run_id
         ORDER BY AVG(fm.pr_auc) DESC,
                  AVG(fm.brier_score) ASC,
                  CASE mr.cv_mode WHEN 'Stratified_Nested_CV' THEN 0 ELSE 1 END ASC
         LIMIT 1",
            rusqlite::params![target],
            |r| r.get(0),
        )
        .optional()?;

    let Some(best_run_id) = best_run_id else {
        log::warn!(
            "No completed non-dummy run available for {} screening priority",
            target
        );
        return Ok(());
    };

    for sid in sample_ids {
        let mut stmt_p = conn.prepare(
            "SELECT wp.predicted_probability
             FROM well_predictions wp
             WHERE wp.run_id = ?1 AND wp.sample_id = ?2",
        )?;
        let mut probs: Vec<f64> = stmt_p
            .query_map(rusqlite::params![&best_run_id, sid], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<f64>>>()?;

        if probs.is_empty() {
            continue;
        }
        probs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let median = percentile_sorted(&probs, 50.0);
        let lower_ci = percentile_sorted(&probs, 2.5);
        let upper_ci = percentile_sorted(&probs, 97.5);

        let class = screening_priority_class(median);
        let reason = screening_priority_reason(class);

        conn.execute(
            "INSERT OR REPLACE INTO screening_priority
             (sample_id, target_contaminant, best_validated_run_id,
              predicted_probability_median, predicted_probability_lower_ci,
              predicted_probability_upper_ci, screening_priority_class, priority_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                sid,
                target,
                &best_run_id,
                median,
                lower_ci,
                upper_ci,
                class,
                reason,
            ],
        )?;
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════

fn open_db(path: &str) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    schema::create_all_tables(&conn)?;
    Ok(conn)
}
