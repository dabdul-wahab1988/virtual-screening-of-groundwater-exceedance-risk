"""Re-run the MAIN pipeline partitions with the exact-TreeSHAP binary into a
fresh canonical database (after archiving the pre-TreeSHAP record), then merge
and export. Model predictions are unchanged; SHAP values and base values are
regenerated.

Usage:
    python rerun_main_treeshap.py
"""

import os
import shutil
import subprocess
import sys
import concurrent.futures
from pathlib import Path

ROOT = Path(r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening")
GVS = ROOT / "target_rev2" / "release" / "gvs.exe"
OUT = ROOT / "outputs" / "rev2" / "main"
ARCHIVE = ROOT / "outputs" / "rev2" / "main_pre_treeshap_archive"
MAX_WORKERS = 10
TARGETS = ["Na", "Cl", "TDS", "B", "F"]
CV_MODES = ["Stratified_Nested_CV", "Spatial_Group_CV"]


def run_one(target, cv_mode):
    db = OUT / "dbs" / f"{target}_{cv_mode}.db"
    log = OUT / "dbs" / f"{target}_{cv_mode}.treeshap.log"
    cmds = [
        [str(GVS), "init-db", "--db", str(db)],
        [str(GVS), "ingest-inputs",
         "--data", str(ROOT / "data/raw/pru.csv"),
         "--outline", str(ROOT / "manuscript/Outline.txt"),
         "--thresholds", str(ROOT / "configs/thresholds.yaml"),
         "--leakage", str(ROOT / "configs/leakage_rules.yaml"),
         "--config", str(ROOT / "configs/config.yaml"),
         "--db", str(db)],
        [str(GVS), "run-pipeline", "--db", str(db),
         "--config", str(ROOT / "configs/config.yaml"),
         "--targets", target, "--cv-modes", cv_mode],
    ]
    with open(log, "w") as f:
        for cmd in cmds:
            f.write("CMD: " + " ".join(cmd) + "\n")
            f.flush()
            r = subprocess.run(cmd, cwd=str(ROOT), stdout=f, stderr=subprocess.STDOUT,
                               timeout=4 * 3600)
            f.write(f"EXIT: {r.returncode}\n")
            if r.returncode != 0:
                return f"FAIL {target}/{cv_mode} ({r.returncode})"
    return f"OK {target}/{cv_mode}"


def main():
    # 1. Archive the pre-TreeSHAP canonical DB + exports.
    ARCHIVE.mkdir(parents=True, exist_ok=True)
    canonical = OUT / "main.db"
    if canonical.exists():
        shutil.copy2(canonical, ARCHIVE / "main_pre_treeshap.db")
        print("archived pre-TreeSHAP main.db")

    # 2. Fresh canonical ingest.
    if canonical.exists():
        canonical.unlink()
    # Purge old partition DBs/exports (pre-TreeSHAP schema) so no stale rows
    # or mislabelled files survive into the new run.
    for sub in ["dbs", "r_exports", "gis_exports"]:
        d = OUT / sub
        if d.exists():
            shutil.rmtree(d)
        d.mkdir(parents=True, exist_ok=True)
    for cmd in [
        [str(GVS), "init-db", "--db", str(canonical)],
        [str(GVS), "ingest-inputs",
         "--data", str(ROOT / "data/raw/pru.csv"),
         "--outline", str(ROOT / "manuscript/Outline.txt"),
         "--thresholds", str(ROOT / "configs/thresholds.yaml"),
         "--leakage", str(ROOT / "configs/leakage_rules.yaml"),
         "--config", str(ROOT / "configs/config.yaml"),
         "--db", str(canonical)],
    ]:
        subprocess.run(cmd, cwd=str(ROOT), check=True, capture_output=True)
    print("fresh canonical DB initialised")

    # 3. Partition runs.
    jobs = [(t, c) for t in TARGETS for c in CV_MODES]
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_WORKERS) as ex:
        futs = {ex.submit(run_one, t, c): (t, c) for t, c in jobs}
        for fut in concurrent.futures.as_completed(futs):
            t, c = futs[fut]
            try:
                r = fut.result()
                print(f"  {r}")
                results.append(r)
            except Exception as e:
                print(f"  ERROR {t}/{c}: {e}")
                results.append(f"ERROR {t}/{c}")
    fails = [r for r in results if not r.startswith("OK")]
    if fails:
        print("FAILURES:", fails)
        sys.exit(1)

    # 4. Merge + export.
    parts = sorted((OUT / "dbs").glob("*.db"))
    parts = [p for p in parts if not p.name.startswith("main")]
    subprocess.run([sys.executable, str(ROOT / "scripts/merge_partition_dbs.py"),
                    str(canonical)] + [str(p) for p in parts], check=True, cwd=str(ROOT))
    subprocess.run([str(GVS), "export-r", "--db", str(canonical),
                    "--out", str(OUT / "r_exports")], check=True, cwd=str(ROOT),
                   capture_output=True)
    subprocess.run([str(GVS), "export-gis", "--db", str(canonical),
                    "--out", str(OUT / "gis_exports")], check=True, cwd=str(ROOT),
                   capture_output=True)
    print("MERGED + EXPORTED — TreeSHAP main run complete")


if __name__ == "__main__":
    main()
