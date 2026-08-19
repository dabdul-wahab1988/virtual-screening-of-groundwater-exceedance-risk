"""Orchestrate the revised-pipeline runs in parallel partitions.

For each analysis configuration (main, WHO-guideline thresholds, BDL
substitution variants, k-sensitivity), each (target x cv_mode) partition runs
its own SQLite database (init -> ingest -> run-pipeline with filters), then
all partitions are merged into a canonical database with
merge_partition_dbs.py.

Usage:
    python run_revision_pipeline.py
"""

import csv
import os
import shutil
import subprocess
import sys
import concurrent.futures
from pathlib import Path

ROOT = Path(r"C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening")
GVS = ROOT / "target_rev2" / "release" / "gvs.exe"
OUT = ROOT / "outputs" / "rev2"
MAX_WORKERS = 10

ALL_TARGETS = ["Na", "Cl", "TDS", "B", "F", "NO3"]
TIERS_ALL = ["Tier1_Field", "Tier2_Reduced", "Tier3_Full"]
TIERS_NO_TIER1 = ["Tier2_Reduced", "Tier3_Full"]


def threshold_for(cfg_name: str, target: str) -> float | None:
    """Read threshold from the YAML for eligibility pre-checks."""
    path = ROOT / "configs" / f"thresholds{'_who_guideline' if cfg_name == 'who_guideline' else ''}.yaml"
    text = (path if path.exists() else ROOT / "configs" / "thresholds.yaml").read_text()
    import re
    m = re.search(rf"{target}:\n\s+column:.*\n\s+threshold_value:\s*([\d.]+)", text)
    return float(m.group(1)) if m else None


def positives_at(csv_path: Path, target: str, threshold: float) -> int:
    with open(csv_path, newline="") as f:
        rows = list(csv.DictReader(f))
    return sum(1 for r in rows if _num(r.get(target)) is not None and _num(r[target]) > threshold)


def _num(v):
    try:
        return float(v)
    except (TypeError, ValueError):
        return None


def run_one(cfg_name: str, target: str, cv_mode: str, thresholds_yaml: str,
            tiers: list[str], extra_args: list[str], cfg_yaml: str, log_path: Path) -> str:
    db_dir = OUT / cfg_name / "dbs"
    db_dir.mkdir(parents=True, exist_ok=True)
    db = db_dir / f"{target}_{cv_mode}.db"
    cmds = [
        [str(GVS), "init-db", "--db", str(db)],
        [str(GVS), "ingest-inputs",
         "--data", str(ROOT / "data/raw/pru.csv"),
         "--outline", str(ROOT / "manuscript/Outline.txt"),
         "--thresholds", str(ROOT / "configs" / thresholds_yaml),
         "--leakage", str(ROOT / "configs/leakage_rules.yaml"),
         "--config", cfg_yaml,
         "--db", str(db)],
        [str(GVS), "run-pipeline",
         "--db", str(db), "--config", cfg_yaml,
         "--targets", target, "--cv-modes", cv_mode,
         "--tiers", ",".join(tiers)] + extra_args,
    ]
    with open(log_path, "w") as f:
        for cmd in cmds:
            f.write("CMD: " + " ".join(cmd) + "\n")
            f.flush()
            try:
                r = subprocess.run(cmd, cwd=str(ROOT), stdout=f, stderr=subprocess.STDOUT,
                                   timeout=4 * 3600)
                f.write(f"EXIT: {r.returncode}\n")
                if r.returncode != 0:
                    return f"FAIL {target}/{cv_mode} ({r.returncode})"
            except subprocess.TimeoutExpired:
                f.write("EXIT: TIMEOUT\n")
                return f"TIMEOUT {target}/{cv_mode}"
    return f"OK {target}/{cv_mode}"


def plan(cfg_name: str, thresholds_yaml: str, tiers, extra_args, targets=None,
         spatial_only=False, cfg_yaml=None) -> list[tuple]:
    jobs = []
    for t in (targets or ALL_TARGETS):
        # eligibility pre-check for sensitivity configs
        thr = threshold_for(cfg_name, t) if cfg_name != "main" else None
        if thr is not None:
            p = positives_at(ROOT / "data/raw/pru.csv", t, thr)
            if p < 5 or (81 - p) < 5:
                print(f"  [{cfg_name}] {t}: {p} positives at {thr} -> not eligible, skipped")
                continue
        for cv in (["Spatial_Group_CV"] if spatial_only else ["Stratified_Nested_CV", "Spatial_Group_CV"]):
            log = OUT / cfg_name / "dbs" / f"{t}_{cv}.log"
            jobs.append((cfg_name, t, cv, thresholds_yaml, tiers, extra_args,
                         cfg_yaml or str(ROOT / "configs/config.yaml"), log))
    return jobs


def merge_partitions(cfg_name: str, thresholds_yaml: str, cfg_yaml: str) -> Path:
    db_dir = OUT / cfg_name / "dbs"
    canonical = OUT / cfg_name / f"{cfg_name}.db"
    if canonical.exists():
        canonical.unlink()
    subprocess.run([str(GVS), "init-db", "--db", str(canonical)], check=True, cwd=str(ROOT),
                   capture_output=True)
    subprocess.run([str(GVS), "ingest-inputs",
                    "--data", str(ROOT / "data/raw/pru.csv"),
                    "--outline", str(ROOT / "manuscript/Outline.txt"),
                    "--thresholds", str(ROOT / "configs" / thresholds_yaml),
                    "--leakage", str(ROOT / "configs/leakage_rules.yaml"),
                    "--config", cfg_yaml,
                    "--db", str(canonical)], check=True, cwd=str(ROOT), capture_output=True)
    parts = sorted(db_dir.glob("*.db"))
    subprocess.run([sys.executable, str(ROOT / "scripts/merge_partition_dbs.py"),
                    str(canonical)] + [str(p) for p in parts], check=True, cwd=str(ROOT))
    # Export R and GIS outputs from the canonical DB
    for sub, out in [("export-r", "r_exports"), ("export-gis", "gis_exports")]:
        subprocess.run([str(GVS), sub, "--db", str(canonical),
                        "--out", str(OUT / cfg_name / out)], check=True, cwd=str(ROOT),
                       capture_output=True)
    return canonical


def run_jobs(jobs, label):
    print(f"=== {label}: {len(jobs)} partitions ===")
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_WORKERS) as ex:
        futs = {ex.submit(run_one, *j): j for j in jobs}
        for fut in concurrent.futures.as_completed(futs):
            j = futs[fut]
            try:
                results.append(fut.result())
                print(f"  done {j[1]}/{j[2]} ({j[0]})")
            except Exception as e:
                results.append(f"ERROR {j[1]}/{j[2]}: {e}")
    fails = [r for r in results if not r.startswith("OK")]
    print(f"  completed {len(results) - len(fails)}/{len(results)}; failures: {fails}")
    return fails


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    all_fails = []

    # 1) MAIN analysis (canonical thresholds, all tiers, all variants)
    jobs = plan("main", "thresholds.yaml", TIERS_ALL, [])
    all_fails += run_jobs(jobs, "MAIN")
    if not all_fails:
        merge_partitions("main", "thresholds.yaml", str(ROOT / "configs/config.yaml"))
        print("merged main")

    # 2) WHO-guideline threshold sensitivity (no variants)
    jobs = plan("who_guideline", "thresholds_who_guideline.yaml", TIERS_ALL, ["--no-variants"])
    all_fails += run_jobs(jobs, "WHO_GUIDELINE")
    if not all_fails:
        merge_partitions("who_guideline", "thresholds_who_guideline.yaml",
                         str(ROOT / "configs/config.yaml"))
        print("merged who_guideline")

    # 3) BDL substitution variants (Tier2+Tier3 only; Tier1 has no F predictor)
    bdl_variants = {
        "bdl_lod0p5": "thresholds_bdl_lod0p5.yaml",
        "bdl_lod0p2": "thresholds_bdl_lod0p2.yaml",
        "bdl_limit_direct": "thresholds_bdl_limit_direct.yaml",
    }
    for variant, yaml_name in bdl_variants.items():
        jobs = plan(variant, yaml_name, TIERS_NO_TIER1, ["--no-variants"])
        all_fails += run_jobs(jobs, f"BDL {variant}")
        if not all_fails:
            merge_partitions(variant, yaml_name, str(ROOT / "configs/config.yaml"))
            print(f"merged {variant}")

    # 4) k-sensitivity (forced cluster counts; spatial CV only, no variants)
    for k in [3, 5, 6, 7, 8]:
        cfg_k = OUT / f"config_k{k}.yaml"
        text = (ROOT / "configs/config.yaml").read_text()
        import re
        text = re.sub(r"number_of_spatial_clusters:\s*null",
                      f"number_of_spatial_clusters: {k}", text)
        text = text.replace("run_coordinate_ablation:     true",
                            "run_coordinate_ablation:     false")
        cfg_k.write_text(text)
        jobs = plan(f"k{k}", "thresholds.yaml", TIERS_ALL, ["--no-variants"],
                    spatial_only=True, cfg_yaml=str(cfg_k))
        all_fails += run_jobs(jobs, f"K={k}")
        if not all_fails:
            # merge into a k-specific canonical DB (spatial partitions only)
            db_dir = OUT / f"k{k}" / "dbs"
            canonical = OUT / f"k{k}" / f"k{k}.db"
            if canonical.exists():
                canonical.unlink()
            subprocess.run([str(GVS), "init-db", "--db", str(canonical)], check=True,
                           cwd=str(ROOT), capture_output=True)
            subprocess.run([str(GVS), "ingest-inputs",
                            "--data", str(ROOT / "data/raw/pru.csv"),
                            "--outline", str(ROOT / "manuscript/Outline.txt"),
                            "--thresholds", str(ROOT / "configs/thresholds.yaml"),
                            "--leakage", str(ROOT / "configs/leakage_rules.yaml"),
                            "--config", str(cfg_k),
                            "--db", str(canonical)], check=True, cwd=str(ROOT),
                           capture_output=True)
            parts = sorted(db_dir.glob("*.db"))
            subprocess.run([sys.executable, str(ROOT / "scripts/merge_partition_dbs.py"),
                            str(canonical)] + [str(p) for p in parts], check=True, cwd=str(ROOT))
            subprocess.run([str(GVS), "export-r", "--db", str(canonical),
                            "--out", str(OUT / f"k{k}" / "r_exports")], check=True,
                           cwd=str(ROOT), capture_output=True)
            print(f"merged k{k}")

    if all_fails:
        print("FAILURES:", all_fails)
        sys.exit(1)
    print("ALL REVISION RUNS COMPLETE")


if __name__ == "__main__":
    main()
