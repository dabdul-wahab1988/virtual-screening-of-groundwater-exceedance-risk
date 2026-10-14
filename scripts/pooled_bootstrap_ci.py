"""Pooled (instance-level) cluster-bootstrap confidence intervals.

Why this exists
---------------
`bootstrap_ci.py` resamples wells using each well's *consolidated median*
out-of-fold probability.  Its point estimate is therefore the well-level
PR-AUC (n = 81), which is NOT the pooled out-of-fold PR-AUC (n = 81 x
n_repeats) reported in Table 3 and the Abstract.  Pairing a pooled point
estimate with a well-level interval produces an interval that need not
contain its own point estimate.

This script produces the interval that belongs to the pooled estimate:

  * Estimand      : pooled out-of-fold PR-AUC / ROC-AUC over every held-out
                    prediction instance (the Table 3 definition).
  * Sampling unit : the WELL (cluster bootstrap).  A drawn well contributes
                    all of its held-out prediction instances, so the
                    dependence between repeated predictions of the same well
                    is preserved and repeated CV cannot inflate precision.
  * Interval      : 2.5 / 97.5 percentiles over N_BOOT draws.

The PR-AUC and ROC-AUC functions replicate `src/metrics.rs`
(`compute_pr_auc`, `compute_roc_auc`) exactly, including the pessimistic
PR tie-break (label 0 ranked before label 1).  The observed-sample point
estimates are asserted against `pooled_metrics` in the database, so the
script fails loudly if the replication ever drifts from the Rust pipeline.

Output:
  outputs/rev2/main/r_exports/pooled_bootstrap_ci_summary.csv
  outputs/rev2/tables/TableS8b_pooled_bootstrap_ci.csv
"""

import os
import sqlite3

import numpy as np
import pandas as pd

ROOT = r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening"
DB = os.path.join(ROOT, "outputs", "rev2", "main", "main.db")
OUT_EXPORT = os.path.join(ROOT, "outputs", "rev2", "main", "r_exports",
                          "pooled_bootstrap_ci_summary.csv")
OUT_TABLE = os.path.join(ROOT, "outputs", "rev2", "tables",
                         "TableS8b_pooled_bootstrap_ci.csv")

N_BOOT = 2000
SEED = 20260819
TOL = 1e-6

CANONICAL_TIERS = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"]
EXTRA_TIERS = ["Tier2_Reduced_TDS_EC_strict", "Tier2_Reduced_Na_EC_strict", "Tier2_Reduced_Cl_EC_strict"]


def compute_pr_auc(y, p):
    """Port of src/metrics.rs::compute_pr_auc (trapezoid, pessimistic ties)."""
    n_pos = int(np.sum(y))
    if n_pos == 0:
        return 0.0
    order = np.lexsort((y, -p))  # descending p; within a tie, label 0 first
    labels = y[order]
    tp = fp = 0
    auc = 0.0
    prev_recall, prev_precision = 0.0, 1.0
    for label in labels:
        if label == 1:
            tp += 1
        else:
            fp += 1
        recall = tp / n_pos
        precision = tp / (tp + fp)
        auc += (recall - prev_recall) * (precision + prev_precision) / 2.0
        prev_recall, prev_precision = recall, precision
    return auc


def compute_roc_auc(y, p):
    """Port of src/metrics.rs::compute_roc_auc (trapezoid over tie groups)."""
    n = len(y)
    n_pos = int(np.sum(y))
    n_neg = n - n_pos
    if n_pos == 0 or n_neg == 0:
        return 0.5
    order = np.argsort(-p, kind="mergesort")
    probs = p[order]
    labels = y[order]
    auc = 0.0
    tp = fp = prev_tp = prev_fp = 0
    prev_prob = np.inf
    for prob, label in zip(probs, labels):
        if abs(prob - prev_prob) > 1e-10:
            auc += (fp - prev_fp) * (tp + prev_tp) / 2.0
            prev_tp, prev_fp, prev_prob = tp, fp, prob
        if label == 1:
            tp += 1
        else:
            fp += 1
    auc += (fp - prev_fp) * (tp + prev_tp) / 2.0
    return auc / (n_pos * n_neg)


def headline_runs(con):
    """Best canonical non-dummy algorithm per (target, tier) by pooled OOF
    PR-AUC under stratified nested CV, plus the TDS EC-strict recommendation.
    Mirrors the selection rule in bootstrap_ci.py."""
    df = pd.read_sql_query("""
        SELECT mr.run_id, mr.target_contaminant, mr.predictor_tier, mr.algorithm,
               pm.pr_auc, pm.roc_auc
        FROM pooled_metrics pm
        JOIN model_runs mr ON mr.run_id = pm.run_id
        WHERE mr.cv_mode = 'Stratified_Nested_CV'
          AND pm.scope = 'all_held_out'
          AND mr.algorithm NOT LIKE 'Dummy%'
    """, con)
    keep = df[df["predictor_tier"].isin(CANONICAL_TIERS + EXTRA_TIERS)]
    multi = keep[keep["algorithm"] != "EcOnlyLogistic"]
    best = (multi.sort_values("pr_auc", ascending=False)
                 .groupby(["target_contaminant", "predictor_tier"], sort=False)
                 .head(1))
    base = keep[(keep["algorithm"] == "EcOnlyLogistic") & (keep["predictor_tier"] == "Tier1_Field")]
    return pd.concat([best, base], ignore_index=True)


def main():
    con = sqlite3.connect(DB)
    runs = headline_runs(con)
    print(f"Headline selections: {len(runs)}")

    rng = np.random.default_rng(SEED)
    records = []
    for _, row in runs.iterrows():
        preds = pd.read_sql_query(
            "SELECT sample_id, true_label, predicted_probability "
            "FROM well_predictions WHERE run_id = ?", con, params=(row["run_id"],))
        preds = preds.dropna(subset=["true_label", "predicted_probability"])
        if preds.empty:
            continue

        y = preds["true_label"].to_numpy(dtype=int)
        p = preds["predicted_probability"].to_numpy(dtype=float)

        # Observed pooled point estimates, asserted against the database.
        pr_point = compute_pr_auc(y, p)
        roc_point = compute_roc_auc(y, p)
        if abs(pr_point - row["pr_auc"]) > TOL:
            raise SystemExit(
                f"PR-AUC replication drift for {row['run_id']}: "
                f"{pr_point!r} vs DB {row['pr_auc']!r}")
        if abs(roc_point - row["roc_auc"]) > TOL:
            raise SystemExit(
                f"ROC-AUC replication drift for {row['run_id']}: "
                f"{roc_point!r} vs DB {row['roc_auc']!r}")

        # Cluster bootstrap: resample WELLS, carry all their instances.
        wells = preds["sample_id"].to_numpy()
        uniq = np.unique(wells)
        by_well = {w: np.flatnonzero(wells == w) for w in uniq}

        pr_draws, roc_draws = [], []
        for _ in range(N_BOOT):
            drawn = rng.choice(uniq, size=len(uniq), replace=True)
            idx = np.concatenate([by_well[w] for w in drawn])
            y_b, p_b = y[idx], p[idx]
            if y_b.sum() == 0 or y_b.sum() == len(y_b):
                continue
            pr_draws.append(compute_pr_auc(y_b, p_b))
            roc_draws.append(compute_roc_auc(y_b, p_b))

        if len(pr_draws) < 100:
            print(f"  skip {row['run_id']}: too few valid draws")
            continue

        pr_a = np.asarray(pr_draws)
        roc_a = np.asarray(roc_draws)
        records.append({
            "target": row["target_contaminant"],
            "tier": row["predictor_tier"],
            "algorithm": row["algorithm"],
            "n_wells": int(len(uniq)),
            "n_instances": int(len(preds)),
            "n_valid_draws": int(len(pr_a)),
            "pr_auc_point": float(pr_point),
            "pr_auc_ci_lo": float(np.percentile(pr_a, 2.5)),
            "pr_auc_ci_hi": float(np.percentile(pr_a, 97.5)),
            "roc_auc_point": float(roc_point),
            "roc_auc_ci_lo": float(np.percentile(roc_a, 2.5)),
            "roc_auc_ci_hi": float(np.percentile(roc_a, 97.5)),
        })
        print(f"  {row['target_contaminant']:4s} {row['predictor_tier']:28s} "
              f"{row['algorithm']:22s} PR-AUC {pr_point:.3f} "
              f"[{records[-1]['pr_auc_ci_lo']:.3f}, {records[-1]['pr_auc_ci_hi']:.3f}]")

    summary = pd.DataFrame(records).sort_values(["target", "tier"]).reset_index(drop=True)

    # Sanity gate: every interval must contain its own point estimate.
    bad = summary[(summary["pr_auc_point"] < summary["pr_auc_ci_lo"] - 1e-9)
                  | (summary["pr_auc_point"] > summary["pr_auc_ci_hi"] + 1e-9)]
    if not bad.empty:
        raise SystemExit(f"Interval does not contain its point estimate:\n{bad}")

    summary.to_csv(OUT_EXPORT, index=False)
    summary.to_csv(OUT_TABLE, index=False)
    print(f"\nWrote {OUT_EXPORT} ({len(summary)} rows)")
    print(f"Wrote {OUT_TABLE} ({len(summary)} rows)")
    con.close()


if __name__ == "__main__":
    main()
