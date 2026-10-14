"""Merge partition databases (one per target/CV-mode pipeline run) into a
canonical database produced by a standard ingest-inputs.

Partition runs write identical ingest-level rows (raw_samples, measurements,
labels, clusters, ...) and disjoint model rows keyed by deterministic run_id.
Merging uses INSERT OR IGNORE with explicit column lists so autoincrement ids
are regenerated in the canonical database.

Usage:
    python merge_partition_dbs.py <canonical.db> <partition1.db> [partition2.db ...]
"""

import sqlite3
import sys

# Tables merged from partitions, with explicit column lists (id columns omitted).
TABLES = {
    "raw_samples": ["sample_id", "original_row_index", "raw_json", "imported_at", "input_file_hash"],
    "column_dictionary": ["column_name", "inferred_type", "scientific_role", "unit",
                          "is_coordinate", "is_target_candidate", "is_field_variable",
                          "is_lab_variable", "is_derived_variable", "notes"],
    "cleaned_measurements": ["sample_id", "variable_name", "raw_value_text", "cleaned_value_real",
                             "unit", "bdl_flag", "bdl_rule", "missing_flag", "cleaning_note"],
    "target_definitions": ["target_contaminant", "source_column", "threshold_value", "threshold_unit",
                           "threshold_source", "exceedance_direction", "notes"],
    "target_labels": ["target_contaminant", "sample_id", "measured_value", "threshold_value",
                      "true_label", "label_status"],
    "target_eligibility": ["target_contaminant", "n_samples", "n_positive", "n_negative", "prevalence",
                           "cv_feasible", "ml_status", "reporting_level", "eligibility_note"],
    "predictor_tiers": ["tier_name", "tier_description", "intended_use"],
    "candidate_features": ["feature_name", "source_variable", "tier_name", "feature_type",
                           "formula_if_derived", "unit", "enabled"],
    "leakage_rules_applied": ["target_contaminant", "tier_name", "feature_name", "action",
                              "reason", "rule_source"],
    "spatial_cluster_metadata": ["spatial_cluster_id", "centroid_latitude", "centroid_longitude",
                                 "number_of_wells", "max_intra_cluster_distance_km",
                                 "clustering_method", "projection"],
    "sample_spatial_assignment": ["sample_id", "latitude", "longitude", "spatial_cluster_id"],
    "fold_assignments": ["cv_mode", "repeat_index", "fold_index", "sample_id", "split_role",
                         "target_contaminant", "tier_name", "random_seed"],
    "model_runs": ["run_id", "timestamp", "target_contaminant", "predictor_tier", "algorithm",
                   "cv_mode", "threshold_value", "threshold_source", "random_seed", "git_commit_hash",
                   "input_data_hash", "outline_hash", "leakage_rules_hash", "config_hash",
                   "n_features", "n_train_samples", "n_outer_folds", "n_repeats",
                   "cpu_threads_used", "total_execution_time_ms", "run_status", "skip_reason",
                   "labeling_variant"],
    "model_hyperparameters": ["run_id", "repeat_index", "fold_index", "hyperparameter_name",
                              "hyperparameter_value"],
    "fold_base_values": ["run_id", "cv_mode", "repeat_index", "fold_index",
                         "shap_base_value_logit", "shap_base_value_prob", "train_fold_prevalence"],
    "fold_metrics": ["run_id", "cv_mode", "repeat_index", "fold_index", "roc_auc", "pr_auc",
                     "balanced_accuracy", "recall_sensitivity", "specificity", "f1_score",
                     "f2_score", "brier_score", "brier_null", "brier_skill",
                     "calibration_slope", "calibration_intercept", "n_test_pos", "n_test_neg",
                     "train_time_ms", "shap_computation_time_ms"],
    "fold_composition": ["run_id", "cv_mode", "repeat_index", "fold_index", "fold_status",
                         "metrics_defined", "n_train", "n_train_pos", "n_train_neg",
                         "n_test", "n_test_pos", "n_test_neg", "member_cluster_ids"],
    "well_predictions": ["run_id", "sample_id", "target_contaminant", "predictor_tier", "algorithm",
                         "cv_mode", "repeat_index", "fold_index", "true_label",
                         "predicted_probability", "predicted_label_default_0_5", "spatial_cluster_id"],
    "fold_feature_tracking": ["run_id", "cv_mode", "repeat_index", "fold_index", "feature_name",
                              "is_selected", "mean_absolute_shap", "selection_reason"],
    "shap_values": ["run_id", "sample_id", "cv_mode", "repeat_index", "fold_index",
                    "feature_name", "shap_value", "feature_raw_value",
                    "feature_cleaned_value"],
    "operational_thresholds": ["run_id", "cv_mode", "threshold_selection_rule",
                               "optimized_probability_cutoff", "resulting_sensitivity",
                               "resulting_specificity", "false_negatives_count",
                               "false_positives_count"],
    "nested_operational_thresholds": ["run_id", "cv_mode", "repeat_index", "fold_index",
                                      "threshold_selection_rule", "cutoff_selection_source",
                                      "optimized_probability_cutoff", "sensitivity", "specificity",
                                      "precision", "f1_score", "f1_5_score", "f2_score", "f3_score",
                                      "false_negatives_count", "false_positives_count",
                                      "n_test_pos", "n_test_neg"],
    "pooled_metrics": ["run_id", "cv_mode", "scope", "n_wells", "n_pos", "n_neg", "roc_auc",
                       "pr_auc", "recall", "specificity", "f1_score", "f2_score",
                       "balanced_accuracy", "brier_score", "brier_null", "brier_skill"],
    "well_level_metrics": ["run_id", "sample_id", "target_contaminant", "predictor_tier",
                           "algorithm", "cv_mode", "true_label", "n_predictions",
                           "median_probability", "mean_probability", "lower_ci", "upper_ci"],
    "well_level_operating": ["run_id", "cv_mode", "threshold_selection_rule", "cutoff_source",
                             "optimized_probability_cutoff", "tp_count", "tn_count", "fp_count",
                             "fn_count", "sensitivity", "specificity", "precision", "f2_score",
                             "n_wells", "n_pos_wells", "n_neg_wells"],
    "threshold_sensitivity": ["run_id", "cv_mode", "probability_cutoff", "sensitivity",
                              "specificity", "precision", "f1_score", "f1_5_score", "f2_score",
                              "f3_score", "false_negatives_count", "false_positives_count"],
    "screening_priority": ["sample_id", "target_contaminant", "best_validated_run_id",
                           "predicted_probability_median", "predicted_probability_lower_ci",
                           "predicted_probability_upper_ci", "screening_priority_class",
                           "priority_reason"],
    "gis_exports_manifest": ["export_file", "export_type", "created_at", "source_query",
                             "row_count", "sha256_hash"],
}


def merge(canonical: str, partitions: list[str]) -> None:
    main = sqlite3.connect(canonical)
    main.execute("PRAGMA foreign_keys=OFF")
    for i, part_path in enumerate(partitions):
        alias = f"p{i}"
        main.execute(f"ATTACH DATABASE ? AS {alias}", (part_path,))
        for table, cols in TABLES.items():
            col_sql = ", ".join(cols)
            try:
                main.execute(
                    f"INSERT OR IGNORE INTO {table} ({col_sql}) SELECT {col_sql} FROM {alias}.{table}"
                )
            except sqlite3.OperationalError as e:
                print(f"  WARN {table} from {part_path}: {e}")
        main.commit()
        main.execute(f"DETACH DATABASE {alias}")
        main.commit()
        print(f"merged {part_path}")
    main.execute("PRAGMA foreign_keys=ON")
    main.commit()
    main.close()


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)
    canonical = sys.argv[1]
    partitions = sys.argv[2:]
    merge(canonical, partitions)
    print("DONE")
