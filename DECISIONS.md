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
