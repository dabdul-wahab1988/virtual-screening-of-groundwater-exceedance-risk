"""
Summarize spatial-group CV pooled PR-AUC across alternative spatial cluster
counts (k = 3, 5, 6, 7, 8 from outputs/rev2/k{k}/k{k}.db, plus k = 4 from
outputs/rev2/main/main.db) for every canonical (target, tier, algorithm)
combination, using pooled metrics across all unique held-out wells
(scope = 'all_held_out'). Non-evaluable cells (no evaluable spatial fold
coverage) are marked NE.

Output: outputs/rev2/main/r_exports/spatial_k_sensitivity_summary.csv
"""

import os
import sqlite3

import pandas as pd

ROOT = r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening"
OUT_CSV = ROOT + r"\outputs\rev2\main\r_exports\spatial_k_sensitivity_summary.csv"

K_VALUES = [3, 4, 5, 6, 7, 8]

QUERY = """
SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm,
       pm.pr_auc
FROM pooled_metrics pm
JOIN model_runs mr ON pm.run_id = mr.run_id
WHERE mr.run_status = 'completed'
  AND mr.algorithm NOT LIKE 'Dummy%'
  AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  AND pm.cv_mode = 'Spatial_Group_CV'
  AND pm.scope = 'all_held_out'
"""


def read_k(k: int) -> pd.DataFrame:
    if k == 4:
        db = ROOT + r"\outputs\rev2\main\main.db"
    else:
        db = ROOT + rf"\outputs\rev2\k{k}\k{k}.db"
    if not os.path.exists(db):
        print(f"MISSING db for k={k}: {db}")
        return pd.DataFrame()
    con = sqlite3.connect(db)
    df = pd.read_sql_query(QUERY, con)
    con.close()
    return df


def main():
    frames = {}
    for k in K_VALUES:
        frames[k] = read_k(k)

    all_keys = set()
    for k, df in frames.items():
        for _, r in df.iterrows():
            all_keys.add((r["target_contaminant"], r["predictor_tier"], r["algorithm"]))

    rows = []
    for (t, tier, algo) in sorted(all_keys):
        rec = {"target_contaminant": t, "predictor_tier": tier, "algorithm": algo}
        vals = []
        for k in K_VALUES:
            df = frames[k]
            if df.empty:
                rec[f"k{k}"] = "NE"
                continue
            sub = df[(df.target_contaminant == t) & (df.predictor_tier == tier)
                     & (df.algorithm == algo)]
            if sub.empty:
                rec[f"k{k}"] = "NE"
            else:
                v = float(sub.pr_auc.iloc[0])
                rec[f"k{k}"] = round(v, 3)
                vals.append(v)
        rec["range"] = round(max(vals) - min(vals), 3) if len(vals) >= 2 else None
        rows.append(rec)

    out = pd.DataFrame(rows)
    out.to_csv(OUT_CSV, index=False)
    print(f"Wrote {OUT_CSV} ({len(out)} rows)")
    print(out.to_string(index=False))


if __name__ == "__main__":
    main()
