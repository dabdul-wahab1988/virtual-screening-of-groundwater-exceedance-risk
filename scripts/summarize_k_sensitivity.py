"""
Summarize spatial-group CV PR-AUC across alternative spatial cluster counts (k)
for the Table-3 headline algorithm per target/tier, to answer reviewer Q2
(sensitivity of spatial validation to cluster construction).
"""
import pandas as pd
import glob
import os

BASE = r"C:\Users\THINKP~1\AppData\Local\Temp\claude\sensitivity_k"

# Table 3 headline (target, tier, algorithm) selections
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
]

rows = []

# baseline k=4 from the original committed outputs
baseline = pd.read_csv("outputs/r_exports/model_performance_summary.csv")
baseline = baseline[baseline["cv_mode"] == "Spatial_Group_CV"]
for target, tier, algo in SELECTIONS:
    sub = baseline[(baseline["target_contaminant"] == target) & (baseline["predictor_tier"] == tier) & (baseline["algorithm"] == algo)]
    if not sub.empty:
        rows.append({"k": 4, "target": target, "tier": tier, "algorithm": algo, "spatial_pr_auc": sub.iloc[0]["mean_pr_auc"]})

for k in [3, 5, 6, 7, 8]:
    csv_path = f"{BASE}/k{k}/r_exports/model_performance_summary.csv"
    if not os.path.exists(csv_path):
        print(f"MISSING: {csv_path}")
        continue
    df = pd.read_csv(csv_path)
    df = df[df["cv_mode"] == "Spatial_Group_CV"]
    for target, tier, algo in SELECTIONS:
        sub = df[(df["target_contaminant"] == target) & (df["predictor_tier"] == tier) & (df["algorithm"] == algo)]
        if not sub.empty:
            rows.append({"k": k, "target": target, "tier": tier, "algorithm": algo, "spatial_pr_auc": sub.iloc[0]["mean_pr_auc"]})
        else:
            rows.append({"k": k, "target": target, "tier": tier, "algorithm": algo, "spatial_pr_auc": None})

out = pd.DataFrame(rows)
pivot = out.pivot_table(index=["target", "tier", "algorithm"], columns="k", values="spatial_pr_auc")
pivot = pivot.reindex(columns=[3, 4, 5, 6, 7, 8])
pivot["range"] = pivot.max(axis=1) - pivot.min(axis=1)
pivot = pivot.round(4)
pivot.to_csv("outputs/r_exports/spatial_k_sensitivity_summary.csv")
print(pivot.to_string())
