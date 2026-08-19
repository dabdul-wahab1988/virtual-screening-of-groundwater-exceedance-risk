# Response to Reviewers

**Manuscript:** "Leakage-controlled and spatially validated machine learning for groundwater-quality exceedance-risk screening in the semi-arid Pru Basin, Ghana"

We thank the Editor and all four reviewers for their careful reading and constructive comments. We have performed a full end-to-end revision: every methodological change below was implemented in the archived analysis pipeline, all results were regenerated from the raw data into a new reproducibility database (schema version 2.0), and every number cited in the revised manuscript is traceable to that database or to its exports. The revised manuscript is self-contained: a reviewer can judge the validity of the primary methodology from the main text alone, with the Supplementary Material providing depth.

Our response structure follows the reviewer-issue matrix (see REVISION_AUDIT_REPORT.md and REVIEWER_ISSUE_MATRIX.csv): each comment is reproduced verbatim (or abridged where it lists many DOIs), classified (AGREE / CLARIFY / DISAGREE with evidence), and answered with the manuscript location of the change. Section numbers refer to the revised manuscript.

---

# Reviewer 1

**R1-01 (Language).** *"The MS language should be made native, and grammatical errors should be polished."*

**Response (AGREE).** The entire manuscript was proofread and rewritten in native English during the revision. Chemical notation, acronym definitions at first use, and consistent British spelling (e.g., "prioritise", "standardisation") were checked throughout (Phase-10 forensic audit; see REVISION_AUDIT_REPORT.md).

**R1-02 (Abstract with numbers).** *"The abstract should be revised, and interesting findings should be included numerically."*

**Response (AGREE).** The abstract now reports the key numerical findings: pooled out-of-fold PR-AUC of 0.978 (Cl) and 0.992 (TDS) with field-only predictors; spatial PR-AUC of 0.977 (Cl) and 0.964 (TDS full tier); EC-strict TDS PR-AUC 0.976 with well-level bootstrap 95% CI 1.000–1.000 and spatial PR-AUC 0.923; B best PR-AUC 0.813; F 0.636 stratified versus 0.490 spatial; and the Brier skill of the EC-strict TDS model (0.770). An explicit operational recommendation is now stated (Abstract).

**R1-03 (Alphabetical keywords).** *"It is recommended to write keywords alphabetically."*

**Response (AGREE).** Keywords are now ordered alphabetically (Abstract).

**R1-04 (Novelty).** *"The novelty of the work should be highlighted, and it should be stated what literature you have that has not been addressed so far."*

**Response (AGREE).** Section 1 now contains an explicit novelty statement: the integrated validation architecture (target-specific leakage control, tiered field-to-chemistry predictors, repeated nested CV with fully nested cutoff selection, transparent spatial-group validation including single-class folds, well-level decision metrics, prevalence-benchmarked Brier scores, exact TreeSHAP attribution) has not previously been combined in a single small-sample groundwater-screening study. The knowledge gap relative to earlier deterministic hydrochemical classification (Sunkari et al., 2021) and to recent Ghanaian ensemble studies that lacked spatial cross-validation (Ansah-Narh et al., 2026) is stated.

**R1-05 to R1-17 (Suggested references).** *Lists of DOIs for the opening paragraph, knowledge gap, methodology, Discussion and Introduction.*

**Response (CLARIFY).** We verified every suggested DOI against Crossref. All 32 DOIs resolve to registered publications; none is fabricated. We adopted those that are directly relevant to this study and declined those that are off-topic, to keep the reference list scientifically coherent:

- *Included:* Abdipour et al. (2026; drinking-water health-risk assessment, Applied Water Science; cited in Section 1); Zhan et al. (2025; groundwater quality drivers, Nature Food; Section 1); Abbaszadeh Shahri et al. (2022; uncertainty quantification in groundwater deep learning, Natural Resources Research; Sections 1 and 4.4); Yin et al. (2025; physics-informed subsurface imaging, Geophysical Research Letters; Sections 1 and 4.4); Abbasi & Madani (2026; data-driven fractured-reservoir modelling, International Communications in Heat and Mass Transfer; Section 1).
- *Not included (with reasons):* 10.1016/j.engappai.2025.112554 (shield-tunnelling geomechanics), 10.1016/j.compgeo.2026.108039 (levee failure mechanics), 10.1016/j.ijmst.2026.03.015 (mine gas drainage), 10.1007/s10489-025-06839-x (vitiligo imaging), 10.1016/j.patcog.2026.113096 (computer vision), 10.1109/tifs.2025.3626979 (blockchain security), 10.1002/nag.70125 (pile-reinforced slopes), 10.1016/j.est.2025.118038 (supercapacitor state-of-charge), and the water-treatment/adsorption papers (10.1080/03067319.2024.2432583, 10.30955/gnj.005098, 10.1007/s13201-025-02661-5, 10.1016/j.cep.2025.110160, 10.1080/00986445.2024.2445230) are outside the groundwater-quality-modelling scope; several of these (e.g., 10.1016/j.est.2025.118038, 10.1109/tifs.2025.3626979) have no topical connection to environmental modelling. Full verification records (title, authors, journal, year, relevance, decision) are tabulated in REVISION_AUDIT_REPORT.md (Phase 8) and the reviewer-issue matrix.

**R1-07 (Knowledge-gap references).** The ten DOIs for the knowledge gap were verified; the groundwater-relevant subset (Zhan et al., 2025; Yin et al., 2025; Abbaszadeh Shahri et al., 2022; Abbasi & Madani, 2026) is cited in Sections 1 and 4.4; the geomechanics/water-treatment items are declined in the response matrix with reasons.

**R1-08 (Methodology reference).** 10.1016/j.eswa.2025.130999 (three-level feature selection) was verified; it is a generic ML-methodology paper without groundwater content and was not adopted as methodology; the decision is documented in the reviewer-issue matrix.

**R1-09 (Discussion reference).** 10.1007/s10489-025-06839-x was verified (vitiligo stage diagnosis); it is medical imaging ML, off-topic for groundwater screening, and was not cited.

**R1-10 (Discussion reference).** 10.1080/17538947.2026.2646387 was verified (InSAR tropospheric-delay correction); it is remote-sensing methodology with marginal relevance and was not cited.

**R1-11 (Discussion reference).** 10.1016/j.eswa.2026.131177 was verified (multi-label feature selection); its label-interaction focus is only marginally relevant to our single-label-per-target design and it was not cited.

**R1-12 (Discussion reference).** 10.1016/j.patcog.2026.113096 was verified (few-shot open-set recognition in computer vision); it is off-topic and was not cited.

**R1-13 (Discussion reference).** 10.1016/j.petsci.2026.05.001 was verified (well-log imputation); it is petroleum-geophysics ML with marginal relevance and was not cited.

**R1-14 (Discussion reference).** 10.1109/tifs.2025.3626979 was verified (DeFi exploit detection); it is blockchain security, off-topic, and was not cited.

**R1-15 (Discussion reference).** 10.1002/nag.70125 was verified (pile-reinforced slopes); it is geotechnical, off-topic, and was not cited.

**R1-16 (Discussion reference).** 10.1016/j.est.2025.118038 was verified (supercapacitor state-of-charge); it is energy-storage engineering, off-topic, and was not cited.

**R1-06 (Update old references).** *"Some of the references are very old, dating back over 20 years. Please update them."*

**Response (CLARIFY).** We audited the reference list. Foundational methodological references (Breiman 2001; Friedman 2001; Brier 1950; Davis & Goadrich 2006; Varma & Simon 2006; Helsel 2012; Hornung & Reed 1990; Roberts et al. 2017; WHO 2022) are retained because they are the primary sources of the methods we use; replacing them with secondary citations would be scientifically improper. Recent literature (2019–2026) has been added where directly relevant (see R1-05), including four 2025–2026 references in the Introduction and Discussion.

---

# Reviewer 2

**R2-01 (Operational recommendation in abstract).** *"...could be strengthened by explicitly stating the final recommendation for operational use..."*

**Response (AGREE).** The Abstract now ends with an explicit recommendation: "We recommend the EC-strict TDS model and the full-chemistry random-forest model for B as the primary screening tools, with target-specific cutoffs applied as described in Table 5."

**R2-02 (TDS threshold justification).** *"...the TDS threshold of 750 mg/L ... should be more prominently stated as a study-specific screening choice rather than a regulatory limit..."*

**Response (AGREE).** Section 2.2 now states prominently that all thresholds are study-defined precautionary screening cut-offs (75% of WHO/Ghana guideline values), that the TDS threshold is an acceptability-based dissolved-solids alert, and that none of the thresholds is an updated regulatory compliance limit. The B threshold is explicitly flagged as configuration-inherited and not a WHO value (the current WHO boron guideline is 2.4 mg/L). Supplementary Table S2 documents every threshold source.

**R2-03 (Fourth tier excluding coordinates).** *"...a fourth tier that also excludes coordinates to truly test if spatial information alone drives performance..."*

**Response (AGREE).** We implemented coordinate-excluded counterparts of all three tiers for all targets and CV modes (coordinate ablation). Results (Section 3.8; Supplementary Table S10, Figure S4): removing longitude/latitude changed pooled PR-AUC only modestly and sometimes improved it (e.g., Na Tier 2: 0.960 without coordinates versus 0.958 with; B Tier 3: 0.818 versus 0.813), so explicit coordinates do not drive discrimination.

**R2-04 (Spatial groups may split hydrologically connected wells).** *"...could the resulting groups inadvertently separate wells that are hydrologically connected along a flow path, potentially overestimating the spatial penalty?"*

**Response (AGREE).** Section 2.6 and Section 4.4 now state explicitly that spatial clusters are statistical validation partitions, not hydrogeological flow-path units, and that blocked evaluation is a conservative transferability test that may overstate the penalty if hydrologically connected wells are split across blocks.

**R2-05 (Emphasise B/F k-sensitivity spread).** *"...the substantial spread for B and F (0.14–0.27) ... should be emphasized more strongly in the discussion."*

**Response (AGREE).** Sections 3.5 and 4.3 now report the k-sensitivity ranges explicitly (B: 0.76 to 0.86; F: 0.47 to 0.58) and interpret them as evidence of spatial instability for these targets (Supplementary Table S9).

**R2-06 (EC-strict TDS: which predictors drive performance).** *"...could you elaborate on what other predictors (e.g., Na, Cl, HCO3) are driving this performance..."*

**Response (AGREE).** Section 3.9 and the Discussion now report the principal non-EC contributors of the EC-strict TDS model from the attribution outputs (Na, Cl, HCO₃ and SO₄), with the practical implication that these analytes carry the laboratory information when conductivity readings are unavailable.

**R2-07 (Validate Saabas attribution with exact Shapley for a subset).** *"...have you considered validating these attributions with a more exact Shapley value approximation for a subset of models to ensure consistency?"*

**Response (AGREE).** Rather than validating an approximation, we implemented the exact method and replaced the approximation entirely. Exact interventional TreeSHAP (Lundberg et al., 2018) is now computed for every tree model, with the training-fold data as the background distribution; linear-model attributions are exact SHAP values (Lundberg & Lee, 2017). The implementation is validated by unit tests against brute-force Shapley enumeration (including trees with a repeated split feature and class-weighted backgrounds), and the stored values satisfy exact additivity (base value + Σ SHAP = model output). All attribution outputs (Figure 6, Supplementary Table S5, database, exports) were regenerated from the exact values, and the terminology is now used correctly throughout.

**R2-08 (Calibration uncertainty source).** *"...does this uncertainty primarily stem from the small sample size, or is it indicative of a more fundamental lack of structure..."*

**Response (AGREE).** Section 3.7 and 4.6 now attribute the bin-level calibration uncertainty for B and F primarily to the small numbers of positive wells in individual probability bins (13 of 81 for F), supported by the fold-level spread (Supplementary Table S4) and bootstrap intervals (Supplementary Table S8); the discrimination results indicate structure is present, so the uncertainty is sample-size-dominated.

**R2-09 (How a water manager uses the cutoffs).** *"...how you envision a water manager actually using the probability cutoffs in Table 5, especially when they differ so widely (e.g., 0.29 for B vs. 0.43 for Cl)."*

**Response (AGREE).** A new decision-support subsection (Section 4.5) and an end-user flowchart (Figure 8) explain the operational procedure: choose target and tier, apply the model, read the well-level median probability, apply the target-specific cutoff, classify priority. The wide cutoff range is explained: low-prevalence targets require lower cutoffs to keep recall high under the sensitivity-first rule, trading specificity for missed-exceedance protection.

**R2-10 (Simpler non-ML nitrate rule).** *"...could you discuss whether a simpler, non-ML approach like a simple rule based on land use or proximity to agricultural areas might be more appropriate for nitrate risk..."*

**Response (CLARIFY).** We agree that a rule-based nitrate risk flag (land use, proximity to agricultural sources) is a reasonable non-ML alternative, and Section 4.1 now says so explicitly. No such land-use covariate layer is available in this dataset, so we could not evaluate it quantitatively; the ML pathway remains inapplicable because only two wells exceed the nitrate threshold.

**R2-11 (BDL sensitivity, alternative MLE).** *"...have you tested the sensitivity of the results, particularly for F, to an alternative method like maximum likelihood estimation?"*

**Response (AGREE).** We ran a BDL substitution sensitivity analysis for F (Section 3.8; Supplementary Table S12): assumed detection limits of 0.2 and 0.5 mg/L and direct substitution of the fallback limit. No target label changes (all substitutions are far below the 1.125 mg/L threshold), and headline pooled PR-AUCs change by ≤ 0.01. A full censored maximum-likelihood model is not justified because only five F records are affected and the labels are invariant to the substitution (Supplementary Methods S2). The BDL wells are spread across three spatial clusters, so they do not concentrate in one held-out block (Section 3.8; this also addresses R2-18).

**R2-12 (Direct comparison with cited studies).** *"...a more direct comparison of your PR-AUC results for Na and Cl (≈1.000) with those from the cited studies..."*

**Response (CLARIFY).** Section 4.4 now includes a cautious quantitative benchmark against comparator studies (Ansah-Narh et al., 2026; Abbaszadeh Shahri et al., 2022), with the explicit caveat that datasets, targets and threshold definitions differ, so head-to-head PR-AUC equality cannot be claimed. (We also note the headline Na/Cl PR-AUC values in the revised analysis are 0.958–0.978 under pooled out-of-fold evaluation.)

**R2-13 (Single simplified end-user flowchart).** *"...the reader would benefit from a single, simplified flowchart ... (Step 1: Choose target; Step 2: Select tier; Step 3: Apply cutoff; Step 4: Map priority)."*

**Response (AGREE).** Figure 8 provides the end-user decision flowchart (seven steps: target → tier → leakage filter → model → cutoff → priority class → confirmatory testing), and Section 4.5 walks through it.

**R2-14 (F2 rationale and optimality).** *"...the exact rationale for using the F2 score ... and why the choice of β=2 is optimal..."*

**Response (AGREE).** Section 2.9 now states that β = 2 is an explicit a priori surveillance preference weighting recall over precision, not a mathematically optimised cost ratio, and that no optimality claim is made. A β-sensitivity analysis (F1, F1.5, F2, F3) is reported (Section 3.7; Supplementary Table S13, Figure S6): target ranking and qualitative conclusions are unchanged across β.

**R2-15 (Would fully nested selection be overly conservative?).** *"...would a fully nested selection procedure (within each outer fold) have led to overly conservative and less generalizable cutoffs?"**

**Response (AGREE, with evidence).** We implemented the fully nested procedure (cutoffs chosen on inner-CV validation predictions inside each outer fold, frozen, applied to held-out data). Section 3.7 and Supplementary Figure S7 compare nested pooled operating characteristics with the previous post-hoc pooled cutoffs: nested estimates are more conservative (as expected), e.g., B Tier 3 nested specificity 0.426 versus 0.760 post-hoc at matched sensitivity ≈ 0.89–0.91, and slightly less "optimised" per fold — but they are unbiased by construction and are the values we recommend operationally. The post-hoc results are retained as a labelled comparison.

**R2-16 (Na Tier 2 vs Tier 3).** *"...does this suggest that adding more predictors introduces noise ... or is it a random effect of the small sample?"**

**Response (AGREE).** Section 3.6 now states that the small Tier-2-versus-Tier-3 differences for Na are consistent with small-sample variation and are not interpreted as evidence that additional predictors introduce noise.

**R2-17 (How priority classes were defined).** *"...I'd like to see a brief description of how the 'low, moderate, high' screening priority classes were defined..."*

**Response (AGREE).** Section 2.10 and the caption of Figure 7 now document the exact pre-specified probability bands applied to well-level median out-of-fold probabilities: Low < 0.25, Moderate 0.25–0.50, High 0.50–0.75, Very high ≥ 0.75. The maps show four bands; the previous figure silently omitted the Very-high class (a figure bug identified in the forensic audit), which is corrected in the revised Figure 7.

**R2-18 (F BDL wells geographically clustered?).** *"...could this spatial fragility be due to the fact that the five below-detection-limit wells are geographically clustered..."*

**Response (AGREE, with evidence).** Section 3.8 reports the spatial distribution of the five F BDL wells: they fall in three different spatial clusters (0, 1 and 2), so they are not geographically concentrated in one held-out block. Combined with the label-invariant substitution analysis (R2-11), the F spatial fragility is not an artefact of BDL handling.

**R2-19 (NO3 descriptive stats and spatial distribution).** *"...it would be beneficial to report its descriptive statistics and spatial distribution..."*

**Response (AGREE).** Section 3.1 now reports NO₃ descriptive statistics (range and median), and Supplementary Figure S5 shows the NO₃ histogram with the screening threshold and a map of NO₃ concentrations with BDL wells marked.

**R2-20 (How to interpret the Brier score).** *"...what would be considered a 'good' Brier score for a screening tool of this nature..."**

**Response (AGREE).** Section 2.9 and Section 3.7 now explain that raw Brier scores are prevalence-dependent and have no universal cutoff; we report a prevalence-only null Brier (constant training-prevalence prediction) and the Brier skill score 1 − Brier/Brier_null for every headline model (Table 3). All headline models have positive skill (e.g., 0.770 for the EC-strict TDS model), meaning they beat the "always predict prevalence" baseline.

**R2-21 (Explicit availability of the SQLite database).** *"...whether the exact SQLite reproducibility database is also publicly accessible or only available upon request."*

**Response (AGREE).** The Data and Code Availability statement now states explicitly that the SQLite reproducibility database (schema version 2.0) is archived and publicly accessible in full alongside the code and manuscript at https://doi.org/10.5281/zenodo.21442254, not merely available on request.

**R2-22 (Lumped-parameter model as predictor).** *"...a simple lumped-parameter model of flow and transport could be integrated directly as a predictor..."**

**Response (CLARIFY).** Section 4.7 addresses this directly: a lumped-parameter flow/transport predictor is scientifically attractive, especially for F, but requires hydrogeological parameterisation (recharge, transmissivity, travel times) that is not available for this dataset; we frame it as a data-collection-motivating future direction rather than an immediately integrable predictor.

---

# Reviewer 3

**Preliminary note.** Several of Reviewer 3's claims are contradicted by the submitted work. Rather than accepting them at face value, we verified each against the repository and regenerated outputs, per our revision protocol. Where a claim is contradicted, we respond with evidence and improved visibility rather than redesigning the study.

**R3-01 (Self-citation in model set sentence).** *"'The model set comprised baseline, interpretable and nonlinear classifiers (Abdul-Wahab and Sunkari, 2026).'????? That is enough to REJECT this submission."*

**Response (AGREE).** The model-set sentence now cites the primary methodological literature (Breiman, 2001; Friedman, 2001). The software record (Abdul-Wahab & Sunkari, 2026) is cited only in the Data and Code Availability statement, where it belongs.

**R3-02 ("no replication evidence ... optimality approval, curve guarantees, error monitoring, early convergence, trapping").** *[Verbatim.]*

**Response (DISAGREE, with evidence).** The claim of no replication evidence is contradicted by the submission: repeated stratified nested CV (5 repeats × 5 folds = 25 held-out folds per classifier), deterministic seeds, dummy baselines, bootstrap confidence intervals (2,000 well-level resamples), calibration diagnostics and threshold sweeps are all present and now highly visible in the revised main text (Sections 2.8–2.9, 3.4, 3.7). The demands for "optimality approval, curve guarantees, error monitoring, early convergence, trapping" describe the engineering guarantees of optimisation libraries for large-scale learning; they have no meaningful counterpart for the small-sample, fully deterministic estimators evaluated here (fixed-iteration gradient descent with verified convergence behaviour on a ≤ 81-sample problem, shallow trees, bounded grids). We note that our native implementations are deterministic and archived, so every result is exactly reproducible (verified: the pipeline reproduces the archived database run-for-run).

**R3-03 ("No solid comparison with advanced approaches ... only relies on confusion matrix ... lacks supportive metrics ...").** *[Abridged.]*

**Response (DISAGREE, with evidence).** The submitted work does not rely on confusion matrices: it reports ROC-AUC, PR-AUC, recall, specificity, F1/F1.5/F2/F3, balanced accuracy, Brier score with null baseline and skill score, calibration slope/intercept, fold-level mean ± SD, bootstrap CIs, spatial CV with pooled estimates, threshold sweeps, and technical limitations (Section 2.9, 3.4–3.7; Tables 3–5; Supplementary Tables S4, S8, S14–S16). The cited spatial-distribution paper (Abbaszadeh Shahri et al., 2022, Natural Resources Research) is now included and discussed as a comparator (Section 4.4). Visibility of the metrics inventory is improved in the revised main text.

**R3-04 (Temporal variability / dynamic filtering / updating).** *"What about temporal variability of groundwater quality??? ... you need an updating procedure and dynamic filtering ..."*

**Response (DISAGREE, with a scope statement).** The dataset is cross-sectional (a single dry-season campaign; no repeated sampling), so temporal modelling, dynamic filtering or model updating cannot be conducted or claimed. Sections 2.1, 4.6 and 5 now state this explicitly: the framework makes no temporal-transferability claim, and any longitudinal deployment would require model updating and recalibration. Robustness to noise and missing data is discussed in Section 2.5 (BDL and missing-value handling), Section 3.8 (BDL sensitivity) and Section 4.6. Adding dynamic filtering without longitudinal data would be unsupported (Gate 12).

**R3-05 ("Where is the claimed 'Leakage-controlled and spatially'???? This work basically doesn't have any spatial analysis and mapping.").** *[Verbatim.]*

**Response (DISAGREE, with evidence).** The claim is contradicted by the submission: spatial clustering (Section 2.6), spatial-group cross-validation with per-fold transparency and pooled evaluation (Sections 2.8, 3.5; Supplementary Table S15), a spatial-penalty analysis (Figure 4), cluster metadata (Supplementary Table S6), k-sensitivity (Supplementary Table S9) and GIS-ready screening-priority maps (Figure 7) are all present. The revised main text makes these elements unmissable (Methods sections 2.6/2.8 and Results section 3.5).

**R3-06 (English revision; undefined acronyms).** *"English requires substantial revision / many undefined acronyms and parameters at first appearance."*

**Response (AGREE).** Full proofreading was performed and every acronym is defined at first use (e.g., EC, TDS, BDL, PR-AUC, ROC-AUC, CV, OOF, SAR, TP/TN/FP/FN, WHO, GS).

**R3-07 (Improper citations; Sunkari et al. 2021 "obviously is REJECTED").** *"Trivial improper citations. Just as an example, the claimed citation to Sunkari et al., 2021 (L54, L57) or … obviously is REJECTED and it has been recognized much earlier than 2021."*

**Response (CLARIFY).** We verified the Sunkari, Abu & Zango (2021) reference against the published record: Journal of Contaminant Hydrology, Volume 236, Article 103742, https://doi.org/10.1016/j.jconhyd.2020.103742 (published 2021; the DOI year 2020 reflects online-first assignment). It is the source study of the Pru Basin dataset and is cited as such. The claim that this citation is improper is not supported by the published record; if the reviewer refers to a different work, we would be glad to check it. All citations were re-verified in the revision (Phase 8 audit; REVISION_AUDIT_REPORT.md).

**R3-08 (No embedded figures/tables in context).** *"No embedded Fig/Tables/captions within the context."*

**Response (AGREE).** The revised manuscript embeds all figures and tables with captions, in-text citations and logical numbering (Figures 1–8; Tables 1–5); the Phase-10 audit verified citation order and existence.

**R3-09 (Opacity of methods).** *"Full opacity in terms of datasets, diversity, … randomization, unifications, time interval span, … opacity in binary risk-alert thresholds, spatial grouping strategy for cross-validation, spatial autocorrelation, selection of predictor variables, robustness to class imbalance, external or independent validation, …."*

**Response (AGREE).** The main text now contains everything needed to judge the methodology: dataset description and completeness (Table 1), thresholds and their scientific status (Section 2.2; Table 1), predictor selection and tiers (Section 2.3), leakage rules (Section 2.4), randomisation (deterministic seeds; Section 2.10), time-span statement (cross-sectional; Sections 2.1 and 4.6), spatial grouping (Section 2.6), spatial-autocorrelation treatment via spatial CV (Section 2.8), class-imbalance handling (Section 2.5), external-validation status (Sections 4.6), and full reproducibility records (Section 2.10).

**R3-10 ("More suitable for screening than definitive contamination prediction").** *[Verbatim.]*

**Response (AGREE).** This is precisely the study's framing. The revised text maintains the distinction "screening probability ≠ contamination status" throughout (Sections 2.10, 3.9, 4.5, 5; Figure 7 caption).

**R3-11 (Unjustified conclusion).** *[Verbatim.]*

**Response (AGREE).** Conclusions were rewritten to be strictly proportional to the evidence, with explicit uncertainty (fold spread, bootstrap CIs, k-sensitivity ranges, calibration limitations) stated in Sections 3.4–3.7 and reflected in Section 5 (e.g., "showed the strongest evidence of screenability within this dataset" rather than unconditional superiority claims).

---

# Reviewer 4

**R4-01 (Threshold justification + sensitivity at guideline values).** *"...provide a sensitivity analysis using current drinking-water guideline values."*

**Response (AGREE).** Section 2.2 strengthens the threshold rationale (75% of WHO/Ghana guideline values as precautionary early-warning cut-offs; B explicitly configuration-inherited). Section 3.8 and Supplementary Table S11 report a full relabelling at current WHO guideline values: Na (200 mg/L; 27 positives), Cl (250; 12), TDS (1000; 8) retained sufficient prevalence and modelling; discrimination was retained or improved (Cl pooled PR-AUC 0.986 at 250 mg/L); F (3 positives at 1.5 mg/L) and B (3 positives at 2.4 mg/L) fell below the modelling threshold and are reported descriptively — showing that the precautionary thresholds are what make B and F modellable at all in this dataset, and that screenability rankings are not inflated by the study thresholds.

**R4-02 (Overfitting risk; sample-to-predictor ratio).** *"...Please further discuss the risk of overfitting and the adequacy of the sample-to-predictor ratio, particularly for B and F."*

**Response (AGREE).** A small-sample/dimensionality audit is now reported (Section 4.6; Supplementary Table S14): per-target/tier training sizes, positive counts per fold, predictor counts, training-per-predictor ratios, fold-level PR-AUC spread and chosen hyperparameters. The random-forest tuning grid was restricted to max_depth ≤ 4 for the small-sample setting (Section 2.7). For F Tier 3, folds can contain as few as 6 positive training cases with 20 predictors; the Discussion states that point estimates for B and F must be read with the reported uncertainty.

**R4-03 (Spatial clustering implementation).** *"...please describe the exact implementation and justify why this approach is appropriate for geographic coordinates."*

**Response (AGREE).** Section 2.6 now describes the exact implementation and its justification: coordinates are projected onto a local equirectangular plane (km) centred on the basin centroid, and Euclidean k-means with deterministic k-means++ initialisation is applied; the coordinate-mean centroid is the true least-squares centre on the projected plane. We identified that the previous description ("k-means on Haversine distances with coordinate-mean centroids") was mathematically incoherent (mean lat/lon is not the least-squares centre under great-circle distance) and replaced it; cluster metadata now record the method and projection (Supplementary Table S6). The number of clusters is chosen by the documented inertia-elbow rule; sensitivity to k is reported (Supplementary Table S9).

**R4-04 (Single-class spatial folds; bias in transferability estimates).** *"...Please explain how exclusion of spatial folds lacking both classes may influence the reported spatial PR-AUC..."**

**Response (AGREE).** Spatial folds are no longer excluded anywhere. Every fold is reported with n, positives, negatives and status (Supplementary Table S15); fold-level metrics are reported only where mathematically defined; pooled spatial PR-AUC is computed across all unique held-out wells (the headline estimate) and separately across binary-valid folds only, so the effect of single-class-fold handling is quantified (Section 2.6, 3.5). Under the main k = 4 partition all five targets have both classes in every block, so the two pooled estimates coincide; single-class blocks arise at finer partitions (e.g., TDS at k = 7) and are reported rather than discarded.

**R4-05 (Proxy-driven discrimination).** *"...The possibility of proxy-driven discrimination should be discussed more explicitly."**

**Response (AGREE).** Section 4.2 and the EC-ablation results (Section 3.8; Supplementary Table S10) discuss proxy-driven discrimination explicitly: EC is a legitimate operational proxy, not circular target reconstruction, because EC-strict models retain most of the screenability (TDS 0.976, Na 0.913, Cl 0.847 pooled PR-AUC with EC excluded). The Discussion names the exact non-EC predictors that carry the remaining signal (Na, Cl, HCO₃, SO₄).

**R4-06 (Fully nested threshold selection).** *"Operational probability thresholds were selected from pooled out-of-fold predictions ... A fully nested threshold-selection strategy would provide a less optimistic estimate..."**

**Response (AGREE).** Fully nested threshold selection is implemented (Section 2.8): cutoff chosen on inner-CV validation predictions inside each outer fold, frozen, applied to held-out test data; per-fold results archived (Supplementary Table S16); nested pooled operating estimates reported and compared with the post-hoc pooled approach (Section 3.7; Supplementary Figure S7), which is retained only as a labelled comparison. As predicted by the reviewer, the nested estimates are more conservative (e.g., B Tier 3: nested pooled specificity 0.426 vs post-hoc 0.760 at matched sensitivity).

**R4-07 (Updated literature).** *"...consider relevant recent works such as 10.1016/j.icheatmasstransfer.2026.112082, 10.1016/j.pce.2026.104704, and 10.1016/j.advwatres.2026.105393..."**

**Response (CLARIFY).** All three DOIs were verified against Crossref (all real; Abbasi et al., 2026, International Communications in Heat and Mass Transfer / Physics and Chemistry of the Earth / Advances in Water Resources). The most relevant (10.1016/j.icheatmasstransfer.2026.112082, data-driven fractured-reservoir modelling) is cited in Section 1. The other two are analytical geothermal/reservoir-model studies; we did not cite them because their analytical scope does not add to the screening methodology, and we prefer a coherent reference list over citation accumulation.

**R4-08 (SHAP vs Saabas terminology).** *"...these methods are not equivalent; terminology should be corrected or exact TreeSHAP values should be calculated."**

**Response (AGREE).** Exact interventional TreeSHAP (Lundberg et al., 2018) is now calculated for every tree model with the training-fold data as the background distribution, and linear-model attributions are exact SHAP values (Lundberg & Lee, 2017). The implementation is validated by unit tests against brute-force Shapley enumeration and exact additivity checks, and the term SHAP is now used only for these genuine SHAP values — in the manuscript, Supplementary Methods S10, Figure 6, Supplementary Table S5, the database schema (`shap_values`, `fold_base_values`) and exports (`shap_values_long.csv`), all regenerated (Gate 5).

**R4-09 (FP/FN counts under repeated CV).** *"...whether repeated predictions from the same well contributed more than once to these counts."**

**Response (AGREE).** The previous table counts were prediction-instance counts (a well contributed once per repeat). The revised Table 5 presents well-level consolidated metrics: median out-of-fold probability per well (pre-specified rule), each well counted exactly once, with unique-well TP/TN/FP/FN at the well-level sensitivity-first cutoff; pooled prediction-instance counts are retained only in clearly labelled supplementary columns (Section 2.10, 3.7). Unique-well counts can never exceed the 81-well dataset (Gate 8).

**R4-10 (Results/Discussion repetition).** *"...The Results and Discussion sections contain substantial repetition... These sections could be shortened and made more analytical..."**

**Response (AGREE).** The Results section now reports once, with the Discussion providing interpretation and cross-referencing (e.g., the screenability of Na/Cl/TDS and the F spatial penalty are stated once in Results 3.4–3.5 and interpreted in Discussion 4.1–4.3, without repeating the numerical descriptions).

---

# Editor-level summary of the main scientific changes

1. **Fully nested operational-threshold selection** replaces post-hoc pooled cutoff selection (R2-15, R4-06; instructions Phase 3A).
2. **Spatial validation redesign**: all folds reported; pooled spatial metrics across unique held-out wells; binary-valid-only comparison; no fold discarded (R4-04; Phase 3B).
3. **Spatial clustering corrected**: projected equirectangular Euclidean k-means replaces the incoherent Haversine/coordinate-mean variant; method documented; k-sensitivity retained (R4-03; Phase 3C).
4. **Coordinate ablation** for all tiers (R2-03; Phase 3D).
5. **Concentration-threshold sensitivity** at current WHO guideline values (R4-01; Phase 3E).
6. **Small-sample audit** with per-fold diagnostics and restricted tree depth (R4-02; Phase 3F).
7. **Exact interventional TreeSHAP implemented** for all tree models (validated against brute-force Shapley; exact additivity), with genuine SHAP terminology restored for trees and exact linear SHAP for linear models (R2-07, R4-08; Phase 3G).
8. **BDL sensitivity** for F with label invariance demonstrated (R2-11, R2-18; Phase 3H).
9. **F₂ rationale + β sensitivity** (F1, F1.5, F2, F3) (R2-14; Phase 3I).
10. **Brier null baseline and skill score** (R2-20; Phase 3J).
11. **Well-level decision metrics** with unique-well TP/TN/FP/FN (R4-09; Phase 3K).
12. **Screening-priority bands documented** (and the Very-high class restored to the map) (R2-17; Phase 3L).
13. **EC ablation for Na and Cl** added; TDS EC-strict contributors reported (R2-06, R4-05; Phase 3M).
14. **Reference list verified and updated**; off-topic suggestions declined with reasons (R1-05…R1-17, R4-07; Phase 8).
15. **Claim discipline** throughout: screening probability ≠ contamination status; no temporal or external-validation claims (R3-04, R3-10, R3-11; Phase 6).

Every number in the revised manuscript was regenerated from the revised pipeline and is traceable to the archived database or exports; the reproducibility audit (Phase 1) confirmed that the previously submitted headline numbers were correctly reproduced from the original code and data before the methodological revision.


