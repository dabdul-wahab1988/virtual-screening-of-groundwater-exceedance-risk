# Virtual Screening of Groundwater Exceedance Risk

Reproducible, leakage-controlled machine learning for prioritising groundwater wells for confirmatory monitoring in Ghana's Pru Basin.

The project provides a Rust command-line pipeline that turns groundwater chemistry measurements into validated exceedance-risk estimates. It compares practical predictor tiers, evaluates models under both repeated nested and spatially grouped cross-validation, records the full analysis in SQLite, and exports analysis-ready CSV files for statistical graphics and GIS workflows.

> **Research-use notice:** this is a screening and prioritisation tool. Its predictions do not replace laboratory testing, hydrogeological investigation, or regulatory decision-making.

## Why this pipeline exists

Groundwater risk models can appear accurate when a target analyte, a direct derivative of that analyte, or a strongly coupled proxy leaks into the predictors. This project makes leakage control an explicit, auditable part of the workflow:

- target-specific exclusion rules are declared in YAML;
- preprocessing is fitted within each training fold;
- hyperparameters are selected by inner cross-validation;
- conventional repeated nested CV and spatial group CV are reported separately;
- dummy classifiers are retained as performance baselines;
- every input, run, fold assignment, prediction, metric, and exclusion decision is recorded in SQLite;
- fixed random seeds and SHA-256 input hashes support reproducibility.

## Analysis workflow

```mermaid
flowchart LR
    A["Groundwater CSV<br/>+ analysis outline"] --> B["Validate inputs"]
    C["Thresholds, leakage rules,<br/>and runtime configuration"] --> B
    B --> D["Ingest and audit<br/>SQLite provenance store"]
    D --> E["Target labels and<br/>predictor tiers"]
    E --> F["Leakage filter"]
    F --> G["Repeated nested CV"]
    F --> H["Spatial group CV"]
    G --> I["Metrics, calibration,<br/>predictions, attribution"]
    H --> I
    I --> J["R-ready exports"]
    I --> K["GIS-ready exports"]
    J --> L["Publication figures<br/>and tables"]
```

## What is modelled

The configured screening targets are sodium (`Na`), chloride (`Cl`), total dissolved solids (`TDS`), boron (`B`), fluoride (`F`), and nitrate (`NO3`). Threshold values, units, provenance notes, and below-detection-limit handling are defined in [`configs/thresholds.yaml`](configs/thresholds.yaml).

Three predictor tiers represent increasing analytical effort:

| Tier | Intended setting | Candidate information |
|---|---|---|
| `Tier1_Field` | Rapid field screening | pH, temperature, EC, and coordinates |
| `Tier2_Reduced` | Reduced laboratory panel | Tier 1 plus major ions |
| `Tier3_Full` | Full chemistry panel | Tier 2 plus additional analytes and derived ionic ratios |

Before a model is fitted, [`configs/leakage_rules.yaml`](configs/leakage_rules.yaml) removes the target, target-derived variables, label-like fields, and other prohibited predictors. TDS additionally has EC-inclusive and EC-strict sensitivity variants. EC is a legitimate predictor for TDS only as an identity: in this dataset TDS ≈ 0.9 × EC (median TDS/EC ratio 0.900, n = 81 in `data/raw/pru.csv`), so the EC-inclusive TDS result largely reflects that identity and the EC-strict variant is the check against EC-proxy circularity.

The pipeline evaluates majority and stratified baselines, EC-only logistic regression, regularised logistic regression, random forest, and gradient-boosted trees. Model implementations are native Rust and deterministic under the configured seed.

## Repository layout

```text
.
├── Cargo.toml                 Rust package and dependencies
├── Cargo.lock                 Reproducible Rust dependency resolution
├── configs/
│   ├── config.yaml            Runtime, CV, spatial, and output settings
│   ├── leakage_rules.yaml     Target-wise leakage exclusions
│   └── thresholds.yaml        Exceedance definitions and BDL handling
├── src/                       Rust CLI, models, validation, CV, and exports
├── scripts/                   Python and R sensitivity/post-processing tools
├── manuscript/                R code for publication figures and tables
└── DECISIONS.md               Important implementation and interpretation notes
```

Raw data, databases, compiled binaries, and generated exports are intentionally not versioned. They are reproducible products or study inputs and are excluded by `.gitignore`.

## Requirements

### Core pipeline

- a current stable [Rust toolchain](https://www.rust-lang.org/tools/install) with Cargo;
- no separate SQLite installation—the Rust dependency is built with bundled SQLite.

### Optional post-processing

- Python 3 with `numpy`, `pandas`, `scikit-learn`, and `python-docx`;
- R with `DBI`, `RSQLite`, `ggplot2`, `dplyr`, `tidyr`, `readr`, `scales`, `patchwork`, `viridis`, `ggrepel`, `RColorBrewer`, and `gridExtra`.

Install the Python analysis dependencies with:

```bash
python -m pip install numpy pandas scikit-learn python-docx
```

## Quick start

Clone and build the release executable:

```bash
git clone https://github.com/dabdul-wahab1988/virtual-screening-of-groundwater-exceedance-risk.git
cd virtual-screening-of-groundwater-exceedance-risk
cargo build --release
```

On Linux and macOS, the executable is `target/release/gvs`; on Windows it is `target\release\gvs.exe`. The examples below use `cargo run --release --`, which works on every supported platform.

### 1. Prepare the inputs

Place the study CSV at `data/raw/pru.csv` and the analysis outline at `manuscript/Outline.txt`, or substitute your own paths in the commands.

At minimum, the CSV should contain:

- `SampleID`: unique well/sample identifier;
- `Dx`, `Dy`: longitude and latitude used for spatial validation;
- the configured target columns: `Na`, `Cl`, `TDS`, `B`, `F`, and `NO3`;
- predictor columns needed by the selected tiers, such as `pH`, `Temp.`, `EC`, `K`, `Mg`, `Ca`, `HCO3`, `SO4`, and `CO3`.

Plain `bdl` values are recognised case-insensitively. Review the detection-limit policy in `configs/thresholds.yaml` before analysing a new dataset.

### 2. Validate and ingest

```bash
cargo run --release -- validate-inputs \
  --data data/raw/pru.csv \
  --outline manuscript/Outline.txt \
  --thresholds configs/thresholds.yaml \
  --leakage configs/leakage_rules.yaml \
  --config configs/config.yaml

cargo run --release -- init-db \
  --db outputs/groundwater_screening.db

cargo run --release -- ingest-inputs \
  --data data/raw/pru.csv \
  --outline manuscript/Outline.txt \
  --thresholds configs/thresholds.yaml \
  --leakage configs/leakage_rules.yaml \
  --config configs/config.yaml \
  --db outputs/groundwater_screening.db
```

PowerShell users can either enter each command on one line or replace the Bash continuation character (`\`) with PowerShell's backtick.

### 3. Audit leakage and run the models

```bash
cargo run --release -- audit-leakage \
  --db outputs/groundwater_screening.db

cargo run --release -- run-pipeline \
  --db outputs/groundwater_screening.db \
  --config configs/config.yaml
```

For a focused run:

```bash
cargo run --release -- run-target \
  --target F \
  --tier Tier1_Field \
  --cv-mode Spatial_Group_CV \
  --db outputs/groundwater_screening.db
```

Use `cargo run --release -- --help` or append `--help` to any subcommand for the complete CLI reference.

### 4. Export results

```bash
cargo run --release -- export-r \
  --db outputs/groundwater_screening.db \
  --out outputs/r_exports

cargo run --release -- export-gis \
  --db outputs/groundwater_screening.db \
  --out outputs/gis_exports
```

The R export includes target prevalence, the leakage audit, fold-level and aggregate performance, out-of-fold predictions, calibration inputs, operational thresholds, threshold sensitivity, feature attributions, and the screening-priority table. The GIS export contains well-level priority points, target-specific probabilities, spatial-cluster metadata, and a checksum manifest.

## Configuration

[`configs/config.yaml`](configs/config.yaml) controls:

- repeated and spatial CV fold counts;
- the number of repeats and deterministic random seed;
- minimum positive cases required for modelling;
- coordinate fields and spatial clustering;
- enabled model families;
- output switches and paths.

For a new study, copy the configuration files, document every threshold source, review all target-wise leakage exclusions, and change the seed only when a deliberately independent run is required.

## Reproducibility and interpretation

- Input files are registered with SHA-256 hashes.
- Fold assignments and model metadata are persisted in SQLite.
- Imputation and scaling are learned inside each training fold.
- PR-AUC is the inner-CV tuning criterion because exceedances are imbalanced.
- Both discrimination and calibration metrics are exported.
- Spatial CV estimates geographic-transfer performance and should not be conflated with randomly stratified CV.
- Tree explanations are exact path-dependent (cover-weighted) TreeSHAP values (Lundberg et al., 2018). The background distribution is the training fold for gradient-boosted trees and, for random forests, the bootstrap covers (node training counts of each tree's bootstrap sample). Random-forest values are on the probability scale (base value = mean of tree root probabilities); gradient-boosted values are on the log-odds scale. Linear-model explanations are exact SHAP values (Lundberg & Lee, 2017) on the log-odds scale. Each decomposition is exactly additive (base value + sum of SHAP values = model output). See [`DECISIONS.md`](DECISIONS.md) for this and other implementation choices.
- Operational probability cutoffs are selected inside each outer fold from inner-CV validation predictions only (fully nested threshold selection), frozen, and applied to the held-out test fold; the pooled post-hoc cutoffs are retained as a labelled comparison. `nested_operational_thresholds.cutoff_selection_source` is `inner_cv_validation` normally and `training_in_sample_fallback` when inner CV could not be formed (flagged).
- Spatial validation is leave-one-cluster-out: each projected k-means cluster is held out once (`spatial_outer_folds: null` in `configs/config.yaml`). The earlier round-robin merge of clusters into three folds is superseded.
- Fold metrics that are mathematically undefined for a single-class held-out fold (ROC-AUC, PR-AUC, balanced accuracy, calibration; recall without positives; specificity without negatives; F-scores without positives) are stored as NULL in `fold_metrics`, not as sentinel values. SQL `AVG` ignores NULLs, so fold averages cover only defined folds; check the fold count before interpreting them.
- Spatial folds containing a single outcome class are retained and flagged (`fold_composition`); pooled spatial metrics are computed across all unique held-out wells (`pooled_metrics`, scope `all_held_out`) and separately across binary-valid folds only.
- The principal decision-support presentation uses well-level consolidated out-of-fold predictions (median probability per well; each well contributes once) with unique-well TP/TN/FP/FN counts.
- Screening priority is based on the best completed canonical non-dummy run, using pre-specified probability bands (Low < 0.25, Moderate 0.25-0.50, High 0.50-0.75, Very_High >= 0.75).
- Spatial clusters are built by Euclidean k-means (k-means++ init, deterministic seed) on coordinates projected to a local equirectangular plane (km); great-circle distances are used only for reporting intra-cluster distances.
- Brier scores are always accompanied by a prevalence-only null Brier and Brier skill score. For each held-out fold, the null is the prevalence of that fold's outer-training data.

To verify the Rust code locally:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Publication graphics

The scripts under `manuscript/` and `scripts/` consume the generated SQLite database or CSV exports. Some manuscript-era R scripts retain an absolute `PROJECT_ROOT`; update that variable for your checkout before running them. `scripts/make_q1_figures.R` resolves paths relative to the repository root and is the most portable entry point for the risk-alert figures.

## Citation

If you use this software, please cite:

> Abdul-Wahab, D. & Sunkari, E. D. (2026). *Virtual screening of groundwater exceedance risk* (Version 0.1.1) [Computer software]. Zenodo. [https://doi.org/10.5281/zenodo.21442254](https://doi.org/10.5281/zenodo.21442254)

## License

This project is licensed under the [MIT License](LICENSE).
