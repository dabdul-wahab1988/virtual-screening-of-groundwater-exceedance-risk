# =============================================================================
# Groundwater Virtual Screening — Manuscript Figures & Tables
# Nature-style publication quality: 300 DPI, white bg, 14-16pt fonts
# =============================================================================

suppressPackageStartupMessages({
  library(RSQLite)
  library(DBI)
  library(ggplot2)
  library(dplyr)
  library(tidyr)
  library(patchwork)
  library(RColorBrewer)
  library(viridis)
  library(scales)
  library(ggrepel)
  library(gridExtra)
  library(grid)
})

# ── Paths ──────────────────────────────────────────────────────────────────
PROJECT_ROOT <- "C:/Users/DicksonAbdul-Wahab/Documents/Dr_Sukari_Manuscript/manuscript_4b/groundwater_virtual_screening"
DB_PATH      <- file.path(PROJECT_ROOT, "outputs/groundwater_screening.db")
FIG_DIR      <- file.path(PROJECT_ROOT, "manuscript/artifacts/figures")
TAB_DIR      <- file.path(PROJECT_ROOT, "manuscript/artifacts/tables")
dir.create(FIG_DIR, showWarnings = FALSE, recursive = TRUE)
dir.create(TAB_DIR, showWarnings = FALSE, recursive = TRUE)

# ── Database connection ────────────────────────────────────────────────────
con <- dbConnect(SQLite(), DB_PATH)

# ── Nature style theme ────────────────────────────────────────────────────
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
      plot.title        = element_text(size = 13, face = "bold"),
      strip.text        = element_text(size = 13, face = "bold"),
      strip.background  = element_rect(fill = "grey92", colour = "black", linewidth = 0.6),
      axis.ticks        = element_line(colour = "black", linewidth = 0.5),
      panel.grid        = element_blank(),
      axis.ticks.length = unit(0.15, "cm")
    )
}

save_fig <- function(p, name, width = 8, height = 6) {
  path <- file.path(FIG_DIR, paste0(name, ".png"))
  ggsave(path, plot = p, width = width, height = height, dpi = 300,
         bg = "white", units = "in")
  message("  Saved: ", path)
}

save_tab <- function(df, name) {
  path <- file.path(TAB_DIR, paste0(name, ".csv"))
  write.csv(df, path, row.names = FALSE)
  message("  Saved: ", path)
}

# ── Palette & label helpers ───────────────────────────────────────────────
ALGO_LABELS <- c(
  "LogisticRegression"   = "Logistic Reg.",
  "RandomForest"         = "Random Forest",
  "GradientBoostedTrees" = "Gradient Boosted",
  "EcOnlyLogistic"       = "EC-only Logistic",
  "DummyMajority"        = "Dummy (Majority)",
  "DummyStratified"      = "Dummy (Stratified)"
)
ALGO_COLORS <- c(
  "LogisticRegression"   = "#1f77b4",
  "RandomForest"         = "#2ca02c",
  "GradientBoostedTrees" = "#d62728",
  "EcOnlyLogistic"       = "#9467bd",
  "DummyMajority"        = "#aec7e8",
  "DummyStratified"      = "#c5b0d5"
)
TIER_LABELS <- c(
  "Tier1_Field"   = "Tier 1\n(Field)",
  "Tier2_Reduced" = "Tier 2\n(Reduced)",
  "Tier3_Full"    = "Tier 3\n(Full)"
)
TARGET_ORDER <- c("Na", "Cl", "TDS", "B", "F", "NO3")

cat("=== Connected to database. Starting figure generation... ===\n\n")

# =============================================================================
# FIGURE 1 — Study workflow schematic
# =============================================================================
cat("Figure 1: Study workflow...\n")

boxes <- data.frame(
  x     = c(0.5, 0.5,   3,   3,   5.5, 5.5, 5.5,  8,   8,   8),
  y     = c(8,   5.5,   9.5, 7,   9.5, 7,   4.5,  9.5, 7,   4.5),
  label = c(
    "Raw Groundwater\nData (n = 81 wells)\npH, EC, TDS, Ions,\nCoordinates",
    "Target Definition\n6 binary exceedance\noutcomes\n(75% of WHO threshold)",
    "Leakage Control\nTarget-wise exclusion\nof direct + derived\npredictor variables",
    "Predictor Tiers\nTier 1: Field (5 vars)\nTier 2: Reduced (12 vars)\nTier 3: Full (20+ vars)",
    "Repeated Stratified\nNested CV\n10×5 outer folds\n(random splits)",
    "Spatial Block CV\n4 spatial clusters\n(geographic independence)",
    "Model Training\nLogistic Reg.\nRandom Forest\nGradient Boosted",
    "Performance\nPR-AUC, Recall\nF2-score, Brier\nCalibration",
    "Explainability\nSHAP values\n(fold-level)\nHydrochemical alignment",
    "Screening\nPriority Map\n(3-class output)\nSQLite audit trail"
  ),
  fill  = c("data", "target", "leakage", "tier",
            "cv", "cv", "model",
            "output", "output", "output"),
  width = 2.2,
  height= 1.6,
  stringsAsFactors = FALSE
)

fill_vals <- c(
  data    = "#D6EAF8", target = "#D5F5E3", leakage = "#FADBD8",
  tier    = "#FEF9E7", cv     = "#E8DAEF", model   = "#FDEBD0",
  output  = "#D5D8DC"
)

arrows <- data.frame(
  x1 = c(1.6, 1.6, 4.1, 4.1, 6.6, 6.6, 6.6),
  y1 = c(8,   5.5,  9.5, 7,   9.5, 7,   4.5),
  x2 = c(1.9, 1.9,  4.4, 4.4,  6.9, 6.9,  6.9),
  y2 = c(8,   5.5,  9.5, 7,   9.5, 7,   4.5)
)

p1 <- ggplot() +
  geom_rect(data = boxes,
            aes(xmin = x - width/2, xmax = x + width/2,
                ymin = y - height/2, ymax = y + height/2, fill = fill),
            colour = "grey30", linewidth = 0.6, alpha = 0.95) +
  geom_text(data = boxes,
            aes(x = x, y = y, label = label),
            size = 3.2, lineheight = 1.2, fontface = "plain") +
  geom_segment(data = arrows,
               aes(x = x1, y = y1, xend = x2, yend = y2),
               arrow = arrow(length = unit(0.18, "cm"), type = "closed"),
               colour = "grey30", linewidth = 0.7) +
  scale_fill_manual(values = fill_vals, guide = "none") +
  scale_x_continuous(expand = expansion(mult = 0.02)) +
  scale_y_continuous(expand = expansion(mult = 0.05)) +
  labs(title = NULL) +
  theme_void() +
  theme(plot.background = element_rect(fill = "white", colour = NA),
        panel.background = element_rect(fill = "white", colour = NA))

save_fig(p1, "Figure1_workflow_schematic", width = 10, height = 7)


# =============================================================================
# FIGURE 2 — Exceedance prevalence by target
# =============================================================================
cat("Figure 2: Exceedance prevalence...\n")

elig <- dbGetQuery(con, "SELECT * FROM target_eligibility")
tdef <- dbGetQuery(con, "
  SELECT target_contaminant, threshold_value, threshold_unit, threshold_source
  FROM target_definitions
")

elig <- elig %>%
  left_join(tdef, by = "target_contaminant") %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    pct_positive = prevalence * 100,
    pct_negative = (1 - prevalence) * 100,
    ml_label = case_when(
      ml_status == "modelled"          ~ "Modelled",
      ml_status == "descriptive_only"  ~ "Descriptive only",
      TRUE                             ~ ml_status
    ),
    threshold_label = paste0(threshold_value, " ", threshold_unit)
  )

elig_long <- elig %>%
  select(target_contaminant, n_positive, n_negative, threshold_label, ml_label) %>%
  pivot_longer(cols = c(n_positive, n_negative),
               names_to = "class", values_to = "count") %>%
  mutate(class = ifelse(class == "n_positive", "Exceeds threshold", "Below threshold"))

p2 <- ggplot(elig_long, aes(x = target_contaminant, y = count, fill = class)) +
  geom_col(width = 0.65, colour = "white", linewidth = 0.3) +
  geom_text(data = elig,
            aes(x = target_contaminant, y = 81 + 2.5,
                label = paste0(round(pct_positive, 1), "%")),
            inherit.aes = FALSE, size = 4.8, fontface = "bold", colour = "grey20") +
  geom_text(data = elig,
            aes(x = target_contaminant, y = -4,
                label = threshold_label),
            inherit.aes = FALSE, size = 3.8, colour = "grey30", fontface = "italic") +
  annotate("text", x = 0.4, y = -4, label = "Threshold:", size = 3.8,
           colour = "grey30", fontface = "bold", hjust = 1) +
  scale_fill_manual(values = c("Exceeds threshold" = "#d62728",
                               "Below threshold"   = "#aec7e8"),
                    name = NULL) +
  scale_y_continuous(limits = c(-7, 90), breaks = c(0, 20, 40, 60, 80),
                     expand = c(0, 0)) +
  labs(x = "Groundwater quality parameter",
       y = "Number of wells (n = 81)") +
  nature_theme() +
  theme(legend.position = c(0.82, 0.88),
        axis.text.x     = element_text(size = 14, face = "bold"),
        axis.ticks.x    = element_blank())

save_fig(p2, "Figure2_exceedance_prevalence", width = 8, height = 5.5)


# =============================================================================
# FIGURE 3 — Model performance heatmap (PR-AUC) across targets and tiers
# =============================================================================
cat("Figure 3: Performance heatmap...\n")

perf_raw <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         AVG(fm.pr_auc)           AS pr_auc,
         AVG(fm.roc_auc)          AS roc_auc,
         AVG(fm.recall_sensitivity) AS recall,
         AVG(fm.f2_score)         AS f2_score,
         AVG(fm.brier_score)      AS brier_score
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
    AND mr.cv_mode = 'Stratified_Nested_CV'
  GROUP BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
")

perf_raw <- perf_raw %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    predictor_tier     = factor(predictor_tier,
                                levels = c("Tier1_Field","Tier2_Reduced","Tier3_Full"),
                                labels = c("Tier 1\n(Field)","Tier 2\n(Reduced)","Tier 3\n(Full)")),
    algorithm          = factor(algorithm,
                                levels = c("LogisticRegression","RandomForest","GradientBoostedTrees","EcOnlyLogistic"),
                                labels = c("Logistic Reg.","Random Forest","Grad. Boosted","EC-only Logistic"))
  )

p3 <- ggplot(perf_raw, aes(x = algorithm, y = target_contaminant, fill = pr_auc)) +
  geom_tile(colour = "white", linewidth = 0.5) +
  geom_text(aes(label = sprintf("%.2f", pr_auc)), size = 4, colour = "white", fontface = "bold") +
  scale_fill_gradientn(
    colours = c("#2c3e50","#2980b9","#27ae60","#f1c40f","#e74c3c"),
    limits  = c(0, 1), name = "PR-AUC",
    guide   = guide_colourbar(barwidth = 1, barheight = 8,
                               title.position = "top", title.hjust = 0.5)
  ) +
  facet_wrap(~ predictor_tier, nrow = 1) +
  labs(x = NULL, y = NULL) +
  scale_x_discrete(position = "bottom") +
  nature_theme() +
  theme(axis.text.x  = element_text(size = 11, angle = 30, hjust = 1),
        axis.text.y  = element_text(size = 14, face = "bold"),
        legend.position = "right",
        strip.text   = element_text(size = 12, face = "bold"),
        panel.border = element_rect(colour = "grey60", linewidth = 0.5))

save_fig(p3, "Figure3_performance_heatmap_PR_AUC", width = 11, height = 5)


# =============================================================================
# FIGURE 4 — Stratified CV vs Spatial CV performance penalty
# =============================================================================
cat("Figure 4: Spatial CV penalty...\n")

perf_both <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         AVG(fm.pr_auc)             AS pr_auc,
         AVG(fm.recall_sensitivity) AS recall,
         AVG(fm.f2_score)           AS f2_score
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.algorithm != 'EcOnlyLogistic'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
")

perf_wide <- perf_both %>%
  pivot_wider(names_from = cv_mode, values_from = c(pr_auc, recall, f2_score)) %>%
  rename(
    pr_auc_strat   = `pr_auc_Stratified_Nested_CV`,
    pr_auc_spatial = `pr_auc_Spatial_Group_CV`,
    recall_strat   = `recall_Stratified_Nested_CV`,
    recall_spatial = `recall_Spatial_Group_CV`
  ) %>%
  filter(!is.na(pr_auc_strat) & !is.na(pr_auc_spatial)) %>%
  mutate(
    penalty        = pr_auc_strat - pr_auc_spatial,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm      = factor(algorithm,
                            levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                            labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

# Panel A: Scatter strat vs spatial
pA <- ggplot(perf_wide, aes(x = pr_auc_strat, y = pr_auc_spatial,
                             colour = target_contaminant, shape = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_point(size = 3.5, alpha = 0.85, stroke = 0.5) +
  scale_colour_brewer(palette = "Set1", name = "Target") +
  scale_shape_manual(values = c(16, 17, 15), name = "Algorithm") +
  scale_x_continuous(limits = c(0, 1.02), breaks = c(0, 0.25, 0.5, 0.75, 1.0),
                     labels = c("0", "0.25", "0.50", "0.75", "1.00")) +
  scale_y_continuous(limits = c(0, 1.02), breaks = c(0, 0.25, 0.5, 0.75, 1.0),
                     labels = c("0", "0.25", "0.50", "0.75", "1.00")) +
  labs(x = "PR-AUC (Stratified CV)", y = "PR-AUC (Spatial Block CV)") +
  annotate("text", x = 0.85, y = 0.06, label = "Spatial CV\npenalised",
           size = 3.8, colour = "grey40", hjust = 0.5) +
  nature_theme() +
  theme(legend.position = "right",
        legend.margin   = margin(0, 0, 0, 0))

# Panel B: Penalty magnitude by target
pen_summary <- perf_wide %>%
  group_by(target_contaminant, predictor_tier) %>%
  summarise(mean_penalty = mean(penalty, na.rm = TRUE),
            se_penalty   = sd(penalty, na.rm = TRUE) / sqrt(n()),
            .groups = "drop") %>%
  mutate(
    tier_label = case_when(
      predictor_tier == "Tier1_Field"   ~ "T1",
      predictor_tier == "Tier2_Reduced" ~ "T2",
      predictor_tier == "Tier3_Full"    ~ "T3"
    )
  )

pB <- ggplot(pen_summary, aes(x = target_contaminant, y = mean_penalty, fill = tier_label)) +
  geom_col(position = position_dodge(0.75), width = 0.65, colour = "white", linewidth = 0.3) +
  geom_errorbar(aes(ymin = mean_penalty - se_penalty, ymax = mean_penalty + se_penalty),
                position = position_dodge(0.75), width = 0.25, linewidth = 0.6) +
  geom_hline(yintercept = 0, linetype = "solid", colour = "grey30", linewidth = 0.6) +
  scale_fill_manual(values = c("T1" = "#3498db", "T2" = "#2ecc71", "T3" = "#e74c3c"),
                    name = "Predictor\ntier") +
  scale_y_continuous(labels = function(x) sprintf("%.2f", x)) +
  labs(x = "Target parameter",
       y = expression(Delta * "PR-AUC (Stratified − Spatial CV)")) +
  nature_theme() +
  theme(legend.position  = c(0.88, 0.80),
        axis.text.x      = element_text(size = 14, face = "bold"))

p4 <- pA + pB + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))

save_fig(p4, "Figure4_spatial_cv_penalty", width = 13, height = 5.5)


# =============================================================================
# FIGURE 5 — Calibration / reliability diagrams
# =============================================================================
cat("Figure 5: Calibration curves...\n")

preds_all <- dbGetQuery(con, "
  SELECT wp.predicted_probability, wp.true_label,
         mr.target_contaminant, mr.algorithm, mr.cv_mode, mr.predictor_tier
  FROM well_predictions wp
  JOIN model_runs mr ON wp.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm IN ('RandomForest','GradientBoostedTrees','LogisticRegression')
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")

n_bins <- 8

calib_df <- preds_all %>%
  mutate(bin = cut(predicted_probability,
                   breaks = seq(0, 1, length.out = n_bins + 1),
                   include.lowest = TRUE, labels = FALSE)) %>%
  group_by(target_contaminant, algorithm, bin) %>%
  summarise(
    mean_pred  = mean(predicted_probability),
    obs_frac   = mean(true_label),
    n          = n(),
    se         = sqrt(obs_frac * (1 - obs_frac) / n),
    .groups    = "drop"
  ) %>%
  filter(n >= 3) %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm = factor(algorithm,
                       levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                       labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

p5 <- ggplot(calib_df, aes(x = mean_pred, y = obs_frac, colour = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_errorbar(aes(ymin = obs_frac - 1.96 * se, ymax = obs_frac + 1.96 * se),
                width = 0.04, alpha = 0.5, linewidth = 0.6) +
  geom_line(linewidth = 1.6, alpha = 0.85) +
  geom_point(size = 3.5, alpha = 0.9) +
  scale_colour_manual(
    values = c("Logistic Reg." = "#1f77b4",
               "Random Forest"   = "#2ca02c",
               "Grad. Boosted"   = "#d62728"),
    name = "Algorithm"
  ) +
  scale_x_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  scale_y_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  labs(x = "Mean predicted probability",
       y = "Observed fraction (positive class)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3) +
  nature_theme() +
  theme(legend.position  = c(0.84, 0.22),
        strip.text       = element_text(size = 13, face = "bold"),
        panel.spacing    = unit(0.4, "lines"))

save_fig(p5, "Figure5_calibration_curves", width = 11, height = 7)


# =============================================================================
# FIGURE 6 — SHAP feature importance (dot/lollipop) for top targets
# =============================================================================
cat("Figure 6: SHAP importance...\n")

shap_raw <- dbGetQuery(con, "
  SELECT sv.feature_name,
         sv.shap_value,
         sv.feature_cleaned_value,
         mr.target_contaminant,
         mr.algorithm,
         mr.predictor_tier
  FROM shap_values sv
  JOIN model_runs mr ON sv.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.target_contaminant IN ('Na','Cl','TDS')
    AND mr.algorithm = 'RandomForest'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
")

shap_mean <- shap_raw %>%
  group_by(target_contaminant, feature_name) %>%
  summarise(
    mean_abs_shap = mean(abs(shap_value), na.rm = TRUE),
    .groups = "drop"
  ) %>%
  group_by(target_contaminant) %>%
  slice_max(mean_abs_shap, n = 10) %>%
  ungroup() %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = c("Na","Cl","TDS")),
    feature_name = case_when(
      feature_name == "EC"           ~ "EC",
      feature_name == "HCO3_Cl_ratio"~ "HCO3/Cl",
      feature_name == "Na_Cl_ratio"  ~ "Na/Cl",
      feature_name == "Mg_Ca_ratio"  ~ "Mg/Ca",
      feature_name == "Ca_Mg_ratio"  ~ "Ca/Mg",
      feature_name == "Temp."        ~ "Temp.",
      TRUE                           ~ feature_name
    )
  ) %>%
  group_by(target_contaminant) %>%
  mutate(feature_name = reorder(feature_name, mean_abs_shap)) %>%
  ungroup()

p6 <- ggplot(shap_mean, aes(x = mean_abs_shap, y = feature_name,
                             colour = target_contaminant)) +
  geom_segment(aes(xend = 0, yend = feature_name),
               linewidth = 1.5, alpha = 0.7) +
  geom_point(size = 5, alpha = 0.95) +
  scale_colour_brewer(palette = "Set1", name = "Target") +
  scale_x_continuous(labels = function(x) sprintf("%.3f", x),
                     expand = expansion(mult = c(0.02, 0.08))) +
  labs(x = expression("|SHAP value|" ~ (mean ~ absolute)),
       y = NULL) +
  facet_wrap(~ target_contaminant, scales = "free_y", nrow = 1) +
  nature_theme() +
  theme(legend.position = "none",
        strip.text      = element_text(size = 14, face = "bold"),
        axis.text.y     = element_text(size = 13),
        axis.text.x     = element_text(size = 12))

save_fig(p6, "Figure6_SHAP_importance", width = 12, height = 5.5)


# =============================================================================
# FIGURE 7 — Screening priority map
# =============================================================================
cat("Figure 7: Screening priority map...\n")

sp_map <- dbGetQuery(con, "
  SELECT sp.sample_id, sp.target_contaminant,
         sp.predicted_probability_median,
         sp.screening_priority_class,
         ssa.latitude AS lat, ssa.longitude AS lon
  FROM screening_priority sp
  JOIN sample_spatial_assignment ssa ON sp.sample_id = ssa.sample_id
  WHERE sp.target_contaminant IN ('Na','Cl','TDS','B','F')
")

cluster_meta <- dbGetQuery(con, "SELECT * FROM spatial_cluster_metadata")

sp_map <- sp_map %>%
  mutate(
    target_contaminant  = factor(target_contaminant, levels = TARGET_ORDER),
    priority_class_ord  = factor(screening_priority_class,
                                 levels = c("Low","Moderate","High"),
                                 labels = c("Low priority","Moderate priority","High priority"))
  )

clust_pts <- cluster_meta %>%
  rename(lon = centroid_longitude, lat = centroid_latitude) %>%
  mutate(label = paste0("Cluster ", spatial_cluster_id, "\n(n=", number_of_wells, ")"))

p7 <- ggplot(sp_map, aes(x = lon, y = lat)) +
  geom_point(aes(colour = priority_class_ord, size = predicted_probability_median),
             alpha = 0.85, shape = 16) +
  geom_point(data = clust_pts,
             aes(x = lon, y = lat),
             shape = 3, size = 5, colour = "black", stroke = 1.5,
             inherit.aes = FALSE) +
  geom_text_repel(data = clust_pts,
                  aes(x = lon, y = lat, label = label),
                  size = 3.5, colour = "black",
                  box.padding = 0.5, point.padding = 0.3,
                  segment.colour = "grey50", inherit.aes = FALSE) +
  scale_colour_manual(
    values = c("Low priority"      = "#2ecc71",
               "Moderate priority" = "#f39c12",
               "High priority"     = "#e74c3c"),
    name = "Screening\npriority"
  ) +
  scale_size_continuous(
    range = c(2.5, 7),
    name  = "Median\nscreening\nprobability",
    breaks = c(0.2, 0.5, 0.8),
    labels = c("0.20", "0.50", "0.80")
  ) +
  labs(x = "Longitude (°E)", y = "Latitude (°N)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3) +
  nature_theme() +
  theme(legend.position = "right",
        strip.text      = element_text(size = 13, face = "bold"),
        panel.border    = element_rect(colour = "grey60", linewidth = 0.5),
        panel.spacing   = unit(0.4, "lines"),
        axis.text       = element_text(size = 11))

save_fig(p7, "Figure7_screening_priority_map", width = 13, height = 8)


# =============================================================================
# SUPPLEMENTARY FIGURE S1 — Correlation matrix (leakage-excluded)
# =============================================================================
cat("Supplementary Figure S1: Correlation matrix...\n")

raw_data <- dbGetQuery(con, "
  SELECT cm.sample_id, cm.variable_name, cm.cleaned_value_real
  FROM cleaned_measurements cm
  WHERE cm.missing_flag = 0 AND cm.bdl_flag = 0
    AND cm.variable_name IN ('pH','Temp.','EC','Na','K','Mg','Ca','Cl','SO4','HCO3','CO3','F','B')
")

wide_data <- raw_data %>%
  pivot_wider(names_from = variable_name, values_from = cleaned_value_real) %>%
  select(-sample_id) %>%
  filter(complete.cases(.))

corr_mat  <- cor(wide_data, use = "pairwise.complete.obs", method = "spearman")
corr_long <- as.data.frame(as.table(corr_mat)) %>%
  rename(Var1 = Var1, Var2 = Var2, corr = Freq) %>%
  mutate(Var1 = factor(Var1, levels = colnames(corr_mat)),
         Var2 = factor(Var2, levels = rev(colnames(corr_mat))))

pS1 <- ggplot(corr_long, aes(x = Var1, y = Var2, fill = corr)) +
  geom_tile(colour = "white", linewidth = 0.4) +
  geom_text(aes(label = sprintf("%.2f", corr)),
            size = 3.2, colour = ifelse(abs(corr_long$corr) > 0.55, "white", "grey20")) +
  scale_fill_gradient2(
    low    = "#3498db", mid = "white", high = "#e74c3c",
    midpoint = 0, limits = c(-1, 1),
    name = "Spearman\ncorrelation",
    guide = guide_colourbar(barwidth = 1, barheight = 8)
  ) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x  = element_text(size = 12, angle = 45, hjust = 1),
        axis.text.y  = element_text(size = 12),
        panel.border = element_blank(),
        axis.ticks   = element_blank(),
        legend.position = "right")

save_fig(pS1, "FigS1_correlation_matrix", width = 9, height = 8)


# =============================================================================
# SUPPLEMENTARY FIGURE S2 — Threshold sensitivity curves
# =============================================================================
cat("Supplementary Figure S2: Threshold sensitivity curves...\n")

thresh_sens <- dbGetQuery(con, "
  SELECT ts.probability_cutoff, ts.sensitivity, ts.specificity, ts.f2_score,
         ts.false_negatives_count,
         mr.target_contaminant, mr.algorithm, mr.predictor_tier, ts.cv_mode
  FROM threshold_sensitivity ts
  JOIN model_runs mr ON ts.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm = 'RandomForest'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")

thresh_long <- thresh_sens %>%
  select(probability_cutoff, sensitivity, specificity, f2_score,
         target_contaminant, algorithm) %>%
  pivot_longer(cols = c(sensitivity, specificity, f2_score),
               names_to = "metric", values_to = "value") %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    metric_label = case_when(
      metric == "sensitivity" ~ "Sensitivity (Recall)",
      metric == "specificity" ~ "Specificity",
      metric == "f2_score"    ~ "F2-score"
    )
  )

pS2 <- ggplot(thresh_long, aes(x = probability_cutoff, y = value,
                                colour = metric_label, linetype = metric_label)) +
  geom_line(linewidth = 1.8, alpha = 0.9) +
  geom_vline(xintercept = 0.5, linetype = "dotted", colour = "grey50", linewidth = 0.8) +
  scale_colour_manual(
    values = c("Sensitivity (Recall)" = "#e74c3c",
               "Specificity"          = "#2980b9",
               "F2-score"             = "#27ae60"),
    name = NULL
  ) +
  scale_linetype_manual(
    values = c("Sensitivity (Recall)" = "solid",
               "Specificity"          = "solid",
               "F2-score"             = "dashed"),
    name = NULL
  ) +
  scale_x_continuous(breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  scale_y_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  labs(x = "Probability threshold", y = "Metric value") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3) +
  nature_theme() +
  theme(legend.position = c(0.82, 0.15),
        strip.text      = element_text(size = 13, face = "bold"),
        panel.spacing   = unit(0.4, "lines"))

save_fig(pS2, "FigS2_threshold_sensitivity_curves", width = 12, height = 7)


# =============================================================================
# SUPPLEMENTARY FIGURE S3 — Per-target confusion matrices
# =============================================================================
cat("Supplementary Figure S3: Confusion matrices...\n")

ot_data <- dbGetQuery(con, "
  SELECT ot.threshold_selection_rule, ot.cv_mode,
         ot.resulting_sensitivity, ot.resulting_specificity,
         ot.false_negatives_count, ot.false_positives_count,
         mr.target_contaminant, mr.predictor_tier, mr.algorithm
  FROM operational_thresholds ot
  JOIN model_runs mr ON ot.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND ot.threshold_selection_rule = 'sens_ge_0.90'
    AND ot.cv_mode = 'Stratified_Nested_CV'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.algorithm IN ('LogisticRegression','RandomForest','GradientBoostedTrees')
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
")

ot_data2 <- ot_data %>%
  mutate(
    n_pos       = as.integer(round(false_negatives_count / (1 - resulting_sensitivity))),
    n_neg       = as.integer(round(false_positives_count / (1 - resulting_specificity))),
    TP          = n_pos - false_negatives_count,
    FN          = false_negatives_count,
    FP          = false_positives_count,
    TN          = n_neg - false_positives_count,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm   = factor(algorithm,
                         levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                         labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

# Best model per target (highest sensitivity)
best_model <- ot_data2 %>%
  group_by(target_contaminant) %>%
  slice_max(resulting_sensitivity, n = 1, with_ties = FALSE) %>%
  ungroup()

cm_long <- best_model %>%
  select(target_contaminant, algorithm, TP, FN, FP, TN) %>%
  pivot_longer(cols = c(TP, FN, FP, TN),
               names_to = "cell", values_to = "count") %>%
  mutate(
    actual    = ifelse(cell %in% c("TP","FN"), "Actual +", "Actual −"),
    predicted = ifelse(cell %in% c("TP","FP"), "Predicted +", "Predicted −"),
    fill_cat  = cell,
    actual    = factor(actual, levels = c("Actual +","Actual −")),
    predicted = factor(predicted, levels = c("Predicted +","Predicted −"))
  )

pS3 <- ggplot(cm_long, aes(x = predicted, y = actual, fill = fill_cat)) +
  geom_tile(colour = "white", linewidth = 0.8) +
  geom_text(aes(label = count), size = 5.5, fontface = "bold", colour = "white") +
  scale_fill_manual(
    values = c("TP" = "#27ae60", "TN" = "#2980b9",
               "FP" = "#f39c12", "FN" = "#e74c3c"),
    guide = "none"
  ) +
  facet_wrap(~ target_contaminant + algorithm, nrow = 2,
             labeller = labeller(
               target_contaminant = as_labeller(setNames(
                 paste0(levels(cm_long$target_contaminant)),
                 levels(cm_long$target_contaminant)
               ))
             )) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x    = element_text(size = 12),
        axis.text.y    = element_text(size = 12),
        strip.text     = element_text(size = 11, face = "bold"),
        panel.border   = element_rect(colour = "grey40", linewidth = 0.5),
        panel.spacing  = unit(0.3, "lines"))

save_fig(pS3, "FigS3_confusion_matrices", width = 14, height = 7)


# =============================================================================
# TABLE 1 — Dataset summary and exceedance prevalence
# =============================================================================
cat("\nTable 1: Dataset summary...\n")

raw_df <- dbGetQuery(con, "
  SELECT cm.variable_name, cm.cleaned_value_real, cm.bdl_flag, cm.missing_flag
  FROM cleaned_measurements cm
")

col_dict <- dbGetQuery(con, "SELECT * FROM column_dictionary")
targ_def <- dbGetQuery(con, "SELECT * FROM target_definitions")
elig_t   <- dbGetQuery(con, "SELECT * FROM target_eligibility")

summary_stats <- raw_df %>%
  filter(missing_flag == 0) %>%
  mutate(value = ifelse(bdl_flag == 1, NA_real_, cleaned_value_real)) %>%
  group_by(variable_name) %>%
  summarise(
    n_valid   = sum(!is.na(value)),
    n_bdl     = sum(bdl_flag),
    n_missing = sum(missing_flag),
    min_val   = round(min(value, na.rm = TRUE), 3),
    max_val   = round(max(value, na.rm = TRUE), 3),
    median_v  = round(median(value, na.rm = TRUE), 3),
    .groups = "drop"
  )

tab1 <- col_dict %>%
  filter(!column_name %in% c("SampleID", "Location")) %>%
  left_join(summary_stats, by = c("column_name" = "variable_name")) %>%
  left_join(targ_def %>% select(target_contaminant, threshold_value, threshold_unit, threshold_source),
            by = c("column_name" = "target_contaminant")) %>%
  left_join(elig_t %>% select(target_contaminant, n_positive, prevalence, ml_status),
            by = c("column_name" = "target_contaminant")) %>%
  select(
    Variable   = column_name,
    Unit       = unit,
    Role       = scientific_role,
    n_Valid    = n_valid,
    n_BDL      = n_bdl,
    n_Missing  = n_missing,
    Min        = min_val,
    Median     = median_v,
    Max        = max_val,
    Threshold  = threshold_value,
    Threshold_Unit = threshold_unit,
    n_Exceeding = n_positive,
    Prevalence_pct = prevalence,
    ML_Status  = ml_status
  ) %>%
  mutate(
    Prevalence_pct = ifelse(!is.na(Prevalence_pct), round(Prevalence_pct * 100, 1), NA),
    Threshold_Source = targ_def$threshold_source[match(Variable, targ_def$target_contaminant)]
  )

save_tab(tab1, "Table1_dataset_summary")


# =============================================================================
# TABLE 2 — Target-wise leakage matrix
# =============================================================================
cat("Table 2: Leakage matrix...\n")

leak_rules <- dbGetQuery(con, "SELECT * FROM leakage_rules_applied")

tab2 <- leak_rules %>%
  filter(action == "excluded") %>%
  select(Target = target_contaminant, Tier = tier_name,
         Excluded_Feature = feature_name, Reason = reason) %>%
  arrange(Target, Tier, Excluded_Feature)

# Also add n retained features per target/tier
retained <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.n_features
  FROM model_runs mr
  WHERE mr.run_status = 'completed'
    AND mr.algorithm = 'RandomForest'
    AND mr.cv_mode = 'Stratified_Nested_CV'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY mr.target_contaminant, mr.predictor_tier
") %>%
  rename(Target = target_contaminant, Tier = predictor_tier,
         N_Features_Retained = n_features)

tab2_summary <- leak_rules %>%
  filter(action == "excluded") %>%
  group_by(target_contaminant, tier_name) %>%
  summarise(n_excluded = n_distinct(feature_name), .groups = "drop") %>%
  rename(Target = target_contaminant, Tier = tier_name) %>%
  left_join(retained, by = c("Target","Tier")) %>%
  arrange(Target, Tier)

save_tab(tab2, "Table2_leakage_matrix_full")
save_tab(tab2_summary, "Table2_leakage_matrix_summary")


# =============================================================================
# TABLE 3 — Model performance by target, tier, and validation mode
# =============================================================================
cat("Table 3: Model performance...\n")

tab3 <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         ROUND(AVG(fm.pr_auc), 3)               AS PR_AUC,
         ROUND(AVG(fm.roc_auc), 3)              AS ROC_AUC,
         ROUND(AVG(fm.recall_sensitivity), 3)   AS Recall,
         ROUND(AVG(fm.f2_score), 3)             AS F2_Score,
         ROUND(AVG(fm.brier_score), 4)          AS Brier_Score,
         ROUND(AVG(fm.balanced_accuracy), 3)    AS Balanced_Acc,
         ROUND(AVG(fm.calibration_slope), 3)    AS Cal_Slope,
         ROUND(AVG(fm.calibration_intercept), 3) AS Cal_Intercept
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
  ORDER BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
")

save_tab(tab3, "Table3_model_performance")


# =============================================================================
# TABLE 4 — Operational threshold rules and false-negative trade-offs
# =============================================================================
cat("Table 4: Operational thresholds...\n")

tab4 <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, ot.cv_mode,
         ot.threshold_selection_rule,
         ROUND(ot.optimized_probability_cutoff, 2) AS Prob_Cutoff,
         ROUND(ot.resulting_sensitivity, 3)        AS Sensitivity,
         ROUND(ot.resulting_specificity, 3)        AS Specificity,
         ot.false_negatives_count                  AS False_Negatives,
         ot.false_positives_count                  AS False_Positives
  FROM operational_thresholds ot
  JOIN model_runs mr ON ot.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  ORDER BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, ot.threshold_selection_rule
")

save_tab(tab4, "Table4_operational_thresholds")


# =============================================================================
# TABLE 5 — Reduced-variable model comparison (Tier 1 vs 2 vs 3)
# =============================================================================
cat("Table 5: Tier comparison...\n")

tab5_base <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         ROUND(AVG(fm.pr_auc), 3)             AS PR_AUC,
         ROUND(AVG(fm.recall_sensitivity), 3) AS Recall,
         ROUND(AVG(fm.f2_score), 3)           AS F2_Score,
         mr.n_features                        AS N_Features
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm IN ('LogisticRegression','RandomForest','GradientBoostedTrees')
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
    AND mr.cv_mode = 'Stratified_Nested_CV'
  GROUP BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
  ORDER BY mr.target_contaminant, mr.algorithm, mr.predictor_tier
")

# Compute PR-AUC % retention vs Tier 3
tab5 <- tab5_base %>%
  group_by(target_contaminant, algorithm) %>%
  mutate(
    tier3_pr_auc  = PR_AUC[predictor_tier == "Tier3_Full"],
    pct_retained  = round(PR_AUC / tier3_pr_auc * 100, 1)
  ) %>%
  ungroup() %>%
  select(-tier3_pr_auc)

save_tab(tab5, "Table5_tier_comparison")


# =============================================================================
# SUPPLEMENTARY TABLES
# =============================================================================
cat("\nSupplementary Tables...\n")

# S1 — Full data dictionary
tabS1 <- dbGetQuery(con, "SELECT * FROM column_dictionary") %>%
  select(Variable = column_name, Inferred_Type = inferred_type,
         Scientific_Role = scientific_role, Unit = unit,
         Is_Coordinate = is_coordinate, Is_Target = is_target_candidate,
         Is_Field_Var = is_field_variable, Is_Lab_Var = is_lab_variable,
         Is_Derived = is_derived_variable, Notes = notes)
save_tab(tabS1, "TableS1_data_dictionary")

# S2 — Threshold sources
tabS2 <- targ_def %>%
  select(Target = target_contaminant,
         Source_Column = source_column,
         Threshold_Value = threshold_value,
         Unit = threshold_unit,
         Source = threshold_source,
         Direction = exceedance_direction,
         Notes = notes)
save_tab(tabS2, "TableS2_threshold_sources")

# S3 — Hyperparameter search space
tabS3 <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         mp.hyperparameter_name, mp.hyperparameter_value
  FROM model_hyperparameters mp
  JOIN model_runs mr ON mp.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
  GROUP BY mr.algorithm, mp.hyperparameter_name, mp.hyperparameter_value
  ORDER BY mr.algorithm, mp.hyperparameter_name
")
save_tab(tabS3, "TableS3_hyperparameter_search")

# S4 — Fold-level metrics
tabS4 <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         fm.repeat_index, fm.fold_index,
         ROUND(fm.pr_auc, 4)              AS PR_AUC,
         ROUND(fm.roc_auc, 4)            AS ROC_AUC,
         ROUND(fm.recall_sensitivity, 4) AS Recall,
         ROUND(fm.f2_score, 4)           AS F2_Score,
         ROUND(fm.brier_score, 5)        AS Brier_Score,
         ROUND(fm.balanced_accuracy, 4)  AS Balanced_Acc,
         fm.n_test_pos, fm.n_test_neg
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  ORDER BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
           fm.repeat_index, fm.fold_index
")
save_tab(tabS4, "TableS4_fold_level_metrics")

# S5 — Fold SHAP base values
tabS5 <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         fb.repeat_index, fb.fold_index,
         ROUND(fb.shap_base_value_prob, 5) AS SHAP_Base_Prob,
         ROUND(fb.train_fold_prevalence, 4) AS Train_Prevalence
  FROM fold_base_values fb
  JOIN model_runs mr ON fb.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
  ORDER BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, fb.repeat_index, fb.fold_index
")
save_tab(tabS5, "TableS5_SHAP_base_values")

# S6 — Spatial cluster metadata
tabS6 <- dbGetQuery(con, "
  SELECT sc.spatial_cluster_id, sc.number_of_wells,
         ROUND(sc.centroid_latitude, 5)                    AS Centroid_Lat,
         ROUND(sc.centroid_longitude, 5)                   AS Centroid_Lon,
         ROUND(sc.max_intra_cluster_distance_km, 3)        AS Max_Intra_Dist_km
  FROM spatial_cluster_metadata sc
") %>%
  left_join(
    dbGetQuery(con, "
      SELECT spatial_cluster_id, GROUP_CONCAT(sample_id, '; ') AS Well_IDs
      FROM sample_spatial_assignment
      GROUP BY spatial_cluster_id
    "),
    by = "spatial_cluster_id"
  )
save_tab(tabS6, "TableS6_spatial_clusters")

# S7 — Full leakage audit
tabS7 <- dbGetQuery(con, "SELECT * FROM leakage_rules_applied") %>%
  select(Target = target_contaminant, Tier = tier_name,
         Feature = feature_name, Action = action,
         Reason = reason, Rule_Source = rule_source)
save_tab(tabS7, "TableS7_leakage_audit")


# =============================================================================
# Close connection
# =============================================================================
dbDisconnect(con)

cat("\n=== All figures and tables generated successfully! ===\n")
cat(paste0("  Figures: ", FIG_DIR, "\n"))
cat(paste0("  Tables:  ", TAB_DIR, "\n"))
