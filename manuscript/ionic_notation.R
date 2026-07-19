# Ionic notation fix – regenerates ALL figures with proper chemical ion symbols
# Na→Na⁺  Cl→Cl⁻  F→F⁻  NO₃→NO₃⁻  SO₄→SO₄²⁻  HCO₃→HCO₃⁻  CO₃→CO₃²⁻
# K→K⁺  Mg→Mg²⁺  Ca→Ca²⁺  B/TDS/EC/pH/Temp. unchanged

suppressPackageStartupMessages({
  library(RSQLite); library(DBI); library(ggplot2); library(dplyr)
  library(tidyr); library(patchwork); library(RColorBrewer)
  library(viridis); library(scales); library(ggrepel); library(grid)
})

PROJECT_ROOT <- "C:/Users/DicksonAbdul-Wahab/Documents/Dr_Sukari_Manuscript/manuscript_4b/groundwater_virtual_screening"
DB_PATH <- file.path(PROJECT_ROOT, "outputs/groundwater_screening.db")
FIG_DIR <- file.path(PROJECT_ROOT, "manuscript/artifacts/figures")
con <- dbConnect(SQLite(), DB_PATH)

TARGET_ORDER <- c("Na", "Cl", "TDS", "B", "F", "NO3")

# ── Plotmath label maps ────────────────────────────────────────────────────
# Target ions (for facets and axes)
ION_PM <- c(
  "Na"  = "Na^'+'",
  "Cl"  = "Cl^'-'",
  "TDS" = "TDS",
  "B"   = "B",
  "F"   = "F^'-'",
  "NO3" = "NO[3]^'-'"
)

# Predictor/feature ions (for SHAP y-axis, correlation matrix)
FEAT_PM <- c(
  "EC"             = "EC",
  "B"              = "B",
  "Na"             = "Na^'+'",
  "K"              = "K^'+'",
  "Mg"             = "Mg^{'2+'}",
  "Ca"             = "Ca^{'2+'}",
  "Cl"             = "Cl^'-'",
  "SO4"            = "SO[4]^{'2-'}",
  "HCO3"           = "HCO[3]^'-'",
  "CO3"            = "CO[3]^{'2-'}",
  "NO3"            = "NO[3]^'-'",
  "F"              = "F^'-'",
  "Na_Cl_ratio"    = "Na^'+'/Cl^'-'",
  "HCO3_Cl_ratio"  = "HCO[3]^'-'/Cl^'-'",
  "Mg_Ca_ratio"    = "Mg^{'2+'}/Ca^{'2+'}",
  "Ca_Mg_ratio"    = "Ca^{'2+'}/Mg^{'2+'}",
  "SAR"            = "SAR",
  "Temp."          = "Temp.",
  "pH"             = "pH",
  "Dy"             = "Lat.",
  "Dx"             = "Lon.",
  # already-cleaned names from Figure 6 recode
  "HCO3/Cl"        = "HCO[3]^'-'/Cl^'-'",
  "Na/Cl"          = "Na^'+'/Cl^'-'",
  "Ca/Mg"          = "Ca^{'2+'}/Mg^{'2+'}",
  "Mg/Ca"          = "Mg^{'2+'}/Ca^{'2+'}"
)

# ── Helper functions ───────────────────────────────────────────────────────
# For scale_x/y_discrete(labels = pm_scale(MAP))
pm_scale <- function(map) {
  function(x) {
    mapped <- ifelse(x %in% names(map), map[x], x)
    parse(text = mapped)
  }
}

# For facet labellers
pm_labeller <- function(map) {
  as_labeller(function(x) ifelse(x %in% names(map), map[x], x), label_parsed)
}

# ── Nature theme ───────────────────────────────────────────────────────────
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

cat("=== Applying ionic notation to all figures ===\n\n")

# =============================================================================
# FIGURE 2 — Exceedance prevalence (ionic x-axis)
# =============================================================================
cat("Figure 2...\n")

elig <- dbGetQuery(con, "SELECT * FROM target_eligibility")
tdef <- dbGetQuery(con, "SELECT target_contaminant, threshold_value FROM target_definitions")

elig <- elig %>%
  left_join(tdef, by = "target_contaminant") %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    pct_positive       = round(prevalence * 100, 1),
    threshold_label    = paste0(threshold_value, " mg/L")
  )

elig_long <- elig %>%
  select(target_contaminant, n_positive, n_negative, threshold_label) %>%
  pivot_longer(cols = c(n_positive, n_negative),
               names_to = "class", values_to = "count") %>%
  mutate(class = ifelse(class == "n_positive", "Exceeds threshold", "Below threshold"))

p2 <- ggplot(elig_long, aes(x = target_contaminant, y = count, fill = class)) +
  geom_col(width = 0.65, colour = "white", linewidth = 0.3) +
  geom_text(data = elig,
            aes(x = target_contaminant, y = 81 + 2.5,
                label = paste0(pct_positive, "%")),
            inherit.aes = FALSE, size = 4.8, fontface = "bold", colour = "grey20") +
  geom_text(data = elig,
            aes(x = target_contaminant, y = -4, label = threshold_label),
            inherit.aes = FALSE, size = 3.8, colour = "grey30", fontface = "italic") +
  annotate("text", x = 0.4, y = -4, label = "Threshold:",
           size = 3.8, colour = "grey30", fontface = "bold", hjust = 1) +
  scale_fill_manual(
    values = c("Exceeds threshold" = "#d62728", "Below threshold" = "#aec7e8"),
    name = NULL
  ) +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  scale_y_continuous(limits = c(-7, 90), breaks = c(0, 20, 40, 60, 80), expand = c(0, 0)) +
  labs(x = "Groundwater quality parameter",
       y = "Number of wells (n = 81)") +
  nature_theme() +
  theme(legend.position  = "right",
        legend.direction = "vertical",
        legend.margin    = margin(0, 0, 0, 8),
        axis.text.x      = element_text(size = 14, face = "bold"),
        axis.ticks.x     = element_blank())

save_fig(p2, "Figure2_exceedance_prevalence", width = 9, height = 5.5)


# =============================================================================
# FIGURE 3 — Performance heatmap (ionic y-axis)
# =============================================================================
cat("Figure 3...\n")

perf_raw <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         AVG(fm.pr_auc) AS pr_auc
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
                                levels = c("LogisticRegression","RandomForest",
                                           "GradientBoostedTrees","EcOnlyLogistic"),
                                labels = c("Logistic Reg.","Random Forest",
                                           "Grad. Boosted","EC-only Logistic"))
  )

p3 <- ggplot(perf_raw, aes(x = algorithm, y = target_contaminant, fill = pr_auc)) +
  geom_tile(colour = "white", linewidth = 0.5) +
  geom_text(aes(label = sprintf("%.2f", pr_auc)), size = 4,
            colour = "white", fontface = "bold") +
  scale_fill_gradientn(
    colours = c("#2c3e50","#2980b9","#27ae60","#f1c40f","#e74c3c"),
    limits  = c(0, 1), name = "PR-AUC",
    guide   = guide_colourbar(barwidth = 1, barheight = 8,
                               title.position = "top", title.hjust = 0.5)
  ) +
  scale_y_discrete(labels = pm_scale(ION_PM)) +
  facet_wrap(~ predictor_tier, nrow = 1) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x     = element_text(size = 11, angle = 30, hjust = 1),
        axis.text.y     = element_text(size = 14, face = "bold"),
        legend.position = "right",
        strip.text      = element_text(size = 12, face = "bold"),
        panel.border    = element_rect(colour = "grey60", linewidth = 0.5))

save_fig(p3, "Figure3_performance_heatmap_PR_AUC", width = 11, height = 5)


# =============================================================================
# FIGURE 4 — Spatial CV penalty (ionic Panel B x-axis)
# =============================================================================
cat("Figure 4...\n")

perf_both <- dbGetQuery(con, "
  SELECT mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode,
         AVG(fm.pr_auc) AS pr_auc
  FROM fold_metrics fm
  JOIN model_runs mr ON fm.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.algorithm NOT LIKE 'Dummy%'
    AND mr.algorithm != 'EcOnlyLogistic'
    AND mr.predictor_tier IN ('Tier1_Field','Tier2_Reduced','Tier3_Full')
  GROUP BY mr.target_contaminant, mr.predictor_tier, mr.algorithm, mr.cv_mode
")

perf_wide <- perf_both %>%
  pivot_wider(names_from = cv_mode, values_from = pr_auc) %>%
  rename(pr_auc_strat   = Stratified_Nested_CV,
         pr_auc_spatial = Spatial_Group_CV) %>%
  filter(!is.na(pr_auc_strat) & !is.na(pr_auc_spatial)) %>%
  mutate(
    penalty            = pr_auc_strat - pr_auc_spatial,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm          = factor(algorithm,
                                levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                                labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

pA <- ggplot(perf_wide, aes(x = pr_auc_strat, y = pr_auc_spatial,
                             colour = target_contaminant, shape = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_point(size = 3.5, alpha = 0.85, stroke = 0.5) +
  scale_colour_brewer(palette = "Set1", name = "Target",
                      labels = pm_scale(ION_PM)) +
  scale_shape_manual(values = c(16, 17, 15), name = "Algorithm") +
  scale_x_continuous(limits = c(0, 1.02),
                     breaks = c(0, 0.25, 0.5, 0.75, 1.0),
                     labels = c("0","0.25","0.50","0.75","1.00")) +
  scale_y_continuous(limits = c(0, 1.02),
                     breaks = c(0, 0.25, 0.5, 0.75, 1.0),
                     labels = c("0","0.25","0.50","0.75","1.00")) +
  labs(x = "PR-AUC (Stratified CV)", y = "PR-AUC (Spatial Block CV)") +
  annotate("text", x = 0.85, y = 0.06, label = "Spatial CV\npenalised",
           size = 3.8, colour = "grey40", hjust = 0.5) +
  nature_theme() +
  theme(legend.position = "right", legend.margin = margin(0, 0, 0, 0))

pen_summary <- perf_wide %>%
  group_by(target_contaminant, predictor_tier) %>%
  summarise(mean_penalty = mean(penalty, na.rm = TRUE),
            se_penalty   = sd(penalty, na.rm = TRUE) / sqrt(n()),
            .groups = "drop") %>%
  mutate(tier_label = case_when(
    predictor_tier == "Tier1_Field"   ~ "T1",
    predictor_tier == "Tier2_Reduced" ~ "T2",
    predictor_tier == "Tier3_Full"    ~ "T3"
  ))

pB <- ggplot(pen_summary,
             aes(x = target_contaminant, y = mean_penalty, fill = tier_label)) +
  geom_col(position = position_dodge(0.75), width = 0.65,
           colour = "white", linewidth = 0.3) +
  geom_errorbar(aes(ymin = mean_penalty - se_penalty,
                    ymax = mean_penalty + se_penalty),
                position = position_dodge(0.75), width = 0.25, linewidth = 0.6) +
  geom_hline(yintercept = 0, linetype = "solid", colour = "grey30", linewidth = 0.6) +
  scale_fill_manual(
    values = c("T1" = "#3498db", "T2" = "#2ecc71", "T3" = "#e74c3c"),
    name = "Predictor\ntier"
  ) +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  scale_y_continuous(labels = function(x) sprintf("%.2f", x)) +
  labs(x = "Target parameter",
       y = expression(Delta * "PR-AUC (Stratified − Spatial CV)")) +
  nature_theme() +
  theme(legend.position = c(0.12, 0.18),
        legend.margin   = margin(4, 6, 4, 6),
        axis.text.x     = element_text(size = 14, face = "bold"))

p4 <- pA + pB +
  plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))

save_fig(p4, "Figure4_spatial_cv_penalty", width = 13, height = 5.5)


# =============================================================================
# FIGURE 5 — Calibration curves (ionic facet labels)
# =============================================================================
cat("Figure 5...\n")

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

calib_df <- preds_all %>%
  mutate(bin = cut(predicted_probability,
                   breaks = seq(0, 1, length.out = 9),
                   include.lowest = TRUE, labels = FALSE)) %>%
  group_by(target_contaminant, algorithm, bin) %>%
  summarise(mean_pred = mean(predicted_probability),
            obs_frac  = mean(true_label),
            n         = n(),
            se        = sqrt(obs_frac * (1 - obs_frac) / n),
            .groups = "drop") %>%
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
               "Random Forest"  = "#2ca02c",
               "Grad. Boosted"  = "#d62728"),
    name = "Algorithm"
  ) +
  scale_x_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  scale_y_continuous(limits = c(0, 1), breaks = c(0, 0.25, 0.5, 0.75, 1)) +
  labs(x = "Mean predicted probability",
       y = "Observed fraction (positive class)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = c(0.84, 0.22),
        strip.text      = element_text(size = 13, face = "bold"),
        panel.spacing   = unit(0.4, "lines"))

save_fig(p5, "Figure5_calibration_curves", width = 11, height = 7)


# =============================================================================
# FIGURE 6 — SHAP importance (ionic facets + ionic feature labels)
# =============================================================================
cat("Figure 6...\n")

shap_raw <- dbGetQuery(con, "
  SELECT sv.feature_name, sv.shap_value,
         mr.target_contaminant, mr.algorithm, mr.predictor_tier
  FROM shap_values sv
  JOIN model_runs mr ON sv.run_id = mr.run_id
  WHERE mr.run_status = 'completed'
    AND mr.target_contaminant IN ('Na','Cl','TDS','B','F')
    AND mr.algorithm = 'RandomForest'
    AND mr.predictor_tier = 'Tier3_Full'
    AND mr.cv_mode = 'Stratified_Nested_CV'
")

shap_mean <- shap_raw %>%
  group_by(target_contaminant, feature_name) %>%
  summarise(mean_abs_shap = mean(abs(shap_value), na.rm = TRUE), .groups = "drop") %>%
  group_by(target_contaminant) %>%
  slice_max(mean_abs_shap, n = 10) %>%
  ungroup() %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = c("Na","Cl","TDS","B","F"))
  ) %>%
  group_by(target_contaminant) %>%
  mutate(feature_name = reorder(feature_name, mean_abs_shap)) %>%
  ungroup()

p6 <- ggplot(shap_mean, aes(x = mean_abs_shap, y = feature_name,
                             colour = target_contaminant)) +
  geom_segment(aes(xend = 0, yend = feature_name),
               linewidth = 1.5, alpha = 0.7) +
  geom_point(size = 5, alpha = 0.95) +
  scale_colour_brewer(palette = "Set1", name = "Target",
                      labels = pm_scale(ION_PM)) +
  scale_y_discrete(labels = pm_scale(FEAT_PM)) +
  scale_x_continuous(labels = function(x) sprintf("%.3f", x),
                     expand = expansion(mult = c(0.02, 0.08))) +
  labs(x = expression("|SHAP value|"~(mean~absolute)), y = NULL) +
  facet_wrap(~ target_contaminant, scales = "free_y", nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = "none",
        strip.text      = element_text(size = 14, face = "bold"),
        axis.text.y     = element_text(size = 13),
        axis.text.x     = element_text(size = 12))

save_fig(p6, "Figure6_SHAP_importance", width = 14, height = 9)


# =============================================================================
# FIGURE 7 — Screening priority map (ionic facet labels)
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
    priority = factor(screening_priority_class, levels = c("Low","Moderate","High"))
  )
clust_pts <- cluster_meta %>%
  rename(lon = centroid_longitude, lat = centroid_latitude) %>%
  mutate(label = paste0("C", spatial_cluster_id, "\n(n=", number_of_wells, ")"))

p7 <- ggplot(sp_map, aes(x = lon, y = lat)) +
  geom_point(aes(fill = priority, size = predicted_probability_median),
             shape = 21, colour = "white", stroke = 0.4, alpha = 0.88) +
  geom_point(data = clust_pts, aes(x = lon, y = lat),
             shape = 3, size = 4.5, colour = "black", stroke = 1.8,
             inherit.aes = FALSE) +
  geom_text_repel(data = clust_pts, aes(x = lon, y = lat, label = label),
                  size = 3.4, colour = "black", box.padding = 0.5,
                  segment.colour = "grey50", inherit.aes = FALSE) +
  scale_fill_manual(
    values = c(Low = "#27ae60", Moderate = "#e67e22", High = "#c0392b"),
    name = "Screening\npriority",
    guide = guide_legend(override.aes = list(size = 5, colour = "white"), order = 1)
  ) +
  scale_size_continuous(range = c(2.5, 8), name = "Median\nprobability",
                        breaks = c(0.2, 0.5, 0.8),
                        guide = guide_legend(override.aes = list(fill = "grey50",
                                                                  colour = "white"),
                                             order = 2)) +
  labs(x = "Longitude (°E)", y = "Latitude (°N)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = "right",
        strip.text      = element_text(size = 13, face = "bold"),
        panel.border    = element_rect(colour = "grey60", linewidth = 0.5),
        panel.spacing   = unit(0.4, "lines"),
        axis.text       = element_text(size = 11),
        legend.key      = element_rect(fill = "white", colour = NA))

save_fig(p7, "Figure7_screening_priority_map", width = 13, height = 8)


# =============================================================================
# SUPPLEMENTARY FIGURE S1 — Correlation matrix (ionic axes)
# =============================================================================
cat("FigS1...\n")

raw_df <- dbGetQuery(con, "
  SELECT cm.sample_id, cm.variable_name, cm.cleaned_value_real
  FROM cleaned_measurements cm
  WHERE cm.missing_flag = 0 AND cm.bdl_flag = 0
    AND cm.variable_name IN ('pH','Temp.','EC','Na','K','Mg','Ca',
                              'Cl','SO4','HCO3','CO3','F','B')
")

wide_data <- raw_df %>%
  pivot_wider(names_from = variable_name, values_from = cleaned_value_real) %>%
  select(-sample_id) %>% filter(complete.cases(.))

corr_mat  <- cor(wide_data, use = "pairwise.complete.obs", method = "spearman")
corr_long <- as.data.frame(as.table(corr_mat)) %>%
  rename(Var1 = Var1, Var2 = Var2, corr = Freq) %>%
  mutate(Var1 = factor(Var1, levels = colnames(corr_mat)),
         Var2 = factor(Var2, levels = rev(colnames(corr_mat))))

pS1 <- ggplot(corr_long, aes(x = Var1, y = Var2, fill = corr)) +
  geom_tile(colour = "white", linewidth = 0.4) +
  geom_text(aes(label = sprintf("%.2f", corr)),
            size = 3.2,
            colour = ifelse(abs(corr_long$corr) > 0.55, "white", "grey20")) +
  scale_fill_gradient2(low = "#3498db", mid = "white", high = "#e74c3c",
                       midpoint = 0, limits = c(-1, 1),
                       name = "Spearman\ncorrelation",
                       guide = guide_colourbar(barwidth = 1, barheight = 8)) +
  scale_x_discrete(labels = pm_scale(FEAT_PM)) +
  scale_y_discrete(labels = pm_scale(FEAT_PM)) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x  = element_text(size = 12, angle = 45, hjust = 1),
        axis.text.y  = element_text(size = 12),
        panel.border = element_blank(),
        axis.ticks   = element_blank(),
        legend.position = "right")

save_fig(pS1, "FigS1_correlation_matrix", width = 9, height = 8)


# =============================================================================
# SUPPLEMENTARY FIGURE S2 — Threshold sensitivity curves (ionic facets)
# =============================================================================
cat("FigS2...\n")

thresh_sens <- dbGetQuery(con, "
  SELECT ts.probability_cutoff, ts.sensitivity, ts.specificity, ts.f2_score,
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
  geom_vline(xintercept = 0.5, linetype = "dotted",
             colour = "grey50", linewidth = 0.8) +
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
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = c(0.82, 0.15),
        strip.text      = element_text(size = 13, face = "bold"),
        panel.spacing   = unit(0.4, "lines"))

save_fig(pS2, "FigS2_threshold_sensitivity_curves", width = 12, height = 7)


# =============================================================================
# SUPPLEMENTARY FIGURE S3 — Confusion matrices (ionic panel labels)
# =============================================================================
cat("FigS3...\n")

ot_data <- dbGetQuery(con, "
  SELECT ot.resulting_sensitivity, ot.resulting_specificity,
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
    n_pos = as.integer(round(false_negatives_count /
                               pmax(1 - resulting_sensitivity, 1e-6))),
    n_neg = as.integer(round(false_positives_count /
                               pmax(1 - resulting_specificity, 1e-6))),
    TP = n_pos - false_negatives_count,
    FN = false_negatives_count,
    FP = false_positives_count,
    TN = n_neg - false_positives_count,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm = factor(algorithm,
                       levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                       labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

best_model <- ot_data2 %>%
  group_by(target_contaminant) %>%
  slice_max(resulting_sensitivity, n = 1, with_ties = FALSE) %>%
  ungroup()

cm_long <- best_model %>%
  select(target_contaminant, algorithm, TP, FN, FP, TN,
         resulting_sensitivity, resulting_specificity) %>%
  pivot_longer(cols = c(TP, FN, FP, TN),
               names_to = "cell", values_to = "count") %>%
  mutate(
    actual    = ifelse(cell %in% c("TP","FN"), "Actual\nExceeds", "Actual\nBelow"),
    predicted = ifelse(cell %in% c("TP","FP"), "Predicted\nExceeds", "Predicted\nBelow"),
    actual    = factor(actual,    levels = c("Actual\nExceeds","Actual\nBelow")),
    predicted = factor(predicted, levels = c("Predicted\nExceeds","Predicted\nBelow"))
  )

sens_labels <- best_model %>%
  mutate(sens_label = sprintf("Sens=%.0f%%  Spec=%.0f%%",
                              resulting_sensitivity * 100,
                              resulting_specificity * 100))

cm_long <- cm_long %>%
  left_join(sens_labels %>% select(target_contaminant, algorithm, sens_label),
            by = c("target_contaminant","algorithm"))

pS3 <- ggplot(cm_long, aes(x = predicted, y = actual, fill = cell)) +
  geom_tile(colour = "white", linewidth = 1.2, width = 0.95, height = 0.95) +
  geom_text(aes(label = count), size = 8, fontface = "bold", colour = "white") +
  geom_text(aes(label = cell),  size = 4.5, colour = "white", nudge_y = -0.22) +
  geom_text(data = sens_labels,
            aes(x = 1.5, y = 2.58, label = sens_label),
            size = 3.6, colour = "grey20", fontface = "italic",
            inherit.aes = FALSE) +
  scale_fill_manual(
    values = c("TP" = "#27ae60","TN" = "#2980b9",
               "FP" = "#e67e22","FN" = "#c0392b"),
    guide = "none"
  ) +
  scale_y_discrete(expand = expansion(add = c(0.6, 0.7))) +
  facet_wrap(~ target_contaminant + algorithm, nrow = 2,
             labeller = labeller(
               target_contaminant = as_labeller(
                 function(x) ifelse(x %in% names(ION_PM), ION_PM[x], x),
                 label_parsed
               ),
               algorithm = label_value
             )) +
  labs(x = NULL, y = NULL,
       caption = "Counts are cumulative across all outer CV fold test sets (10 repeats × 5 folds).\nGreen = correct; Red/Orange = errors. Threshold rule: sensitivity ≥ 0.90 (Tier 3 Full model).") +
  nature_theme() +
  theme(axis.text.x   = element_text(size = 12, face = "bold"),
        axis.text.y   = element_text(size = 12, face = "bold"),
        strip.text    = element_text(size = 11, face = "bold"),
        panel.border  = element_rect(colour = "grey40", linewidth = 0.7),
        panel.spacing = unit(0.5, "lines"),
        axis.ticks    = element_blank(),
        plot.caption  = element_text(size = 10, colour = "grey40", hjust = 0))

save_fig(pS3, "FigS3_confusion_matrices", width = 15, height = 8)

dbDisconnect(con)
cat("\n=== Done. All figures updated with proper ionic notation. ===\n")
