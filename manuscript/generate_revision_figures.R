# =============================================================================
# generate_revision_figures.R
# Regenerates ALL publication figures for the revised manuscript from the
# revised reproducibility database (outputs/rev2/main/main.db).
#
# Terminology: exact interventional TreeSHAP values (Lundberg et al., 2018)
# for tree models; exact linear SHAP values for linear models.
# Screening priority uses the four documented probability bands
# (Low <0.25, Moderate 0.25-0.50, High 0.50-0.75, Very_High >=0.75).
# Spatial headline metrics are POOLED across all unique held-out wells
# (pooled_metrics scope = 'all_held_out').
# =============================================================================

suppressPackageStartupMessages({
  library(RSQLite); library(DBI); library(ggplot2); library(dplyr)
  library(tidyr); library(patchwork); library(RColorBrewer)
  library(viridis); library(scales); library(ggrepel); library(grid)
})

PROJECT_ROOT <- normalizePath(".")   # run with workdir = repository root
DB_PATH <- file.path(PROJECT_ROOT, "outputs/rev2/main/main.db")
FIG_DIR <- file.path(PROJECT_ROOT, "outputs/rev2/figures")
dir.create(FIG_DIR, recursive = TRUE, showWarnings = FALSE)
con <- dbConnect(SQLite(), DB_PATH)

TARGET_ORDER <- c("Na", "Cl", "TDS", "B", "F", "NO3")
MODEL_ORDER  <- c("EcOnlyLogistic", "LogisticRegression", "RandomForest", "GradientBoostedTrees")
TIER_ORDER   <- c("Tier1_Field", "Tier2_Reduced", "Tier3_Full")

# ── Plotmath label maps (ionic notation) ────────────────────────────────────
ION_PM <- c(
  "Na"  = "Na^'+'", "Cl" = "Cl^'-'", "TDS" = "TDS", "B" = "B",
  "F"   = "F^'-'", "NO3" = "NO[3]^'-'"
)
FEAT_PM <- c(
  "EC" = "EC", "B" = "B", "Na" = "Na^'+'", "K" = "K^'+'",
  "Mg" = "Mg^{'2+'}", "Ca" = "Ca^{'2+'}", "Cl" = "Cl^'-'",
  "SO4" = "SO[4]^{'2-'}", "HCO3" = "HCO[3]^'-'", "CO3" = "CO[3]^{'2-'}",
  "NO3" = "NO[3]^'-'", "F" = "F^'-'",
  "Na_Cl_ratio" = "Na^'+'/Cl^'-'", "HCO3_Cl_ratio" = "HCO[3]^'-'/Cl^'-'",
  "Mg_Ca_ratio" = "Mg^{'2+'}/Ca^{'2+'}", "Ca_Mg_ratio" = "Ca^{'2+'}/Mg^{'2+'}",
  "SAR" = "SAR", "Temp." = "Temp.", "pH" = "pH", "Dy" = "Lat.", "Dx" = "Lon.",
  "HCO3/Cl" = "HCO[3]^'-'/Cl^'-'", "Na/Cl" = "Na^'+'/Cl^'-'",
  "Ca/Mg" = "Ca^{'2+'}/Mg^{'2+'}", "Mg/Ca" = "Mg^{'2+'}/Ca^{'2+'}"
)
pm_scale <- function(map) function(x) ifelse(x %in% names(map), map[x], x)
pm_labeller <- function(map) as_labeller(function(x) ifelse(x %in% names(map), map[x], x), label_parsed)

nature_theme <- function(base_size = 11) {
  theme_classic(base_size = base_size) +
    theme(
      plot.background   = element_rect(fill = "white", colour = NA),
      panel.background  = element_rect(fill = "white", colour = NA),
      panel.border      = element_rect(colour = "black", fill = NA, linewidth = 0.8),
      axis.text         = element_text(size = 14, colour = "black"),
      axis.title        = element_text(size = 16, colour = "black"),
      legend.text       = element_text(size = 14),
      legend.title      = element_text(size = 14, face = "bold"),
      legend.background = element_rect(fill = "white", colour = "grey80"),
      legend.key        = element_rect(fill = "white"),
      strip.text        = element_text(size = 13, face = "bold"),
      strip.background  = element_rect(fill = "grey92", colour = "black", linewidth = 0.6),
      axis.ticks        = element_line(colour = "black", linewidth = 0.5)
    )
}
save_fig <- function(p, name, width = 8, height = 6) {
  path <- file.path(FIG_DIR, paste0(name, ".png"))
  ggsave(path, plot = p, width = width, height = height, dpi = 300,
         bg = "white", units = "in")
  message("  Saved: ", basename(path))
}

cat("=== Generating revised figures ===\n\n")

# =============================================================================
# FIGURE 1 — Workflow schematic (static, updated for the revised design)
# =============================================================================
cat("Figure 1...\n")
box_df <- data.frame(
  x = c(1, 3, 5, 7, 9, 11, 13, 15),
  y = c(6, 6, 6, 6, 6, 6, 6, 6),
  label = c(
    "81 georeferenced\nwell samples\n(Sunkari et al., 2021)",
    "Six binary\nprecautionary\nscreening labels\n(study-specific thresholds)",
    "Target-wise\nleakage filtering\n(Table 2)",
    "Projected\nk-means spatial\nclusters (k = 4)\n(equirectangular)",
    "Repeated nested CV\n(5 repeats x 5 folds)\n+ inner CV tuning",
    "Spatial-group CV\n(3 held-out blocks)\n+ pooled evaluation",
    "Path-based feature\nattribution\n(Saabas-style)",
    "Well-level median\nOOF probabilities\n+ screening-priority\nclasses"
  ),
  stringsAsFactors = FALSE
)
p1 <- ggplot(box_df, aes(x = x, y = y)) +
  geom_label(aes(label = label), size = 3.4, fill = "grey95", colour = "black",
             label.size = 0.6, lineheight = 0.9, label.padding = unit(0.55, "lines")) +
  geom_segment(aes(x = x + 1.1, xend = x + 1.9, y = y, yend = y),
               arrow = arrow(length = unit(0.18, "cm"), type = "closed"),
               linewidth = 0.7, colour = "grey30") +
  coord_cartesian(xlim = c(0, 17), ylim = c(3.5, 8.5)) +
  labs(x = NULL, y = NULL) +
  theme_void() +
  theme(plot.margin = margin(10, 10, 10, 10))
save_fig(p1, "Figure1_workflow_schematic", width = 16, height = 5)

# =============================================================================
# FIGURE 2 — Exceedance prevalence
# =============================================================================
cat("Figure 2...\n")
elig <- dbGetQuery(con, "SELECT * FROM target_eligibility")
tdef <- dbGetQuery(con, "SELECT target_contaminant, threshold_value FROM target_definitions")
elig <- elig %>%
  left_join(tdef, by = "target_contaminant") %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    pct_positive       = round(prevalence * 100, 1),
    threshold_label    = paste0(format(threshold_value, trim = TRUE), " mg/L")
  )
elig_long <- elig %>%
  select(target_contaminant, n_positive, n_negative, threshold_label) %>%
  pivot_longer(cols = c(n_positive, n_negative),
               names_to = "class", values_to = "count") %>%
  mutate(class = ifelse(class == "n_positive", "Exceeds threshold", "Below threshold"))

p2 <- ggplot(elig_long, aes(x = target_contaminant, y = count, fill = class)) +
  geom_col(width = 0.65, colour = "white", linewidth = 0.3) +
  geom_text(data = elig,
            aes(x = target_contaminant, y = 81 + 2.5, label = paste0(pct_positive, "%")),
            inherit.aes = FALSE, size = 4.8, fontface = "bold", colour = "grey20") +
  geom_text(data = elig,
            aes(x = target_contaminant, y = -4, label = threshold_label),
            inherit.aes = FALSE, size = 3.8, colour = "grey30", fontface = "italic") +
  annotate("text", x = 0.4, y = -4, label = "Threshold:",
           size = 3.8, colour = "grey30", fontface = "bold", hjust = 1) +
  scale_fill_manual(values = c("Exceeds threshold" = "#d62728",
                               "Below threshold" = "#aec7e8"), name = NULL) +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  scale_y_continuous(limits = c(-7, 90), breaks = c(0, 20, 40, 60, 80), expand = c(0, 0)) +
  labs(x = "Groundwater quality parameter", y = "Number of wells (n = 81)") +
  nature_theme() +
  theme(legend.position = "right", axis.text.x = element_text(size = 14, face = "bold"),
        axis.ticks.x = element_blank())
save_fig(p2, "Figure2_exceedance_prevalence", width = 9, height = 5.5)

# =============================================================================
# FIGURE 3 — PR-AUC performance heatmap (pooled OOF, canonical tiers)
# =============================================================================
cat("Figure 3...\n")
perf <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm,
         pm.pr_auc
  FROM pooled_metrics pm
  JOIN model_runs mr ON pm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
    AND mr.labeling_variant = 'main'
    AND pm.cv_mode = 'Stratified_Nested_CV'
    AND pm.scope = 'all_held_out'
")
perf <- perf %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    predictor_tier = factor(predictor_tier, levels = TIER_ORDER,
                            labels = c("Tier 1\n(Field)", "Tier 2\n(Reduced)", "Tier 3\n(Full)")),
    algorithm = factor(algorithm, levels = MODEL_ORDER,
                       labels = c("EC-only\nLogistic", "Logistic\nReg.", "Random\nForest", "Grad.\nBoosted"))
  )
p3 <- ggplot(perf, aes(x = algorithm, y = target_contaminant, fill = pr_auc)) +
  geom_tile(colour = "white", linewidth = 0.5) +
  geom_text(aes(label = sprintf("%.2f", pr_auc)), size = 4,
            colour = "white", fontface = "bold") +
  scale_fill_gradientn(colours = c("#2c3e50", "#2980b9", "#27ae60", "#f1c40f", "#e74c3c"),
                       limits = c(0, 1), name = "PR-AUC",
                       guide = guide_colourbar(barwidth = 1, barheight = 8,
                                               title.position = "top", title.hjust = 0.5)) +
  scale_y_discrete(labels = pm_scale(ION_PM)) +
  facet_wrap(~ predictor_tier, nrow = 1) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x = element_text(size = 11),
        axis.text.y = element_text(size = 14, face = "bold"),
        legend.position = "right",
        strip.text = element_text(size = 12, face = "bold"),
        panel.border = element_rect(colour = "grey60", linewidth = 0.5))
save_fig(p3, "Figure3_performance_heatmap_PR_AUC", width = 11, height = 5)

# =============================================================================
# FIGURE 4 — Spatial validation penalty (pooled metrics, both CV modes)
# =============================================================================
cat("Figure 4...\n")
perf_both <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, pm.cv_mode,
         pm.pr_auc
  FROM pooled_metrics pm
  JOIN model_runs mr ON pm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.algorithm != 'EcOnlyLogistic'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
    AND mr.labeling_variant = 'main'
    AND pm.scope = 'all_held_out'
")
perf_wide <- perf_both %>%
  pivot_wider(names_from = cv_mode, values_from = pr_auc) %>%
  rename(pr_auc_strat = Stratified_Nested_CV, pr_auc_spatial = Spatial_Group_CV) %>%
  filter(!is.na(pr_auc_strat) & !is.na(pr_auc_spatial)) %>%
  mutate(
    penalty = pr_auc_strat - pr_auc_spatial,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm = factor(algorithm,
                       levels = c("LogisticRegression", "RandomForest", "GradientBoostedTrees"),
                       labels = c("Logistic Reg.", "Random Forest", "Grad. Boosted"))
  )
pA <- ggplot(perf_wide, aes(x = pr_auc_strat, y = pr_auc_spatial,
                            colour = target_contaminant, shape = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_point(size = 3.5, alpha = 0.85, stroke = 0.5) +
  scale_colour_brewer(palette = "Set1", name = "Target", labels = pm_scale(ION_PM)) +
  scale_shape_manual(values = c(16, 17, 15), name = "Algorithm") +
  scale_x_continuous(limits = c(0, 1.02), breaks = c(0, 0.25, 0.5, 0.75, 1.0)) +
  scale_y_continuous(limits = c(0, 1.02), breaks = c(0, 0.25, 0.5, 0.75, 1.0)) +
  labs(x = "PR-AUC (repeated stratified CV, pooled OOF)",
       y = "PR-AUC (spatial-group CV, pooled OOF)") +
  annotate("text", x = 0.85, y = 0.06, label = "Spatial CV\npenalised",
           size = 3.8, colour = "grey40", hjust = 0.5) +
  nature_theme() +
  theme(legend.position = "right")
pen_summary <- perf_wide %>%
  group_by(target_contaminant, predictor_tier) %>%
  summarise(mean_penalty = mean(penalty, na.rm = TRUE),
            se_penalty = sd(penalty, na.rm = TRUE) / sqrt(n()), .groups = "drop") %>%
  mutate(tier_label = case_when(predictor_tier == "Tier1_Field" ~ "T1",
                                predictor_tier == "Tier2_Reduced" ~ "T2",
                                predictor_tier == "Tier3_Full" ~ "T3"))
pB <- ggplot(pen_summary, aes(x = target_contaminant, y = mean_penalty, fill = tier_label)) +
  geom_col(position = position_dodge(0.75), width = 0.65, colour = "white", linewidth = 0.3) +
  geom_errorbar(aes(ymin = mean_penalty - se_penalty, ymax = mean_penalty + se_penalty),
                position = position_dodge(0.75), width = 0.25, linewidth = 0.6) +
  geom_hline(yintercept = 0, linetype = "solid", colour = "grey30", linewidth = 0.6) +
  scale_fill_manual(values = c("T1" = "#3498db", "T2" = "#2ecc71", "T3" = "#e74c3c"),
                    name = "Predictor\ntier") +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  labs(x = "Target parameter",
       y = expression(Delta * "PR-AUC (Stratified " * "\u2212" * " Spatial CV)")) +
  nature_theme() +
  theme(legend.position = c(0.12, 0.18), axis.text.x = element_text(size = 14, face = "bold"))
p4 <- pA + pB + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))
save_fig(p4, "Figure4_spatial_cv_penalty", width = 13, height = 5.5)

# =============================================================================
# FIGURE 5 — Calibration curves
# =============================================================================
cat("Figure 5...\n")
preds_all <- dbGetQuery(con, "
  SELECT wp.predicted_probability, wp.true_label,
         mr.target_contaminant, mr.algorithm
  FROM well_predictions wp
  JOIN model_runs mr ON wp.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm IN ('RandomForest','GradientBoostedTrees','LogisticRegression')
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")
calib_df <- preds_all %>%
  mutate(bin = cut(predicted_probability, breaks = seq(0, 1, length.out = 9),
                   include.lowest = TRUE, labels = FALSE)) %>%
  group_by(target_contaminant, algorithm, bin) %>%
  summarise(mean_pred = mean(predicted_probability),
            obs_frac = mean(true_label), n = n(),
            se = sqrt(obs_frac * (1 - obs_frac) / n), .groups = "drop") %>%
  filter(n >= 3) %>%
  mutate(target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
         algorithm = factor(algorithm,
                            levels = c("LogisticRegression", "RandomForest", "GradientBoostedTrees"),
                            labels = c("Logistic Reg.", "Random Forest", "Grad. Boosted")))
p5 <- ggplot(calib_df, aes(x = mean_pred, y = obs_frac, colour = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_errorbar(aes(ymin = obs_frac - 1.96 * se, ymax = obs_frac + 1.96 * se),
                width = 0.04, alpha = 0.5, linewidth = 0.6) +
  geom_line(linewidth = 1.6, alpha = 0.85) +
  geom_point(size = 3.5, alpha = 0.9) +
  scale_colour_manual(values = c("Logistic Reg." = "#1f77b4",
                                 "Random Forest" = "#2ca02c",
                                 "Grad. Boosted" = "#d62728"), name = "Algorithm") +
  scale_x_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  scale_y_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  labs(x = "Mean predicted probability", y = "Observed fraction (positive class)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3, labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = c(0.84, 0.22), strip.text = element_text(size = 13, face = "bold"))
save_fig(p5, "Figure5_calibration_curves", width = 11, height = 7)

# =============================================================================
# FIGURE 6 — SHAP feature importance (exact interventional TreeSHAP)
# =============================================================================
cat("Figure 6...\n")
attr_raw <- dbGetQuery(con, "
  SELECT fa.feature_name, fa.shap_value,
         mr.target_contaminant
  FROM shap_values fa
  JOIN model_runs mr ON fa.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
    AND mr.algorithm = 'RandomForest'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
")
attr_mean <- attr_raw %>%
  group_by(target_contaminant, feature_name) %>%
  summarise(mean_abs_shap = mean(abs(shap_value), na.rm = TRUE), .groups = "drop") %>%
  group_by(target_contaminant) %>%
  slice_max(mean_abs_shap, n = 10) %>%
  ungroup() %>%
  mutate(target_contaminant = factor(target_contaminant,
                                     levels = c("Na", "Cl", "TDS", "B", "F"))) %>%
  group_by(target_contaminant) %>%
  mutate(feature_name = reorder(feature_name, mean_abs_shap)) %>%
  ungroup()
p6 <- ggplot(attr_mean, aes(x = mean_abs_shap, y = feature_name,
                            colour = target_contaminant)) +
  geom_segment(aes(xend = 0, yend = feature_name), linewidth = 1.5, alpha = 0.7) +
  geom_point(size = 5, alpha = 0.95) +
  scale_colour_brewer(palette = "Set1", name = "Target", labels = pm_scale(ION_PM)) +
  scale_y_discrete(labels = pm_scale(FEAT_PM)) +
  scale_x_continuous(labels = function(x) sprintf("%.3f", x),
                     expand = expansion(mult = c(0.02, 0.08))) +
  labs(x = expression("|SHAP value|" ~ (mean ~ absolute)), y = NULL) +
  facet_wrap(~ target_contaminant, scales = "free_y", nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = "none", strip.text = element_text(size = 14, face = "bold"),
        axis.text.y = element_text(size = 13), axis.text.x = element_text(size = 12))
save_fig(p6, "Figure6_SHAP_importance", width = 14, height = 9)

# =============================================================================
# FIGURE 7 — Screening priority maps (four documented bands)
# =============================================================================
cat("Figure 7...\n")
sp_map <- dbGetQuery(con, "
  SELECT sp.sample_id, sp.target_contaminant,
         sp.predicted_probability_median, sp.screening_priority_class,
         ssa.latitude AS lat, ssa.longitude AS lon
  FROM screening_priority sp
  JOIN sample_spatial_assignment ssa ON sp.sample_id = ssa.sample_id
  WHERE sp.target_contaminant IN ('Na','Cl','TDS','B','F')
")
cluster_meta <- dbGetQuery(con, "SELECT * FROM spatial_cluster_metadata")
sp_map <- sp_map %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    priority = factor(screening_priority_class,
                      levels = c("Low", "Moderate", "High", "Very_High"),
                      labels = c("Low", "Moderate", "High", "Very high"))
  )
clust_pts <- cluster_meta %>%
  rename(lon = centroid_longitude, lat = centroid_latitude) %>%
  mutate(label = paste0("C", spatial_cluster_id, "\n(n=", number_of_wells, ")"))
p7 <- ggplot(sp_map, aes(x = lon, y = lat)) +
  geom_point(aes(fill = priority, size = predicted_probability_median),
             shape = 21, colour = "white", stroke = 0.4, alpha = 0.88) +
  geom_point(data = clust_pts, aes(x = lon, y = lat), shape = 3, size = 4.5,
             colour = "black", stroke = 1.8, inherit.aes = FALSE) +
  geom_text_repel(data = clust_pts, aes(x = lon, y = lat, label = label),
                  size = 3.4, colour = "black", box.padding = 0.5,
                  segment.colour = "grey50", inherit.aes = FALSE) +
  scale_fill_manual(
    values = c(Low = "#2ca02c", Moderate = "#ffbb33", High = "#e67e22", "Very high" = "#c0392b"),
    name = "Screening\npriority",
    guide = guide_legend(override.aes = list(size = 5, colour = "white"), order = 1)) +
  scale_size_continuous(range = c(2.5, 8), name = "Median\nprobability",
                        breaks = c(0.2, 0.5, 0.8),
                        guide = guide_legend(override.aes = list(fill = "grey50",
                                                                 colour = "white"), order = 2)) +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3, labeller = pm_labeller(ION_PM)) +
  labs(x = "Longitude", y = "Latitude") +
  nature_theme() +
  theme(legend.position = "right", strip.text = element_text(size = 14, face = "bold"))
save_fig(p7, "Figure7_screening_priority_map", width = 13, height = 9)

# =============================================================================
# FIGURE 8 — End-user decision flowchart (static)
# =============================================================================
cat("Figure 8...\n")
step_df <- data.frame(
  x = c(1, 2, 3, 4, 5, 6, 7),
  y = rep(4, 7),
  label = c("Step 1\nChoose target\n(Na, Cl, TDS, B, F)",
            "Step 2\nChoose tier\n(1 field, 2 reduced,\n3 full chemistry)",
            "Step 3\nCheck leakage\nfilter (Table 2)",
            "Step 4\nApply model\n(tuned in inner CV)",
            "Step 5\nApply target-specific\nprobability cutoff\n(Table 4)",
            "Step 6\nClassify priority\n(Low / Moderate /\nHigh / Very high)",
            "Step 7\nPrioritise wells for\nconfirmatory\nlaboratory testing"),
  stringsAsFactors = FALSE
)
p8 <- ggplot(step_df, aes(x = x, y = y)) +
  geom_label(aes(label = label), size = 3.2, fill = "lightyellow",
             colour = "black", label.size = 0.6, lineheight = 0.9,
             label.padding = unit(0.5, "lines")) +
  geom_segment(aes(x = x + 0.55, xend = x + 0.95, y = y, yend = y),
               arrow = arrow(length = unit(0.18, "cm"), type = "closed"),
               linewidth = 0.7, colour = "grey30") +
  coord_cartesian(xlim = c(0.3, 7.7), ylim = c(2, 6)) +
  labs(x = NULL, y = NULL) +
  theme_void() +
  theme(plot.margin = margin(10, 10, 10, 10))
save_fig(p8, "Figure8_enduser_decision_flowchart", width = 15, height = 4.5)

# =============================================================================
# FIGURE S1 — Spearman correlation matrix
# =============================================================================
cat("Figure S1...\n")
cm <- dbGetQuery(con, "
  SELECT sample_id, variable_name, cleaned_value_real
  FROM cleaned_measurements
  WHERE variable_name IN ('pH','Temp.','EC','TDS','Na','K','Mg','Ca','Cl',
                          'SO4','HCO3','CO3','NO3','F','B')
    AND cleaned_value_real IS NOT NULL
")
cm_wide <- cm %>%
  pivot_wider(names_from = variable_name, values_from = cleaned_value_real) %>%
  select(-sample_id) %>%
  select(any_of(c("pH","Temp.","EC","TDS","Na","K","Mg","Ca","Cl","SO4","HCO3","CO3","NO3","F","B")))
cor_mat <- cor(cm_wide, method = "spearman", use = "pairwise.complete.obs")
cor_df <- as.data.frame(as.table(cor_mat)) %>%
  rename(var1 = Var1, var2 = Var2, rho = Freq) %>%
  mutate(both = apply(., 1, function(r) paste(sort(c(r["var1"], r["var2"])), collapse = "|"))) %>%
  distinct(both, .keep_all = TRUE) %>%
  select(-both) %>%
  mutate(var1 = factor(var1, levels = rev(colnames(cor_mat))),
         var2 = factor(var2, levels = colnames(cor_mat)))
pS1 <- ggplot(cor_df, aes(x = var2, y = var1, fill = rho)) +
  geom_tile(colour = "white", linewidth = 0.4) +
  geom_text(aes(label = sprintf("%.2f", rho)), size = 3.2, colour = "grey15") +
  scale_fill_gradient2(low = "#2166ac", mid = "white", high = "#b2182b",
                       midpoint = 0, limits = c(-1, 1), name = "Spearman rho") +
  scale_x_discrete(labels = pm_scale(FEAT_PM)) +
  scale_y_discrete(labels = pm_scale(FEAT_PM)) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x = element_text(angle = 45, hjust = 1, size = 11),
        axis.text.y = element_text(size = 11),
        panel.border = element_blank())
save_fig(pS1, "FigS1_correlation_matrix", width = 11, height = 9)

# =============================================================================
# FIGURE S2 — Threshold-sensitivity curves (Tier 3, stratified, best models)
# =============================================================================
cat("Figure S2...\n")
ts <- dbGetQuery(con, "
  SELECT ts.probability_cutoff, ts.sensitivity, ts.specificity, ts.f2_score,
         mr.target_contaminant
  FROM threshold_sensitivity ts
  JOIN model_runs mr ON ts.run_id = mr.run_id
  WHERE mr.algorithm = 'GradientBoostedTrees'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")
ts_long <- ts %>%
  pivot_longer(cols = c(sensitivity, specificity, f2_score),
               names_to = "metric", values_to = "value") %>%
  mutate(metric = factor(metric, levels = c("sensitivity", "specificity", "f2_score"),
                         labels = c("Sensitivity", "Specificity", "F2 score")),
         target_contaminant = factor(target_contaminant, levels = TARGET_ORDER))
pS2 <- ggplot(ts_long, aes(x = probability_cutoff, y = value, colour = metric)) +
  geom_line(linewidth = 1.2) +
  geom_vline(xintercept = 0.5, linetype = "dashed", colour = "grey40") +
  scale_colour_manual(values = c("Sensitivity" = "#d62728",
                                 "Specificity" = "#1f77b4",
                                 "F2 score" = "#2ca02c"), name = NULL) +
  scale_x_continuous(breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Probability cutoff", y = "Value") +
  nature_theme() +
  theme(legend.position = "bottom", axis.text.x = element_text(size = 10))
save_fig(pS2, "FigS2_threshold_sensitivity_curves", width = 14, height = 4.5)

# =============================================================================
# FIGURE S3 — Well-level confusion matrices at the operating cutoff
# =============================================================================
cat("Figure S3...\n")
wlo <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm,
         w.tp_count, w.tn_count, w.fp_count, w.fn_count,
         round(w.optimized_probability_cutoff, 2) AS cutoff
  FROM well_level_operating w
  JOIN model_runs mr ON w.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.cv_mode = 'Stratified_Nested_CV'
")
# Best model per target/tier by F2 at the well level
best_wlo <- wlo %>%
  mutate(f2 = (5 * (tp_count / (tp_count + fn_count)) * (tp_count / (tp_count + fp_count))) /
           (4 * (tp_count / (tp_count + fp_count)) + (tp_count / (tp_count + fn_count)) + 1e-9)) %>%
  group_by(target_contaminant, predictor_tier) %>%
  slice_max(f2, n = 1, with_ties = FALSE) %>%
  ungroup() %>%
  filter(predictor_tier == "Tier3_Full")
cm_long <- best_wlo %>%
  rowwise() %>%
  mutate(cells = list(data.frame(
    actual = c("Positive", "Positive", "Negative", "Negative"),
    predicted = c("Positive", "Negative", "Positive", "Negative"),
    count = c(tp_count, fn_count, fp_count, tn_count)))) %>%
  unnest(cells) %>%
  mutate(target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
         actual = factor(actual, levels = c("Positive", "Negative")),
         predicted = factor(predicted, levels = c("Positive", "Negative")),
         is_err = (actual != predicted))
pS3 <- ggplot(cm_long, aes(x = predicted, y = actual, fill = is_err)) +
  geom_tile(colour = "white", linewidth = 0.8) +
  geom_text(aes(label = count), size = 6, fontface = "bold") +
  scale_fill_manual(values = c("TRUE" = "#f4a582", "FALSE" = "#92c5de"), guide = "none") +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Predicted (at well-level operating cutoff)",
       y = "Observed") +
  nature_theme() +
  theme(axis.text = element_text(size = 12), strip.text = element_text(size = 13))
save_fig(pS3, "FigS3_confusion_matrices", width = 13, height = 4.5)

# =============================================================================
# FIGURE S4 — Coordinate ablation comparison
# =============================================================================
cat("Figure S4...\n")
coord <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         CASE WHEN mr.predictor_tier LIKE '%no_coords%' THEN 'without coordinates'
              ELSE 'with coordinates' END AS coords,
         pm.pr_auc
  FROM pooled_metrics pm
  JOIN model_runs mr ON pm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND pm.scope = 'all_held_out'
    AND (mr.predictor_tier LIKE '%no_coords%' OR mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full'))
")
coord <- coord %>%
  mutate(base_tier = case_when(predictor_tier %in% c("Tier1_Field","Tier1_Field_no_coords") ~ "Tier 1",
                               predictor_tier %in% c("Tier2_Reduced","Tier2_Reduced_no_coords") ~ "Tier 2",
                               TRUE ~ "Tier 3"),
         target_contaminant = factor(target_contaminant, levels = TARGET_ORDER))
pS4 <- ggplot(coord, aes(x = coords, y = pr_auc, colour = algorithm)) +
  geom_jitter(width = 0.08, height = 0, size = 2.2, alpha = 0.75) +
  stat_summary(aes(group = interaction(coords, algorithm)), fun = mean, geom = "line",
               linewidth = 0.8, alpha = 0.5) +
  scale_colour_manual(values = c("EcOnlyLogistic" = "#7f7f7f",
                                 "LogisticRegression" = "#1f77b4",
                                 "RandomForest" = "#2ca02c",
                                 "GradientBoostedTrees" = "#d62728")) +
  facet_grid(cv_mode ~ target_contaminant + base_tier, scales = "free_x",
             labeller = labeller(target_contaminant = pm_scale(ION_PM))) +
  labs(x = NULL, y = "PR-AUC (pooled OOF)",
       colour = "Algorithm") +
  nature_theme() +
  theme(axis.text.x = element_text(size = 10, angle = 25, hjust = 1),
        strip.text = element_text(size = 10))
save_fig(pS4, "FigS4_coordinate_ablation", width = 15, height = 7)

# =============================================================================
# FIGURE S5 — NO3 descriptive statistics and spatial distribution
# =============================================================================
cat("Figure S5...\n")
no3 <- dbGetQuery(con, "
  SELECT m.sample_id, m.cleaned_value_real AS value, m.bdl_flag,
         ssa.latitude AS lat, ssa.longitude AS lon
  FROM cleaned_measurements m
  LEFT JOIN sample_spatial_assignment ssa ON ssa.sample_id = m.sample_id
  WHERE m.variable_name = 'NO3'
")
no3 <- no3 %>% mutate(bdl = bdl_flag == 1)
pS5a <- ggplot(no3, aes(x = value)) +
  geom_histogram(bins = 12, fill = "#1f77b4", colour = "white") +
  geom_vline(xintercept = 37.5, linetype = "dashed", colour = "#d62728", linewidth = 0.8) +
  annotate("text", x = 39, y = Inf, label = "Screening threshold 37.5 mg/L",
           hjust = 0, vjust = 1.5, size = 3.6, colour = "#d62728") +
  labs(x = expression(NO[3]^"-" * " (mg/L)"), y = "Wells") +
  nature_theme()
pS5b <- ggplot(no3, aes(x = lon, y = lat, colour = bdl)) +
  geom_point(aes(size = value), alpha = 0.8) +
  scale_colour_manual(values = c("TRUE" = "#d62728", "FALSE" = "#1f77b4"),
                      name = "Below detection\nlimit", labels = c("FALSE" = "No", "TRUE" = "Yes")) +
  scale_size_continuous(range = c(1.5, 6), name = expression(NO[3]^"-" * " (mg/L)")) +
  labs(x = "Longitude", y = "Latitude") +
  nature_theme()
pS5 <- pS5a + pS5b + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))
save_fig(pS5, "FigS5_NO3_descriptive", width = 12, height = 5)

# =============================================================================
# FIGURE S6 — F-beta sensitivity (F1, F1.5, F2, F3 cutoffs)
# =============================================================================
cat("Figure S6...\n")
fb <- dbGetQuery(con, "
  SELECT ts.probability_cutoff, ts.f1_score, ts.f1_5_score, ts.f2_score, ts.f3_score,
         mr.target_contaminant
  FROM threshold_sensitivity ts
  JOIN model_runs mr ON ts.run_id = mr.run_id
  WHERE mr.algorithm = 'GradientBoostedTrees'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")
fb_long <- fb %>%
  pivot_longer(cols = c(f1_score, f1_5_score, f2_score, f3_score),
               names_to = "beta", values_to = "score") %>%
  mutate(beta = factor(beta, levels = c("f1_score", "f1_5_score", "f2_score", "f3_score"),
                       labels = c("F1 (beta = 1)", "F1.5 (beta = 1.5)",
                                  "F2 (beta = 2)", "F3 (beta = 3)")),
         target_contaminant = factor(target_contaminant, levels = TARGET_ORDER))
pS6 <- ggplot(fb_long, aes(x = probability_cutoff, y = score, colour = beta)) +
  geom_line(linewidth = 1.1) +
  scale_colour_manual(values = c("F1 (beta = 1)" = "#1f77b4",
                                 "F1.5 (beta = 1.5)" = "#2ca02c",
                                 "F2 (beta = 2)" = "#ff7f0e",
                                 "F3 (beta = 3)" = "#d62728"), name = NULL) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Probability cutoff", y = "F-beta score") +
  nature_theme() +
  theme(legend.position = "bottom", axis.text.x = element_text(size = 10))
save_fig(pS6, "FigS6_beta_sensitivity", width = 14, height = 4.5)

# =============================================================================
# FIGURE S7 — Nested vs pooled operating characteristics
# =============================================================================
cat("Figure S7...\n")
nested_pooled <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         ot.threshold_selection_rule, ot.optimized_probability_cutoff,
         ot.resulting_sensitivity, ot.resulting_specificity,
         ot.false_negatives_count, ot.false_positives_count
  FROM operational_thresholds ot
  JOIN model_runs mr ON ot.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.predictor_tier = 'Tier3_Full'
    AND ot.threshold_selection_rule IN ('sens_ge_0.90','nested_pooled_operating')
")
op_plot <- nested_pooled %>%
  mutate(rule = ifelse(threshold_selection_rule == "sens_ge_0.90",
                       "Pooled (post-hoc)", "Nested (fold-local cutoffs)"),
         target_contaminant = factor(target_contaminant, levels = TARGET_ORDER))
pS7a <- ggplot(op_plot, aes(x = resulting_sensitivity, y = resulting_specificity,
                            colour = rule, shape = algorithm)) +
  geom_point(size = 3.5, alpha = 0.9) +
  scale_colour_manual(values = c("Pooled (post-hoc)" = "#1f77b4",
                                 "Nested (fold-local cutoffs)" = "#d62728"), name = NULL) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Sensitivity at operating cutoff", y = "Specificity at operating cutoff") +
  nature_theme() +
  theme(legend.position = "bottom", axis.text.x = element_text(size = 10))
pS7b <- ggplot(op_plot, aes(x = rule, y = optimized_probability_cutoff,
                            colour = target_contaminant)) +
  geom_jitter(width = 0.1, size = 3, alpha = 0.85) +
  scale_colour_brewer(palette = "Set1", name = "Target", labels = pm_scale(ION_PM)) +
  labs(x = NULL, y = "Probability cutoff") +
  nature_theme() +
  theme(legend.position = "right")
pS7 <- pS7a + pS7b + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))
save_fig(pS7, "FigS7_nested_vs_pooled_cutoffs", width = 15, height = 6)

dbDisconnect(con)
cat("\n=== All revised figures written to", FIG_DIR, "===\n")
