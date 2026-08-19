"""Well-level bootstrap confidence intervals for headline classifiers.

Sampling unit: the WELL. Each bootstrap draw resamples wells with replacement
and uses each well's consolidated median out-of-fold probability (one
contribution per well), so repeated cross-validation predictions can never
inflate apparent precision.

Headline selections: for each (target, tier) the best canonical non-dummy
algorithm by pooled OOF PR-AUC under stratified nested CV
(reads outputs/rev2/main/r_exports/pooled_metrics.csv).

Metrics per draw: PR-AUC, ROC-AUC, recall at 0.5, F1/F1.5/F2/F3 at 0.5,
Brier, Brier skill vs the observed prevalence, calibration slope/intercept
(logistic regression of the outcome on logit(p)).

Output: outputs/rev2/main/r_exports/bootstrap_ci_summary.csv
"""

import warnings
warnings.filterwarnings("ignore")
import numpy as np
import pandas as pd
from sklearn.metrics import average_precision_score, roc_auc_score, recall_score, brier_score_loss
from sklearn.linear_model import LogisticRegression

ROOT = r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening"
POOLED = ROOT + r"\outputs\rev2\main\r_exports\pooled_metrics.csv"
WELL = ROOT + r"\outputs\rev2\main\r_exports\well_level_metrics.csv"
OUT = ROOT + r"\outputs\rev2\main\r_exports\bootstrap_ci_summary.csv"

N_BOOT = 2000
SEED = 20260819
THRESH = 0.5


def f_beta(precision, recall, beta):
    b2 = beta * beta
    denom = b2 * precision + recall
    num = (1.0 + b2) * precision * recall
    with np.errstate(divide="ignore", invalid="ignore"):
        out = np.divide(num, denom, out=np.zeros_like(denom), where=denom > 0)
    return out


def metrics_for(y, p):
    n_pos = int(y.sum())
    n_neg = int(len(y) - n_pos)
    out = {}
    if n_pos > 0 and n_neg > 0:
        out["pr_auc"] = average_precision_score(y, p)
        out["roc_auc"] = roc_auc_score(y, p)
        out["recall"] = recall_score(y, (p >= THRESH).astype(int))
        tp = np.sum((y == 1) & (p >= THRESH))
        tn = np.sum((y == 0) & (p < THRESH))
        fp = np.sum((y == 0) & (p >= THRESH))
        fn = np.sum((y == 1) & (p < THRESH))
        prec = tp / (tp + fp) if (tp + fp) > 0 else 0.0
        out["precision"] = prec
        out["f1"] = f_beta(prec, out["recall"], 1.0)
        out["f1_5"] = f_beta(prec, out["recall"], 1.5)
        out["f2"] = f_beta(prec, out["recall"], 2.0)
        out["f3"] = f_beta(prec, out["recall"], 3.0)
        out["brier"] = brier_score_loss(y, p)
        prev = n_pos / len(y)
        null_brier = prev * (1 - prev)
        out["brier_skill"] = 1.0 - out["brier"] / null_brier if null_brier > 1e-12 else 0.0
        lp = np.clip(np.log(p.clip(1e-6, 1 - 1e-6) / (1 - p.clip(1e-6, 1 - 1e-6))), -20, 20)
        lr = LogisticRegression(max_iter=2000)
        lr.fit(lp.reshape(-1, 1), y)
        out["cal_slope"] = lr.coef_[0][0]
        out["cal_intercept"] = lr.intercept_[0]
    return out


def main():
    pooled = pd.read_csv(POOLED)
    well = pd.read_csv(WELL)

    # Headline selections: best canonical non-dummy algorithm per (target, tier)
    # under stratified nested CV by pooled OOF PR-AUC.
    sel = pooled[
        (pooled["cv_mode"] == "Stratified_Nested_CV")
        & (pooled["scope"] == "all_held_out")
        & (~pooled["algorithm"].str.startswith("Dummy"))
        & (pooled["predictor_tier"].isin(["Tier1_Field", "Tier2_Reduced", "Tier3_Full"]))
    ].copy()
    # Exclude sensitivity variants (tier names with suffixes)
    sel = sel[~sel["predictor_tier"].str.contains("no_coords|EC_strict|EC_inclusive")]
    best_idx = (
        sel.sort_values("pr_auc", ascending=False)
        .groupby(["target_contaminant", "predictor_tier"], sort=False)
        .head(1)
        .index
    )
    headline = sel.loc[best_idx]
    # The TDS EC-strict sensitivity model is a headline recommendation and is
    # bootstrapped explicitly (it is excluded from the canonical filter above).
    strict = pooled[
        (pooled["cv_mode"] == "Stratified_Nested_CV")
        & (pooled["scope"] == "all_held_out")
        & (~pooled["algorithm"].str.startswith("Dummy"))
        & (pooled["predictor_tier"] == "Tier2_Reduced_TDS_EC_strict")
    ]
    if not strict.empty:
        strict_best = strict.sort_values("pr_auc", ascending=False).head(1)
        headline = pd.concat([headline, strict_best], ignore_index=True)
    print(f"Headline selections: {len(headline)}")

    rng = np.random.default_rng(SEED)
    records = []
    for _, row in headline.iterrows():
        run_id = row["run_id"]
        wm = well[(well["run_id"] == run_id) & well["median_probability"].notna()
                  & well["true_label"].notna()]
        if len(wm) < 10:
            print(f"  skip {run_id}: {len(wm)} wells")
            continue
        y_all = wm["true_label"].astype(int).to_numpy(dtype=int)
        p_all = wm["median_probability"].to_numpy(dtype=float)
        idx_all = np.arange(len(wm))

        draws = {}
        for _ in range(N_BOOT):
            bi = rng.choice(idx_all, size=len(idx_all), replace=True)
            y_b, p_b = y_all[bi], p_all[bi]
            if y_b.sum() == 0 or (len(y_b) - y_b.sum()) == 0:
                continue
            m = metrics_for(y_b, p_b)
            if not m:
                continue
            for k, v in m.items():
                draws.setdefault(k, []).append(v)
        if len(draws.get("pr_auc", [])) < 100:
            print(f"  skip {run_id}: too few valid draws")
            continue

        rec = {
            "target": row["target_contaminant"],
            "tier": row["predictor_tier"],
            "algorithm": row["algorithm"],
            "n_wells": len(wm),
        }
        for k, vals in draws.items():
            a = np.asarray(vals)
            rec[f"{k}_median"] = float(np.median(a))
            rec[f"{k}_ci_lo"] = float(np.percentile(a, 2.5))
            rec[f"{k}_ci_hi"] = float(np.percentile(a, 97.5))
        # Point estimates on the observed well-level sample
        obs = metrics_for(y_all, p_all)
        for k, v in obs.items():
            rec[f"{k}_point"] = float(v)
        records.append(rec)

    summary = pd.DataFrame(records)
    summary.to_csv(OUT, index=False)
    print(f"Wrote {OUT} ({len(summary)} rows)")


if __name__ == "__main__":
    main()
