**Leakage-controlled and spatially validated machine learning for groundwater-quality exceedance-risk screening in the semi-arid Pru Basin, Ghana**

Emmanuel Daanoba Sunkari^1,2,3*, Dickson Abdul-Wahab^4, Jonas Togyeni Mba^5, Kamal Hiran Kant^6 & Hamidatu Saaka Darimani^7

^1^Mining Engineering, Department of Integrated and Advanced Technology, School of Engineering and Sciences, Sir Padampat Singhania University, Udaipur-313601, Rajasthan, India

^2^Centre of Excellence in Environmental Science and Sustainability, Sir Padampat Singhania University, Udaipur-313601, Rajasthan, India

^3^Department of Chemical Sciences, Faculty of Science, University of Johannesburg, P.O. Box 524, Auckland Park 2006, Johannesburg, South Africa

^4^Graduate School of Nuclear and Allied Sciences, University of Ghana, Legon, Accra, Ghana

^5^Nuclear Regulatory Authority, P.O. Box AE 50, Houses 1 & 2, Neutron Avenue, Kwabenya, Accra, Ghana

^6^Computer Science and Engineering, Department of Computing and Informatics, School of Engineering and Sciences, Sir Padampat Singhania University, Udaipur-313601, Rajasthan, India

^7^Department of Agricultural Engineering, Dr. Hilla Limann Technical University, P.O. Box 553, Wa, Upper West Region, Ghana

\*Corresponding author: emmanueldaanobasunkari@gmail.com (ORCID: 0000-0002-0898-2286)

# Abstract

Machine-learning applications in groundwater-quality assessment often report high predictive performance without demonstrating control of target leakage, spatial dependence, class imbalance and small-sample uncertainty. This study developed a leakage-controlled virtual-screening framework to prioritise wells at elevated probability of exceeding precautionary drinking-water screening thresholds in the semi-arid Pru Basin, Ghana, using 81 georeferenced groundwater samples. Six groundwater-quality variables (Na, Cl, TDS, B, F and NO₃) were converted into binary risk-alert labels at study-defined precautionary thresholds; NO₃ was excluded from modelling because only two wells exceeded its screening threshold. Target-derived predictors were removed before modelling, preprocessing and hyperparameter tuning were contained within training folds, and operational probability cutoffs were selected inside each outer cross-validation fold from internal-validation predictions only, so that no held-out information entered any modelling decision. Na, Cl and TDS were the most screenable targets: pooled out-of-fold PR-AUC reached 0.978 for Cl and 0.992 for TDS with field-only predictors, and spatial-group cross-validation retained PR-AUC values of 0.977 (Cl) and 0.964 (TDS, full tier) for the salinity-linked targets. For TDS, an EC-strict model that excluded the conductivity proxy retained PR-AUC = 0.976 (well-level bootstrap 95% CI 1.000–1.000) under stratified validation and 0.923 under spatial validation. B showed moderate, chemistry-dependent performance (0.813 best), whereas F was more spatially fragile (0.636 stratified vs 0.490 spatial). Removing coordinates did not erase discrimination for any modelled target, and relabelling at current WHO guideline values left the ranking of screenability unchanged wherever sufficient positives remained. Well-level decision metrics (median out-of-fold probability per well, each well counted once) gave interpretable false-positive/false-negative burdens for confirmatory-testing prioritisation, and Brier scores were benchmarked against prevalence-only baselines (Brier skill 0.770 for the EC-strict TDS model). We recommend the EC-strict TDS model and the full-chemistry random-forest model for B as the primary screening tools, applied with the target-specific cutoffs described in the Methods. The framework supports conservative well prioritisation for confirmatory testing in data-limited groundwater surveillance; it does not replace laboratory confirmation or regulatory monitoring.

**Keywords:** confirmatory-testing prioritisation; groundwater screening; precision–recall analysis; SHAP values; spatial validation; well prioritisation

# 1. Introduction

Groundwater is a critical source of drinking water, domestic supply and irrigation water in semi-arid and rural regions where surface-water resources are seasonally constrained or unreliable. Its quality is controlled by coupled geogenic and anthropogenic processes, including lithology, mineral weathering, ion exchange, salinisation, agricultural return flow and localised surface-derived inputs (Sunkari et al., 2021). In the semi-arid Pru Basin, groundwater occurs within a hydrogeologically constrained Voltaian setting where shallow fractured aquifers, ferricrete cover, argillaceous and arenaceous rocks, low primary porosity and variable secondary permeability influence groundwater occurrence and chemistry (Sunkari et al., 2021). Regular groundwater-quality assessment is essential for public-health protection, particularly where laboratory monitoring resources are limited; the WHO drinking-water guideline framework provides the authoritative basis for risk management and national standard setting, but operational surveillance also requires efficient tools that prioritise wells for confirmatory testing rather than replace laboratory analysis (World Health Organization, 2022). Recent assessments in other data-limited regions reinforce this need: drinking-water quality and the associated human health risks of potentially toxic elements remain unevenly characterised across many semi-arid and rural settings (Abdipour et al., 2026), and land-use decisions can measurably alter groundwater quality at regional scales (Zhan et al., 2025).

Machine-learning and data-driven screening approaches are increasingly used to support environmental monitoring, water-quality classification and contaminant-risk prioritisation. Algorithms such as logistic regression, random forests and gradient-boosted trees provide complementary modelling capacities: logistic regression supports interpretable linear screening, random forests provide ensemble-based nonlinear classification, and gradient boosting captures additive nonlinear structure through sequential learning (Breiman, 2001; Friedman, 2001). However, groundwater datasets are often small, spatially clustered and class imbalanced. These conditions make performance estimates vulnerable to methodological bias when target-derived predictors, spatially neighbouring wells or model-selection information enter validation folds (Roberts et al., 2017; Varma & Simon, 2006). Precision–recall analysis is particularly relevant where positive exceedance cases are sparse, because PR-AUC focuses directly on positive-class retrieval rather than being dominated by the majority class (Davis & Goadrich, 2006). Recent methodological work in adjacent fields illustrates the same priorities that guide this study: uncertainty quantification in groundwater-table deep learning (Abbaszadeh Shahri et al., 2022), physics-informed deep learning for subsurface exchange imaging (Yin et al., 2025), and data-driven modelling of fractured-reservoir systems (Abbasi & Madani, 2026) all emphasise calibrated, uncertainty-aware prediction under data limitation rather than raw discrimination alone.

The central problem addressed here is that high reported predictive performance in groundwater machine learning may not be operationally reliable unless leakage, spatial dependence, fold-level uncertainty and threshold selection are explicitly controlled. In the Pru Basin dataset, the modelling challenge is sharpened by a small sample size (81 wells), unequal target prevalence and chemically interdependent predictors. Sodium, chloride and TDS are expected to be more screenable because they are linked to coherent salinity and mineralisation gradients, whereas fluoride, boron and nitrate may be influenced by more localised geochemical, lithological or anthropogenic controls (Sunkari et al., 2021). The relevant question is therefore not whether a single model can classify all groundwater-quality targets uniformly, but which targets can be screened with sufficient reliability under conservative validation, and with what false-positive/false-negative burden at an explicitly defined operating point.

The novelty of this framework lies in an integrated validation architecture rather than in any single algorithm. First, target-specific leakage control removes direct target variables, target-derived ratios and circular proxies before model training. Second, tiered field-to-chemistry modelling evaluates whether low-cost field variables retain useful screening performance relative to reduced and full hydrochemical predictor sets. Third, repeated stratified nested cross-validation is paired with spatial-group validation in which all spatial folds — including single-class folds — are reported transparently and pooled spatial performance is computed across all unique held-out wells. Fourth, operational probability cutoffs are selected inside each outer fold from internal-validation predictions (fully nested threshold selection), eliminating the optimistic bias of post-hoc pooled cutoff selection. Fifth, well-level consolidated out-of-fold probabilities provide unique-well decision metrics for confirmatory-testing prioritisation, and Brier scores are benchmarked against prevalence-only baselines. Finally, exact interventional TreeSHAP attribution (Lundberg et al., 2018) and GIS-ready screening-priority maps translate validated probabilities into decision-support outputs without presenting them as definitive contamination distributions. To our knowledge, no previous groundwater-screening study combines all of these safeguards in a single small-sample framework; earlier work in this basin used deterministic hydrochemical classification without predictive validation (Sunkari et al., 2021), and recent ensemble studies of Ghanaian groundwater quality identified the absence of spatial cross-validation as an explicit limitation (Ansah-Narh et al., 2026).

The aim of this study was to develop and evaluate a leakage-controlled, small-sample virtual-screening framework for identifying wells at elevated risk of exceeding precautionary drinking-water thresholds. The objectives were to (i) assess target-specific screenability under field, reduced-chemistry and full-chemistry predictor tiers; (ii) compare repeated stratified nested cross-validation with spatial-group validation, including a transparent treatment of single-class spatial folds; (iii) quantify the contribution of explicit geographic coordinates through a coordinate-ablation design; (iv) evaluate operational decision thresholds through fully nested cutoff selection and well-level decision metrics, including sensitivity to the recall-weighting parameter of the F-score; and (v) test sensitivity to concentration-threshold definition, below-detection-limit handling and the EC conductivity proxy. The study is significant because it provides a reproducible approach for groundwater surveillance in data-limited semi-arid environments while explicitly avoiding claims that machine-learning screening replaces laboratory confirmation.


# 2. Materials and methods

## 2.1 Study area, geology, hydrogeology and groundwater dataset

The study used the semi-arid Pru Basin groundwater dataset originally generated for hydrochemical assessment of groundwater salinisation in Ghana (Sunkari et al., 2021). The basin lies within the Voltaian hydrogeological province and is dominated by the Oti-Pendjari Group of the Voltaian Supergroup, with reddish-brown interbedded mudstone and sandstone of the Pru Formation, ferricrete cover, shallow fractured aquifers and groundwater occurrence controlled mainly by secondary porosity. Boreholes in the Pru Formation commonly range from 45 to 75 m depth, with aquifers generally encountered between 20 and 30 m. The modelling dataset comprised 81 georeferenced groundwater samples (70 hand-dug wells and 11 boreholes) collected in a single dry-season campaign; the dataset is therefore cross-sectional, and the framework makes no temporal-transferability claims. Field parameters included pH, temperature, electrical conductivity (EC) and total dissolved solids (TDS); laboratory variables included major ions and selected health-relevant constituents. Sampling, analytical methods and quality control are summarised in Supplementary Methods S1, and variable roles, analytical completeness and summary statistics are reported in Table 1 and Supplementary Table S1.

## 2.2 Target definition and modelling eligibility

Six groundwater-quality variables were evaluated as binary screening targets: Na, Cl, TDS, B, F and NO₃. Labels were generated with a greater-than rule against study-defined precautionary screening thresholds:

  ---------------------------------------------
  $$y_i,t = \left\{ \beginmatrix    \(1)\
  1, & x_i,t > \tau_t \\            
  \ 0, & x_i,t \leq \tau_t          
  \endmatrix \right.\ $$              
  ------------------------------------- -------

  ---------------------------------------------

where $y_i,t$ is the binary label for sample *i* and target *t*, $x_i,t$ is the measured concentration and $\tau_t$ is the target-specific threshold. The applied thresholds were 150 mg L⁻¹ for Na, 187.5 mg L⁻¹ for Cl, 750 mg L⁻¹ for TDS, 1.125 mg L⁻¹ for B, 1.125 mg L⁻¹ for F and 37.5 mg L⁻¹ for NO₃ (Table 1; Supplementary Table S2). These values are study-specific early-warning screening cut-offs, not updated regulatory compliance limits: Na, Cl, TDS, F and NO₃ thresholds correspond to 75% of the corresponding WHO/Ghana guideline values, a precautionary margin intended to flag wells before guideline exceedance develops, and the TDS threshold is an acceptability-based dissolved-solids alert (World Health Organization, 2022). The B threshold of 1.125 mg L⁻¹ was inherited from the original modelling configuration and does not correspond to 75% of any current WHO boron guideline (the current WHO guideline value is 2.4 mg L⁻¹); it is therefore interpreted strictly as a study-defined screening threshold. To test the sensitivity of the analysis to threshold definition, all targets were additionally relabelled at current WHO guideline values (Na 200, Cl 250, TDS 1000, B 2.4, F 1.5 and NO₃ 50 mg L⁻¹) and re-evaluated wherever class prevalence remained sufficient (Section 3.8; Supplementary Table S11).

Modelling eligibility required at least five positive and five negative cases, the pre-registered minimum for fold-based machine-learning evaluation. Na (47 positives), Cl (15), TDS (13), F (13) and B (11) were modelled. NO₃ was retained for descriptive reporting only because only two wells exceeded its screening threshold; its descriptive statistics and spatial distribution are reported in Section 3.1 and Supplementary Figure S5.

## 2.3 Predictor-tier design

Three predictor tiers represented increasing analytical effort (Table 2). Tier 1 (field) contained pH, temperature, EC and the coordinates (longitude Dx, latitude Dy). Tier 2 (reduced chemistry) added the major ions Na, K, Mg, Ca, Cl, HCO₃ and SO₄. Tier 3 (full chemistry) added CO₃, NO₃, F, B and the derived ionic ratios Na/Cl, Mg/Ca, HCO₃/Cl, Ca/Mg and SAR, all after target-wise leakage filtering (Section 2.4). For TDS, two Tier-2 sensitivity variants were retained: an EC-inclusive design and an EC-strict design in which EC was excluded as a conductivity proxy for TDS. For Na and Cl, equivalent EC-strict Tier-2 variants were added to distinguish field-proxy screening from richer hydrochemical signal (Section 3.8). For every tier, a coordinate-excluded counterpart (all predictors except Dx and Dy) was run as a coordinate-ablation sensitivity analysis (Section 3.8).

## 2.4 Target-wise leakage control

Target-wise leakage control was a central design requirement. Each target was modelled separately, and predictors were removed where they represented the target itself, a direct target-derived ratio or a circular proxy likely to inflate performance artificially. The principal leakage-control matrix is given in Table 2, with the complete audit in Supplementary Table S7. B was excluded from Tier-3 B models; Cl was excluded from Tier-2 and Tier-3 Cl models, together with Na/Cl and HCO₃/Cl ratios at Tier 3; F and NO₃ were excluded from their respective Tier-3 models; Na was excluded from Tier-2 and Tier-3 Na models, together with Na/Cl and SAR at Tier 3; and EC was excluded in the strict sensitivity designs for TDS, Na and Cl. Leakage filtering was applied before model fitting for every target–tier combination, so models could not rediscover the response variable through chemically circular information.

## 2.5 Data preprocessing and below-detection-limit handling

Below-detection-limit (BDL) entries and missing numeric values were handled at ingestion. Plain BDL markers were substituted with limit/√2, the standard least-biased substitution for left-censored values when the detection limit is known (Hornung & Reed, 1990; Helsel, 2012). Laboratory detection limits were not provided for CO₃, F and NO₃, so the documented fallback used each analyte's minimum detected value as the substitution limit (giving 0.163 mg L⁻¹ for the five F BDL wells, all far below the 1.125 mg L⁻¹ screening threshold). Because the F BDL value is the only BDL that affects a modelled target's predictor set, we tested three alternative F substitutions (min-detected/√2 baseline, an assumed detection limit of 0.5 mg L⁻¹, an assumed detection limit of 0.2 mg L⁻¹, and direct substitution of the fallback limit) and verified that target labels are invariant to the substitution and that headline validation results are insensitive to it (Section 3.8; Supplementary Table S12). A full censored maximum-likelihood model was not justified: only five F records are affected, all substituted values lie far below the screening threshold, and the substitution cannot change any target label (Supplementary Methods S2).

Continuous predictors were median-imputed and standardised inside each training fold only; the fitted imputation and scaling parameters were then applied to the corresponding held-out fold, so no test-fold information entered preprocessing. No whole-dataset oversampling, SMOTE-style resampling or feature selection outside cross-validation was used. Class imbalance was addressed by positive-class weighting within the training data, with the positive-class weight defined as the ratio of negative to positive training cases, floored at 1.0.

## 2.6 Spatial grouping

Spatial dependence was evaluated through coordinate-based spatial clusters. Longitude/latitude pairs were projected onto a local equirectangular plane (kilometres) centred on the basin centroid, and k-means clustering with deterministic k-means++ initialisation was applied in projected Euclidean space; coordinate-mean centroids are the true least-squares cluster centres on the projected plane, which is not the case for great-circle distance. The number of clusters was selected by an inertia-elbow rule (stop at the first k for which the marginal inertia improvement falls below 10% of the total improvement range, k from 2 to 8), yielding 4 clusters with 41, 5, 18 and 17 wells, respectively (maximum intra-cluster distances 35.1, 20.7, 23.4 and 32.1 km; Supplementary Table S6). Sensitivity to the number of clusters was evaluated for k = 3, 5, 6, 7 and 8 (Section 3.5; Supplementary Table S9). Spatial clusters are used strictly as validation partitions: they should not be read as hydrogeological flow-path units, and spatially blocked evaluation is a conservative transferability test that may overstate the penalty if hydrologically connected wells are split across blocks (Roberts et al., 2017).

Spatial-group cross-validation held out each spatial block in turn. Crucially, no spatial fold was discarded: every fold is reported with its sample size, positive and negative counts and a status flag (Supplementary Table S15), and fold-level metrics are reported only where they are mathematically defined (e.g., PR-AUC requires at least one positive held-out case). Pooled spatial performance is computed across all unique held-out wells (each well appears in the held-out set exactly once), and a binary-valid-only pooled estimate is reported alongside to quantify the effect of excluding single-class folds (Section 3.5).

## 2.7 Model development

The model set comprised baseline, interpretable and nonlinear classifiers. Baselines included majority-class and stratified dummies and EC-only logistic regression (where EC was available and non-circular). The interpretable model was L2-regularised logistic regression; nonlinear models were random forest and gradient-boosted trees (Breiman, 2001; Friedman, 2001). All implementations are native, deterministic under the fixed seed, and archived with the analysis (Section 2.10). Hyperparameters were tuned inside the inner cross-validation loop only: inverse-regularisation C for logistic regression (grid 0.001–50), n_estimators and max_depth for random forest (grid restricted to max_depth ≤ 4 because of the small training sets), and n_estimators and learning rate for gradient boosting (max_depth fixed at 3). The tuning criterion was inner-CV PR-AUC. Fold-level hyperparameter choices are archived in Supplementary Table S3. No support-vector machine or deep-learning models were used: with roughly 65 training samples per outer fold and up to 21 predictors, the added complexity is not justified by the data, and the validated model set is sufficient to answer the screening question (Supplementary Methods S6).

## 2.8 Validation protocol

Two complementary validation designs were used. Repeated stratified nested cross-validation (5 repeats × 5 outer folds = 25 held-out folds per classifier) provided the main performance estimate while preserving class balance and separating hyperparameter tuning (inner 3-fold CV) from outer-fold testing (Varma & Simon, 2006). Spatial-group cross-validation (3 held-out spatial blocks) provided a conservative geographic-transferability test. For both designs, preprocessing, class weighting, hyperparameter tuning, probability prediction and metric calculation were conducted within the fold structure, and no held-out fold contributed to imputation, scaling, tuning or model fitting.

Operational probability cutoffs were selected with a fully nested procedure: inside each outer fold, the cutoff was chosen on pooled internal-validation predictions (inner-CV validation folds for the tuned model) using a sensitivity-first rule (maximise specificity subject to sensitivity ≥ 0.90, falling back to the best available sensitivity), frozen, and applied once to that fold's held-out test predictions. Per-fold cutoffs and operating metrics are archived (Supplementary Table S16). This design is leakage-safe by construction; the earlier post-hoc approach (cutoffs selected on pooled out-of-fold predictions) is retained only as a clearly labelled comparison (Section 3.7; Supplementary Figure S7), because the fully nested estimate is expected to be more conservative.

## 2.9 Evaluation metrics

The primary metrics were PR-AUC, recall/sensitivity and the F₂ score. PR-AUC was prioritised because it is informative for imbalanced binary classification (Davis & Goadrich, 2006). Recall and F₂ were emphasised because missed exceedance wells were treated as more serious than false-positive flags for confirmatory testing:

  ------------------------------------------------------------------------------------------------------------------------------------
  $$F_{\beta} = \frac{\left( 1 + \beta^{2} \right) \times Precision \times Recall}{\left( \beta^{2} \times Precision \right) + Recall}$$   \(2)\
  ------------------------------------------------------------------------------------------------------------------------------------

The choice of β = 2 is an explicit a priori surveillance preference that weights recall more heavily than precision; it is not a mathematically optimised cost ratio. Because this choice is consequential, cutoffs were also evaluated under F1, F1.5 and F3 (Section 3.7; Supplementary Table S13 and Figure S6).

Secondary metrics included ROC-AUC, specificity, balanced accuracy, F1 score, Brier score, calibration slope and calibration intercept. Brier scores were never interpreted in isolation: each Brier score was benchmarked against a prevalence-only null model (a constant prediction equal to the training prevalence), and a Brier skill score (1 − Brier/Brier_null) was computed, because the interpretability of a raw Brier score depends on event prevalence and no universal "good" cutoff exists. Calibration slope and intercept were estimated by logistic regression of the binary outcome on the logit of the predicted probability (Supplementary Methods S8).

Headline discrimination metrics are reported as pooled out-of-fold estimates (computed on all unique held-out predictions concatenated across folds), with fold-level mean ± SD reported in Supplementary Table S4 and well-level bootstrap 95% confidence intervals (2,000 resamples of wells using each well's median out-of-fold probability, so repeated predictions never inflate precision) reported in Supplementary Table S8.

## 2.10 Operational thresholds, explainability, screening priority and reproducibility

Decision support is provided at two levels. First, target-specific operating points: the fully nested sensitivity-first cutoffs described in Section 2.8, aggregated across folds. Second, well-level decision metrics: for each well, the median of its repeated out-of-fold probabilities (each well contributing exactly once), followed by unique-well true-positive (TP), true-negative (TN), false-positive (FP) and false-negative (FN) counts at a well-level cutoff selected with the same sensitivity-first rule. The resulting well-level operating characteristics are reported in Section 3.7. Pooled prediction-instance counts, in which a well can contribute several times across repeats, are retained only as explicitly labelled prediction-instance counts and are not used for principal decision support. Screening-priority classes were defined on the well-level median out-of-fold probability of the best canonical (non-dummy, non-sensitivity) model for each target, using pre-specified probability bands: Low (< 0.25), Moderate (0.25–0.50), High (0.50–0.75) and Very high (≥ 0.75). These bands are screening-priority statements for confirmatory testing, not definitive contamination classifications.

Feature attributions were computed as exact interventional TreeSHAP values for the tree ensembles (Lundberg et al., 2018) — exact Shapley values computed with the training-fold data as the background distribution, giving an exactly additive decomposition (base value plus the sum of SHAP values equals the model output) — and, for the linear models, as exact SHAP values (Lundberg & Lee, 2017) given by the additive logit decomposition bias plus Σ wⱼzⱼ on the standardised features, which is exact because the training background has zero mean on standardised features. These are model-behaviour diagnostics, not causal mechanisms (Supplementary Methods S10).

Reproducibility was enforced through input-file SHA-256 hashing, deterministic seeds, stored fold assignments, archived hyperparameters, fold-level metrics, well-level predictions, attribution values, nested and pooled operational thresholds, threshold sweeps and GIS-ready exports, all recorded in an auditable SQLite database (schema version 2.0; Supplementary Methods S12). The complete analytical workflow is summarised in Figure 1; extended mathematical and implementation details are given in Supplementary Methods. An end-user decision flowchart is provided in Section 4.5.


# 3. Results

## 3.1 Dataset summary and exceedance prevalence

The dataset comprised 81 georeferenced groundwater samples with complete coordinate records and complete measurements for most field and major-ion variables (Table 1). Valid measurements were available for pH, temperature, EC, TDS, Na, K, Mg, Ca, Cl, SO₄, HCO₃ and B across all 81 wells. CO₃ had 40 valid observations and 41 BDL records; NO₃ had 39 valid observations and 42 BDL records; and F had 76 valid observations and 5 BDL records (Table 1). No missing coordinate values were reported.

Na had the highest precautionary screening-positive prevalence, with 47 of 81 wells (58.0%) above 150 mg L⁻¹. Cl exceeded 187.5 mg L⁻¹ in 15 wells (18.5%), TDS exceeded 750 mg L⁻¹ in 13 wells (16.0%), F exceeded 1.125 mg L⁻¹ in 13 wells (16.0%) and B exceeded 1.125 mg L⁻¹ in 11 wells (13.6%). NO₃ exceeded 37.5 mg L⁻¹ in only 2 of 39 detected records (2.5% of all wells); its measured concentrations ranged from 0.44 to 70.30 mg L⁻¹ (median 0.44), with BDL wells distributed across the basin rather than in a single cluster (Supplementary Figure S5). Figure 2 summarises the target-prevalence structure graphically.

## 3.2 Target eligibility and modelling feasibility

Na, Cl, TDS, B and F each had at least five positive and five negative cases and were modelled under both validation designs. NO₃ was retained for descriptive reporting only: with two exceedances, fold-based machine-learning evaluation is not statistically supported, and any classifier fitted on this target would produce unstable, potentially misleading performance estimates. This eligibility rule (pre-registered at five minimum positives) prevents over-interpreting models with negligible positive support.

## 3.3 Leakage audit and retained feature availability

Target-wise leakage filtering materially reduced the feature space before modelling (Table 2). B Tier 3 excluded B and retained 20 predictors; Cl Tier 2 excluded Cl (11 retained) and Cl Tier 3 excluded Cl, HCO₃/Cl and Na/Cl (18 retained); F Tier 3 excluded F (20 retained); Na Tier 2 excluded Na (11 retained) and Na Tier 3 excluded Na, Na/Cl and SAR (18 retained); and TDS Tier 3 retained all 21 candidate predictors. The EC-strict sensitivity designs excluded EC from the Tier-2 predictor set for TDS, Na and Cl. No target variable or target-derived ratio entered any corresponding prediction task.

## 3.4 Model performance under repeated stratified nested cross-validation

Table 3 reports, for each target and tier, the best canonical non-dummy classifier by pooled out-of-fold PR-AUC, with pooled discrimination and calibration metrics under stratified nested CV and the corresponding pooled spatial PR-AUC. Salinity-related targets achieved the strongest and most stable screening performance. Cl reached a pooled PR-AUC of 0.978 with EC-only logistic regression (ROC-AUC 0.994, recall 0.720 at the 0.50 default point, F₂ 0.763, Brier 0.0844); TDS reached 0.992 under the same field-only design (0.0355). Because EC is a field-scale proxy for total dissolved ionic content, these EC-only results were probed with the EC-strict designs: with EC excluded, the best TDS model (gradient-boosted trees, Tier 2) retained a pooled PR-AUC of 0.976 (ROC-AUC 0.984; well-level bootstrap 95% CI 1.000–1.000), confirming that TDS screenability is not solely an artefact of the EC–TDS relationship. Na was also highly screenable (0.958 for the best Tier-1 model, 0.958 for the best Tier-2 model), and Na probabilities were the most consistently calibrated of all targets (Brier skill 0.276).

B showed intermediate performance that improved with predictor richness: pooled PR-AUC increased from 0.750 (Tier 1) to 0.787 (Tier 2) and 0.813 (Tier 3), with the Tier-3 random-forest model reaching recall 0.636 and F₂ 0.665. F was the weakest modelled target: pooled PR-AUC increased from 0.426 (Tier 1) to 0.636 (Tier 3), and the best Tier-3 model reached recall 0.631. Figure 3 summarises these patterns; Supplementary Table S4 reports fold-level mean ± SD and Supplementary Table S8 reports well-level bootstrap 95% confidence intervals. Fold-level spread is large for B and F (e.g., Tier-3 F PR-AUC SD 0.253), which qualifies all point estimates for these targets.

## 3.5 Spatial validation: pooled transferability, fold transparency and k sensitivity

Spatial-group validation was evaluated in two complementary ways: pooled spatial PR-AUC across all unique held-out wells (the headline spatial estimate, Table 3), and per-fold transparency (Supplementary Table S15), because spatial folds with a single outcome class leave fold-level metrics undefined. Under the elbow-selected 4-cluster partition, single-class held-out blocks occurred and were reported rather than discarded. Single-class held-out blocks arise at finer partitions (e.g., TDS at k = 7), where they are reported rather than discarded (Supplementary Table S15).

Salinity-linked targets retained strong geographic transferability: the best Cl and TDS models retained pooled spatial PR-AUC values of 0.977 and 1.000 (field tier), and Na retained 0.925 (Tier 1) and 0.925 (Tier 2). The EC-strict TDS model retained a pooled spatial PR-AUC of 0.923. B showed mixed spatial behaviour (0.927 at Tier 3 versus 0.813 stratified), and F showed the clearest spatial penalty at richer tiers (0.490 spatial versus 0.636 stratified; Figure 4).

Sensitivity to the number of spatial clusters (k = 3, 4, 5, 6, 7, 8) showed that the stability of spatial PR-AUC is target- and algorithm-dependent (Supplementary Table S9). Na and Cl retained narrow spatial PR-AUC ranges across all tested partitions (0.90–0.93 and 0.98–1.00 for their headline models; ≤ 0.16 across all models). TDS was highly stable for the EC-only models (0.96–1.00) but its tree-based headline model ranged more widely (0.90–1.00). B and F showed the widest ranges overall — up to 0.35 (B Tier-2 logistic regression) and 0.51 (F Tier-1 random forest) — with the Tier-3 headline models ranging 0.76–0.86 (B) and 0.47–0.58 (F). Fluoride and, to a lesser extent, boron screening are therefore the most sensitive to the spatial partition, and the spatial-validation results for these targets must be read as partition-dependent (Supplementary Table S9). Spatial clusters are validation partitions, not hydrogeological flow-path units; blocked evaluation may overstate the transferability penalty if hydrologically connected wells are split across blocks (Section 2.6).

## 3.6 Reduced-variable screening performance

Table 4 reports the effect of predictor-tier expansion. For Na and Cl, Tier 1 and Tier 2 retained more than 90% of the Tier-3 pooled PR-AUC for most algorithms; the Na Tier-2 model performed comparably to its Tier-3 counterpart (0.958 versus 0.958), a small-sample difference that we do not interpret as evidence that additional predictors introduce noise. TDS retained very high performance under Tier 1 and Tier 2 for tree-based algorithms. B and F benefited more from hydrochemical enrichment: B random forest increased from 0.750 (Tier 1) to 0.813 (Tier 3), and F gradient-boosted trees from 0.426 to 0.636. Field-only screening is therefore most defensible for salinity-linked parameters, whereas F and B require broader hydrochemical context.

## 3.7 Operational thresholds, decision metrics, β sensitivity and calibration

Fully nested threshold selection produced target-specific cutoffs whose operating characteristics are reported per fold (Supplementary Table S16) and aggregated in Table 5 together with the well-level decision metrics. At the well level (median out-of-fold probability per well, each well counted once), the sensitivity-first rule selected cutoffs of 0.14 (B, Tier 3), 0.43 (Cl, Tier 1), 0.17 (F, Tier 3), 0.50 (Na, Tier 1) and 0.74 (TDS, Tier 1), yielding unique-well FP/FN burdens of 12/1, 0/1, 18/1, 9/3 and 0/1 respectively (Table 5). These well-level counts cannot exceed the 81-well dataset and are the principal decision-support presentation. The fully nested pooled operating estimates were more conservative than the previous post-hoc pooled cutoffs: for example, nested pooled sensitivity/specificity of 0.891/0.426 for B versus 0.909/0.760 post-hoc (Supplementary Figure S7). Pooled prediction-instance counts (where a well can contribute several times across the five repeats) are retained only as clearly labelled supplementary values and should not be read as unique-well counts.

Because β = 2 is an a priori recall-weighting choice rather than an optimised cost ratio, cutoffs were re-selected under F1, F1.5 and F3 (Supplementary Table S13). The substantive ranking of targets and the qualitative screening conclusions were unchanged across β: targets with strong discrimination (Na, Cl, TDS) retained high-specificity operating points under all β values, while B and F required lower cutoffs under recall-weighted β values (Supplementary Figure S6).

Brier scores were benchmarked against prevalence-only null forecasts. Brier skill scores (1 − Brier/Brier_null) were positive for all headline models (e.g., 0.276 for Na Tier 2, 0.770 for the EC-strict TDS model, and 0.525 for B Tier 3; Table 3), indicating that the models outperform constant prevalence predictions; however, Brier scores remain prevalence-dependent, and bin-level calibration uncertainty for B and F (Figure 5) stems primarily from the small numbers of positive wells in individual probability bins rather than from any single modelling failure. Raw Brier values should therefore be read together with the null benchmark and calibration curves, never in isolation.

## 3.8 Sensitivity analyses

**Coordinate ablation.** Removing longitude and latitude from every tier changed pooled PR-AUC only modestly for most targets and in some cases slightly improved it: for example, the Na Tier-2 model achieved 0.960 without coordinates versus 0.958 with coordinates, and the Cl Tier-3 model 0.978 versus 0.978 (Supplementary Table S10, Figure S4). For B and F, discrimination was generally retained without coordinates (0.818 and 0.620 at Tier 3). Explicit geographic location therefore does not drive discrimination on its own; spatial structure contributes indirectly through the hydrochemical variables that co-vary with position. Under spatial-group validation the coordinate-free models behaved similarly to their coordinate-inclusive counterparts (Supplementary Table S10).

**EC ablation for Na and Cl.** Excluding EC from the Tier-2 predictor set for Na and Cl reduced but did not eliminate discrimination: pooled PR-AUC fell from 0.958 to 0.912 for Na and from 0.978 to 0.847 for Cl (Supplementary Table S10). The remaining performance is carried by major ions (notably Na and Cl for each other's models and HCO₃/SO₄ for TDS; Figure 6), which we interpret as legitimate hydrochemical signal rather than circular reconstruction: EC is a legitimate operational proxy, and removing it tests whether richer chemistry can substitute for field conductivity readings.

**Concentration-threshold sensitivity.** Relabelling at current WHO guideline values (Na 200, Cl 250, TDS 1000, B 2.4, F 1.5, NO₃ 50 mg L⁻¹) reduced positive counts as expected — to 27 (Na), 12 (Cl), 8 (TDS), 3 (F) and 3 (B) wells — and left Na, Cl and TDS with sufficient positive support for modelling (Supplementary Table S11). Discrimination was retained or improved at the guideline thresholds (pooled PR-AUC 0.962 for Na, 0.986 for Cl and 1.000 for TDS at their best tiers), and the ranking of screenability (Na ≈ Cl ≈ TDS > B > F) was unchanged. F (three positives at 1.5 mg L⁻¹) and B (three positives at 2.4 mg L⁻¹) are reported descriptively at the guideline thresholds because their positive classes are too small for stable modelling; this is itself informative, because it shows that the precautionary 75%-of-guideline thresholds are what make B and F modellable at all in this dataset. These results confirm that the study-defined thresholds do not inflate screenability relative to current guideline values.

**BDL substitution sensitivity.** Alternative F substitutions (assumed detection limits of 0.2 and 0.5 mg L⁻¹, and direct substitution of the fallback limit) changed no target label and left headline pooled PR-AUCs unchanged to within 0.007 (Supplementary Table S12). The five F BDL wells are spread across three spatial clusters (clusters 0, 1, 2), so the BDL records do not concentrate in one held-out block; the spatial fragility of F is not an artefact of BDL substitution or of BDL-well geography.

## 3.9 Explainability and screening-priority mapping

Exact interventional TreeSHAP values (tree ensembles) and exact linear SHAP values (logistic models) identified hydrochemically coherent leading predictors (Figure 6). For Na, Cl and TDS models, EC, Na, Cl and salinity-linked ratios dominated; for the EC-strict TDS model, the principal non-EC contributors were Cl, Na, Ca, Mg and K, followed by HCO₃ and SO₄, indicating which laboratory analytes add the most information when conductivity readings are unavailable. Fluoride models showed a more mixed predictor structure, including SAR, B, K, Mg, Na, SO₄, HCO₃ and EC, consistent with F's more spatially sensitive behaviour. SHAP values describe model behaviour, not causal processes (Supplementary Methods S10).

Figure 7 translates validated probabilities into target-specific screening-priority maps for Na, Cl, TDS, B and F across the 81 wells, using the documented probability bands (Low < 0.25; Moderate 0.25–0.50; High 0.50–0.75; Very high ≥ 0.75) applied to well-level median out-of-fold probabilities from the best canonical model for each target. The maps are screening-priority outputs for confirmatory testing, not definitive contamination distributions.





# 4. Discussion

## 4.1 Principal finding

The framework identifies clear, target-specific differences in screenability under leakage-controlled, spatially conservative validation. Na, Cl and TDS are the most operationally screenable targets: they retain pooled out-of-fold PR-AUC values of 0.958–0.978 across predictor tiers, algorithms and spatial validation, and their operating points combine high sensitivity with acceptable false-positive burdens at the well level (Table 5). B is moderately screenable and benefits from expanded hydrochemical predictors. F is more difficult to screen reliably: performance improves with richer chemistry but declines under spatial validation and shows the most partition-dependent spatial performance (headline-model k-range 0.47\u20130.58). NO₃ is not suitable for standalone machine-learning screening in this dataset because the positive class is too small; a simpler rule-based nitrate risk flag based on land use or proximity to agricultural sources would be a reasonable non-ML alternative, but no such land-use covariate layer is available in this dataset to evaluate it. These results answer the central question: in a small semi-arid basin dataset, leakage-controlled virtual screening is useful, but only when target-specific prevalence, hydrochemical coherence and spatial transferability are explicitly evaluated.

## 4.2 Why salinity-related targets were more screenable

The stronger performance of Na, Cl and TDS is consistent with the hydrochemical structure of the Pru Basin dataset, in which water–rock interaction, ion exchange, weathering, salinisation and anthropogenic activity control groundwater chemistry, with salinisation especially evident in parts of the basin (Sunkari et al., 2021). Supplementary Figure S1 shows a strong salinity–sodium–chloride–boron correlation axis, including high EC–Na, EC–Cl and Na–Cl associations. These relationships make salinity-linked exceedance targets more learnable because field and major-ion variables encode a coherent ionic-strength gradient rather than isolated contaminant behaviour.

EC deserves special attention because it provides a field-scale proxy for total dissolved ionic load. EC-only logistic models perform strongly for Cl, TDS and Na (Table 3), which is operationally useful: rapid field conductivity readings can prioritise wells efficiently. But the EC-strict analyses show that most of this screenability is not EC-circular: with EC excluded, TDS retains 0.976 pooled PR-AUC (0.923 spatially), and Na and Cl retain 0.912 and 0.847, with the remaining signal carried by Cl, Na, Ca, Mg, K, HCO₃ and SO₄ (Figure 6). EC is thus a legitimate operational proxy, not circular target reconstruction, and the strongest screening recommendations can name the exact predictors a surveillance programme must measure when conductivity readings are unavailable.

## 4.3 Why F, B and NO₃ require more cautious interpretation

B improves from Tier 1 to Tier 3, indicating that field-only information does not fully capture its exceedance pattern. Its SHAP pattern — SAR, Na, EC, HCO₃/Cl and Cl — suggests association with sodium-rich, mineralised groundwater, but SHAP values identify variables influencing model predictions, not causal processes (Figure 6). B probabilities are useful for prioritisation but should not be read as precise risk estimates; the well-level operating point for B tolerates more false positives than for the salinity targets by design of the sensitivity-first rule (Table 5).

F is even more spatially sensitive. Its performance improves with richer predictor tiers, but pooled spatial PR-AUC declines relative to stratified validation (0.490 versus 0.636) and its spatial PR-AUC range across cluster counts spans 0.47 to 0.58 for the headline model (up to 0.51 across models; Supplementary Table S9). This supports the interpretation that fluoride occurrence is influenced by localised geochemical or lithological controls. Two alternative explanations were tested and excluded: the five F BDL wells are not geographically concentrated (they fall in clusters 0, 1, 2), and alternative BDL substitutions do not change labels or headline results (Supplementary Table S12). The residual uncertainty is dominated by the small number of F-positive wells (13 of 81), which limits both calibration-bin precision and fold-level stability.

NO₃ is different: the main limitation is extreme low prevalence (2 exceedances). Nitrate modelling would produce unstable and potentially misleading performance estimates; the data support descriptive reporting and a rule-based screening alternative, not machine learning.

## 4.4 Importance of leakage control, nested thresholds and spatial validation

Leakage control is essential because groundwater chemistry contains mathematically and chemically related variables: a Na model that could use Na itself, SAR or Na/Cl, or a Cl model that could use Cl-derived ratios, would report circular accuracy. Table 2 documents the exclusions, and the coordinate-ablation results (Section 3.8) further show that performance does not ride on explicit coordinates.

The fully nested threshold-selection design closes a second leak: post-hoc pooled cutoff selection uses all out-of-fold predictions, including the wells being scored, which inflates operating characteristics. The nested estimates are more conservative (Section 3.7; Supplementary Figure S7) and are the values recommended for operational use. Well-level consolidation (median out-of-fold probability per well) then gives decision metrics that are directly interpretable by a water manager: each well contributes once, and the FP/FN burdens in Table 5 are unique-well counts that cannot exceed the 81-well dataset.

Spatial validation is equally important because nearby wells may share geology, recharge history, residence time, land-use influence and hydrochemical evolution; random stratified cross-validation can place spatially similar wells in both training and test folds, producing optimistic transferability estimates (Roberts et al., 2017). The pooled spatial estimates, computed across all unique held-out wells with full fold transparency (Supplementary Table S15), separate general discrimination from geographic transferability. Two caveats are explicit: the spatial blocks are statistical partitions, not flow-path units, and blocked evaluation may overstate the penalty if hydrologically connected wells are split; and within-basin transferability is not external validation across basins.

Recent comparator studies clarify both the contribution and the remaining scope of the present framework. A distribution-aware stacked ensemble for groundwater heavy-metal pollution in Ghana's Densu Basin used nested cross-validation but identified the absence of spatial cross-validation as a limitation (Ansah-Narh et al., 2026); the present study adds target-wise leakage auditing, fully nested cutoff selection, spatial-group validation and well-level decision metrics. Automated deep-learning groundwater-table modelling with explicit uncertainty quantification (Abbaszadeh Shahri et al., 2022) and physics-informed subsurface imaging (Yin et al., 2025) illustrate the calibration-first standards to which screening models should be held; direct head-to-head comparison of PR-AUC values with these studies is not possible because datasets, targets and threshold definitions differ, but the qualitative benchmark is informative: leakage-controlled PR-AUC values in the 0.85–1.00 range for salinity-linked targets are consistent with strong, transferable signal rather than with artefacts of circular chemistry.

## 4.5 Operational relevance for groundwater surveillance

The framework converts validated model probabilities into decision-support outputs at two levels: target-specific operating cutoffs (Table 5) and screening-priority classes for confirmatory testing (Figure 7). A water manager would proceed as follows (Figure 8): choose the target and tier, apply the corresponding model, read the well-level median probability, apply the target-specific cutoff from Table 5, and classify priority with the documented probability bands. The cutoffs differ across targets (e.g., 0.43 for Cl versus 0.14 for B) precisely because prevalence and the sensitivity-first rule interact: low-prevalence targets need lower cutoffs to keep recall high, at the cost of more false-positive flags for laboratory confirmation. Missing an elevated-risk well is treated as more serious than flagging additional wells, which is the appropriate surveillance posture; the maps communicate this prioritisation and should not be interpreted as maps of true contamination distribution.

## 4.6 Limitations

The principal limitation is the small dataset. Eighty-one wells support exploratory, carefully validated screening but limit generalisation, particularly for lower-prevalence targets; the sample-to-predictor ratios and fold-level spreads in Supplementary Table S14 quantify this (e.g., Tier-3 models with 20–21 predictors and roughly 65 training wells per fold, with as few as 10 F-positive training cases in the smallest folds). The absence of independent seasonal sampling restricts temporal transferability: the dataset is cross-sectional, no updating or dynamic-filtering procedure is claimed, and any longitudinal deployment would require model updating and recalibration. Spatial validation improves conservatism but is within-basin only; the smallest spatial cluster contains 5 wells, limiting the stability of geographic testing at fine partitions. Calibration remains uncertain for B and F, where bin-level uncertainty is high (Figure 5). The binary labels were generated from study-defined precautionary screening thresholds and should not be read as current regulatory exceedance classifications; the WHO-guideline relabelling analysis (Section 3.8) shows the ranking of screenability is robust to threshold definition wherever prevalence permits modelling, but reclassification at other values would require complete re-estimation. The B threshold (1.125 mg L⁻¹) is explicitly configuration-inherited and not a WHO value. TreeSHAP values use the training-fold data as the background distribution and remain model-behaviour diagnostics, not causal statements.

## 4.7 Future work

Future work should prioritise independent seasonal and multi-basin validation and prospective testing of the screening-priority classes against new laboratory results. Additional sampling should target nitrate hotspots, fluoride-prone zones and underrepresented spatial clusters. A lumped-parameter flow/transport model could in principle supply a process-based predictor — especially for spatially fragile targets like F — but requires hydrogeological parameterisation (recharge, transmissivity, travel times) that is not available for this dataset; we therefore recommend it as a data-collection-motivating direction rather than an immediately integrable predictor. The framework should also be extended to uncertainty-aware field deployment, where probability intervals, target-specific thresholds and spatial-transferability warnings are displayed alongside screening classes, and to leakage-safe two-stage salinity scores for weakly screenable B and F targets.

# 5. Conclusion

The study demonstrates that leakage-controlled virtual screening can support prioritisation of wells for confirmatory groundwater testing in a small semi-arid basin dataset. Na, Cl and TDS are the most operationally screenable targets, showing strong discrimination under repeated stratified nested cross-validation and retaining high performance under spatial-group validation; their screenability survives removal of the EC conductivity proxy and of explicit coordinates, and is carried by coherent salinity and ionic-strength structure. B is moderately screenable and benefits from expanded hydrochemical predictors; F requires more caution because of weaker field-only performance, a clearer spatial-validation penalty and sensitivity to the spatial partition; NO₃ is not suitable for standalone machine-learning screening in this dataset. Fully nested threshold selection, well-level decision metrics and prevalence-benchmarked Brier scores provide an honest operating picture: every headline number in this study traces to the archived pipeline, database and exports (Section 2.10). The framework provides a practical route for low-cost groundwater surveillance, adaptive sampling and well prioritisation, but it does not replace laboratory confirmation or regulatory monitoring; SHAP values identify variables influencing model predictions, not proven causes of exceedance; and the GIS outputs show screening priority rather than true contamination distribution. Prospective seasonal and external validation is required before operational deployment.

**Acknowledgement**

The authors gratefully acknowledge the support of Sir Padampat Singhania University, Udaipur, India, and the Centre of Excellence in Environmental Science and Sustainability for providing an enabling research environment.

**Author Contributions (CRediT Author Statement)**

**Emmanuel Daanoba Sunkari:** Conceptualization, Data curation, Formal analysis, Investigation, Methodology, Project administration, Resources, Supervision, Validation, Writing – original draft, Writing – review & editing.

**Dickson Abdul-Wahab:** Conceptualization, Data curation, Investigation, Methodology, Software, Validation, Visualization, Writing – original draft, Writing – review & editing.

**Jonas Togyeni Mba:** Conceptualization, Formal analysis, Validation, Writing – original draft, Writing – review & editing.

**Kamal Hiran Kant:** Data curation, Formal analysis, Validation, Writing – original draft, Writing – review & editing.

**Hamidatu Saaka Darimani:** Data curation, Validation, Writing – review & editing.

# Data and Code Availability

The raw groundwater dataset originates from Sunkari et al. (2021) and is archived with the analysis inputs. The leakage-control rules, predictor-tier definitions, spatial-cluster assignments, cross-validation fold assignments, random seeds, hyperparameters, fold-level metrics, well-level predictions, SHAP values, nested and pooled operational-threshold selections, threshold sweeps, well-level decision metrics and GIS-ready exports are stored in an auditable SQLite reproducibility database (schema version 2.0), which is archived alongside the code and this manuscript at https://doi.org/10.5281/zenodo.21442254 (Abdul-Wahab & Sunkari, 2026); the development repository is https://github.com/dabdul-wahab1988/virtual-screening-of-groundwater-exceedance-risk. The database is therefore publicly accessible in full, not merely available on request.

# References

Abbasi, M., & Madani, M. (2026). Data-driven modeling of outlet temperature in fractured granite cores with implications for geothermal reservoir heat extraction. International Communications in Heat and Mass Transfer, 179, Article 112082. https://doi.org/10.1016/j.icheatmasstransfer.2026.112082

Abbaszadeh Shahri, A., Shan, C., & Larsson, S. (2022). A novel approach to uncertainty quantification in groundwater table modeling by automated predictive deep learning. Natural Resources Research, 31(3), 1351–1373. https://doi.org/10.1007/s11053-022-10051-w

Abdipour, H., Mansoorian, H. J., & Zari, S. N. (2026). Human health risk assessment of potentially toxic elements in drinking water of Hamedan province, Iran. Applied Water Science, 16(1). https://doi.org/10.1007/s13201-026-02938-3

Abdul-Wahab, D., & Sunkari, E. D. (2026). Virtual screening of groundwater exceedance risk (Version 0.2.0) [Computer software]. Zenodo. https://doi.org/10.5281/zenodo.21442254

Ansah-Narh, T., Afrifa, G. Y., Tandoh, J. B., Asare, K., Addi, M., Essel-Yorke, K. A., Akpoley, D. M. A., Aidoo, K., & Fosuhene, S. K. (2026). Smart ensemble learning framework for predicting groundwater heavy metal pollution. arXiv. https://doi.org/10.48550/arXiv.2605.00056

Breiman, L. (2001). Random forests. Machine Learning, 45(1), 5–32. https://doi.org/10.1023/A:1010933404324

Brier, G. W. (1950). Verification of forecasts expressed in terms of probability. Monthly Weather Review, 78(1), 1–3. https://doi.org/10.1175/1520-0493(1950)078<0001:VOFEIT>2.0.CO;2

Davis, J., & Goadrich, M. (2006). The relationship between precision–recall and ROC curves. In Proceedings of the 23rd International Conference on Machine Learning (pp. 233–240). Association for Computing Machinery. https://doi.org/10.1145/1143844.1143874

Friedman, J. H. (2001). Greedy function approximation: A gradient boosting machine. The Annals of Statistics, 29(5), 1189–1232. https://doi.org/10.1214/aos/1013203451

Helsel, D. R. (2012). Statistics for censored environmental data using Minitab and R (2nd ed.). Wiley. https://doi.org/10.1002/9781118162729

Hornung, R. W., & Reed, L. D. (1990). Estimation of average concentration in the presence of nondetectable values. Applied Occupational and Environmental Hygiene, 5(1), 46–51. https://doi.org/10.1080/1047322X.1990.10389587

Lundberg, S. M., & Lee, S.-I. (2017). A unified approach to interpreting model predictions. In I. Guyon, U. von Luxburg, S. Bengio, H. Wallach, R. Fergus, S. Vishwanathan, & R. Garnett (Eds.), Advances in neural information processing systems (Vol. 30, pp. 4765–4774). Curran Associates, Inc. https://doi.org/10.5555/3295222.3295230

Lundberg, S. M., Erion, G. G., & Lee, S.-I. (2018). Consistent individualized feature attribution for tree ensembles. arXiv. https://doi.org/10.48550/arXiv.1802.03888

Roberts, D. R., Bahn, V., Ciuti, S., Boyce, M. S., Elith, J., Guillera-Arroita, G., Hauenstein, S., Lahoz-Monfort, J. J., Schröder, B., Thuiller, W., Warton, D. I., Wintle, B. A., Hartig, F., & Dormann, C. F. (2017). Cross-validation strategies for data with temporal, spatial, hierarchical, or phylogenetic structure. Ecography, 40(8), 913–929. https://doi.org/10.1111/ecog.02881

Sunkari, E. D., Abu, M., & Zango, M. S. (2021). Geochemical evolution and tracing of groundwater salinization using different ionic ratios, multivariate statistical and geochemical modeling approaches in a typical semi-arid basin. Journal of Contaminant Hydrology, 236, Article 103742. https://doi.org/10.1016/j.jconhyd.2020.103742

Varma, S., & Simon, R. (2006). Bias in error estimation when using cross-validation for model selection. BMC Bioinformatics, 7, Article 91. https://doi.org/10.1186/1471-2105-7-91

World Health Organization. (2022). Guidelines for drinking-water quality: Fourth edition incorporating the first and second addenda. World Health Organization. https://www.who.int/publications/i/item/9789240045064

Yin, H., Ikard, S. J., Rucker, D. F., et al. (2025). Imaging hyporheic exchange by integrating deep learning and physics-informed inversion of time-lapse self-potential data. Geophysical Research Letters, 52, e2025GL118772. https://doi.org/10.1029/2025GL118772

Zhan, Y., Guo, Z., Podgorski, J., et al. (2025). Changes in meat consumption can improve groundwater quality. Nature Food, 6, 1–12. https://doi.org/10.1038/s43016-025-01188-x










# Tables

**Table 1 | Dataset structure, analytical completeness and exceedance prevalence for groundwater virtual screening.**

| Variable | Unit | Role | Valid n | BDL n | Missing n | Min | Median | Max | Threshold | Exceeding n | Exceeding % | Modelling status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Dx | decimal_degrees_lon | coordinate | 81 | 0 | 0 | -1.233 | -0.939 | -0.65 |  |  |  | Predictor |
| Dy | decimal_degrees_lat | coordinate | 81 | 0 | 0 | 7.779 | 8.033 | 8.37 |  |  |  | Predictor |
| pH | dimensionless | field_parameter | 81 | 0 | 0 | 7.77 | 8.3 | 8.63 |  |  |  | Predictor |
| Temp. | °C | field_parameter | 81 | 0 | 0 | 11.2 | 28.5 | 33.6 |  |  |  | Predictor |
| EC | µS/cm | field_parameter | 81 | 0 | 0 | 306 | 570 | 7914 |  |  |  | Predictor |
| TDS | mg/L | target_contaminant | 81 | 0 | 0 | 275 | 513 | 7123 | 750 | 13.0 | 16.0 | Modelled |
| Na | mg/L | target_contaminant | 81 | 0 | 0 | 25 | 172 | 2115 | 150 | 47.0 | 58.0 | Modelled |
| K | mg/L | laboratory_chemical | 81 | 0 | 0 | 0.98 | 1.63 | 10.8 |  |  |  | Predictor |
| Mg | mg/L | laboratory_chemical | 81 | 0 | 0 | 0.24 | 16.3 | 84.8 |  |  |  | Predictor |
| Ca | mg/L | laboratory_chemical | 81 | 0 | 0 | 8.16 | 33.8 | 505 |  |  |  | Predictor |
| Cl | mg/L | target_contaminant | 81 | 0 | 0 | 1.36 | 54.5 | 4671 | 187.5 | 15.0 | 18.5 | Modelled |
| SO4 | mg/L | laboratory_chemical | 81 | 0 | 0 | 0.01 | 5.29 | 72.6 |  |  |  | Predictor |
| HCO3 | mg/L | laboratory_chemical | 81 | 0 | 0 | 121 | 453 | 641 |  |  |  | Predictor |
| CO3 | mg/L | laboratory_chemical | 81 | 41 | 0 | 0.636 | 0.636 | 14.7 |  |  |  | Predictor |
| NO3 | mg/L | target_contaminant | 81 | 42 | 0 | 0.438 | 0.438 | 70.3 | 37.5 | 2.0 | 2.5 | Descriptive only |
| F | mg/L | target_contaminant | 81 | 5 | 0 | 0.163 | 0.53 | 3.4 | 1.125 | 13.0 | 16.0 | Modelled |
| B | mg/L | target_contaminant | 81 | 0 | 0 | 0.05 | 0.23 | 3.37 | 1.125 | 11.0 | 13.6 | Modelled |

**Table 2 | Leakage-control matrix used to prevent target-derived information from entering predictor sets.**

| Target | Predictor tier | n excluded | n retained | Excluded predictor(s) |
|---|---|---|---|---|
| B | Tier 3: full | 1 | 20 | B |
| B | Tier3_Full_no_coords | 1 | 18 | B |
| Cl⁻ | Tier 2: reduced | 1 | 11 | Cl |
| Cl⁻ | Tier2_Reduced_Cl_EC_strict | 2 | 10 | Cl; EC |
| Cl⁻ | Tier2_Reduced_no_coords | 1 | 9 | Cl |
| Cl⁻ | Tier 3: full | 3 | 18 | Cl; HCO3_Cl_ratio; Na_Cl_ratio |
| Cl⁻ | Tier3_Full_no_coords | 3 | 16 | Cl; HCO3_Cl_ratio; Na_Cl_ratio |
| F⁻ | Tier 3: full | 1 | 20 | F |
| F⁻ | Tier3_Full_no_coords | 1 | 18 | F |
| Na⁺ | Tier 2: reduced | 1 | 11 | Na |
| Na⁺ | Tier2_Reduced_Na_EC_strict | 2 | 10 | EC; Na |
| Na⁺ | Tier2_Reduced_no_coords | 1 | 9 | Na |
| Na⁺ | Tier 3: full | 3 | 18 | Na; Na_Cl_ratio; SAR |
| Na⁺ | Tier3_Full_no_coords | 3 | 16 | Na; Na_Cl_ratio; SAR |
| TDS | Tier2_Reduced_TDS_EC_strict | 1 | 11 | EC |

**Table 3 | Best-performing leakage-controlled classifiers across predictor tiers (pooled out-of-fold estimates under stratified nested CV, with the corresponding pooled spatial PR-AUC).**

| Target | Predictor tier | Best algorithm | PR-AUC | ROC-AUC | Recall | F₂ score | Brier score | Brier skill | Spatial PR-AUC |
|---|---|---|---|---|---|---|---|---|---|
| TDS | Tier 3: full | Gradient-boosted trees | 0.999 | 1.000 | 0.969 | 0.972 | 0.018 | 0.865 | 0.964 |
| TDS | Tier 2: reduced | Gradient-boosted trees | 0.998 | 0.999 | 0.985 | 0.988 | 0.020 | 0.853 | 0.938 |
| TDS | Tier 1: field | Gradient-boosted trees | 0.992 | 0.999 | 0.985 | 0.973 | 0.035 | 0.737 | 0.783 |
| Cl⁻ | Tier 2: reduced | EC-only logistic | 0.978 | 0.994 | 0.720 | 0.763 | 0.084 | 0.441 | 0.977 |
| Cl⁻ | Tier 3: full | EC-only logistic | 0.978 | 0.994 | 0.720 | 0.763 | 0.084 | 0.441 | 0.977 |
| Cl⁻ | Tier 1: field | EC-only logistic | 0.978 | 0.994 | 0.720 | 0.763 | 0.084 | 0.441 | 0.977 |
| Na⁺ | Tier 1: field | EC-only logistic | 0.958 | 0.937 | 0.932 | 0.904 | 0.176 | 0.276 | 0.925 |
| Na⁺ | Tier 2: reduced | EC-only logistic | 0.958 | 0.937 | 0.932 | 0.904 | 0.176 | 0.276 | 0.925 |
| Na⁺ | Tier 3: full | EC-only logistic | 0.958 | 0.937 | 0.932 | 0.904 | 0.176 | 0.276 | 0.925 |
| B | Tier 3: full | Random forest | 0.813 | 0.916 | 0.636 | 0.665 | 0.056 | 0.525 | 0.851 |
| B | Tier 2: reduced | Logistic regression | 0.787 | 0.886 | 0.745 | 0.735 | 0.084 | 0.285 | 0.842 |
| B | Tier 1: field | EC-only logistic | 0.750 | 0.902 | 0.636 | 0.670 | 0.101 | 0.143 | 0.791 |
| F⁻ | Tier 3: full | Gradient-boosted trees | 0.636 | 0.892 | 0.631 | 0.635 | 0.095 | 0.296 | 0.490 |
| F⁻ | Tier 2: reduced | Gradient-boosted trees | 0.550 | 0.878 | 0.477 | 0.483 | 0.107 | 0.209 | 0.537 |
| F⁻ | Tier 1: field | Random forest | 0.426 | 0.847 | 0.169 | 0.197 | 0.111 | 0.175 | 0.578 |

**Table 4 | Effect of predictor-tier expansion on leakage-controlled classifier performance (pooled out-of-fold PR-AUC and F₂, stratified nested CV).**

| Target | Algorithm | PR-AUC T1 | PR-AUC T2 | PR-AUC T3 | F₂ T1 | F₂ T2 | F₂ T3 | T1 retained (%) | T2 retained (%) |
|---|---|---|---|---|---|---|---|---|---|
| B | EC-only logistic | 0.750 | 0.750 | 0.750 | 0.670 | 0.670 | 0.670 | 100 | 100 |
| B | Gradient-boosted trees | 0.544 | 0.749 | 0.760 | 0.511 | 0.664 | 0.688 | 71.700 | 98.600 |
| B | Logistic regression | 0.709 | 0.787 | 0.797 | 0.656 | 0.735 | 0.806 | 89 | 98.800 |
| B | Random forest | 0.631 | 0.746 | 0.813 | 0.432 | 0.640 | 0.665 | 77.700 | 91.700 |
| Cl⁻ | EC-only logistic | 0.978 | 0.978 | 0.978 | 0.763 | 0.763 | 0.763 | 100 | 100 |
| Cl⁻ | Gradient-boosted trees | 0.854 | 0.945 | 0.953 | 0.871 | 0.863 | 0.851 | 89.700 | 99.200 |
| Cl⁻ | Logistic regression | 0.796 | 0.746 | 0.712 | 0.677 | 0.653 | 0.674 | 111.800 | 104.800 |
| Cl⁻ | Random forest | 0.870 | 0.958 | 0.955 | 0.630 | 0.770 | 0.776 | 91.100 | 100.300 |
| F⁻ | EC-only logistic | 0.344 | 0.344 | 0.344 | 0.398 | 0.398 | 0.398 | 100 | 100 |
| F⁻ | Gradient-boosted trees | 0.358 | 0.550 | 0.636 | 0.362 | 0.483 | 0.635 | 56.300 | 86.500 |
| F⁻ | Logistic regression | 0.293 | 0.463 | 0.547 | 0.490 | 0.687 | 0.694 | 53.700 | 84.600 |
| F⁻ | Random forest | 0.426 | 0.523 | 0.598 | 0.197 | 0.340 | 0.548 | 71.200 | 87.400 |
| Na⁺ | EC-only logistic | 0.958 | 0.958 | 0.958 | 0.904 | 0.904 | 0.904 | 100 | 100 |
| Na⁺ | Gradient-boosted trees | 0.901 | 0.956 | 0.946 | 0.882 | 0.899 | 0.895 | 95.200 | 101 |
| Na⁺ | Logistic regression | 0.856 | 0.925 | 0.927 | 0.766 | 0.847 | 0.868 | 92.400 | 99.800 |
| Na⁺ | Random forest | 0.900 | 0.946 | 0.953 | 0.921 | 0.921 | 0.886 | 94.400 | 99.300 |
| TDS | EC-only logistic | 0.976 | 0.976 | 0.976 | 0.681 | 0.681 | 0.681 | 100 | 100 |
| TDS | Gradient-boosted trees | 0.992 | 0.998 | 0.999 | 0.973 | 0.988 | 0.972 | 99.300 | 99.800 |
| TDS | Logistic regression | 0.817 | 0.770 | 0.744 | 0.744 | 0.615 | 0.672 | 109.800 | 103.600 |
| TDS | Random forest | 0.983 | 0.991 | 0.991 | 0.695 | 0.912 | 0.886 | 99.200 | 100 |

**Table 5 | Well-level operating characteristics at the sensitivity-first cutoff (median out-of-fold probability per well; each well counted once), with nested pooled and post-hoc pooled operating estimates.**

| Target | Predictor tier | Model | Cutoff | TP | TN | FP | FN | Sensitivity | Specificity | Nested sensitivity | Nested specificity | Pooled sensitivity | Pooled specificity |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| B | Tier 1: field | EC-only logistic | 0.320 | 10 | 53 | 17 | 1 | 0.909 | 0.757 | 0.909 | 0.426 | 0.909 | 0.663 |
| B | Tier 2: reduced | Logistic regression | 0.140 | 10 | 38 | 32 | 1 | 0.909 | 0.543 | 0.891 | 0.440 | 0.909 | 0.471 |
| B | Tier 3: full | Random forest | 0.140 | 10 | 58 | 12 | 1 | 0.909 | 0.829 | 0.891 | 0.426 | 0.909 | 0.760 |
| Cl⁻ | Tier 1: field | EC-only logistic | 0.430 | 14 | 66 | 0 | 1 | 0.933 | 1 | 0.933 | 0.955 | 0.920 | 0.988 |
| Cl⁻ | Tier 2: reduced | EC-only logistic | 0.430 | 14 | 66 | 0 | 1 | 0.933 | 1 | 0.933 | 0.955 | 0.920 | 0.988 |
| Cl⁻ | Tier 3: full | EC-only logistic | 0.430 | 14 | 66 | 0 | 1 | 0.933 | 1 | 0.933 | 0.955 | 0.920 | 0.988 |
| F⁻ | Tier 1: field | Random forest | 0.280 | 12 | 53 | 15 | 1 | 0.923 | 0.779 | 0.877 | 0.653 | 0.908 | 0.685 |
| F⁻ | Tier 2: reduced | Gradient-boosted trees | 0.240 | 12 | 53 | 15 | 1 | 0.923 | 0.779 | 0.831 | 0.718 | 0.908 | 0.747 |
| F⁻ | Tier 3: full | Gradient-boosted trees | 0.170 | 12 | 50 | 18 | 1 | 0.923 | 0.735 | 0.877 | 0.653 | 0.908 | 0.706 |
| Na⁺ | Tier 1: field | EC-only logistic | 0.500 | 44 | 25 | 9 | 3 | 0.936 | 0.735 | 0.906 | 0.771 | 0.902 | 0.800 |
| Na⁺ | Tier 2: reduced | EC-only logistic | 0.500 | 44 | 25 | 9 | 3 | 0.936 | 0.735 | 0.906 | 0.771 | 0.902 | 0.800 |
| Na⁺ | Tier 3: full | EC-only logistic | 0.500 | 44 | 25 | 9 | 3 | 0.936 | 0.735 | 0.906 | 0.771 | 0.902 | 0.800 |
| TDS | Tier 1: field | Gradient-boosted trees | 0.740 | 12 | 68 | 0 | 1 | 0.923 | 1 | 0.954 | 0.859 | 0.908 | 0.994 |
| TDS | Tier 2: reduced | Gradient-boosted trees | 0.840 | 12 | 68 | 0 | 1 | 0.923 | 1 | 0.938 | 0.976 | 0.908 | 1 |
| TDS | Tier 3: full | Gradient-boosted trees | 0.720 | 12 | 68 | 0 | 1 | 0.923 | 1 | 0.892 | 0.988 | 0.908 | 1 |

# Figures

![Figure 1. Leakage-controlled groundwater virtual-screening workflow. Preprocessing, tuning, class weighting and operational-cutoff selection are contained within each outer training fold; spatial clusters are validation partitions built by projected Euclidean k-means; explanations are exact interventional TreeSHAP values.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure1_workflow_schematic.png)

Figure 1. Leakage-controlled groundwater virtual-screening workflow. Preprocessing, tuning, class weighting and operational-cutoff selection are contained within each outer training fold; spatial clusters are validation partitions built by projected Euclidean k-means; explanations are exact interventional TreeSHAP values.

![Figure 2. Prevalence of groundwater-quality threshold exceedance across 81 wells. Red segments represent wells above the study-defined precautionary screening threshold; blue segments represent wells below it.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure2_exceedance_prevalence.png)

Figure 2. Prevalence of groundwater-quality threshold exceedance across 81 wells. Red segments represent wells above the study-defined precautionary screening threshold; blue segments represent wells below it.

![Figure 3. Pooled out-of-fold PR-AUC performance heatmap for leakage-controlled groundwater exceedance models across predictor tiers and algorithms (stratified nested CV).](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure3_performance_heatmap_PR_AUC.png)

Figure 3. Pooled out-of-fold PR-AUC performance heatmap for leakage-controlled groundwater exceedance models across predictor tiers and algorithms (stratified nested CV).

![Figure 4. Spatial validation penalty: (A) pooled PR-AUC under stratified versus spatial-group CV; (B) mean difference by target and tier.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure4_spatial_cv_penalty.png)

Figure 4. Spatial validation penalty: (A) pooled PR-AUC under stratified versus spatial-group CV; (B) mean difference by target and tier.

![Figure 5. Calibration curves for the Tier-3 models (stratified CV). Error bars are binomial 95% intervals at the bin level.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure5_calibration_curves.png)

Figure 5. Calibration curves for the Tier-3 models (stratified CV). Error bars are binomial 95% intervals at the bin level.

![Figure 6. Mean absolute SHAP values (exact interventional TreeSHAP, training-fold background) for the Tier-3 random-forest models, per target (stratified CV). SHAP values describe model behaviour, not causal processes.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure6_SHAP_importance.png)

Figure 6. Mean absolute SHAP values (exact interventional TreeSHAP, training-fold background) for the Tier-3 random-forest models, per target (stratified CV). SHAP values describe model behaviour, not causal processes.

![Figure 7. GIS-ready screening-priority maps: well-level median out-of-fold probabilities of the best canonical model per target, classified into the documented probability bands (Low < 0.25; Moderate 0.25–0.50; High 0.50–0.75; Very high ≥ 0.75). The maps express screening priority for confirmatory testing, not definitive contamination distributions.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure7_screening_priority_map.png)

Figure 7. GIS-ready screening-priority maps: well-level median out-of-fold probabilities of the best canonical model per target, classified into the documented probability bands (Low < 0.25; Moderate 0.25–0.50; High 0.50–0.75; Very high ≥ 0.75). The maps express screening priority for confirmatory testing, not definitive contamination distributions.

![Figure 8. End-user decision flowchart for the screening framework: choose target and tier, apply the leakage-filtered model, read the well-level median probability, apply the target-specific cutoff (Table 4) and classify priority.](C:\Users\ThinkPad P1 G4\Documents\Dr_Sukari_Manuscript\manuscript_4b\groundwater_virtual_screening\outputs\rev2\figures\Figure8_enduser_decision_flowchart.png)

Figure 8. End-user decision flowchart for the screening framework: choose target and tier, apply the leakage-filtered model, read the well-level median probability, apply the target-specific cutoff (Table 4) and classify priority.
