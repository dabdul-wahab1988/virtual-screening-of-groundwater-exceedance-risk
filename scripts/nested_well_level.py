"""Unique-well operating characteristics under the fully nested cutoff rule.

Pre-specified rule (addresses the Reviewer 2/4 concern that decision metrics
used a post-hoc cutoff chosen on pooled out-of-fold predictions):

  For each Stratified_Nested_CV run, every well has one held-out prediction per
  repeat, made in the outer fold in which that well was held out. Each
  prediction is joined to the frozen cutoff c of the same
  (run_id, repeat_index, fold_index) in nested_operational_thresholds and
  raises an alert if p >= c. A well is flagged at the well level if it raised
  an alert in a strict majority of its repeats (2 * n_alert > n_repeats). If
  the number of repeats is even, a tie (2 * n_alert == n_repeats) is also
  flagged (>= half); this is recorded per row in repeat_vote_rule. Cutoffs
  come only from inner-CV predictions, so no held-out information enters the
  decision rule.

Per run, unique-well TP/TN/FP/FN are reported (they sum to n_wells), together
with sensitivity, specificity, precision, F2 (beta = 2), the mean/SD/min/max of
the per-fold cutoffs (sample SD, ddof = 1) and the number of cutoffs taken from
the in-sample fallback (value 'training_in_sample_fallback' in
cutoff_selection_source; 0 when that column is absent).

Usage:
  python -I nested_well_level.py --db <main.db> --out <rows.csv>

Rows cover all non-dummy Stratified_Nested_CV runs with labeling_variant='main'
(completed runs only). predictor_tier, algorithm and target_contaminant are
kept exactly as stored. The is_best_canonical column is INFORMATIONAL only: it
marks the run with the highest pooled PR-AUC (pooled_metrics scope
'all_held_out') for each (target, Tier1_Field / Tier2_Reduced / Tier3_Full)
INCLUDING the EC-only baseline. The manuscript tables select rows differently
(build_revision_tables.py reports the EC-only baseline separately and the best
multivariable model per tier) and use their own run_id list, not this flag.
"""

import argparse
import os
import sqlite3
import sys

import numpy as np
import pandas as pd

CANONICAL_TIERS = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"]
FALLBACK_SOURCE = "training_in_sample_fallback"
BETA = 2.0
B2 = BETA * BETA

# Run population: non-dummy, completed, main labelling, nested CV.
# Every query that touches runs aliases model_runs as "m".
RUN_FILTER = (
    "m.cv_mode = 'Stratified_Nested_CV' "
    "AND m.algorithm NOT LIKE 'Dummy%' "
    "AND m.run_status = 'completed' "
    "AND m.labeling_variant = 'main'"
)

OUT_COLUMNS = [
    "run_id", "target_contaminant", "predictor_tier", "algorithm", "cv_mode",
    "labeling_variant", "repeat_vote_rule", "n_repeats", "n_wells", "n_pos", "n_neg",
    "TP", "TN", "FP", "FN", "sensitivity", "specificity", "precision", "F2",
    "cutoff_mean", "cutoff_sd", "cutoff_min", "cutoff_max", "n_fold_cutoffs",
    "n_fallback_cutoffs", "pooled_pr_auc_all_held_out", "is_best_canonical",
]


def table_columns(con, table):
    return [r[1] for r in con.execute(f"PRAGMA table_info({table})")]


def safe_ratio(num, den):
    num = num.astype(float)
    den = den.astype(float)
    return (num / den).where(den > 0)


def vote_flags(n_alert, n_rep):
    """Well-level flag from repeat votes: strict majority, or >= half if even."""
    n_alert = np.asarray(n_alert)
    n_rep = np.asarray(n_rep)
    return np.where(n_rep % 2 == 1, 2 * n_alert > n_rep, 2 * n_alert >= n_rep)


def load_inputs(con):
    has_src = "cutoff_selection_source" in table_columns(
        con, "nested_operational_thresholds")
    src_expr = "nt.cutoff_selection_source" if has_src else "NULL"

    runs = pd.read_sql_query(f"""
        SELECT m.run_id, m.target_contaminant, m.predictor_tier, m.algorithm,
               m.cv_mode, m.labeling_variant
        FROM model_runs m
        WHERE {RUN_FILTER}
        ORDER BY m.target_contaminant, m.predictor_tier, m.algorithm, m.run_id
    """, con)

    preds = pd.read_sql_query(f"""
        SELECT wp.run_id, wp.sample_id, wp.repeat_index, wp.fold_index,
               wp.true_label, wp.predicted_probability
        FROM well_predictions wp
        JOIN model_runs m ON m.run_id = wp.run_id
        WHERE {RUN_FILTER}
    """, con)

    cuts = pd.read_sql_query(f"""
        SELECT nt.run_id, nt.repeat_index, nt.fold_index,
               nt.optimized_probability_cutoff,
               {src_expr} AS cutoff_selection_source
        FROM nested_operational_thresholds nt
        JOIN model_runs m ON m.run_id = nt.run_id
        WHERE {RUN_FILTER} AND nt.cv_mode = 'Stratified_Nested_CV'
    """, con)

    pooled = pd.read_sql_query(f"""
        SELECT pm.run_id, pm.pr_auc AS pooled_pr_auc_all_held_out
        FROM pooled_metrics pm
        JOIN model_runs m ON m.run_id = pm.run_id
        WHERE pm.scope = 'all_held_out' AND {RUN_FILTER}
    """, con)

    return runs, preds, cuts, pooled, has_src


def well_level_votes(preds, cuts):
    """One row per (run, well) with repeat-vote alert count and flag."""
    keys = ["run_id", "repeat_index", "fold_index"]
    if cuts.duplicated(keys).any():
        raise ValueError("nested_operational_thresholds has duplicate (run, repeat, fold) keys")

    j = preds.merge(cuts[keys + ["optimized_probability_cutoff"]], on=keys,
                    how="left", validate="many_to_one", indicator=True)
    n_unjoined = int((j["_merge"] != "both").sum())
    if n_unjoined:
        raise ValueError(f"{n_unjoined} predictions lack a frozen cutoff for their (run, repeat, fold)")
    if j["predicted_probability"].isna().any() or j["optimized_probability_cutoff"].isna().any():
        raise ValueError("NaN probability or cutoff after join")

    j["alert"] = (j["predicted_probability"] >= j["optimized_probability_cutoff"]).astype(int)

    well = j.groupby(["run_id", "sample_id"]).agg(
        n_pred=("alert", "size"),
        n_rep_seen=("repeat_index", "nunique"),
        n_alert=("alert", "sum"),
        y_min=("true_label", "min"),
        y_max=("true_label", "max"),
    ).reset_index()
    n_rep_run = j.groupby("run_id")["repeat_index"].nunique().rename("n_repeats")
    well = well.merge(n_rep_run, left_on="run_id", right_index=True, how="left")

    bad = well[(well.n_pred != well.n_repeats) | (well.n_rep_seen != well.n_repeats)
               | (well.y_min != well.y_max)]
    if len(bad):
        raise ValueError(f"{len(bad)} wells lack exactly one prediction per repeat, "
                         "or have inconsistent true labels")

    well["y"] = well["y_min"].astype(int)
    well["flag"] = vote_flags(well["n_alert"], well["n_repeats"]).astype(int)
    well["TP"] = ((well.flag == 1) & (well.y == 1)).astype(int)
    well["FN"] = ((well.flag == 0) & (well.y == 1)).astype(int)
    well["FP"] = ((well.flag == 1) & (well.y == 0)).astype(int)
    well["TN"] = ((well.flag == 0) & (well.y == 0)).astype(int)
    return well


def run_confusion(well):
    g = well.groupby("run_id")
    conf = g[["TP", "TN", "FP", "FN"]].sum()
    conf["n_wells"] = g.size()
    conf["n_pos"] = g["y"].sum()
    conf["n_neg"] = conf["n_wells"] - conf["n_pos"]
    conf["n_repeats"] = g["n_repeats"].first()
    return conf


def cutoff_stats(cuts, has_src):
    g = cuts.groupby("run_id")
    out = g["optimized_probability_cutoff"].agg(
        cutoff_mean="mean", cutoff_sd="std", cutoff_min="min", cutoff_max="max",
        n_fold_cutoffs="size")
    if has_src:
        out["n_fallback_cutoffs"] = g["cutoff_selection_source"].apply(
            lambda s: int((s == FALLBACK_SOURCE).sum()))
    else:
        out["n_fallback_cutoffs"] = 0
    return out


def mark_best_canonical(rows):
    """Mirror build_revision_tables.py: top pooled PR-AUC per (target, canonical tier)."""
    cand = rows[rows["predictor_tier"].isin(CANONICAL_TIERS)]
    cand = cand.sort_values(["pooled_pr_auc_all_held_out", "run_id"], ascending=[False, True])
    best = cand.groupby(["target_contaminant", "predictor_tier"], sort=False).head(1)

    top = cand.groupby(["target_contaminant", "predictor_tier"])["pooled_pr_auc_all_held_out"]\
        .transform("max")
    ties = cand[cand["pooled_pr_auc_all_held_out"] == top]\
        .groupby(["target_contaminant", "predictor_tier"]).size()
    if (ties > 1).any():
        print("WARNING: tied top pooled PR-AUC within a (target, tier) group:")
        print(ties[ties > 1].to_string())

    rows["is_best_canonical"] = rows["run_id"].isin(best["run_id"]).astype(int)
    return rows


def compute_rows(db_path):
    con = sqlite3.connect(db_path)
    try:
        runs, preds, cuts, pooled, has_src = load_inputs(con)
    finally:
        con.close()

    if runs.empty:
        raise ValueError("no runs matched the filter")

    well = well_level_votes(preds, cuts)
    conf = run_confusion(well)
    if len(conf) != len(runs):
        raise ValueError(f"well-level results for {len(conf)} runs, expected {len(runs)}")

    cstat = cutoff_stats(cuts, has_src)

    rows = runs.merge(conf.reset_index(), on="run_id", how="left", validate="one_to_one")
    rows = rows.merge(cstat.reset_index(), on="run_id", how="left", validate="one_to_one")
    rows = rows.merge(pooled, on="run_id", how="left", validate="one_to_one")

    tp, tn, fp, fn = rows["TP"], rows["TN"], rows["FP"], rows["FN"]
    rows["sensitivity"] = safe_ratio(tp, tp + fn)
    rows["specificity"] = safe_ratio(tn, tn + fp)
    rows["precision"] = safe_ratio(tp, tp + fp)
    rows["F2"] = safe_ratio((1 + B2) * tp, (1 + B2) * tp + B2 * fn + fp)
    rows["repeat_vote_rule"] = np.where(rows["n_repeats"] % 2 == 1,
                                        "strict_majority", "ge_half")

    if not (rows["TP"] + rows["TN"] + rows["FP"] + rows["FN"] == rows["n_wells"]).all():
        raise ValueError("TP+TN+FP+FN != n_wells for some run")
    if not (rows["TP"] + rows["FN"] == rows["n_pos"]).all():
        raise ValueError("TP+FN != n_pos for some run")

    rows = mark_best_canonical(rows)

    for col in ["sensitivity", "specificity", "precision", "F2", "cutoff_mean",
                "cutoff_sd", "cutoff_min", "cutoff_max", "pooled_pr_auc_all_held_out"]:
        rows[col] = rows[col].round(6)
    for col in ["n_wells", "n_pos", "n_neg", "TP", "TN", "FP", "FN", "n_repeats",
                "n_fold_cutoffs", "n_fallback_cutoffs", "is_best_canonical"]:
        rows[col] = rows[col].astype(int)

    rows["cv_mode"] = "Stratified_Nested_CV"
    rows = rows[OUT_COLUMNS].sort_values(
        ["target_contaminant", "predictor_tier", "algorithm"]).reset_index(drop=True)
    return rows, has_src


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--db", required=True, help="path to main.db")
    parser.add_argument("--out", required=True, help="output CSV path")
    args = parser.parse_args(argv)

    if not os.path.isfile(args.db):
        sys.exit(f"DB not found: {args.db}")

    rows, has_src = compute_rows(args.db)

    out_dir = os.path.dirname(os.path.abspath(args.out))
    os.makedirs(out_dir, exist_ok=True)
    rows.to_csv(args.out, index=False)

    print(f"rows: {len(rows)}  (cutoff_selection_source column present: {has_src})")
    print("check TP+TN+FP+FN == n_wells: OK for all rows")
    print(f"n_wells range: {rows['n_wells'].min()}-{rows['n_wells'].max()}")
    print(f"wrote {args.out}")
    print("\nis_best_canonical == 1:")
    best = rows[rows["is_best_canonical"] == 1]
    cols = ["target_contaminant", "predictor_tier", "algorithm", "run_id",
            "pooled_pr_auc_all_held_out", "TP", "TN", "FP", "FN",
            "sensitivity", "specificity", "F2"]
    print(best[cols].to_string(index=False))


if __name__ == "__main__":
    main()
