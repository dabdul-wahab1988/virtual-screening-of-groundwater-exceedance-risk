#!/usr/bin/env python3
"""Raw-data audit for the revision.

Produces:
  TableS17_TDS_EC_identity.csv  - TDS/EC identity, TDS and EC label audits, ion-sum relations
  TableS18_B_threshold_audit.csv - count of wells above boron and fluoride screening thresholds

Input : data/raw/pru.csv (81 wells; 'bdl' and blanks treated as non-numeric)
Stdlib only. Run as: python -I scripts/raw_data_audit.py
"""
import csv
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "data" / "raw" / "pru.csv"
OUT_DIR = ROOT / "outputs" / "rev2" / "tables"
OUT_S17 = OUT_DIR / "TableS17_TDS_EC_identity.csv"
OUT_S18 = OUT_DIR / "TableS18_B_threshold_audit.csv"

MAJOR_IONS = ["Na", "K", "Mg", "Ca", "Cl", "SO4", "HCO3"]
ION_SUM_8 = MAJOR_IONS + ["CO3"]  # CO3 bdl/blank counted as 0

TDS_LABEL_CUTOFF = 750.0
TDS_LABEL_CUTOFF_2 = 1000.0
SCAN_RANGE = range(700, 1401)  # integer EC cutoffs (uS/cm)


def to_num(s):
    """Return a float, or None for blank, 'bdl' or other non-numeric text."""
    if s is None:
        return None
    t = s.strip()
    if t == "" or t.lower() == "bdl":
        return None
    try:
        return float(t)
    except ValueError:
        return None


def mean(xs):
    return sum(xs) / len(xs)


def sample_sd(xs):
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def pearson(xs, ys):
    mx, my = mean(xs), mean(ys)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    sxx = sum((x - mx) ** 2 for x in xs)
    syy = sum((y - my) ** 2 for y in ys)
    return sxy / math.sqrt(sxx * syy)


def ols(xs, ys):
    """Return (slope, intercept, R^2) for ys regressed on xs."""
    mx, my = mean(xs), mean(ys)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    sxx = sum((x - mx) ** 2 for x in xs)
    slope = sxy / sxx
    intercept = my - slope * mx
    r = pearson(xs, ys)
    return slope, intercept, r * r


def f4(x):
    return f"{x:.4f}"


def ec_cutoff_scan(ec, tds, tds_cut):
    """Scan integer EC cutoffs 700-1400; predict TDS>tds_cut as EC>c and count mismatches."""
    labels = [t > tds_cut for t in tds]
    mismatches = {}
    for c in SCAN_RANGE:
        mismatches[c] = sum((e > c) != lab for e, lab in zip(ec, labels))
    zero = [c for c in SCAN_RANGE if mismatches[c] == 0]
    min_mism = min(mismatches.values())
    min_cutoffs = [c for c in SCAN_RANGE if mismatches[c] == min_mism]
    return {
        "zero": zero,
        "min_mism": min_mism,
        "min_cutoffs": min_cutoffs,
        "mism_at_833": mismatches[833],
    }


def fmt_list(cs):
    """Compact representation of integer cutoffs as ranges."""
    if not cs:
        return "none"
    runs, start, prev = [], cs[0], cs[0]
    for c in cs[1:]:
        if c == prev + 1:
            prev = c
            continue
        runs.append((start, prev))
        start = prev = c
    runs.append((start, prev))
    return ", ".join(str(a) if a == b else f"{a}-{b}" for a, b in runs)


def main():
    with open(RAW, encoding="utf-8-sig", newline="") as fh:
        rows = list(csv.DictReader(fh))
    n_total = len(rows)

    # Wells with both EC and TDS numeric
    both = []
    for r in rows:
        ec_v, tds_v = to_num(r["EC"]), to_num(r["TDS"])
        if ec_v is not None and tds_v is not None:
            both.append((r, ec_v, tds_v))
    n_both = len(both)
    ec = [e for _, e, _ in both]
    tds = [t for _, _, t in both]
    ratios = [t / e for e, t in zip(ec, tds)]

    # Ion sums (CO3 bdl/blank -> 0); 7-ion variant without CO3 as sensitivity
    sum8, sum7, imputed_co3 = [], [], 0
    for r, _, _ in both:
        vals = {}
        for ion in ION_SUM_8:
            v = to_num(r[ion])
            if v is None:
                if ion == "CO3":
                    imputed_co3 += 1
                v = 0.0
            vals[ion] = v
        sum8.append(sum(vals[i] for i in ION_SUM_8))
        sum7.append(sum(vals[i] for i in MAJOR_IONS))
    ion_ratio_8 = [s / t for s, t in zip(sum8, tds)]
    ion_ratio_7 = [s / t for s, t in zip(sum7, tds)]

    slope, intercept, r2 = ols(ec, tds)
    r_ec_tds = pearson(ec, tds)
    n_tds_750 = sum(1 for t in tds if t > TDS_LABEL_CUTOFF)
    n_tds_1000 = sum(1 for t in tds if t > TDS_LABEL_CUTOFF_2)

    scan750 = ec_cutoff_scan(ec, tds, TDS_LABEL_CUTOFF)
    scan1000 = ec_cutoff_scan(ec, tds, TDS_LABEL_CUTOFF_2)

    s17 = []  # (statistic, value, note)
    s17.append(("n_wells_total", n_total, "rows in data/raw/pru.csv"))
    s17.append(("n_wells_both_EC_TDS", n_both, "wells with numeric EC and TDS; all statistics below use this set"))
    s17.append(("TDS_EC_ratio_min", f4(min(ratios)), "min of per-well TDS/EC (mg/L per uS/cm)"))
    s17.append(("TDS_EC_ratio_max", f4(max(ratios)), "max of per-well TDS/EC"))
    s17.append(("TDS_EC_ratio_mean", f4(mean(ratios)), "arithmetic mean of per-well TDS/EC"))
    s17.append(("TDS_EC_ratio_SD", f4(sample_sd(ratios)), "sample SD (n-1) of per-well TDS/EC"))
    s17.append(("pearson_r_EC_TDS", f4(r_ec_tds), "Pearson r between EC and TDS"))
    s17.append(("OLS_slope_TDS_on_EC", f4(slope), "TDS (mg/L) = slope x EC + intercept; ordinary least squares"))
    s17.append(("OLS_intercept_TDS_on_EC", f4(intercept), "intercept of TDS on EC (mg/L)"))
    s17.append(("OLS_R2_TDS_on_EC", f4(r2), "R^2 of the OLS fit (equals r^2)"))
    s17.append(("n_TDS_gt_750", n_tds_750, "wells with TDS > 750 mg/L (strict)"))
    s17.append((
        "TDS750_EC_cutoff_lowest_zero_mismatch",
        str(min(scan750["zero"])) if scan750["zero"] else "none",
        "lowest integer EC cutoff in 700-1400 uS/cm reproducing TDS>750 label with zero mismatches (rule: EC > cutoff)",
    ))
    s17.append((
        "TDS750_EC_cutoff_highest_zero_mismatch",
        str(max(scan750["zero"])) if scan750["zero"] else "none",
        "highest integer EC cutoff in 700-1400 uS/cm reproducing TDS>750 label with zero mismatches",
    ))
    s17.append((
        "TDS750_EC_cutoff_zero_mismatch_set",
        fmt_list(scan750["zero"]),
        "all integer cutoffs with zero mismatches for TDS>750",
    ))
    s17.append((
        "TDS750_mismatches_at_EC_gt_833",
        scan750["mism_at_833"],
        "label mismatches (TDS>750 vs EC>833) at the 833 uS/cm cutoff",
    ))
    s17.append((
        "TDS750_min_mismatches_in_scan",
        scan750["min_mism"],
        f"minimum mismatches over 700-1400 scan; attained at cutoff(s): {fmt_list(scan750['min_cutoffs'])}",
    ))
    s17.append(("n_TDS_gt_1000", n_tds_1000, "wells with TDS > 1000 mg/L (strict)"))
    s17.append((
        "TDS1000_EC_cutoff_lowest_zero_mismatch",
        str(min(scan1000["zero"])) if scan1000["zero"] else "none",
        "lowest integer EC cutoff in 700-1400 uS/cm reproducing TDS>1000 label with zero mismatches; 'none' means no cutoff in scan range works",
    ))
    s17.append((
        "TDS1000_EC_cutoff_highest_zero_mismatch",
        str(max(scan1000["zero"])) if scan1000["zero"] else "none",
        "highest integer EC cutoff in 700-1400 uS/cm reproducing TDS>1000 label with zero mismatches",
    ))
    s17.append((
        "TDS1000_EC_cutoff_zero_mismatch_set",
        fmt_list(scan1000["zero"]),
        "all integer cutoffs with zero mismatches for TDS>1000",
    ))
    s17.append((
        "TDS1000_mismatches_at_EC_gt_833",
        scan1000["mism_at_833"],
        "label mismatches (TDS>1000 vs EC>833) at the 833 uS/cm cutoff",
    ))
    s17.append((
        "TDS1000_min_mismatches_in_scan",
        scan1000["min_mism"],
        f"minimum mismatches over 700-1400 scan; attained at cutoff(s): {fmt_list(scan1000['min_cutoffs'])}",
    ))
    s17.append((
        "ion_list_used",
        "Na+K+Mg+Ca+Cl+SO4+HCO3+CO3",
        f"primary ion sum; CO3 bdl/blank counted as 0 ({imputed_co3} wells); numeric values used as reported (mg/L)",
    ))
    s17.append(("pearson_r_TDS_ionsum", f4(pearson(sum8, tds)), "Pearson r between TDS and 8-ion sum (primary)"))
    s17.append(("mean_ratio_ionsum_over_TDS", f4(mean(ion_ratio_8)), "mean of per-well (8-ion sum)/TDS"))
    s17.append(("pearson_r_EC_ionsum", f4(pearson(ec, sum8)), "Pearson r between EC and 8-ion sum (primary)"))
    s17.append((
        "pearson_r_TDS_ionsum_7ion_noCO3",
        f4(pearson(sum7, tds)),
        "sensitivity: Pearson r between TDS and 7-ion sum (Na+K+Mg+Ca+Cl+SO4+HCO3, no CO3)",
    ))
    s17.append(("mean_ratio_ionsum7_over_TDS", f4(mean(ion_ratio_7)), "sensitivity: mean of per-well 7-ion sum/TDS"))

    # TableS18: boron and fluoride threshold counts (strictly greater than)
    b_vals = [to_num(r["B"]) for r in rows]
    f_vals = [to_num(r["F"]) for r in rows]
    n_b_nonnum = sum(1 for v in b_vals if v is None)
    n_f_nonnum = sum(1 for v in f_vals if v is None)

    def count_gt(vals, t):
        return sum(1 for v in vals if v is not None and v > t)

    def row_s18(label, vals, t, nonnum, note):
        pos = count_gt(vals, t)
        neg = n_total - pos
        suffix = f" {nonnum} non-numeric (bdl/blank) entries counted as negative." if nonnum else ""
        return (f"{label} > {t} mg/L", pos, neg, note + suffix)

    s18 = [
        row_s18("B", b_vals, 1.125, n_b_nonnum,
                "study-defined threshold inherited from original modelling config (configs/thresholds.yaml); "
                "numerically equals 0.75 x 1.5 mg/L (WHO fluoride guideline); no boron-specific derivation in project configuration"),
        row_s18("B", b_vals, 1.5, n_b_nonnum, "sensitivity value"),
        row_s18("B", b_vals, 1.8, n_b_nonnum, "75% of 2.4 mg/L (WHO boron guideline)"),
        row_s18("B", b_vals, 2.0, n_b_nonnum, "sensitivity value"),
        row_s18("B", b_vals, 2.4, n_b_nonnum, "WHO boron guideline value"),
        row_s18("F", f_vals, 1.125, n_f_nonnum, "75% of 1.5 mg/L WHO fluoride guideline"),
        row_s18("F", f_vals, 1.5, n_f_nonnum, "WHO fluoride guideline value"),
        ("NOTE", "NA", "NA",
         "1.125 mg/L equals 0.75 x 1.5 mg/L (WHO fluoride guideline). The boron threshold 1.125 mg/L therefore coincides "
         "numerically with 75% of the fluoride guideline and has no boron-specific derivation in the project configuration."),
    ]

    # Write CSVs
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    with open(OUT_S17, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["statistic", "value", "note"])
        w.writerows(s17)
    with open(OUT_S18, "w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["threshold", "n_positive", "n_negative", "note"])
        w.writerows(s18)

    # Print both tables
    print(f"== TableS17_TDS_EC_identity.csv ({OUT_S17}) ==")
    w1 = max(len(s) for s, _, _ in s17)
    w2 = max(len(str(v)) for _, v, _ in s17)
    print(f"{'statistic':<{w1}}  {'value':<{w2}}  note")
    for s, v, n in s17:
        print(f"{s:<{w1}}  {str(v):<{w2}}  {n}")
    print()
    print(f"== TableS18_B_threshold_audit.csv ({OUT_S18}) ==")
    w1 = max(len(s) for s, _, _, _ in s18)
    print(f"{'threshold':<{w1}}  n_positive  n_negative  note")
    for t, p, n, note in s18:
        print(f"{t:<{w1}}  {str(p):<10}  {str(n):<10}  {note}")


if __name__ == "__main__":
    main()
