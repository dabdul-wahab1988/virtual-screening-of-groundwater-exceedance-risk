# Response to Reviewers

**Manuscript:** "Leakage-controlled and spatially validated machine learning for groundwater-quality exceedance-risk screening in the semi-arid Pru Basin, Ghana"

We thank the Editor and all four reviewers for their careful reading and constructive comments. We have performed a full end-to-end revision. The methodological changes below were implemented in the archived analysis pipeline, and all results were regenerated from the raw data into a new reproducibility database (schema version 2.0). Every number cited in the revised manuscript is traceable to that database or to its exports. Several of the revised analyses narrow the conclusions of the original submission; we state this explicitly in the responses below. The revised manuscript is written to be judged from the main text, with the Supplementary Material providing depth.

Each comment is reproduced verbatim (or abridged where it lists many DOIs), classified as AGREE, PARTLY ACCEPTED, CLARIFY or DISAGREE (with evidence), and answered with the manuscript location of the change. Section numbers refer to the revised manuscript, and Supplementary Table and Figure labels are the final numbers.

---

# Reviewer 1

**R1-01 (Language).** *"The MS language should be made native, and grammatical errors should be polished."*

**Response (AGREE).** The manuscript text was revised for English throughout, with consistent British spelling. We recommend a professional language edit at proof stage, because native-level polish has not been independently verified.

**R1-02 (Abstract with numbers).** *"The abstract should be revised, and interesting findings should be included numerically."*

**Response (AGREE).** The Abstract now reports the key numerical findings. The EC-only logistic baseline reached pooled PR-AUC 0.958 for Na and 0.978 for Cl (leave-one-cluster-out spatial PR-AUC 0.935 and 0.958), and no multivariable model exceeded it at any predictor tier. TDS equals 0.90 × EC in these data, so its high scores reproduce a unit conversion; an EC-excluded TDS model reached 0.976 (95% CI 0.919–1.000). B reached 0.813 (95% CI 0.557–0.973) and F 0.636 (95% CI 0.372–0.859; spatial 0.514), both described as exploratory. Under the fully nested cutoff rule with each well counted once, the EC-only Cl model detected 14 of 15 positive wells with 2 false positives, and the EC-only Na model detected 42 of 47 with 6.

**R1-03 (Alphabetical keywords).** *"It is recommended to write keywords alphabetically."*

**Response (AGREE).** Keywords are now ordered alphabetically (Abstract).

**R1-04 (Novelty).** *"The novelty of the work should be highlighted, and it should be stated what literature you have that has not been addressed so far."*

**Response (AGREE).** Section 1 (paragraph 4) contains an explicit novelty statement. The contribution is the integrated validation architecture: target-specific leakage control, tiered field-to-chemistry predictors, repeated nested cross-validation with fully nested cutoff selection, transparent spatial-group validation that retains single-class blocks, unique-well decision metrics, prevalence-benchmarked Brier scores and exact TreeSHAP attribution. We are not aware of a previous small-sample groundwater-screening study that combines these elements, although we have not conducted a systematic review. The knowledge gap relative to earlier deterministic hydrochemical classification (Sunkari et al., 2021) and to a recent Ghanaian ensemble study that used random rather than spatial cross-validation (Ansah-Narh et al., 2026) is stated.

**R1-05 (Water-quality references, six DOIs).** *"In initial paragraph written on the topic water quality, it is recommended to use these articles."* (DOIs listed in the comment.)

**Response (PARTLY ACCEPTED).** Each of the six DOIs was looked up in Crossref; the records are listed in Appendix A. One is cited: 10.1007/s13201-026-02938-3 (Abdipour et al., 2026, Applied Water Science; drinking-water health-risk assessment; Section 1, paragraph 1). The other five are water-treatment process studies (adsorption, electrodialysis, catalytic ozonation, dye degradation and photocatalysis) rather than groundwater-quality assessments, and were not cited.

**R1-07 (Knowledge-gap references, ten DOIs).** *"In the context of this research, use the following valuable articles to fill the knowledge gap."*

**Response (PARTLY ACCEPTED).** All ten DOIs have Crossref records (Appendix A). Two are cited: 10.1038/s43016-025-01188-x (Zhan et al., 2025, Nature Food; Section 1) and 10.1029/2025GL118772 (Yin et al., 2025, Geophysical Research Letters; Sections 1 and 4.5). The other eight were not cited because they concern geomechanics, mining, soil-hydraulic or geoenvironmental engineering rather than groundwater-quality modelling. Correction: Abbaszadeh Shahri et al. (2022) was supplied by Reviewer 3 and Abbasi & Madani (2026) by Reviewer 4, not in the Reviewer-1 knowledge-gap list; they are addressed under R3-03 and R4-07.

**R1-08 (Methodology reference).** 10.1016/j.eswa.2025.130999 has a Crossref record (Expert Systems with Applications; Crossref year 2026, DOI year 2025): a generic fuzzy-covering feature-selection method with no groundwater application. It was not adopted as methodology (Appendix A).

**R1-09 (Discussion reference).** 10.1007/s10489-025-06839-x (Applied Intelligence, 2025): a multimodal model with uncertainty calibration for vitiligo stage diagnosis. Medical imaging, off-topic for groundwater screening; not cited (Appendix A).

**R1-10 (Discussion reference).** 10.1080/17538947.2026.2646387 (International Journal of Digital Earth, 2026): tropospheric-delay correction for time-series InSAR. Remote-sensing method with no InSAR component in this study; not cited.

**R1-11 (Discussion reference).** 10.1016/j.eswa.2026.131177 (Expert Systems with Applications, 2026): label-interaction multi-label feature selection. The study models one binary label per target, so the method does not apply; not cited.

**R1-12 (Discussion reference).** 10.1016/j.patcog.2026.113096 (Pattern Recognition, 2026): few-shot open-set recognition. Computer-vision classification, off-topic; not cited.

**R1-13 (Discussion reference).** 10.1016/j.petsci.2026.05.001 (Petroleum Science, 2026): diffusion-based imputation of well-log data. Petroleum well-log imputation; the study uses median imputation inside each training fold (Section 2.5), so it is not cited.

**R1-14 (Discussion reference).** 10.1109/tifs.2025.3626979 (IEEE Transactions on Information Forensics and Security, 2025): DeFi protocol exploit detection. Blockchain security, off-topic; not cited.

**R1-15 (Discussion reference).** 10.1002/nag.70125 (International Journal for Numerical and Analytical Methods in Geomechanics, 2025): probabilistic analysis of pile-reinforced slopes. Geotechnical reliability, off-topic; not cited.

**R1-16 (Discussion reference).** 10.1016/j.est.2025.118038 (Journal of Energy Storage, 2025): online state-of-charge estimation of supercapacitors. Energy-storage engineering, off-topic; not cited.

**R1-17 (Introduction reference).** 10.1038/s41467-026-68798-2 (Nature Communications, 2026): global meta-analysis of plastic-mulch productivity and sustainability. Agronomy, not groundwater-related; not cited.

**R1-06 (Update old references).** *"Some of the references are very old, dating back over 20 years. Please update them."*

**Response (PARTLY ACCEPTED).** We audited the reference list. Foundational methodological references (Breiman 2001; Friedman 2001; Brier 1950; Davis & Goadrich 2006; Varma & Simon 2006; Helsel 2012; Hornung & Reed 1990; Roberts et al. 2017; WHO 2022) are retained because they are the primary sources of the methods and thresholds used; replacing them with secondary citations would not be appropriate. We accept that the update was limited. The revision adds five 2025–2026 studies (Abdipour et al., 2026; Zhan et al., 2025; Yin et al., 2025; Abbasi & Madani, 2026; Ansah-Narh et al., 2026), cited in the Introduction and, where relevant, in Section 4.5. We have not performed a systematic update of the 2019–2026 literature.

---

# Reviewer 2

**R2-01 (Operational recommendation in abstract).** *"...could be strengthened by explicitly stating the final recommendation for operational use..."*

**Response (AGREE, with a different recommendation).** The Abstract now ends with an explicit recommendation: "We recommend EC-based screening, with its nested cutoff, for Na and Cl and a direct EC threshold for TDS; the B and F models are exploratory and are not recommended for operational use." We did not adopt the specific recommendation suggested (the EC-strict TDS model and Tier-3 logistic regression for B). The EC-strict TDS result shows that dissolved ions reconstruct conductivity, and TDS equals 0.90 × EC in all 81 wells, so a TDS model is not an independent screening tool. The B models rest on 11 positive wells and a threshold without a boron-specific basis (R2-02, R4-01), so none is recommended for operational use.

**R2-02 (TDS threshold justification).** *"...the TDS threshold of 750 mg/L ... should be more prominently stated as a study-specific screening choice rather than a regulatory limit..."*

**Response (AGREE).** Section 2.2 now states prominently that all thresholds are study-defined precautionary screening cut-offs, not updated regulatory compliance limits. The Na, Cl, TDS, F and NO₃ thresholds correspond to 75% of the corresponding WHO guideline values. The TDS threshold is additionally an acceptability-based dissolved-solids alert. The B threshold of 1.125 mg L⁻¹ has no boron-specific basis: it equals 75% of the WHO fluoride guideline (1.5 mg L⁻¹), not 75% of the WHO boron guideline (2.4 mg L⁻¹), so all B results are conditional on it. Supplementary Table S2 documents threshold sources, and Supplementary Table S3 gives the B and F counts at alternative thresholds.

**R2-03 (Fourth tier excluding coordinates).** *"...a fourth tier that also excludes coordinates to truly test if spatial information alone drives performance..."*

**Response (AGREE).** We implemented coordinate-excluded counterparts of all three tiers for every target and validation design (coordinate ablation). Removing longitude and latitude changed the best pooled PR-AUC at Tier 3 by at most 0.032 (Section 3.8; Supplementary Tables S19 and S20; Figure S6). For example, the best Na Tier-2 model scored 0.956 with coordinates and 0.960 without, and the best B Tier-3 model scored 0.813 with and 0.818 without. The only sizeable change was for F at Tier 1 (0.426 with coordinates against 0.347 without), where location carries some information that chemistry otherwise supplies. Explicit coordinates therefore do not drive discrimination for Na, Cl, TDS or B, and only marginally for F. This does not rule out that the chemical predictors themselves encode location.

**R2-04 (Spatial groups may split hydrologically connected wells).** *"...could the resulting groups inadvertently separate wells that are hydrologically connected along a flow path, potentially overestimating the spatial penalty?"*

**Response (AGREE).** Section 2.6 and Section 4.4 now state explicitly that spatial clusters are statistical validation partitions, not hydrogeological flow-path units, and that blocked evaluation is a conservative transferability test that may overstate the penalty if hydrologically connected wells are split across blocks.

**R2-05 (Emphasise B/F k-sensitivity spread).** *"...the substantial spread for B and F (0.14–0.27) ... should be emphasized more strongly in the discussion."*

**Response (AGREE).** Section 3.5 reports the k-sensitivity results (k = 3 to 8), and Section 4.3 interprets B and F spatial transfer as uneven. The largest range of pooled leave-one-cluster-out PR-AUC across the partitions was 0.28 for B and 0.34 for F, against 0.15 for Na and 0.19 for Cl. The Tier-3 random forest for B ranged 0.80–0.86, and the Tier-3 gradient-boosted model for F ranged 0.51–0.72 (Supplementary Table S9). Spatial estimates for B and F are therefore partition-dependent. We state this as a limitation rather than tuning it away; k was chosen by the inertia elbow, never by classifier performance.

**R2-06 (EC-strict TDS: which predictors drive performance).** *"...could you elaborate on what other predictors (e.g., Na, Cl, HCO3) are driving this performance..."*

**Response (AGREE, with a qualification).** Section 3.9 and Section 4.2 report the principal contributors to the EC-strict TDS model from exact SHAP shares: Cl (31.1%) and Na (24.4%) dominated, followed by Ca, Mg, K and HCO₃ (Supplementary Table S21). The EC-strict TDS model reached PR-AUC 0.976 (Table 3), but this is not evidence of independent screenability. TDS is an exact linear function of EC in these 81 wells (TDS/EC = 0.900; r = 1.000; Supplementary Table S5), so the EC-strict result shows that the ions carry the conductivity signal. Where conductivity readings are unavailable, Na and Cl are the analytes that most recover that signal, which is the practical implication for laboratory effort.

**R2-07 (Validate Saabas attribution with exact Shapley for a subset).** *"...have you considered validating these attributions with a more exact Shapley value approximation for a subset of models to ensure consistency?"*

**Response (AGREE).** Rather than validating an approximation, we replaced the path-dependent attribution with exact values for all models. Tree ensembles now use exact path-dependent (cover-weighted) TreeSHAP, computed by exact Shapley-weighted subset enumeration over the features on each sample's path. This is feasible because the trees are shallow (depth at most 4 for random forests and 3 for boosted trees). Linear models use exact linear SHAP values (Lundberg & Lee, 2017). The implementation is unit-tested against brute-force Shapley enumeration, and the stored values satisfy exact additivity (base value plus the sum of SHAP values equals the model output). Figure 6, Supplementary Tables S21 and S22, and the archived database were regenerated from these values. The attributions are not interventional, and they describe model behaviour rather than causal effects (Supplementary Methods S10).

**R2-08 (Calibration uncertainty source).** *"...does this uncertainty primarily stem from the small sample size, or is it indicative of a more fundamental lack of structure..."*

**Response (PARTLY ACCEPTED; the data cannot settle the cause).** Section 3.7 and Section 4.3 attribute the bin-level calibration uncertainty for B and F to the small number of positive wells in each probability bin (11 and 13 positives in total), and Supplementary Tables S14 and S16 give the fold-level and bootstrap spread. We do not claim that the uncertainty is primarily sample-size-driven. Discrimination above the prevalence baseline is observed for both targets, but the data cannot distinguish limited sample size from a lack of learnable structure, and the revised text says so.

**R2-09 (How a water manager uses the cutoffs).** *"...how you envision a water manager actually using the probability cutoffs in Table 5, especially when they differ so widely (e.g., 0.29 for B vs. 0.43 for Cl)."*

**Response (AGREE).** A decision-support subsection (Section 4.6) and an end-user flowchart (Figure 8) now describe the procedure: choose the target and tier, apply the leakage-filtered model, read the well-level probability, apply the target-specific cutoff (Table 5) and classify priority. The cutoffs differ because they are specific to model, target and fold; they are not interchangeable scores. The mean nested cutoff is 0.40 for the EC-only Cl model and 0.08 for the Tier-3 B random forest (Table 5). A cutoff should be used only for the model and target from which it was derived. The sensitivity-first rule trades specificity for protection against missed exceedances; this is a modest cost for Na and Cl but a large one for B (Section 4.6).

**R2-10 (Simpler non-ML nitrate rule).** *"...could you discuss whether a simpler, non-ML approach like a simple rule based on land use or proximity to agricultural areas might be more appropriate for nitrate risk..."*

**Response (CLARIFY).** We agree that a rule-based nitrate risk flag (land use, proximity to agricultural sources) is a reasonable non-ML alternative, and Sections 3.2 and 4.3 now say so. No such land-use or agricultural covariate is available in this dataset, so we could not evaluate it quantitatively. The ML pathway remains inapplicable because only two wells exceed the nitrate threshold.

**R2-11 (BDL sensitivity, alternative MLE).** *"...have you tested the sensitivity of the results, particularly for F, to an alternative method like maximum likelihood estimation?"*

**Response (PARTLY ACCEPTED).** We ran a sensitivity analysis over four F substitution schemes: the minimum-detected/√2 baseline, assumed detection limits of 0.5 and 0.2 mg L⁻¹, and direct substitution of the fallback limit (Sections 2.5 and 3.8; Supplementary Table S7). No target label changes, and the headline pooled PR-AUCs for the Tier-2 and Tier-3 models change by at most 0.014. We did not fit a censored maximum-likelihood model: only five F records are affected, all substituted values lie far below the 1.125 mg L⁻¹ threshold, and the substitution cannot change any label (Supplementary Methods S2). The five BDL wells fall in three spatial clusters (Section 3.8; see also R2-18).

**R2-12 (Direct comparison with cited studies).** *"...a more direct comparison of your PR-AUC results for Na and Cl (≈1.000) with those from the cited studies..."*

**Response (CLARIFY; a direct numerical comparison is not possible).** Section 4.5 explains why. Ansah-Narh et al. (2026) regressed a continuous heavy-metal pollution index in the Densu Basin with random cross-validation (R² = 0.96 for their best model) and reported no PR-AUC for Na or Cl. Their targets, metrics and validation design differ from ours, so a head-to-head PR-AUC comparison cannot be made. The adjacent-field studies cited (Abbaszadeh Shahri et al., 2022; Yin et al., 2025; Abbasi & Madani, 2026) are not groundwater-quality screening benchmarks. We also correct the premise: our headline Na and Cl values are not ≈1.000. The EC-only logistic baseline gives 0.958 (Na) and 0.978 (Cl) (Table 3). That baseline is the comparator any high-accuracy Na or Cl study should be measured against.

**R2-13 (Single simplified end-user flowchart).** *"...the reader would benefit from a single, simplified flowchart ... (Step 1: Choose target; Step 2: Select tier; Step 3: Apply cutoff; Step 4: Map priority)."*

**Response (AGREE).** Figure 8 is the end-user decision flowchart, with seven steps: choose target; choose tier; check the leakage filter (Table 2); apply the model; apply the target-specific cutoff (Table 5); classify priority; and prioritise wells for confirmatory testing. Section 4.6 walks through it.

**R2-14 (F2 rationale and optimality).** *"...the exact rationale for using the F2 score ... and why the choice of β=2 is optimal..."*

**Response (AGREE).** Section 2.9 states that β = 2 is an explicit a priori surveillance preference that weights recall over precision, not a mathematically optimised cost ratio, and no optimality claim is made. Cutoffs were re-selected under F1, F1.5 and F3 and compared with the F2 choice (Section 3.7; Supplementary Table S13 and Figure S3). The ranking of targets and the qualitative conclusions did not change across β. The cutoffs move as expected: for the Tier-3 F gradient-boosted model the pooled max-F1 cutoff was 0.51 and the max-F3 cutoff 0.16.

**R2-15 (Would fully nested selection be overly conservative?).** *"...would a fully nested selection procedure (within each outer fold) have led to overly conservative and less generalizable cutoffs?"*

**Response (AGREE, with evidence; the effect is target-specific).** We implemented the fully nested procedure. The cutoff is chosen on inner-CV validation predictions inside each outer fold, frozen, and applied to that fold's held-out wells (Section 2.8; per-fold cutoffs in Supplementary Table S12). Table 5 gives the nested unique-well operating characteristics and, in a comparison-only column, the post-hoc specificity. Supplementary Figure S2 shows the comparison (Section 3.7). Nested specificity was lower than post-hoc specificity in most rows. For the Tier-3 B random forest it was 0.400 nested against 0.829 post hoc. Across the 23 target–model rows of Table 5, post-hoc specificity was higher in 15, equal in 6, and lower in 2 (the EC-only Na and TDS baselines). The post-hoc values were therefore optimistic for every B and F row, and the nested values are the ones we recommend for operational use. The nested rule is conservative relative to the post-hoc rule, but not uniformly so.

**R2-16 (Na Tier 2 vs Tier 3).** *"...does this suggest that adding more predictors introduces noise ... or is it a random effect of the small sample?"*

**Response (AGREE).** Section 3.6 now states that the small Tier-2-versus-Tier-3 differences for Na (0.956 against 0.953) are consistent with small-sample variation and are not interpreted as evidence that additional predictors add or remove information.

**R2-17 (How priority classes were defined).** *"...I'd like to see a brief description of how the 'low, moderate, high' screening priority classes were defined..."*

**Response (AGREE).** Section 2.10 and the caption of Figure 7 document the pre-specified probability bands applied to the well-level median out-of-fold probability of the best canonical (non-dummy, non-sensitivity) model for each target: Low < 0.25; Moderate 0.25–0.50; High 0.50–0.75; Very high ≥ 0.75. For Na and Cl this is the EC-only logistic model, and the TDS panel is effectively a conductivity map (Section 3.9). The bands are fixed probabilities that do not depend on any target-specific cutoff or on outcome data. They are screening-priority statements for confirmatory testing, not contamination classes. The revised Figure 7 shows all four bands, including the Very-high class, which the earlier version did not display (identified in our internal checks).

**R2-18 (F BDL wells geographically clustered?).** *"...could this spatial fragility be due to the fact that the five below-detection-limit wells are geographically clustered..."*

**Response (AGREE, with evidence).** Section 3.8 reports that the five F BDL wells fall in three different spatial clusters (0, 1 and 2), so they are not concentrated in one held-out block. Section 4.3 notes that the remaining explanation is the combination of few positives and strong partition dependence: leaving the largest cluster out leaves only 10 positive training wells. Together with the label-invariant substitution analysis (R2-11), the F spatial fragility is not an artefact of BDL handling.

**R2-19 (NO3 descriptive stats and spatial distribution).** *"...it would be beneficial to report its descriptive statistics and spatial distribution..."*

**Response (AGREE).** Section 3.1 now reports the NO₃ descriptive statistics: 39 quantified values ranging from 0.62 to 70.30 mg L⁻¹ (median 2.20 mg L⁻¹), 42 BDL records, and 2 exceedances of 37.5 mg L⁻¹. Supplementary Figure S1 shows the histogram with the screening threshold and a map of NO₃ concentrations with the BDL wells marked. The BDL wells are distributed across the basin rather than concentrated in one cluster.

**R2-20 (How to interpret the Brier score).** *"...what would be considered a 'good' Brier score for a screening tool of this nature..."*

**Response (AGREE).** Sections 2.9 and 3.7 explain that raw Brier scores depend on prevalence and have no universal "good" cutoff. We report a prevalence-only null Brier score, computed from each fold's own outer-training prevalence so that no test label enters the reference, and the Brier skill score (1 − Brier/Brier_null) for every model in Table 3. Brier skill is positive for 21 of 23 models (range 0.145 to 0.865). The two exceptions do not beat a prevalence-only forecast: the F EC-only baseline (−0.585) and the B Tier-1 model (−0.013). For the headline models the skill scores are 0.276 (Na baseline), 0.441 (Cl baseline), 0.771 (EC-strict TDS), 0.526 (Tier-3 B) and 0.298 (Tier-3 F). Because class-weighted models are not calibrated, skill is a calibration diagnostic and can disagree with PR-AUC.

**R2-21 (Explicit availability of the SQLite database).** *"...whether the exact SQLite reproducibility database is also publicly accessible or only available upon request."*

**Response (AGREE in the statement; deposit pending).** The Data and Code Availability statement now says that the revised pipeline (version 0.1.1) and the complete SQLite database (schema version 2.0) that produced every number in the manuscript will be deposited in the Zenodo record and made publicly accessible in full before publication. We are explicit that the existing Zenodo DOI (10.5281/zenodo.21442254) still corresponds to version 0.1.0. Until the deposit is made, the database is available from the corresponding author on request. The software reference will be updated to the deposited version at that point.

**R2-22 (Lumped-parameter model as predictor).** *"...a simple lumped-parameter model of flow and transport could be integrated directly as a predictor..."*

**Response (CLARIFY).** Section 4.8 addresses this. A lumped-parameter flow and transport predictor is scientifically attractive, especially for F, but would require recharge, transmissivity and travel-time information that this dataset does not contain. We therefore frame it as a data-collection-motivating direction rather than an immediately integrable predictor.

---

# Reviewer 3

**Preliminary note.** Several of Reviewer 3's claims are contradicted by the submitted work. Rather than accepting them at face value, we verified each against the repository and regenerated outputs. Where a claim is contradicted, we respond with evidence and improved visibility rather than redesigning the study.

**R3-01 (Self-citation in model set sentence).** *"'The model set comprised baseline, interpretable and nonlinear classifiers (Abdul-Wahab and Sunkari, 2026).'????? That is enough to REJECT this submission."*

**Response (AGREE).** The model-set sentence in Section 2.7 no longer carries the self-citation. The random forest and gradient-boosting methods are cited to Breiman (2001) and Friedman (2001) in the following sentence. The software record (Abdul-Wahab & Sunkari, 2026) is cited only in the Data and Code Availability statement and the reference list, where it belongs.

**R3-02 ("no replication evidence ... optimality approval, curve guarantees, error monitoring, early convergence, trapping...").** *[Verbatim.]*

**Response (DISAGREE, with evidence).** The claim of no replication evidence is contradicted by the submission. Repeated stratified nested CV (5 repeats × 5 folds = 25 held-out folds per classifier), deterministic seeds, dummy baselines, bootstrap confidence intervals (2,000 resamples; Section 2.9), calibration diagnostics and threshold sweeps are all present and are now stated in the main text (Sections 2.8–2.9, 3.4 and 3.7). The demands for optimality approval, convergence guarantees, error monitoring and trapping describe the engineering guarantees of large-scale optimisation libraries. They do not map onto the small, deterministic estimators evaluated here (shallow trees, bounded hyperparameter grids, at most 81 samples). Because the models are deterministic and the database is archived, the analysis can be rerun exactly once the v0.1.1 deposit is made (R2-21).

**R3-03 ("No solid comparison with advanced approaches ... only relies on confusion matrix ... lacks supportive metrics ...").** *[Abridged.]*

**Response (DISAGREE, with evidence).** The submitted work does not rely on confusion matrices. It reports ROC-AUC, PR-AUC, recall, specificity, F1, F1.5, F2, F3, balanced accuracy, the Brier score with a prevalence-null baseline and skill score, calibration slope and intercept, fold-level mean ± SD, bootstrap CIs, spatial CV with pooled estimates, threshold sweeps and technical limitations (Section 2.9; Tables 3–5; Supplementary Tables S14–S16). The cited Abbaszadeh Shahri et al. (2022; Natural Resources Research, 10.1007/s11053-022-10051-w) is a groundwater-table uncertainty-quantification study using deep learning, not a spatial-distribution analysis. Our earlier description of it was wrong and is corrected. It is cited in Section 1 and discussed as a qualitative comparator in Section 4.5, which notes that head-to-head PR-AUC comparison is not possible. Deep-learning and support-vector models were not used, because the training sets (about 65 samples per outer fold and up to 21 predictors) do not justify them (Section 2.7; Supplementary Methods S6). We agree that the comparison with more advanced approaches is qualitative only, and this is stated in Section 4.5. Spatial distribution is addressed in R3-05.

**R3-04 (Temporal variability / dynamic filtering / updating).** *"What about temporal variability of groundwater quality??? ... you need an updating procedure and dynamic filtering ..."*

**Response (DISAGREE for dynamic filtering, with a scope statement; PARTLY ACCEPTED for robustness).**

(a) *Dynamic filtering and sequential updating.* The dataset is a single cross-sectional campaign (81 wells sampled in one dry-season round; Section 2.1), with no repeated sampling and no time series. Dynamic filtering and sequential model updating therefore cannot be fitted or evaluated on it, and the framework makes no temporal-transferability claim (Sections 2.1 and 4.7). We reviewed the two suggested papers at the level of their Crossref records (Appendix A). The Sensors (2024) paper addresses automated normalisation of sensor-based measurement-while-drilling data into a unified database; the Soft Computing (2026) paper proposes a data-normalisation transformation. Neither is presented here as a dynamic-filtering or updating method for groundwater time series, neither was read in full, and neither has been adopted. Section 4.7 states that any longitudinal deployment would require model updating and recalibration.

(b) *Robustness to noise, missing data and malfunction.* The study handles missing and below-detection-limit values in specific ways. Plain BDL markers were replaced by limit/√2, and where detection limits were not provided, by the documented fallback limit (Section 2.5). A sensitivity analysis of alternative F substitutions left all target labels unchanged (Section 3.8; Supplementary Table S7). Continuous predictors were median-imputed and standardised within each training fold only, so imputation cannot transmit test-fold information (Section 2.5). Spatial-block validation provides a geographic robustness test (Sections 2.6 and 2.8). No noise was injected into predictors or labels, sensor malfunction was not simulated, and performance was not tested across graded levels of missing data. Robustness to measurement noise, and to missingness beyond the BDL substitution, is therefore not established. Section 4.7 now states this limitation.

**R3-05 ("Where is the claimed 'Leakage-controlled and spatially'???? This work basically doesn't have any spatial analysis and mapping.").** *[Verbatim.]*

**Response (DISAGREE, with evidence).** The claim is contradicted by the submission. Spatial clustering (Section 2.6), spatial-group cross-validation with per-fold transparency and pooled evaluation (Sections 2.8 and 3.5; Supplementary Table S10), a spatial-penalty analysis (Figure 4), cluster metadata (Supplementary Table S8), k-sensitivity (Supplementary Table S9) and GIS-ready screening-priority maps (Figure 7) are all present. The revised main text presents these elements in Sections 2.6, 2.8 and 3.5 and in Figures 4 and 7.

**R3-06 (English revision; undefined acronyms).** *"English requires substantial revision / many undefined acronyms and parameters at first appearance."*

**Response (AGREE, with one gap).** The text was revised for English. Acronyms are defined at first use, including EC, TDS, BDL, PR-AUC, ROC-AUC, CV, SAR, TP/TN/FP/FN, WHO, GIS and SHAP. The chemical symbols B and F are not yet expanded to boron and fluoride at their first appearance in the Abstract and Section 2.2; this is an author action to be completed before resubmission. Equations are native Word equations in the DOCX.

**R3-07 (Improper citations; Sunkari et al. 2021 "obviously is REJECTED").** *"Trivial improper citations. Just as an example, the claimed citation to Sunkari et al., 2021 (L54, L57) or … obviously is REJECTED and it has been recognized much earlier than 2021."*

**Response (CLARIFY).** We checked the Sunkari, Abu & Zango (2021) reference against the published record: Journal of Contaminant Hydrology, Volume 236, Article 103742, https://doi.org/10.1016/j.jconhyd.2020.103742 (published 2021; the DOI year 2020 reflects online-first assignment). It is the source study of the Pru Basin dataset and is cited as such (Sections 1 and 2.1). We do not find the citation improper. If the reviewer refers to a different work, we would be glad to check it.

**R3-08 (No embedded figures/tables in context).** *"No embedded Fig/Tables/captions within the context."*

**Response (AGREE).** Figures 1–8 and Tables 1–5 are embedded in the main text near their first citation in the DOCX, each with a caption and in-text citations in logical numbering.

**R3-09 (Opacity of methods).** *"Full opacity in terms of datasets, diversity, … randomization, unifications, time interval span, … opacity in binary risk-alert thresholds, spatial grouping strategy for cross-validation, spatial autocorrelation, selection of predictor variables, robustness to class imbalance, external or independent validation, …."*

**Response (AGREE).** The main text now contains what is needed to judge the methodology: dataset description and completeness (Section 2.1; Table 1); thresholds and their scientific status (Section 2.2; Table 1; Supplementary Table S2); predictor selection and tiers (Section 2.3); leakage rules (Section 2.4; Table 2); preprocessing, BDL handling and class weighting (Section 2.5); spatial grouping (Section 2.6); the spatial-autocorrelation treatment, which is blocked validation rather than explicit modelling (Sections 2.8 and 4.4); randomisation through deterministic seeds (Section 2.10); the time span, a single dry-season campaign (Sections 2.1 and 4.7); and external validation, which has not been performed (Section 4.7).

**R3-10 ("More suitable for screening than definitive contamination prediction").** *[Verbatim.]*

**Response (AGREE).** This is the study's framing. The revised text maintains the distinction between screening probability and contamination status in the Abstract, Sections 2.10, 3.9, 4.6 and 5, and the Figure 7 caption.

**R3-11 (Unjustified conclusion).** *[Verbatim.]*

**Response (AGREE).** The conclusions were rewritten to be proportional to the evidence. Section 5 now states that the EC-only logistic model screens Na and Cl as well as any richer model tested, that TDS is a conductivity conversion, that B and F are exploratory with wide intervals and unstable cutoffs, and that NO₃ cannot be modelled. The uncertainty (fold spread, bootstrap CIs, k-sensitivity ranges and calibration limits) is stated in Sections 3.4–3.7.

---

# Reviewer 4

**R4-01 (Threshold justification + sensitivity at guideline values).** *"...provide a sensitivity analysis using current drinking-water guideline values."*

**Response (AGREE).** Section 2.2 gives the scientific status of each threshold: 75% of the corresponding WHO guideline values as precautionary early-warning cut-offs, with the TDS threshold additionally an acceptability-based alert. The B threshold has no boron-specific basis (R2-02). Section 3.8 and Supplementary Table S4 relabel all targets at current WHO guideline values (Na 200 mg L⁻¹, 27 positives; Cl 250 mg L⁻¹, 12; TDS 1000 mg L⁻¹, 8). Na, Cl and TDS remained screenable at these values: the best pooled PR-AUC was 0.962 (Na, Tier-3 random forest), 0.986 (Cl, Tier-2 random forest; 0.975 at Tier 3) and 1.000 (TDS). F (3 positives at 1.5 mg L⁻¹) and B (3 positives at 2.4 mg L⁻¹) are too sparse for modelling and are described only. This means the 75%-of-guideline thresholds are what make F modellable in this dataset. The threshold choice nevertheless remains a limitation.

**R4-02 (Overfitting risk; sample-to-predictor ratio).** *"...Please further discuss the risk of overfitting and the adequacy of the sample-to-predictor ratio, particularly for B and F."*

**Response (AGREE).** Section 3.4 ("Small-sample adequacy") and Supplementary Table S17 report the audit: predictor counts, training-set sizes, positive training cases per outer fold, events per variable, and the fold-level PR-AUC range for each target and tier. The random-forest grid was restricted to max_depth ≤ 4 for the small training sets (Section 2.7). The Tier-3 B random forest had 20 predictors and at least 8 positive training wells per outer fold (0.40–0.44 events per variable). The Tier-3 F gradient-boosted model had 20 predictors and at least 10 positive training wells (0.50–0.52 events per variable). Both are far below the conventional ten events per variable, so both models are treated as exploratory (Sections 3.4 and 4.3), and their point estimates must be read with the reported intervals.

**R4-03 (Spatial clustering implementation).** *"...please describe the exact implementation and justify why this approach is appropriate for geographic coordinates."*

**Response (AGREE).** Section 2.6 describes the exact implementation. Coordinates are projected onto a local equirectangular plane (kilometres) centred on the basin centroid, and Euclidean k-means with deterministic k-means++ initialisation is applied in that projected space. Coordinate-mean centroids are then the least-squares cluster centres on the plane, which is not the case for great-circle distance. We found that the earlier description (k-means on Haversine distances with coordinate-mean centroids) was internally inconsistent, and replaced it. The number of clusters was chosen by the documented inertia-elbow rule (k = 4; cluster sizes 41, 5, 18 and 17), and sensitivity to k is reported (Supplementary Tables S8 and S9).

**R4-04 (Single-class spatial folds; bias in transferability estimates).** *"...Please explain how exclusion of spatial folds lacking both classes may influence the reported spatial PR-AUC..."*

**Response (AGREE; the reviewer was right).** Spatial folds are no longer excluded. The design is leave-one-cluster-out on the four clusters, each held out once, and every fold is reported with its sample size, positive and negative counts and status (Supplementary Table S10). Pooled spatial PR-AUC is computed across all unique held-out wells, and a binary-valid-only estimate is reported alongside it to quantify the effect of excluding single-class folds (Sections 2.6 and 3.5; Supplementary Table S18). At the main k = 4 partition, cluster 1 (five wells) contains no positive well for B or F. Excluding that block would have raised the pooled spatial PR-AUC of the Tier-3 B random forest from 0.831 to 0.844, and of the Tier-3 F gradient-boosted model from 0.514 to 0.523. Excluding single-class blocks therefore inflates the estimated geographic transferability of the two targets with the fewest positives, by up to 0.027 PR-AUC across the Tier-3 models, and by nothing for Na, Cl and TDS, which have both classes in every block (Section 3.5). At finer partitions (k = 5 to 8), single-class held-out blocks become common (one to three per target for the Tier-3 random forest). Spatial estimates for B and F then rest on very few wells and positives.

**R4-05 (Proxy-driven discrimination).** *"...The possibility of proxy-driven discrimination should be discussed more explicitly."*

**Response (AGREE).** Section 4.2 and the EC-ablation results discuss proxy-driven discrimination explicitly, and the distinction matters for each target. For TDS, EC is an exact linear function of TDS in these data (TDS/EC = 0.900; r = 1.000). Any TDS model that can see EC therefore reproduces a unit conversion, and the EC-strict TDS result (PR-AUC 0.976) shows only that the ions carry the same conductivity signal. For Na and Cl the labels are not arithmetic functions of EC, so EC is a separate and legitimate operational proxy. Excluding EC reduced pooled PR-AUC to 0.912 for Na and 0.847 for Cl (Table 3), so ion-based models recover much but not all of the conductivity signal. The non-EC contributors are Cl and Na for TDS, Cl, HCO₃ and SO₄ for Na, and Na, Ca and Mg for Cl (Section 3.9).

**R4-06 (Fully nested threshold selection).** *"Operational probability thresholds were selected from pooled out-of-fold predictions ... A fully nested threshold-selection strategy would provide a less optimistic estimate..."*

**Response (AGREE, with evidence).** Fully nested threshold selection is implemented (Section 2.8). The cutoff is chosen on inner-CV validation predictions inside each outer fold, frozen, and applied once to that fold's held-out wells. Per-fold cutoffs and operating metrics are archived (Supplementary Table S12). Table 5 reports the nested unique-well operating characteristics and, in a comparison-only column, the post-hoc specificity; Supplementary Figure S2 shows the comparison (Section 3.7). The reviewer's expectation holds for B and F: nested specificity is lower than the post-hoc value in every B and F row (for the Tier-3 B random forest, 0.400 against 0.829). The effect is not uniform across targets (see R2-15).

**R4-07 (Updated literature).** *"...consider relevant recent works such as 10.1016/j.icheatmasstransfer.2026.112082, 10.1016/j.pce.2026.104704, and 10.1016/j.advwatres.2026.105393..."**

**Response (CLARIFY).** Crossref records were retrieved for all three DOIs (Appendix A). 10.1016/j.icheatmasstransfer.2026.112082 (Abbasi & Madani, 2026, International Communications in Heat and Mass Transfer; data-driven modelling of fractured granite cores) is cited in Section 1. 10.1016/j.pce.2026.104704 (Abbasi, 2026, Physics and Chemistry of the Earth; analytical thermal model of naturally fractured geothermal reservoirs) and 10.1016/j.advwatres.2026.105393 (Abbasi, 2026, Advances in Water Resources; analytical well-test modelling of fractured reservoirs) were not cited. Our earlier reason for declining both, that they are geothermal or reservoir-model studies whose scope does not add to the screening methodology, was based on the titles alone, and the second title does not describe a geothermal study. Neither article has been read in full, so whether either is relevant is not yet verified; this is flagged for author action.

**R4-08 (SHAP vs Saabas terminology).** *"...these methods are not equivalent; terminology should be corrected or exact TreeSHAP values should be calculated."*

**Response (AGREE).** Exact path-dependent (cover-weighted) TreeSHAP (Lundberg et al., 2018) is now computed for every tree ensemble, and exact linear SHAP (Lundberg & Lee, 2017) for the logistic models. The term SHAP is used only for these values, and the terminology has been corrected throughout the manuscript, including Figure 6 and Supplementary Methods S10. The implementation is unit-tested against brute-force Shapley enumeration with exact additivity checks. The method is path-dependent, not interventional (see R2-07). The EC-strict and headline attribution shares are in Supplementary Tables S21 and S22.

**R4-09 (FP/FN counts under repeated CV).** *"...whether repeated predictions from the same well contributed more than once to these counts."*

**Response (AGREE).** The earlier table counted prediction instances, so a well contributed once per repeat. Table 5 now reports unique-well counts. Each well is predicted once per repeat by the outer fold in which it is held out. Each prediction is compared with the frozen, inner-CV-selected cutoff of that fold, and the well is flagged if the prediction met or exceeded its cutoff in a strict majority of the five repeats. TP, TN, FP and FN therefore sum to 81 for every row (Sections 2.10 and 3.7). Pooled prediction-instance counts, in which a well contributes once per repeat, are not used for decision support. The median out-of-fold probability is used only for the screening-priority classes and the well-level bootstrap (Section 2.9), not for the operating counts.

**R4-10 (Results/Discussion repetition).** *"...The Results and Discussion sections contain substantial repetition... These sections could be shortened and made more analytical..."*

**Response (PARTLY ACCEPTED).** We reduced repetition: the Results report the numbers, and the Discussion (Sections 4.1–4.3) interprets them with fewer repeated descriptions. Some key figures, such as the Na and Cl baseline PR-AUCs, still recur in the Abstract, Results, Discussion and Conclusion by design, so that each section can be read on its own. We have not reduced these repeats further.

---

# Editor-level summary of the main scientific changes

1. **Conclusions narrowed.** An EC-only logistic baseline is now reported as its own row. No multivariable model beat it for Na or Cl at any predictor tier, so the added value of predictor tiers and nonlinear learners is not demonstrated for these two targets (R2-06, R2-12).
2. **TDS reclassified.** TDS equals 0.90 × EC in all 81 wells, so it is a benchmark and unit-conversion target, not an independent screening target. The recommendation changed accordingly to EC-based screening with its nested cutoff for Na and Cl, a direct EC threshold for TDS, and exploratory B and F models (R2-01, R2-06, R4-05).
3. **Fully nested operational cutoffs.** Cutoffs are chosen inside each outer fold and frozen. The earlier post-hoc cutoffs are retained only as a labelled comparison, and they were optimistic for every B and F row (R2-15, R4-06).
4. **Unique-well decision counts.** Table 5 counts each well once by strict majority of repeats, with TP + TN + FP + FN = 81 for every row (R4-09).
5. **Spatial validation redesigned.** Leave-one-cluster-out on four clusters, with no fold discarded. Retaining single-class blocks changes the pooled B and F spatial estimates, and excluding them would have inflated them by up to 0.027 PR-AUC (R4-04).
6. **Spatial clustering corrected.** Projected Euclidean k-means replaces the earlier Haversine-with-coordinate-mean variant, which was internally inconsistent. k-sensitivity is reported, and it shows B and F spatial results to be partition-dependent (R4-03, R2-05).
7. **Exact TreeSHAP.** Exact path-dependent (cover-weighted) TreeSHAP for tree ensembles and exact linear SHAP for logistic models, unit-tested against brute-force Shapley enumeration. Interventional wording has been removed (R2-07, R4-08).
8. **Coordinate ablation for all tiers** (R2-03).
9. **Threshold sensitivity at current WHO guideline values.** Na, Cl and TDS remain screenable; F and B are too sparse to model. The B threshold has no boron-specific basis, and B results are conditional on it (R2-02, R4-01).
10. **Small-sample audit.** Events per variable for the Tier-3 B and F models are 0.40–0.52, so both are treated as exploratory. Random-forest depth is restricted to 4 (R4-02).
11. **BDL sensitivity for F.** Four substitution schemes leave every target label unchanged, and headline PR-AUCs change by at most 0.014. A censored maximum-likelihood model was not fitted, for the reasons given (R2-11, R2-18).
12. **F2 rationale and β sensitivity.** β = 2 is an a priori surveillance preference. Cutoffs under F1, F1.5 and F3 are reported, and the target ranking does not change (R2-14).
13. **Brier null and skill.** The null uses each fold's training prevalence. Skill is positive for 21 of 23 models; the F EC-only baseline and the B Tier-1 model do not beat the prevalence forecast (R2-20).
14. **Screening-priority bands documented.** Bands are fixed; the maps use the best canonical model per target, with the Very-high class shown (R2-17).
15. **Data availability.** The v0.1.1 pipeline and complete database will be deposited before publication. The existing Zenodo DOI still corresponds to v0.1.0 (R2-21).
16. **Reference suggestions checked against Crossref** (Appendix A). Of the 32 DOIs and URLs supplied by Reviewers 1, 3 and 4, five are cited and the rest are not, for the reasons given there. Two Reviewer-4 items are flagged for author action.
17. **Claim discipline.** Screening probability is distinguished from contamination status, and temporal, dynamic-filtering and external-validation claims are withdrawn. The limitations are stated in Section 4.7 (R3-04, R3-10, R3-11).
18. **Presentation.** Figures and tables are embedded in the main text near first citation, equations are native Word equations, and internal working notes have been removed (R3-06, R3-08).

---

# Appendix A. Per-DOI and per-URL disposition of reference suggestions

**Scope and method.** The table covers every DOI or URL supplied by Reviewer 1 (26 DOIs), Reviewer 3 (3 URLs) and Reviewer 4 (3 DOIs), 32 items in total. Each item was looked up in the Crossref REST API (api.crossref.org) on 14 October 2026, and the journal and year are taken from the Crossref record. Titles are paraphrased except where quoted. A "cited" disposition was determined by searching the reference list and text of the revised manuscript (Manuscript_Revised.md). Subject judgements are based on the Crossref title and journal only; no full text was read unless stated. The URL-to-DOI mapping for the MDPI item (10.3390/s24041209) follows the MDPI page address, and the Crossref record for that DOI was retrieved.

**Totals.** Cited: 5 (Reviewer 1: 3; Reviewer 3: 1; Reviewer 4: 1). Not cited: 27, of which 2 (Reviewer 4) are marked "Not verified" because their relevance has not been checked against full text and requires author action. Not verified at record level: 0. Every item returned a Crossref record.

| Reviewer | DOI / URL | Disposition | Reason (Crossref record: journal, year, first author; title paraphrased) |
|---|---|---|---|
| Reviewer 1 | 10.1080/03067319.2024.2432583 | Not cited | Int. J. Environ. Anal. Chem., 2024 (Torabideh): mercury adsorption by ZIF-8 optimised by response-surface methodology. Water-treatment process study, not a groundwater-quality assessment. |
| Reviewer 1 | 10.30955/gnj.005098 | Not cited | Global NEST Journal, 2024 (no author in record): electrodialysis removal of nickel and chromium(VI) from industrial wastewater. Treatment process study. |
| Reviewer 1 | 10.1007/s13201-025-02661-5 | Not cited | Applied Water Science, 2025 (Abdipour): catalytic ozonation with zero-valent iron nanoparticles for combined arsenic and nitrate removal. Treatment process study. |
| Reviewer 1 | 10.1007/s13201-026-02938-3 | Cited (Section 1, paragraph 1) | Applied Water Science, 2026 (Abdipour): health-risk assessment of potentially toxic elements in drinking water in Hamedan province, Iran. Drinking-water quality context. |
| Reviewer 1 | 10.1016/j.cep.2025.110160 | Not cited | Chemical Engineering and Processing: Process Intensification, 2025 (Rajabizadeh): persulfate and dithionite activation under magnetic fields for Rhodamine B removal. Treatment process study. |
| Reviewer 1 | 10.1080/00986445.2024.2445230 | Not cited | Chemical Engineering Communications, 2025 (Moein): photocatalytic removal of amoxicillin using magnetic Fe-TiO2 nanoparticles. Treatment process study. |
| Reviewer 1 | 10.1016/j.geoderma.2023.116713 | Not cited | Geoderma, 2023 (Zhao): uncertainty in process-based soil-hydraulic modelling of a semi-arid steppe. Soil process modelling rather than groundwater-quality classification; judged peripheral. |
| Reviewer 1 | 10.1080/19942060.2024.2447389 | Not cited | Engineering Applications of Computational Fluid Mechanics, 2025 (Di): hydraulic response of silted urban stormwater pipes and manholes (GASM-TranGRU and CFD-DEM). Urban drainage, not groundwater. |
| Reviewer 1 | 10.1016/j.engappai.2025.112554 | Not cited | Engineering Applications of Artificial Intelligence, 2025 (Zou): deep-learning prediction of limestone stress-strain response under shield-tunnelling loads. Geomechanics. |
| Reviewer 1 | 10.1016/j.compgeo.2026.108039 | Not cited | Computers and Geotechnics, 2026 (Zheng): two-stage material point method for cascading failure in levee systems. Geotechnical failure mechanics. |
| Reviewer 1 | 10.1038/s43016-025-01188-x | Cited (Section 1, paragraph 1) | Nature Food, 2025 (Zhan): changes in meat consumption and groundwater quality. Groundwater-quality drivers. |
| Reviewer 1 | 10.1007/s10230-025-01066-9 | Not cited | Mine Water and the Environment, 2025 (Feng): spatio-temporal evolution of water bodies and water-conservation mining in the Xinjiang coalfield. Mine-water case study; no machine learning. |
| Reviewer 1 | 10.1016/j.ijmst.2026.03.015 | Not cited | International Journal of Mining Science and Technology, 2026 (Zhang): air leakage in coal-mine gas-drainage boreholes and sealing. Mining gas engineering. |
| Reviewer 1 | 10.1002/ep.13149 | Not cited | Environmental Progress and Sustainable Energy, 2019 (Xiao): curing conditions and properties of solidified clay in landfill-leachate environments. Geoenvironmental engineering (landfill leachate and clay curing), not water treatment. |
| Reviewer 1 | 10.1029/2025GL118772 | Cited (Section 1, paragraph 2; Section 4.5) | Geophysical Research Letters, 2025 (Yin): imaging hyporheic exchange by combining deep learning with physics-informed inversion of time-lapse self-potential data. Physics-informed subsurface imaging. |
| Reviewer 1 | 10.1061/IJGNAI.GMENG-9175 | Not cited | International Journal of Geomechanics, 2024 (Wu): rainfall-induced seepage into an underlying bedrock slope (seepage-deformation coupling). Slope geomechanics. |
| Reviewer 1 | 10.1016/j.eswa.2025.130999 | Not cited | Expert Systems with Applications, Crossref year 2026 (DOI year 2025) (Xie): systematic feature selection with fused uncertainty measures for fuzzy coverings. Generic ML method with no groundwater application. |
| Reviewer 1 | 10.1007/s10489-025-06839-x | Not cited | Applied Intelligence, 2025 (Li): multimodal model with uncertainty calibration for vitiligo stage diagnosis. Medical imaging, off-topic. |
| Reviewer 1 | 10.1080/17538947.2026.2646387 | Not cited | International Journal of Digital Earth, 2026 (Guo): tropospheric-delay correction for time-series InSAR in mountainous regions. Remote sensing; no InSAR component in this study. |
| Reviewer 1 | 10.1016/j.eswa.2026.131177 | Not cited | Expert Systems with Applications, 2026 (Yu): label-interaction-aware multi-label feature selection (LIMFS). This study models one binary label per target, so the method does not apply. |
| Reviewer 1 | 10.1016/j.patcog.2026.113096 | Not cited | Pattern Recognition, 2026 (Li): contextual metric learning for few-shot open-set recognition. Computer-vision classification, off-topic. |
| Reviewer 1 | 10.1016/j.petsci.2026.05.001 | Not cited | Petroleum Science, 2026 (Feng): conditional diffusion framework for well-log imputation (Diffusion-IDEA). Petroleum well-log imputation; the study uses median imputation inside each training fold (Section 2.5). |
| Reviewer 1 | 10.1109/tifs.2025.3626979 | Not cited | IEEE Transactions on Information Forensics and Security, 2025 (Li): detecting DeFi protocol exploits through cross-contract analysis. Blockchain security, off-topic. |
| Reviewer 1 | 10.1002/nag.70125 | Not cited | International Journal for Numerical and Analytical Methods in Geomechanics, 2025 (Zhao): probabilistic analysis of pile-reinforced slopes with anisotropic soil properties. Geotechnical reliability, off-topic. |
| Reviewer 1 | 10.1016/j.est.2025.118038 | Not cited | Journal of Energy Storage, 2025 (Wang): online state-of-charge estimation of supercapacitors with uncertain bias compensation. Energy-storage engineering, off-topic. |
| Reviewer 1 | 10.1038/s41467-026-68798-2 | Not cited | Nature Communications, 2026 (Wang): global meta-analysis of plastic-mulch productivity and sustainability trade-offs. Agronomy, not groundwater-related. |
| Reviewer 3 | https://link.springer.com/article/10.1007/s11053-022-10051-w (DOI 10.1007/s11053-022-10051-w) | Cited (Section 1, paragraph 2; Section 4.5) | Natural Resources Research, 2022 (Abbaszadeh Shahri): uncertainty quantification in groundwater-table modelling by automated predictive deep learning. A groundwater-table uncertainty study, not a spatial-distribution analysis (earlier description corrected). |
| Reviewer 3 | https://www.mdpi.com/1424-8220/24/4/1209 (DOI 10.3390/s24041209) | Not cited | Sensors, 2024 (Abbaszadeh Shahri): automated normalisation of large-scale sensor-based measurement-while-drilling data into a unified database. A data-normalisation method, not dynamic filtering or model updating for time-series groundwater data; the Pru dataset has no time series. Title-level assessment; full text not read. |
| Reviewer 3 | https://link.springer.com/article/10.1007/s00500-026-11232-y (DOI 10.1007/s00500-026-11232-y) | Not cited | Soft Computing, 2026 (Mand): data-normalisation transformation based on a piecewise continuous symmetric tanh-type function. A preprocessing transform, not a dynamic-filtering or updating method; the study standardises predictors within training folds (Section 2.5). Title-level assessment; full text not read. |
| Reviewer 4 | 10.1016/j.icheatmasstransfer.2026.112082 | Cited (Section 1, paragraph 2) | International Communications in Heat and Mass Transfer, 2026 (Abbasi & Madani): data-driven modelling of outlet temperature in fractured granite cores. Data-driven modelling of fractured-reservoir systems. |
| Reviewer 4 | 10.1016/j.pce.2026.104704 | Not verified (not cited; author action) | Physics and Chemistry of the Earth, 2026 (Abbasi): analytical time-dependent thermal model of naturally fractured geothermal reservoirs. Title-level only; full text not read; relevance requires author check. |
| Reviewer 4 | 10.1016/j.advwatres.2026.105393 | Not verified (not cited; author action) | Advances in Water Resources, 2026 (Abbasi): analytical modelling of well-test responses in fractured reservoirs with gravity drainage and matrix-fracture transfer. Title-level only; the title describes well-test and reservoir-hydraulics modelling rather than geothermal heat extraction; full text not read; relevance requires author check. |
