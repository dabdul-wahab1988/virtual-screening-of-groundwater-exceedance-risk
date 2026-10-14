"""
Summarise spatial-group CV pooled PR-AUC across spatial cluster counts k.

Reads the project's per-k databases read-only:
    outputs/rev2/k3/k3.db, outputs/rev2/k5/k5.db, outputs/rev2/k6/k6.db,
    outputs/rev2/k7/k7.db, outputs/rev2/k8/k8.db
    outputs/rev2/main/main.db  (k = 4, the canonical run)

For every canonical (target, predictor tier, algorithm) combination it takes the
pooled Spatial_Group_CV PR-AUC over all unique held-out wells
(pooled_metrics, scope = 'all_held_out'), reports it for each k, and the range
across the evaluable k. Cells with no completed run, or an undefined (NULL)
PR-AUC, are marked NE.

Output: outputs/rev2/main/r_exports/spatial_k_sensitivity_summary.csv

Paths are resolved relative to this script, so it runs from any working
directory. A missing database or a duplicated canonical run is an error; no
partial summary is written.
"""

import csv
import sqlite3
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
REV2 = PROJECT_ROOT / "outputs" / "rev2"
OUT_CSV = REV2 / "main" / "r_exports" / "spatial_k_sensitivity_summary.csv"

# k -> database holding the Spatial_Group_CV runs for that cluster count.
K_DATABASES = {
    3: REV2 / "k3" / "k3.db",
    4: REV2 / "main" / "main.db",
    5: REV2 / "k5" / "k5.db",
    6: REV2 / "k6" / "k6.db",
    7: REV2 / "k7" / "k7.db",
    8: REV2 / "k8" / "k8.db",
}

# Canonical predictor tiers only. Sensitivity variants (no_coords,
# *_EC_strict, *_EC_inclusive) are stored under their own tier names and are
# excluded here.
CANONICAL_TIERS = ("Tier1_Field", "Tier2_Reduced", "Tier3_Full")

QUERY = """
SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, pm.pr_auc
FROM pooled_metrics pm
JOIN model_runs mr ON pm.run_id = mr.run_id
WHERE mr.run_status = 'completed'
  AND mr.algorithm NOT LIKE 'Dummy%'
  AND mr.predictor_tier IN (?, ?, ?)
  AND mr.cv_mode = 'Spatial_Group_CV'
  AND pm.cv_mode = 'Spatial_Group_CV'
  AND pm.scope = 'all_held_out'
"""


def connect_readonly(db: Path) -> sqlite3.Connection:
    """Open an existing database read-only so a running pipeline is never affected."""
    uri = db.resolve().as_uri() + "?mode=ro"
    return sqlite3.connect(uri, uri=True)


def load_k(k: int, db: Path) -> dict:
    """Return {(target, tier, algorithm): pooled PR-AUC or None} for one k."""
    con = connect_readonly(db)
    try:
        rows = con.execute(QUERY, CANONICAL_TIERS).fetchall()
    finally:
        con.close()
    values = {}
    for target, tier, algorithm, pr_auc in rows:
        key = (target, tier, algorithm)
        if key in values:
            raise RuntimeError(f"duplicate canonical run for {key} in k={k} ({db})")
        values[key] = pr_auc
    return values


def main() -> None:
    missing = [str(p) for p in K_DATABASES.values() if not p.is_file()]
    if missing:
        sys.exit("Missing database(s); no summary written:\n  " + "\n  ".join(missing))

    per_k = {k: load_k(k, db) for k, db in K_DATABASES.items()}
    keys = sorted(set().union(*(v.keys() for v in per_k.values())))

    fieldnames = ["target", "tier", "algorithm"] + [f"k{k}" for k in K_DATABASES] + ["range"]
    rows = []
    for key in keys:
        target, tier, algorithm = key
        row = {"target": target, "tier": tier, "algorithm": algorithm}
        evaluable = []
        for k in K_DATABASES:
            value = per_k[k].get(key)
            if value is None:
                row[f"k{k}"] = "NE"
            else:
                row[f"k{k}"] = f"{value:.3f}"
                evaluable.append(value)
        row["range"] = f"{max(evaluable) - min(evaluable):.3f}" if len(evaluable) >= 2 else "NE"
        rows.append(row)

    OUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUT_CSV.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"Wrote {OUT_CSV} ({len(rows)} rows)")


if __name__ == "__main__":
    main()
