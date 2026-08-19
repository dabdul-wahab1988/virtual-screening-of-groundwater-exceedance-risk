# DECISIONS.md — groundwater_virtual_screening

## 2026-05-31 — Stage A5: Package Implementation

**Role:** R2 (Architect)
**Decision:** Implemented complete Rust package from scratch without external ML crates.
**Justification:** Using pure Rust implementations (logistic regression via gradient descent,
CART decision trees, random forest with bootstrap aggregation, gradient boosted trees with
Newton-step leaves) avoids heavy compilation dependencies and gives full control over the
leakage-control guardrails. The csv crate was used instead of Polars since the dataset is small
(n=81) and the csv crate is sufficient for reliable column-typed reading.

**Decision:** Saabas path-based attributions used as TreeSHAP approximation.
**Justification:** Exact interventional TreeSHAP (Lundberg 2018) requires per-tree path enumeration
with exponential subset summation. For max_depth=3–5 and n=81 samples, the Saabas approximation
gives directionally correct attributions that are adequate for SHAP summary plots.
The paper should describe this as "path-based feature attribution" not exact Shapley values.

**Decision:** Spatial k-means uses Haversine distance but computes inertia in coordinate space.
**Justification:** For the Pru Basin (< 100 km extent), geodetic curvature error is negligible.
If wells were spread over > 1000 km, spherical k-means would be required.

**Decision:** Inner CV uses PR-AUC as tuning criterion.
**Justification:** PR-AUC is appropriate for imbalanced binary classification (most targets
have < 30% prevalence). ROC-AUC is less discriminating at extreme class ratios.

**Decision:** BDL substitution uses value / sqrt(2) for numeric BDL markers.
**Justification:** Standard censored-data substitution following Helsel (2012). The exact BDL
value is unknown; divide by sqrt(2) is the least-biased unbiased estimator for a uniform
distribution between 0 and the detection limit.

---

## 2026-08-19 — Revision round 2 (Track D)

### Attribution terminology and exact TreeSHAP (replaces the earlier SHAP decisions)
**Decision (final):** Exact interventional TreeSHAP (Lundberg et al., 2018) is implemented natively for the tree models, with the training-fold data as the background distribution. Per-sample contributions use Shapley-weighted sums over subsets of the unique features on the sample's path (features off the path contribute zero); expected values marginalise non-fixed features over the background covers (node training counts, weighted by the positive-class weight for classification trees, so the decomposition is exactly additive on the fitted (weighted) tree values). Random forests are decomposed on the probability scale (base = mean of tree root probabilities); gradient-boosted trees on the log-odds scale (per-tree values scaled by the learning rate; base = initial log-odds + learning rate × sum of tree root means). Linear models use the exact linear SHAP decomposition (bias + Σ wⱼzⱼ on standardised features; the training background has zero mean, so it is exact). Database schema uses SHAP naming (`shap_values`, `fold_base_values`, `mean_absolute_shap`, `shap_value`, `shap_computation_time_ms`); exports use `shap_values_long.csv`; Figure 6 is `Figure6_SHAP_importance.png`. The implementation is unit-tested against brute-force Shapley enumeration (including repeated split features and class-weighted covers) and exact additivity.
**Justification:** Reviewers 2/4 flagged that Saabas-style values must never be labelled SHAP (R2-07, R4-08; Gate 5). With max_depth ≤ 4 trees the exact algorithm is computationally trivial, so the scientifically stronger option (genuine SHAP) is taken instead of relabelling an approximation. The prior "exponential subset summation" concern applied to naive full Shapley enumeration over all features; the path-based TreeSHAP formulation with the off-path-weighting is polynomial and exact.

### Spatial clustering (revises the 2026-05-31 decision)
**Decision:** The previous k-means variant (Haversine assignment distances with coordinate-mean
centroids) is mathematically incoherent because the coordinate mean is not the least-squares
centre under great-circle distance. Replaced with: (1) project (lon, lat) to a local
equirectangular plane (kilometres) centred on the basin centroid; (2) ordinary Euclidean
k-means with deterministic k-means++ initialisation; (3) inertia-elbow rule unchanged in spirit
(marginal improvement < 10% of total range, capped at 8 clusters). Cluster metadata still report
great-circle intra-cluster distances.
**Justification:** R4-03; coherent least-squares clustering on a planar approximation whose
error is negligible at < 100 km extent.

### Fully nested operational-threshold selection
**Decision:** Operational cutoffs are now selected inside each outer fold from inner-CV
validation predictions only (sensitivity >= 0.90 rule, specificity maximised; fallback = best
available sensitivity), frozen, and applied to the outer held-out test fold. Per-fold cutoffs
and operating metrics are stored; a run-level pooled operating estimate (each fold's own cutoff
applied to its test predictions) is stored; the old pooled post-hoc cutoffs are retained as a
clearly labelled comparison analysis.
**Justification:** R2-15, R4-06, instructions Phase 3A. Pooled cutoff selection leaks fold-level
information into the operating estimate.

### Spatial validation transparency
**Decision:** Single-class spatial folds are never dropped and never abort a run. Every fold is
stored with n, positives, negatives, status flag, and metric definability
(`fold_composition`); pooled spatial metrics are computed across ALL unique held-out wells
(`pooled_metrics`, scope 'all_held_out') and separately across binary-valid folds only
(scope 'binary_valid_only') to quantify any exclusion bias.
**Justification:** R4-04, instructions Phase 3B.

### Well-level decision metrics
**Decision:** The principal decision-support presentation uses well-level consolidated
out-of-fold predictions: median OOF probability per well (pre-specified rule), each well
contributing exactly once, with unique-well TP/TN/FP/FN at a well-level cutoff selected with
the sensitivity >= 0.90 rule. Pooled prediction-instance counts (where a well can contribute
multiple times across repeats) are retained only as explicitly labelled prediction-instance
counts.
**Justification:** R4-09, instructions Phase 3K (Gate 8).

### Brier interpretation
**Decision:** A prevalence-only null Brier score (constant prediction equal to the training
prevalence) and Brier skill score = 1 - Brier_model / Brier_null are computed per fold and
pooled, so Brier scores are never interpreted in isolation.
**Justification:** R2-20, instructions Phase 3J.

### F-beta sensitivity
**Decision:** Threshold sweeps now compute F1, F1.5, F2 and F3; cutoff rules max_F1, max_F1_5,
max_F2, max_F3 are all reported. F2 (beta = 2) is described as an a priori surveillance
preference giving greater weight to recall, not an optimised cost ratio.
**Justification:** R2-14, instructions Phase 3I.

### Coordinate ablation
**Decision:** For every tier, a "no_coords" sensitivity variant (coordinates excluded) is run
for every target and CV mode to test whether explicit geographic information drives
discrimination. Reported as sensitivity analysis, not as a new operational tier.
**Justification:** R2-03, instructions Phase 3D.

### EC ablation for Na and Cl
**Decision:** Tier-2 EC-strict sensitivity variants are run for Na and Cl (EC excluded),
mirroring the TDS EC-strict design, to distinguish field-proxy screening from richer
hydrochemical signal.
**Justification:** R4-05, instructions Phase 3M.

### Small-sample diagnostics
**Decision:** Fold-level train sizes and positive counts, sample-to-predictor ratios,
feature counts and fold-level PR-AUC spread are exported
(`small_sample_diagnostics.csv`, `fold_composition.csv`). The random-forest tuning grid was
restricted to max_depth <= 4 (removed depth 5) for the small-sample setting.
**Justification:** R4-02, instructions Phase 3F (Gate 7).

### Screening-priority classes
**Decision:** The pre-specified probability bands are retained and documented precisely:
Low (< 0.25), Moderate (0.25–0.50), High (0.50–0.75), Very_High (>= 0.75), applied to the
well-level median OOF probability of the best canonical run. Classes express screening
priority for confirmatory testing, never definitive contamination status.
**Justification:** R2-17, instructions Phase 3L (Gate 10).

### Run identity
**Decision:** run_id now includes the sensitivity-variant tag and the labelling variant
(canonical = "main"); alternative-threshold and BDL-sensitivity analyses run in separate
databases produced from alternative thresholds YAML files so canonical results are never
overwritten.

### BDL sensitivity
**Decision:** Below-detection-limit substitution remains limit/sqrt(2) with the minimum
detected value as the documented fallback limit (no laboratory LODs were supplied). The
substitution method is now configurable ("limit_div_sqrt2" | "limit_div_2" | "limit_direct"),
and a sensitivity analysis over reasonable F substitution values is run. A full censored
maximum-likelihood model is not justified: only five F records are affected, all substituted
values are far below the F screening threshold (1.125 mg/L), and target labels are invariant
to the substitution (see REMAINING_LIMITATIONS).
**Justification:** R2-11, R2-18, instructions Phase 3H.

### EC-only logistic base value
**Decision:** For linear models the attribution base value is the bias term, giving an exact
additive decomposition of the logit. For tree ensembles the base value is the training
prevalence logit (random forest) or initial log-odds (gradient boosting); the Saabas
decomposition is approximate and documented as such.
