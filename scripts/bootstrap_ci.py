"""
Bootstrap 95% CIs for PR-AUC, ROC-AUC, recall, F2, Brier, and calibration
slope/intercept for the Table-3 headline classifiers, computed by
resampling the existing out-of-fold predictions (no retraining).

Resampling unit is the well (sample_id): for each bootstrap draw we sample
wells with replacement and, for each drawn well, randomly pick one of its
available repeat-fold predictions (stratified nested CV repeats the same
well across several outer folds). This respects that predictions for the
same well across repeats are not independent while still propagating the
finite-sample uncertainty a reviewer is asking about.
"""
import warnings
warnings.filterwarnings("ignore")
import numpy as np
import pandas as pd
from sklearn.metrics import average_precision_score, roc_auc_score, recall_score, brier_score_loss
from sklearn.linear_model import LogisticRegression

RNG = np.random.default_rng(20260719)
N_BOOT = 2000

PRED_PATH = "outputs/r_exports/out_of_fold_predictions.csv"

# Table 3 headline selections (target, tier, algorithm) + the strict-TDS addition for Q1
SELECTIONS = [
    ("B", "Tier1_Field", "EcOnlyLogistic"),
    ("B", "Tier2_Reduced", "LogisticRegression"),
    ("B", "Tier3_Full", "LogisticRegression"),
    ("Cl", "Tier1_Field", "EcOnlyLogistic"),
    ("Cl", "Tier2_Reduced", "EcOnlyLogistic"),
    ("Cl", "Tier3_Full", "EcOnlyLogistic"),
    ("F", "Tier1_Field", "RandomForest"),
    ("F", "Tier2_Reduced", "RandomForest"),
    ("F", "Tier3_Full", "GradientBoostedTrees"),
    ("Na", "Tier1_Field", "EcOnlyLogistic"),
    ("Na", "Tier2_Reduced", "GradientBoostedTrees"),
    ("Na", "Tier3_Full", "EcOnlyLogistic"),
    ("TDS", "Tier1_Field", "EcOnlyLogistic"),
    ("TDS", "Tier2_Reduced", "EcOnlyLogistic"),
    ("TDS", "Tier3_Full", "EcOnlyLogistic"),
    ("TDS", "Tier2_Reduced_TDS_EC_strict", "GradientBoostedTrees"),
]

CV_MODE = "Stratified_Nested_CV"
F2_BETA2 = 4.0  # (1+2^2)


def f2_score(y_true, y_prob, thresh=0.5):
    y_pred = (y_prob >= thresh).astype(int)
    tp = np.sum((y_pred == 1) & (y_true == 1))
    fp = np.sum((y_pred == 1) & (y_true == 0))
    fn = np.sum((y_pred == 0) & (y_true == 1))
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    if precision + recall == 0:
        return 0.0
    return (1 + F2_BETA2) * precision * recall / (F2_BETA2 * precision + recall)


def calibration_slope_intercept(y_true, y_prob):
    eps = 1e-6
    p = np.clip(y_prob, eps, 1 - eps)
    logit = np.log(p / (1 - p))
    if len(np.unique(y_true)) < 2:
        return np.nan, np.nan
    try:
        lr = LogisticRegression(C=1e6, max_iter=1000)
        lr.fit(logit.reshape(-1, 1), y_true)
        return float(lr.coef_[0][0]), float(lr.intercept_[0])
    except Exception:
        return np.nan, np.nan


def bootstrap_one(df, n_boot=N_BOOT):
    wells = df["sample_id"].unique()
    n_wells = len(wells)
    by_well = {w: g[["true_label", "predicted_probability"]].to_numpy() for w, g in df.groupby("sample_id")}

    metrics = {"pr_auc": [], "roc_auc": [], "recall": [], "f2": [], "brier": [], "cal_slope": [], "cal_intercept": []}

    for _ in range(n_boot):
        draw = RNG.choice(wells, size=n_wells, replace=True)
        y_true, y_prob = [], []
        for w in draw:
            rows = by_well[w]
            idx = RNG.integers(0, len(rows))
            y_true.append(rows[idx, 0])
            y_prob.append(rows[idx, 1])
        y_true = np.array(y_true)
        y_prob = np.array(y_prob)

        if len(np.unique(y_true)) < 2:
            continue

        metrics["pr_auc"].append(average_precision_score(y_true, y_prob))
        metrics["roc_auc"].append(roc_auc_score(y_true, y_prob))
        metrics["recall"].append(recall_score(y_true, (y_prob >= 0.5).astype(int), zero_division=0))
        metrics["f2"].append(f2_score(y_true, y_prob))
        metrics["brier"].append(brier_score_loss(y_true, y_prob))
        slope, intercept = calibration_slope_intercept(y_true, y_prob)
        if not np.isnan(slope):
            metrics["cal_slope"].append(slope)
            metrics["cal_intercept"].append(intercept)

    return metrics


def summarize(vals):
    arr = np.array(vals)
    if len(arr) == 0:
        return (np.nan, np.nan, np.nan)
    return (np.median(arr), np.percentile(arr, 2.5), np.percentile(arr, 97.5))


def main():
    preds = pd.read_csv(PRED_PATH)
    preds = preds[preds["cv_mode"] == CV_MODE]

    rows = []
    for target, tier, algo in SELECTIONS:
        sub = preds[(preds["target_contaminant"] == target) & (preds["predictor_tier"] == tier) & (preds["algorithm"] == algo)]
        if sub.empty:
            print(f"WARNING: no rows for {target}/{tier}/{algo}")
            continue
        n_pred_rows = len(sub)
        n_wells = sub["sample_id"].nunique()
        metrics = bootstrap_one(sub)

        row = {"target": target, "tier": tier, "algorithm": algo, "n_wells": n_wells, "n_pred_rows": n_pred_rows}
        for m in ["pr_auc", "roc_auc", "recall", "f2", "brier", "cal_slope", "cal_intercept"]:
            med, lo, hi = summarize(metrics[m])
            row[f"{m}_median"] = round(med, 4) if not np.isnan(med) else np.nan
            row[f"{m}_ci_lo"] = round(lo, 4) if not np.isnan(lo) else np.nan
            row[f"{m}_ci_hi"] = round(hi, 4) if not np.isnan(hi) else np.nan
        rows.append(row)
        print(f"{target:4s} {tier:32s} {algo:20s}  PR-AUC {row['pr_auc_median']:.3f} [{row['pr_auc_ci_lo']:.3f}, {row['pr_auc_ci_hi']:.3f}]")

    out = pd.DataFrame(rows)
    out.to_csv("outputs/r_exports/bootstrap_ci_summary.csv", index=False)
    print("\nWritten: outputs/r_exports/bootstrap_ci_summary.csv")


if __name__ == "__main__":
    main()
