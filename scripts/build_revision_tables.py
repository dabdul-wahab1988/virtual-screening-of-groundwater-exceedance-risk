"""Build all revised manuscript tables (CSV) from the revised database and
exports under outputs/rev2.

Outputs land in outputs/rev2/tables/. Tables are consumed when composing the
revised manuscript and supplementary material.
"""

import os
import sqlite3

import pandas as pd

ROOT = r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening"
DB = ROOT + r"\outputs\rev2\main\main.db"
OUT = ROOT + r"\outputs\rev2\tables"
os.makedirs(OUT, exist_ok=True)

CANONICAL_TIERS = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"]
TIER_LABEL = {"Tier1_Field": "Tier 1 (field)", "Tier2_Reduced": "Tier 2 (reduced)",
              "Tier3_Full": "Tier 3 (full)"}
TARGETS = ["Na", "Cl", "TDS", "B", "F"]

con = sqlite3.connect(DB)


def q(sql):
    return pd.read_sql_query(sql, con)


def save(df, name, fmt_round=None):
    df.to_csv(os.path.join(OUT, name), index=False)
    print(f"  {name}: {len(df)} rows")


# ── Table 1: dataset summary ────────────────────────────────────────────────
print("Table 1")
t1 = q("""
  SELECT c.variable_name AS Variable,
         c.unit AS Unit,
         cd.scientific_role AS Analytical_role,
         SUM(CASE WHEN c.cleaned_value_real IS NOT NULL AND c.missing_flag = 0 THEN 1 ELSE 0 END) AS n_valid,
         SUM(c.bdl_flag) AS n_BDL,
         SUM(c.missing_flag) AS n_missing,
         ROUND(MIN(CASE WHEN c.bdl_flag = 0 THEN c.cleaned_value_real END), 3) AS Min,
         ROUND(AVG(c.cleaned_value_real), 3) AS Median_approx,
         ROUND(MAX(CASE WHEN c.bdl_flag = 0 THEN c.cleaned_value_real END), 3) AS Max
  FROM cleaned_measurements c
  LEFT JOIN column_dictionary cd ON cd.column_name = c.variable_name
  WHERE c.variable_name IN ('Dx','Dy','pH','Temp.','EC','TDS','Na','K','Mg','Ca','Cl',
                            'SO4','HCO3','CO3','NO3','F','B')
  GROUP BY c.variable_name, c.unit, cd.scientific_role
  ORDER BY CASE c.variable_name
             WHEN 'Dx' THEN 1 WHEN 'Dy' THEN 2 WHEN 'pH' THEN 3 WHEN 'Temp.' THEN 4
             WHEN 'EC' THEN 5 WHEN 'TDS' THEN 6 WHEN 'Na' THEN 7 WHEN 'K' THEN 8
             WHEN 'Mg' THEN 9 WHEN 'Ca' THEN 10 WHEN 'Cl' THEN 11 WHEN 'SO4' THEN 12
             WHEN 'HCO3' THEN 13 WHEN 'CO3' THEN 14 WHEN 'NO3' THEN 15 WHEN 'F' THEN 16
             ELSE 17 END
""")
# median computed properly per variable
med = q("""
  SELECT variable_name, ROUND(MIN(median_val), 3) AS Median FROM (
    SELECT variable_name,
           cleaned_value_real AS median_val,
           ROW_NUMBER() OVER (PARTITION BY variable_name ORDER BY cleaned_value_real) AS rn,
           COUNT(*) OVER (PARTITION BY variable_name) AS cnt
    FROM cleaned_measurements
    WHERE cleaned_value_real IS NOT NULL AND missing_flag = 0
  ) WHERE rn IN ((cnt + 1) / 2, (cnt + 2) / 2)
  GROUP BY variable_name
""")
# Median of DETECTED values only (BDL substitutions excluded), so the reported
# median is a measured concentration; n_valid includes BDL substitutions.
med_det = q("""
  SELECT variable_name, ROUND(MIN(median_val), 3) AS Median_detected FROM (
    SELECT variable_name,
           cleaned_value_real AS median_val,
           ROW_NUMBER() OVER (PARTITION BY variable_name ORDER BY cleaned_value_real) AS rn,
           COUNT(*) OVER (PARTITION BY variable_name) AS cnt
    FROM cleaned_measurements
    WHERE cleaned_value_real IS NOT NULL AND missing_flag = 0 AND bdl_flag = 0
  ) WHERE rn IN ((cnt + 1) / 2, (cnt + 2) / 2)
  GROUP BY variable_name
""")
t1 = t1.drop(columns=["Median_approx"]).merge(med.rename(columns={"variable_name": "Variable"}),
                                              on="Variable", how="left")
t1 = t1.merge(med_det.rename(columns={"variable_name": "Variable"}), on="Variable", how="left")
t1["n_detected"] = t1["n_valid"] - t1["n_BDL"]
tdef = q("SELECT target_contaminant, threshold_value FROM target_definitions")
telig = q("SELECT target_contaminant, n_positive, prevalence FROM target_eligibility")
t1 = t1.merge(tdef.rename(columns={"target_contaminant": "Variable"}),
              on="Variable", how="left")
t1 = t1.merge(telig.rename(columns={"target_contaminant": "Variable"}),
              on="Variable", how="left")
t1["n_exceeding"] = t1["n_positive"]
t1["pct"] = (t1["prevalence"] * 100).round(1)
t1["ML_status"] = t1["Variable"].apply(
    lambda v: "Modelled" if v in TARGETS else ("Descriptive only" if v == "NO3" else "Predictor"))
save(t1, "Table1_dataset_summary.csv")

# ── Table 2: leakage matrix summary ─────────────────────────────────────────
print("Table 2")
t2 = q("""
  SELECT target_contaminant, tier_name,
         SUM(CASE WHEN action='excluded' THEN 1 ELSE 0 END) AS n_excluded,
         SUM(CASE WHEN action='included' THEN 1 ELSE 0 END) AS n_retained
  FROM leakage_rules_applied
  GROUP BY target_contaminant, tier_name
  ORDER BY target_contaminant, tier_name
""")
excl = q("""
  SELECT target_contaminant, tier_name, feature_name
  FROM leakage_rules_applied WHERE action = 'excluded'
  ORDER BY target_contaminant, tier_name
""")
excl_list = excl.groupby(["target_contaminant", "tier_name"])["feature_name"].apply(
    lambda s: "; ".join(s)).reset_index(name="excluded_predictors")
t2 = t2.merge(excl_list, on=["target_contaminant", "tier_name"])
save(t2, "Table2_leakage_matrix_summary.csv")
save(excl, "Table2_leakage_matrix_full.csv")

# ── Table 3: best classifiers with pooled metrics ───────────────────────────
print("Table 3")
pooled = q("""
  SELECT mr.run_id, mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         pm.pr_auc, pm.roc_auc, pm.recall, pm.f2_score, pm.brier_score,
         pm.balanced_accuracy, pm.brier_skill, pm.n_pos, pm.n_neg
  FROM pooled_metrics pm JOIN model_runs mr ON pm.run_id = mr.run_id
  WHERE mr.run_status = 'completed' AND mr.algorithm NOT LIKE 'Dummy%'
    AND (mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
         OR mr.predictor_tier LIKE '%_EC_strict')
    AND pm.scope = 'all_held_out'
""")
strat = pooled[pooled.cv_mode == "Stratified_Nested_CV"].copy()
spat = (pooled[pooled.cv_mode == "Spatial_Group_CV"]
        [["target_contaminant", "predictor_tier", "algorithm", "pr_auc"]].rename(
            columns={"pr_auc": "spatial_pr_auc"}))
# Tier-specific best model: chosen among the multivariable models (logistic
# regression, random forest, gradient-boosted trees). The EC-only logistic
# baseline uses only EC, so it is identical at every tier; it is reported as
# its own reference row ('EC_only_baseline') rather than as the "best model"
# of a tier it does not use.
multi = strat[strat.algorithm != "EcOnlyLogistic"]
best_multi = (multi.sort_values("pr_auc", ascending=False)
              .groupby(["target_contaminant", "predictor_tier"], sort=False).head(1))
base = strat[(strat.algorithm == "EcOnlyLogistic") & (strat.predictor_tier == "Tier1_Field")].copy()
base["predictor_tier"] = "EC_only_baseline"
best = pd.concat([base, best_multi], ignore_index=True)
spat_all = spat.copy()
spat_base = spat_all[(spat_all.algorithm == "EcOnlyLogistic")
                     & (spat_all.predictor_tier == "Tier1_Field")].copy()
spat_base["predictor_tier"] = "EC_only_baseline"
spat_all = pd.concat([spat_all, spat_base], ignore_index=True)
best = best.merge(spat_all, on=["target_contaminant", "predictor_tier", "algorithm"], how="left")
TIER_ORDER = {"EC_only_baseline": 0, "Tier1_Field": 1, "Tier2_Reduced": 2,
              "Tier2_Reduced_Na_EC_strict": 3, "Tier2_Reduced_Cl_EC_strict": 3,
              "Tier2_Reduced_TDS_EC_strict": 3, "Tier3_Full": 4}
TGT_ORDER = {"Na": 0, "Cl": 1, "TDS": 2, "B": 3, "F": 4}
best["_o"] = best.target_contaminant.map(TGT_ORDER) * 10 + best.predictor_tier.map(TIER_ORDER)
best = best.sort_values("_o").drop(columns="_o").reset_index(drop=True)
# Pooled calibration slope and intercept (logistic regression of the outcome on
# logit of the out-of-fold probability, Newton/L-BFGS fit; the fold-level values
# stored by the pipeline come from a fixed-step gradient descent that does not
# converge on near-separable models and are therefore not reported).
def _calibration(run_id):
    import numpy as np
    from sklearn.linear_model import LogisticRegression
    d = pd.read_sql_query(
        "SELECT true_label AS y, predicted_probability AS p FROM well_predictions WHERE run_id = ?",
        con, params=(run_id,)).dropna()
    if d["y"].nunique() < 2:
        return (np.nan, np.nan)
    z = np.log(np.clip(d["p"], 1e-3, 1 - 1e-3) / (1 - np.clip(d["p"], 1e-3, 1 - 1e-3)))
    m = LogisticRegression(penalty=None, max_iter=2000).fit(z.to_numpy().reshape(-1, 1), d["y"].astype(int))
    return (float(m.coef_[0][0]), float(m.intercept_[0]))


cal = best["run_id"].map(_calibration)
best["cal_slope"] = [c[0] for c in cal]
best["cal_intercept"] = [c[1] for c in cal]
save(best, "Table3_model_performance.csv")

# ── Table 4: principal decision table = FULLY NESTED, unique-well ──────────
# Each well is flagged if its held-out prediction exceeded the frozen,
# inner-CV-selected cutoff of its own outer fold in a strict majority of the
# repeats (scripts/nested_well_level.py). Post-hoc well-level counts (cutoff
# chosen on all 81 wells' median OOF probabilities) are kept only as a
# clearly labelled comparison.
print("Table 4")
import subprocess, sys
nested_all = os.path.join(OUT, "Table4_nested_well_level_all_runs.csv")
subprocess.run([sys.executable, os.path.join(ROOT, "scripts", "nested_well_level.py"),
                "--db", DB, "--out", nested_all], check=True)
nw = pd.read_csv(nested_all)
wlo = q("""
  SELECT w.run_id, w.optimized_probability_cutoff AS posthoc_cutoff,
         w.tp_count AS posthoc_tp, w.tn_count AS posthoc_tn,
         w.fp_count AS posthoc_fp, w.fn_count AS posthoc_fn,
         w.sensitivity AS posthoc_sensitivity, w.specificity AS posthoc_specificity
  FROM well_level_operating w JOIN model_runs mr ON w.run_id = mr.run_id
  WHERE mr.cv_mode = 'Stratified_Nested_CV' AND mr.algorithm NOT LIKE 'Dummy%'
    AND w.threshold_selection_rule = 'sens_ge_0.90'
""")
t4 = nw.merge(wlo, on="run_id", how="left")
# canonical best models (Table 3 rows) + best EC-strict model per target
keep = set(best["run_id"])
t4 = t4[t4["run_id"].isin(keep)].copy()
t4.loc[t4["run_id"].isin(set(base["run_id"])), "predictor_tier"] = "EC_only_baseline"
t4 = t4.rename(columns={"TP": "tp_count", "TN": "tn_count", "FP": "fp_count", "FN": "fn_count"})
save(t4, "Table4_operational_thresholds.csv")

# ── Table 5: tier comparison ────────────────────────────────────────────────
print("Table 5")
t5 = strat.pivot_table(index=["target_contaminant", "algorithm"],
                       columns="predictor_tier", values="pr_auc").reset_index()
t5 = t5.merge(strat.pivot_table(index=["target_contaminant", "algorithm"],
                                columns="predictor_tier", values="f2_score")
              .reset_index().rename(columns={
                  "Tier1_Field": "f2_T1", "Tier2_Reduced": "f2_T2", "Tier3_Full": "f2_T3"}),
              on=["target_contaminant", "algorithm"])
t5["pct_T1"] = (t5["Tier1_Field"] / t5["Tier3_Full"] * 100).round(1)
t5["pct_T2"] = (t5["Tier2_Reduced"] / t5["Tier3_Full"] * 100).round(1)
save(t5, "Table5_tier_comparison.csv")

# ── TableS1: data dictionary ────────────────────────────────────────────────
print("TableS1")
s1 = q("SELECT * FROM column_dictionary")
save(s1, "TableS1_data_dictionary.csv")

# ── TableS2: threshold sources ──────────────────────────────────────────────
print("TableS2")
s2 = q("""
  SELECT d.target_contaminant, d.source_column, d.threshold_value, d.threshold_unit,
         d.threshold_source, d.exceedance_direction, d.notes,
         e.n_positive, e.n_negative, e.ml_status
  FROM target_definitions d
  LEFT JOIN target_eligibility e ON e.target_contaminant = d.target_contaminant
  ORDER BY d.target_contaminant
""")
save(s2, "TableS2_threshold_sources.csv")

# ── TableS3: hyperparameter search ──────────────────────────────────────────
print("TableS3")
s3 = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode,
         h.hyperparameter_name, h.hyperparameter_value,
         COUNT(*) AS n_folds, COUNT(DISTINCT h.hyperparameter_value) AS n_unique
  FROM model_hyperparameters h JOIN model_runs r ON r.run_id = h.run_id
  WHERE r.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode,
           h.hyperparameter_name, h.hyperparameter_value
  ORDER BY r.target_contaminant, r.predictor_tier, r.algorithm, h.hyperparameter_value
""")
save(s3, "TableS3_hyperparameter_search.csv")

# ── TableS4: fold-level metrics (mean +/- SD) for headline models ───────────
print("TableS4")
s4_raw = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode,
         m.pr_auc, m.roc_auc, m.recall_sensitivity, m.f2_score, m.brier_score,
         m.balanced_accuracy
  FROM fold_metrics m JOIN model_runs r ON r.run_id = m.run_id
  WHERE r.algorithm NOT LIKE 'Dummy%'
    AND r.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
""")
s4 = (s4_raw.groupby(["target_contaminant", "predictor_tier", "algorithm", "cv_mode"])
      .agg(n_folds=("pr_auc", "size"), n_folds_defined=("pr_auc", "count"),
           pr_auc_mean=("pr_auc", "mean"), pr_auc_sd=("pr_auc", "std"),
           roc_auc_mean=("roc_auc", "mean"), roc_auc_sd=("roc_auc", "std"),
           recall_mean=("recall_sensitivity", "mean"), recall_sd=("recall_sensitivity", "std"),
           f2_mean=("f2_score", "mean"), f2_sd=("f2_score", "std"),
           brier_mean=("brier_score", "mean"), brier_sd=("brier_score", "std"),
           balacc_mean=("balanced_accuracy", "mean"), balacc_sd=("balanced_accuracy", "std"))
      .round(4).reset_index()
      .sort_values(["target_contaminant", "predictor_tier", "algorithm", "cv_mode"]))
save(s4, "TableS4_fold_level_metrics.csv")

# ── TableS5: SHAP base-probability stability ────────────────────────────────
print("TableS5")
s5_raw = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm, b.cv_mode,
         b.shap_base_value_prob, b.train_fold_prevalence
  FROM fold_base_values b JOIN model_runs r ON r.run_id = b.run_id
  WHERE r.algorithm NOT LIKE 'Dummy%'
""")
s5 = (s5_raw.groupby(["target_contaminant", "predictor_tier", "algorithm", "cv_mode"])
      .agg(n_folds=("shap_base_value_prob", "size"),
           base_prob_mean=("shap_base_value_prob", "mean"),
           base_prob_sd=("shap_base_value_prob", "std"),
           train_prevalence_mean=("train_fold_prevalence", "mean"),
           train_prevalence_sd=("train_fold_prevalence", "std"))
      .round(4).reset_index()
      .sort_values(["target_contaminant", "predictor_tier", "algorithm", "cv_mode"]))
save(s5, "TableS5_SHAP_base_values.csv")

# ── TableS6: spatial clusters ───────────────────────────────────────────────
print("TableS6")
s6 = q("""
  SELECT s.spatial_cluster_id, s.number_of_wells, s.centroid_latitude, s.centroid_longitude,
         s.max_intra_cluster_distance_km, s.clustering_method,
         (SELECT GROUP_CONCAT(sample_id, '; ') FROM sample_spatial_assignment a
          WHERE a.spatial_cluster_id = s.spatial_cluster_id) AS well_ids
  FROM spatial_cluster_metadata s ORDER BY s.spatial_cluster_id
""")
save(s6, "TableS6_spatial_clusters.csv")

# ── TableS7: leakage audit (full) ───────────────────────────────────────────
print("TableS7")
s7 = q("""
  SELECT target_contaminant, tier_name, feature_name, action, reason, rule_source
  FROM leakage_rules_applied ORDER BY target_contaminant, tier_name, feature_name
""")
save(s7, "TableS7_leakage_audit.csv")

# ── TableS8: bootstrap CIs (from bootstrap_ci_summary.csv) ─────────────────
print("TableS8")
boot_path = ROOT + r"\outputs\rev2\main\r_exports\bootstrap_ci_summary.csv"
if os.path.exists(boot_path):
    s8 = pd.read_csv(boot_path)
    save(s8, "TableS8_bootstrap_ci.csv")

# ── TableS9: k sensitivity (from spatial_k_sensitivity_summary.csv) ─────────
print("TableS9")
ks_path = ROOT + r"\outputs\rev2\main\r_exports\spatial_k_sensitivity_summary.csv"
if os.path.exists(ks_path):
    s9 = pd.read_csv(ks_path)
    save(s9, "TableS9_spatial_k_sensitivity.csv")

# ── TableS10: coordinate ablation ───────────────────────────────────────────
print("TableS10")
s10 = q("""
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         CASE WHEN mr.predictor_tier LIKE '%no_coords%' THEN 0 ELSE 1 END AS has_coords,
         pm.pr_auc, pm.roc_auc, pm.brier_skill
  FROM pooled_metrics pm JOIN model_runs mr ON pm.run_id = mr.run_id
  WHERE mr.algorithm NOT LIKE 'Dummy%' AND pm.scope = 'all_held_out'
    AND (mr.predictor_tier LIKE '%no_coords%'
         OR mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full'))
  ORDER BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
""")
save(s10, "TableS10_coordinate_ablation.csv")
# Paired comparison: best multivariable model with vs without coordinates,
# per target, tier and validation design (EC-only logistic is excluded because
# it never uses coordinates).
s10m = s10[s10.algorithm != "EcOnlyLogistic"].copy()
s10m["tier_base"] = s10m.predictor_tier.str.replace("_no_coords", "", regex=False)
s10m = s10m[s10m.tier_base.isin(CANONICAL_TIERS)]
pairs = []
for (t, tb, cv), g in s10m.groupby(["target_contaminant", "tier_base", "cv_mode"]):
    w = g[g.has_coords == 1].sort_values("pr_auc", ascending=False).head(1)
    n = g[g.has_coords == 0].sort_values("pr_auc", ascending=False).head(1)
    if w.empty or n.empty:
        continue
    pairs.append({"target_contaminant": t, "tier": tb, "cv_mode": cv,
                  "algorithm_with_coords": w.iloc[0].algorithm, "pr_auc_with_coords": w.iloc[0].pr_auc,
                  "algorithm_no_coords": n.iloc[0].algorithm, "pr_auc_no_coords": n.iloc[0].pr_auc,
                  "delta_no_minus_with": n.iloc[0].pr_auc - w.iloc[0].pr_auc})
save(pd.DataFrame(pairs), "TableS10b_coordinate_ablation_pairs.csv")

# ── TableS11: WHO-threshold sensitivity (from who_guideline DB) ─────────────
print("TableS11")
who_db = ROOT + r"\outputs\rev2\who_guideline\who_guideline.db"
if os.path.exists(who_db):
    wcon = sqlite3.connect(who_db)
    who_elig = pd.read_sql_query("""
      SELECT e.target_contaminant, e.n_positive, e.n_negative, e.prevalence, e.ml_status,
             d.threshold_value, d.threshold_source
      FROM target_eligibility e LEFT JOIN target_definitions d
        ON d.target_contaminant = e.target_contaminant
      ORDER BY e.target_contaminant
    """, wcon)
    who_pooled = pd.read_sql_query("""
      SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
             pm.pr_auc, pm.brier_skill
      FROM pooled_metrics pm JOIN model_runs mr ON pm.run_id = mr.run_id
      WHERE mr.algorithm NOT LIKE 'Dummy%' AND pm.scope = 'all_held_out'
    """, wcon)
    who_best = (who_pooled[who_pooled.cv_mode == "Stratified_Nested_CV"]
                .sort_values("pr_auc", ascending=False)
                .groupby(["target_contaminant", "predictor_tier"], sort=False).head(1))
    who_spat = who_pooled[who_pooled.cv_mode == "Spatial_Group_CV"][
        ["target_contaminant", "predictor_tier", "algorithm", "pr_auc"]].rename(
        columns={"pr_auc": "spatial_pr_auc"})
    who_best = who_best.merge(who_spat, on=["target_contaminant", "predictor_tier", "algorithm"],
                              how="left")
    s11 = who_elig.merge(who_best, on="target_contaminant", how="left")
    wcon.close()
    save(s11, "TableS11_who_threshold_sensitivity.csv")

# ── TableS12: BDL sensitivity (across BDL variant DBs) ──────────────────────
print("TableS12")
bdl_rows = []
bdl_variants = {
    "main (min-detected / sqrt2)": (DB, None),
    "bdl_lod0p5 (LOD 0.5 / sqrt2)": (ROOT + r"\outputs\rev2\bdl_lod0p5\bdl_lod0p5.db", 0.5),
    "bdl_lod0p2 (LOD 0.2 / sqrt2)": (ROOT + r"\outputs\rev2\bdl_lod0p2\bdl_lod0p2.db", 0.2),
    "bdl_limit_direct (limit direct)": (ROOT + r"\outputs\rev2\bdl_limit_direct\bdl_limit_direct.db", None),
}
for label, (db, lod) in bdl_variants.items():
    if not os.path.exists(db):
        print(f"  skip missing {db}")
        continue
    c = sqlite3.connect(db)
    sub = pd.read_sql_query("""
      SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm,
             pm.pr_auc, pm.roc_auc
      FROM pooled_metrics pm JOIN model_runs mr ON pm.run_id = mr.run_id
      WHERE mr.algorithm NOT LIKE 'Dummy%'
        AND pm.cv_mode = 'Stratified_Nested_CV' AND pm.scope = 'all_held_out'
        AND mr.predictor_tier IN ('Tier2_Reduced','Tier3_Full')
    """, c)
    c.close()
    sub["variant"] = label
    bdl_rows.append(sub)
if bdl_rows:
    s12 = pd.concat(bdl_rows, ignore_index=True)
    save(s12, "TableS12_bdl_sensitivity.csv")

# ── TableS13: beta sensitivity (max-F1/F1.5/F2/F3 rules, well level) ────────
print("TableS13")
s13 = q("""
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm,
         ot.threshold_selection_rule, ot.optimized_probability_cutoff,
         ot.resulting_sensitivity, ot.resulting_specificity,
         ot.false_negatives_count, ot.false_positives_count
  FROM operational_thresholds ot JOIN model_runs mr ON ot.run_id = mr.run_id
  WHERE mr.cv_mode = 'Stratified_Nested_CV'
    AND ot.threshold_selection_rule IN ('max_F1','max_F1_5','max_F2','max_F3')
    AND mr.predictor_tier = 'Tier3_Full'
  ORDER BY mr.target_contaminant, ot.threshold_selection_rule
""")
save(s13, "TableS13_beta_sensitivity.csv")

# ── TableS14: small-sample diagnostics ──────────────────────────────────────
print("TableS14")
s14 = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode, r.n_features,
         COUNT(*) AS n_folds,
         ROUND(AVG(fc.n_train), 1) AS mean_train_n,
         MIN(fc.n_train_pos) AS min_train_pos,
         ROUND(AVG(fc.n_train_pos), 1) AS mean_train_pos,
         MAX(fc.n_train_pos) AS max_train_pos,
         ROUND(MIN(fc.n_train_pos * 1.0 / r.n_features), 2) AS min_events_per_variable,
         ROUND(AVG(fc.n_train_pos * 1.0 / r.n_features), 2) AS mean_events_per_variable,
         ROUND(MIN(fc.n_train * 1.0 / r.n_features), 2) AS min_train_per_feature,
         ROUND(AVG(fc.n_train * 1.0 / r.n_features), 2) AS mean_train_per_feature,
         ROUND(MIN(m.pr_auc), 4) AS min_pr_auc,
         ROUND(AVG(m.pr_auc), 4) AS mean_pr_auc,
         ROUND(MAX(m.pr_auc), 4) AS max_pr_auc
  FROM model_runs r
  JOIN fold_composition fc ON fc.run_id = r.run_id
  JOIN fold_metrics m ON m.run_id = r.run_id AND m.repeat_index = fc.repeat_index
                     AND m.fold_index = fc.fold_index
  WHERE r.algorithm NOT IN ('DummyMajority','DummyStratified')
    AND r.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode, r.n_features
  ORDER BY r.target_contaminant, r.predictor_tier, r.algorithm, r.cv_mode
""")
save(s14, "TableS14_small_sample_diagnostics.csv")

# ── TableS15: spatial fold transparency ─────────────────────────────────────
print("TableS15")
s15 = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm,
         fc.fold_index, fc.fold_status, fc.metrics_defined,
         fc.n_train, fc.n_train_pos, fc.n_train_neg,
         fc.n_test, fc.n_test_pos, fc.n_test_neg, fc.member_cluster_ids
  FROM fold_composition fc JOIN model_runs r ON r.run_id = fc.run_id
  WHERE r.cv_mode = 'Spatial_Group_CV'
  ORDER BY r.target_contaminant, r.predictor_tier, r.algorithm, fc.fold_index
""")
save(s15, "TableS15_spatial_fold_transparency.csv")
# Pooled spatial estimates over ALL held-out wells versus the binary-valid-only
# subset (single-class held-out blocks excluded): quantifies the bias that
# excluding single-class blocks would have introduced.
s15b = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm,
         a.n_wells AS n_wells_all, a.pr_auc AS pr_auc_all_held_out, a.roc_auc AS roc_auc_all_held_out,
         b.n_wells AS n_wells_binary_valid, b.pr_auc AS pr_auc_binary_valid_only,
         b.roc_auc AS roc_auc_binary_valid_only
  FROM model_runs r
  JOIN pooled_metrics a ON a.run_id = r.run_id AND a.scope = 'all_held_out'
  LEFT JOIN pooled_metrics b ON b.run_id = r.run_id AND b.scope = 'binary_valid_only'
  WHERE r.cv_mode = 'Spatial_Group_CV' AND r.algorithm NOT LIKE 'Dummy%'
    AND r.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  ORDER BY r.target_contaminant, r.predictor_tier, r.algorithm
""")
s15b["delta_binary_valid_minus_all"] = s15b["pr_auc_binary_valid_only"] - s15b["pr_auc_all_held_out"]
save(s15b, "TableS15b_spatial_scope_comparison.csv")

# ── TableS16: nested per-fold threshold summaries ───────────────────────────
print("TableS16")
s16_raw = q("""
  SELECT r.target_contaminant, r.predictor_tier, r.algorithm,
         nt.optimized_probability_cutoff, nt.sensitivity, nt.specificity,
         nt.false_negatives_count, nt.false_positives_count
  FROM nested_operational_thresholds nt JOIN model_runs r ON r.run_id = nt.run_id
  WHERE r.cv_mode = 'Stratified_Nested_CV'
""")
s16 = (s16_raw.groupby(["target_contaminant", "predictor_tier", "algorithm"])
       .agg(n_folds=("optimized_probability_cutoff", "size"),
            cutoff_mean=("optimized_probability_cutoff", "mean"),
            cutoff_sd=("optimized_probability_cutoff", "std"),
            sensitivity_mean=("sensitivity", "mean"),
            specificity_mean=("specificity", "mean"),
            total_fn=("false_negatives_count", "sum"),
            total_fp=("false_positives_count", "sum"))
       .round(4).reset_index()
       .sort_values(["target_contaminant", "predictor_tier", "algorithm"]))
save(s16, "TableS16_nested_threshold_summary.csv")

# ── TableS19: attribution shares for the headline / sensitivity models ──────
print("TableS19")
MODELS = [("TDS_EC_strict", "TDS", "Tier2_Reduced_TDS_EC_strict", "GradientBoostedTrees"),
          ("Na_EC_strict", "Na", "Tier2_Reduced_Na_EC_strict", "RandomForest"),
          ("Cl_EC_strict", "Cl", "Tier2_Reduced_Cl_EC_strict", "GradientBoostedTrees"),
          ("B_Tier3", "B", "Tier3_Full", "RandomForest"),
          ("F_Tier3", "F", "Tier3_Full", "GradientBoostedTrees")]
rows = []
for label, t, tier, alg in MODELS:
    d = q(f"""
      SELECT s.feature_name AS feature, AVG(ABS(s.shap_value)) AS mean_abs_shap
      FROM shap_values s JOIN model_runs r ON r.run_id = s.run_id
      WHERE r.target_contaminant = '{t}' AND r.predictor_tier = '{tier}'
        AND r.algorithm = '{alg}' AND r.cv_mode = 'Stratified_Nested_CV'
      GROUP BY s.feature_name ORDER BY mean_abs_shap DESC
    """)
    d["share_pct"] = 100 * d["mean_abs_shap"] / d["mean_abs_shap"].sum()
    d["rank"] = range(1, len(d) + 1)
    d.insert(0, "model", label)
    d.insert(1, "algorithm", alg)
    rows.append(d)
save(pd.concat(rows, ignore_index=True), "TableS19_attribution_shares.csv")

con.close()
print("ALL TABLES DONE")
