# =============================================================================
# generate_revision_figures.R
# Regenerates ALL publication figures for the revised manuscript from the
# revised reproducibility database (outputs/rev2/main/main.db).
#
# Terminology: exact path-dependent (cover-weighted) TreeSHAP values (Lundberg et al., 2018)
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
# pm_scale MUST parse: ggplot discrete scales/legends accept an expression
# vector. Returning the bare plotmath string (the previous behaviour) printed
# literal "Na^'+'" on every axis and legend while facet strips - which go
# through pm_labeller/label_parsed - rendered correctly.
pm_scale <- function(map) function(x) parse(text = ifelse(x %in% names(map), map[x], x))
pm_labeller <- function(map) as_labeller(function(x) ifelse(x %in% names(map), map[x], x), label_parsed)

# ── Colour-blind-safe palettes (Okabe & Ito 2008) ───────────────────────────
OKABE <- c(blue = "#0072B2", skyblue = "#56B4E9", green = "#009E73",
           orange = "#E69F00", vermillion = "#D55E00", purple = "#CC79A7",
           yellow = "#F0E442", grey = "#7F7F7F", black = "#000000")
TARGET_COL <- c("Na" = "#0072B2", "Cl" = "#56B4E9", "TDS" = "#009E73",
                "B" = "#CC79A7", "F" = "#E69F00")
ALGO_COL <- c("Logistic Reg." = "#0072B2", "Random Forest" = "#009E73",
              "Grad. Boosted" = "#D55E00")
# Sequential, luminance-ordered risk ramp (ColorBrewer YlOrRd, 4-class):
# ordered by lightness so it survives greyscale and all colour-vision types.
PRIORITY_COL <- c("Low" = "#FFFFB2", "Moderate" = "#FECC5C",
                  "High" = "#FD8D3C", "Very high" = "#E31A1C")

# ── Print geometry ──────────────────────────────────────────────────────────
# Figures are authored at their FINAL printed size so that the absolute point
# sizes below are the sizes the reader actually sees. Previously figures were
# drawn 279-406 mm wide and then reduced ~0.42-0.6x at typesetting, which put
# body text at roughly 4.5-6.2 pt - under the >=7 pt most publishers require.
W2 <- 7.09   # double-column, 180 mm
W1 <- 3.54   # single-column, 90 mm
# geom_text/geom_label size is in mm; convert a desired point size.
gs <- function(pt) pt / .pt

nature_theme <- function(base_size = 8) {
  theme_classic(base_size = base_size) +
    theme(
      plot.background   = element_rect(fill = "white", colour = NA),
      panel.background  = element_rect(fill = "white", colour = NA),
      panel.border      = element_rect(colour = "black", fill = NA, linewidth = 0.4),
      axis.text         = element_text(size = 7, colour = "black"),
      axis.title        = element_text(size = 8, colour = "black"),
      legend.text       = element_text(size = 7),
      legend.title      = element_text(size = 7.5, face = "bold"),
      legend.background = element_rect(fill = "white", colour = "grey80",
                                       linewidth = 0.3),
      legend.key        = element_rect(fill = "white"),
      legend.key.size   = unit(0.85, "lines"),
      strip.text        = element_text(size = 7.5, face = "bold"),
      strip.background  = element_rect(fill = "grey92", colour = "black",
                                       linewidth = 0.4),
      axis.ticks        = element_line(colour = "black", linewidth = 0.3),
      panel.spacing     = unit(0.9, "lines")
    )
}
save_fig <- function(p, name, width = W2, height = 5) {
  path <- file.path(FIG_DIR, paste0(name, ".png"))
  ggsave(path, plot = p, width = width, height = height, dpi = 600,
         bg = "white", units = "in")
  message("  Saved: ", basename(path), sprintf("  (%.0f x %.0f mm)",
          width * 25.4, height * 25.4))
}

cat("=== Generating revised figures ===\n\n")

# =============================================================================
# FIGURE 1 — Workflow schematic (static, updated for the revised design)
# =============================================================================
cat("Figure 1...\n")
# Boxes are drawn as rectangles in DATA units so that the connector geometry is
# exact. The previous version used geom_label (absolute sizing) with arrows at
# fixed data-unit offsets, so every arrow was drawn through the following box's
# text, and an arrow was emitted for the last box too - pointing at nothing.
# Box pitch is 1.0, so bw + 2*gap_arrow MUST stay below 1.0; otherwise the
# connector's start overtakes its end and the arrow renders backwards.
flow_panel <- function(df, fill, ncol = 4, bw = 0.80, bh = 0.40,
                       text_pt = 6.4, gap_arrow = 0.035) {
  stopifnot(bw + 2 * gap_arrow < 1)
  n <- nrow(df)
  df$col <- (seq_len(n) - 1) %% ncol
  df$row <- (seq_len(n) - 1) %/% ncol
  df$x <- df$col + 0.5
  df$y <- -df$row
  # Within-row connectors: only between horizontally adjacent boxes.
  same_row <- df$row[-n] == df$row[-1]
  seg_h <- data.frame(
    x    = df$x[-n][same_row] + bw / 2 + gap_arrow,
    xend = df$x[-1][same_row] - bw / 2 - gap_arrow,
    y    = df$y[-n][same_row],
    yend = df$y[-1][same_row]
  )
  # Row-to-row elbow: right edge of last box in a row, down, back to the left.
  elbows <- which(!same_row)
  seg_e <- do.call(rbind, lapply(elbows, function(i) {
    x0 <- df$x[i]; y0 <- df$y[i]; x1 <- df$x[i + 1]; y1 <- df$y[i + 1]
    ymid <- (y0 + y1) / 2
    data.frame(x    = c(x0, x0, x1),
               xend = c(x0, x1, x1),
               y    = c(y0 - bh / 2, ymid, ymid),
               yend = c(ymid, ymid, y1 + bh / 2 + gap_arrow))
  }))
  ggplot() +
    geom_segment(data = seg_h, aes(x = x, xend = xend, y = y, yend = yend),
                 arrow = arrow(length = unit(0.055, "in"), type = "closed"),
                 linewidth = 0.4, colour = "grey25") +
    { if (!is.null(seg_e)) list(
        geom_segment(data = seg_e[seq_len(nrow(seg_e)) %% 3 != 0, ],
                     aes(x = x, xend = xend, y = y, yend = yend),
                     linewidth = 0.4, colour = "grey25"),
        geom_segment(data = seg_e[seq_len(nrow(seg_e)) %% 3 == 0, ],
                     aes(x = x, xend = xend, y = y, yend = yend),
                     arrow = arrow(length = unit(0.055, "in"), type = "closed"),
                     linewidth = 0.4, colour = "grey25")) } +
    geom_rect(data = df, aes(xmin = x - bw / 2, xmax = x + bw / 2,
                             ymin = y - bh / 2, ymax = y + bh / 2),
              fill = fill, colour = "black", linewidth = 0.4) +
    geom_text(data = df, aes(x = x, y = y, label = label),
              size = gs(text_pt), lineheight = 1.05, colour = "black") +
    coord_cartesian(xlim = c(-0.06, ncol + 0.06),
                    ylim = c(-max(df$row) - 0.42, 0.42)) +
    theme_void() +
    theme(plot.margin = margin(3, 3, 3, 3))
}

box_df <- data.frame(
  label = c(
    "81 georeferenced\nwell samples\n(Sunkari et al., 2021)",
    "Six binary precautionary\nscreening labels\n(study-specific thresholds)",
    "Target-wise\nleakage filtering\n(Table 2)",
    "Projected k-means\nspatial clusters (k = 4)\n(equirectangular)",
    "Repeated nested CV\n(5 repeats × 5 folds)\n+ inner-CV tuning",
    "Spatial-group CV\n(3 held-out blocks)\n+ pooled evaluation",
    "Exact path-dependent\nTreeSHAP values\n(Lundberg et al., 2018)",
    "Well-level median\nOOF probabilities\n+ screening-priority classes"
  ),
  stringsAsFactors = FALSE
)
p1 <- flow_panel(box_df, fill = "grey95", ncol = 4)
save_fig(p1, "Figure1_workflow_schematic", width = W2, height = 2.5)

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
    pct_positive       = sprintf("%.1f", prevalence * 100),
    # Drop trailing zeros rather than printing "150.000 mg/L" / "750.000 mg/L".
    threshold_label    = paste0(formatC(threshold_value, format = "fg",
                                        digits = 6, drop0trailing = TRUE),
                                " mg/L")
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
            inherit.aes = FALSE, size = gs(7), fontface = "bold", colour = "grey20") +
  geom_text(data = elig,
            aes(x = target_contaminant, y = -4, label = threshold_label),
            inherit.aes = FALSE, size = gs(6), colour = "grey30", fontface = "italic") +
  annotate("text", x = 0.4, y = -4, label = "Threshold:",
           size = gs(6), colour = "grey30", fontface = "bold", hjust = 1) +
  scale_fill_manual(values = c("Exceeds threshold" = "#D55E00",
                               "Below threshold" = "#9ECAE1"), name = NULL) +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  scale_y_continuous(limits = c(-7, 90), breaks = c(0, 20, 40, 60, 80), expand = c(0, 0)) +
  labs(x = "Groundwater quality parameter", y = "Number of wells (n = 81)") +
  nature_theme() +
  theme(legend.position = "right", axis.text.x = element_text(size = 8, face = "bold"),
        axis.ticks.x = element_blank())
save_fig(p2, "Figure2_exceedance_prevalence", width = W2, height = 3.4)

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
# Viridis replaces the previous blue-green-yellow-red spectral ramp, which is
# perceptually non-uniform and not colour-vision safe. Label colour now flips
# with cell luminance - white-on-yellow was effectively illegible.
p3 <- ggplot(perf, aes(x = algorithm, y = target_contaminant, fill = pr_auc)) +
  geom_tile(colour = "white", linewidth = 0.4) +
  geom_text(aes(label = sprintf("%.2f", pr_auc),
                colour = pr_auc > 0.62), size = gs(6.5), fontface = "bold") +
  scale_colour_manual(values = c(`TRUE` = "grey10", `FALSE` = "white"),
                      guide = "none") +
  scale_fill_viridis_c(option = "viridis", limits = c(0, 1), name = "PR-AUC",
                       guide = guide_colourbar(barwidth = unit(0.35, "cm"),
                                               barheight = unit(3.2, "cm"),
                                               title.position = "top",
                                               title.hjust = 0.5)) +
  scale_y_discrete(labels = pm_scale(ION_PM)) +
  facet_wrap(~ predictor_tier, nrow = 1) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x = element_text(size = 6.5),
        axis.text.y = element_text(size = 8, face = "bold"),
        legend.position = "right",
        panel.border = element_rect(colour = "grey60", linewidth = 0.4))
save_fig(p3, "Figure3_performance_heatmap_PR_AUC", width = W2, height = 3.2)

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
  geom_point(size = 1.7, alpha = 0.85, stroke = 0.4) +
  scale_colour_manual(values = TARGET_COL, name = "Target",
                      labels = pm_scale(ION_PM)) +
  scale_shape_manual(values = c(16, 17, 15), name = "Algorithm") +
  scale_x_continuous(limits = c(0.2, 1.03), breaks = c(0.25, 0.5, 0.75, 1.0)) +
  scale_y_continuous(limits = c(0.2, 1.03), breaks = c(0.25, 0.5, 0.75, 1.0)) +
  labs(x = "PR-AUC (repeated stratified CV, pooled OOF)",
       y = "PR-AUC (spatial-group CV, pooled OOF)") +
  annotate("text", x = 0.86, y = 0.47, label = "below the line:\nspatial CV penalised",
           size = gs(6), colour = "grey40", hjust = 0.5, lineheight = 1.05) +
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
  geom_col(position = position_dodge(0.75), width = 0.65, colour = "white", linewidth = 0.25) +
  geom_errorbar(aes(ymin = mean_penalty - se_penalty, ymax = mean_penalty + se_penalty),
                position = position_dodge(0.75), width = 0.25, linewidth = 0.35) +
  geom_hline(yintercept = 0, linetype = "solid", colour = "grey30", linewidth = 0.4) +
  scale_fill_manual(values = c("T1" = "#0072B2", "T2" = "#009E73", "T3" = "#D55E00"),
                    name = "Predictor tier") +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  labs(x = "Target parameter",
       y = expression(Delta * "PR-AUC (Stratified " * "\u2212" * " Spatial CV)")) +
  nature_theme() +
  theme(legend.position = "top", legend.direction = "horizontal",
        axis.text.x = element_text(size = 8, face = "bold"))
p4 <- pA + pB + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 9, face = "bold"))
save_fig(p4, "Figure4_spatial_cv_penalty", width = W2, height = 3.3)

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
                width = 0.035, alpha = 0.5, linewidth = 0.3) +
  geom_line(linewidth = 0.6, alpha = 0.85) +
  geom_point(size = 1.3, alpha = 0.9) +
  scale_colour_manual(values = ALGO_COL, name = "Algorithm") +
  # Three breaks with short labels: the previous five-break axis collided at
  # every panel boundary, printing "1.000.00".
  scale_x_continuous(limits = c(0, 1), breaks = c(0, 0.5, 1),
                     labels = c("0", "0.5", "1")) +
  scale_y_continuous(limits = c(0, 1), breaks = c(0, 0.5, 1),
                     labels = c("0", "0.5", "1")) +
  labs(x = "Mean predicted probability", y = "Observed fraction (positive class)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3, labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = c(0.84, 0.24))
save_fig(p5, "Figure5_calibration_curves", width = W2, height = 4.6)

# =============================================================================
# FIGURE 6 — SHAP feature importance (exact path-dependent (cover-weighted) TreeSHAP)
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
                                     levels = c("Na", "Cl", "TDS", "B", "F")))

# reorder() is NOT group-aware: called inside group_by() it produced ONE global
# factor level order (the Na panel's), which every other facet then inherited -
# so Cl, TDS, B and F displayed the wrong feature ranking. facet "free_y" frees
# the range, not the level order. Build a per-target ordering key instead
# (equivalent to tidytext::reorder_within, which is not installed here).
attr_mean <- attr_mean %>%
  arrange(target_contaminant, mean_abs_shap) %>%
  mutate(row_key = factor(paste(target_contaminant, feature_name, sep = "___"),
                          levels = paste(target_contaminant, feature_name,
                                         sep = "___")))
strip_key <- function(x) {
  f <- sub("^.*___", "", x)
  parse(text = ifelse(f %in% names(FEAT_PM), FEAT_PM[f], f))
}
p6 <- ggplot(attr_mean, aes(x = mean_abs_shap, y = row_key,
                            colour = target_contaminant)) +
  geom_segment(aes(xend = 0, yend = row_key), linewidth = 0.7, alpha = 0.7) +
  geom_point(size = 1.9, alpha = 0.95) +
  scale_colour_manual(values = TARGET_COL, name = "Target",
                      labels = pm_scale(ION_PM)) +
  scale_y_discrete(labels = strip_key) +
  scale_x_continuous(labels = function(x) sprintf("%.2f", x),
                     expand = expansion(mult = c(0.02, 0.10))) +
  labs(x = expression("|SHAP value|" ~ (mean ~ absolute)), y = NULL) +
  facet_wrap(~ target_contaminant, scales = "free", nrow = 2, ncol = 3,
             labeller = pm_labeller(ION_PM)) +
  nature_theme() +
  theme(legend.position = "none",
        axis.text.y = element_text(size = 6.5), axis.text.x = element_text(size = 6.5))
save_fig(p6, "Figure6_SHAP_importance", width = W2, height = 4.9)

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
# Spatial clusters are identical across targets, so the centroids are drawn in
# the first panel only. Repeating them in all five panels added 20 labels that
# collided with the well symbols and carried no extra information.
clust_pts <- cluster_meta %>%
  rename(lon = centroid_longitude, lat = centroid_latitude) %>%
  mutate(label = paste0("C", spatial_cluster_id, " (n=", number_of_wells, ")"),
         target_contaminant = factor("Na", levels = TARGET_ORDER))

# Map furniture. Latitude/longitude degrees are not equal distances, so the
# panel aspect is fixed at 1/cos(lat) - previously the map was stretched.
LAT0 <- mean(range(sp_map$lat, na.rm = TRUE))
KM_PER_DEG_LON <- 111.320 * cos(LAT0 * pi / 180)
BAR_KM <- 20
bar_deg <- BAR_KM / KM_PER_DEG_LON
xr <- range(sp_map$lon, na.rm = TRUE); yr <- range(sp_map$lat, na.rm = TRUE)
bar_x0 <- xr[1] + 0.02 * diff(xr); bar_y <- yr[1] - 0.055 * diff(yr)
scale_bar <- data.frame(x = bar_x0, xend = bar_x0 + bar_deg, y = bar_y, yend = bar_y)
n_arrow <- data.frame(x = xr[2] - 0.02 * diff(xr),
                      y = yr[2] - 0.16 * diff(yr),
                      yend = yr[2] - 0.02 * diff(yr))
deg_lon <- function(x) paste0(formatC(abs(x), format = "f", digits = 1), "°W")
deg_lat <- function(x) paste0(formatC(x, format = "f", digits = 1), "°N")

p7 <- ggplot(sp_map, aes(x = lon, y = lat)) +
  geom_point(aes(fill = priority, size = predicted_probability_median),
             shape = 21, colour = "grey25", stroke = 0.25, alpha = 0.9) +
  geom_point(data = clust_pts, aes(x = lon, y = lat), shape = 3, size = 1.8,
             colour = "black", stroke = 0.7, inherit.aes = FALSE) +
  geom_text_repel(data = clust_pts, aes(x = lon, y = lat, label = label),
                  size = gs(5.5), colour = "black", box.padding = 0.6,
                  point.padding = 0.3, min.segment.length = 0,
                  max.overlaps = Inf, seed = 42,
                  segment.colour = "grey45", inherit.aes = FALSE) +
  # scale bar + north arrow (drawn once, in the first panel)
  geom_segment(data = transform(scale_bar, target_contaminant = factor("Na", levels = TARGET_ORDER)),
               aes(x = x, xend = xend, y = y, yend = yend),
               linewidth = 0.7, colour = "black", inherit.aes = FALSE) +
  geom_text(data = transform(scale_bar, target_contaminant = factor("Na", levels = TARGET_ORDER)),
            aes(x = (x + xend) / 2, y = y, label = paste0(BAR_KM, " km")),
            vjust = -0.55, size = gs(5.5), inherit.aes = FALSE) +
  geom_segment(data = transform(n_arrow, target_contaminant = factor("Na", levels = TARGET_ORDER)),
               aes(x = x, xend = x, y = y, yend = yend),
               arrow = arrow(length = unit(0.05, "in"), type = "closed"),
               linewidth = 0.5, colour = "black", inherit.aes = FALSE) +
  geom_text(data = transform(n_arrow, target_contaminant = factor("Na", levels = TARGET_ORDER)),
            aes(x = x, y = yend, label = "N"), vjust = -0.35,
            size = gs(6), fontface = "bold", inherit.aes = FALSE) +
  scale_fill_manual(values = PRIORITY_COL, name = "Screening priority",
                    guide = guide_legend(override.aes = list(size = 2.6), order = 1)) +
  scale_size_continuous(range = c(1.1, 3.6), name = "Median probability",
                        breaks = c(0.2, 0.5, 0.8),
                        guide = guide_legend(override.aes = list(fill = "grey60"),
                                             order = 2)) +
  scale_x_continuous(labels = deg_lon) +
  scale_y_continuous(labels = deg_lat) +
  coord_fixed(ratio = 1 / cos(LAT0 * pi / 180), clip = "off") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3, labeller = pm_labeller(ION_PM)) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  # The 3 x 2 grid holds five panels; the legends occupy the empty sixth slot
  # rather than a separate right-hand column.
  theme(legend.position = "inside",
        legend.position.inside = c(0.84, 0.25),
        legend.spacing.y = unit(0.25, "cm"))
save_fig(p7, "Figure7_screening_priority_map", width = W2, height = 4.8)

# =============================================================================
# FIGURE 8 — End-user decision flowchart (static)
# =============================================================================
cat("Figure 8...\n")
step_df <- data.frame(
  label = c("Step 1\nChoose target\n(Na, Cl, TDS, B, F)",
            "Step 2\nChoose tier\n(1 field, 2 reduced,\n3 full chemistry)",
            "Step 3\nCheck leakage\nfilter (Table 2)",
            "Step 4\nApply model\n(tuned in inner CV)",
            "Step 5\nApply target-specific\nprobability cutoff\n(Table 5)",
            "Step 6\nClassify priority\n(Low / Moderate /\nHigh / Very high)",
            "Step 7\nPrioritise wells for\nconfirmatory\nlaboratory testing"),
  stringsAsFactors = FALSE
)
p8 <- flow_panel(step_df, fill = "#FFFCE0", ncol = 4)
save_fig(p8, "Figure8_enduser_decision_flowchart", width = W2, height = 2.5)

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
  geom_text(aes(label = sprintf("%.2f", rho)), size = gs(5), colour = "grey15") +
  scale_fill_gradient2(low = "#2166ac", mid = "white", high = "#b2182b",
                       midpoint = 0, limits = c(-1, 1),
                       name = expression(Spearman~rho)) +
  scale_x_discrete(labels = pm_scale(FEAT_PM)) +
  scale_y_discrete(labels = pm_scale(FEAT_PM)) +
  labs(x = NULL, y = NULL) +
  nature_theme() +
  theme(axis.text.x = element_text(angle = 45, hjust = 1, size = 7),
        axis.text.y = element_text(size = 7),
        panel.border = element_blank())
save_fig(pS1, "FigS1_correlation_matrix", width = W2, height = 5.8)

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
# Sensitivity and F2 previously used red and green - the two curves that
# overlap and cross most - which deuteranopes cannot separate. Okabe-Ito
# vermillion/bluish-green/blue is safe for all common colour-vision types.
pS2 <- ggplot(ts_long, aes(x = probability_cutoff, y = value,
                           colour = metric, linetype = metric)) +
  geom_line(linewidth = 0.55) +
  geom_vline(xintercept = 0.5, linetype = "dotted", colour = "grey45",
             linewidth = 0.35) +
  scale_colour_manual(values = c("Sensitivity" = "#D55E00",
                                 "Specificity" = "#0072B2",
                                 "F2 score" = "#009E73"), name = NULL,
                      labels = parse(text = c("Sensitivity", "Specificity", "F[2]~score"))) +
  scale_linetype_manual(values = c("Sensitivity" = "solid",
                                   "Specificity" = "22",
                                   "F2 score" = "solid"), name = NULL,
                        labels = parse(text = c("Sensitivity", "Specificity", "F[2]~score"))) +
  scale_x_continuous(breaks = c(0, 0.5, 1), labels = c("0", "0.5", "1")) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Probability cutoff (dotted line = 0.50 default)", y = "Value") +
  nature_theme() +
  theme(legend.position = "bottom")
save_fig(pS2, "FigS2_threshold_sensitivity_curves", width = W2, height = 2.6)

# =============================================================================
# FIGURE S3 — Well-level confusion matrices at the operating cutoff
# =============================================================================
cat("Figure S3...\n")
# Read the SAME table the manuscript prints as Table 5, rather than
# re-deriving a "best model" here. The previous version selected by well-level
# F2 and picked gradient-boosted trees for every target, while Table 5 selects
# by pooled PR-AUC (EC-only logistic for Cl, random forest for B). The two
# disagreed: FigS3 showed Cl as 15/0/4/62 against Table 5's 14/1/0/66. Reading
# one source makes divergence impossible.
TAB_DIR <- file.path(PROJECT_ROOT, "outputs/rev2/tables")
best_wlo <- read.csv(file.path(TAB_DIR, "Table4_operational_thresholds.csv"),
                     stringsAsFactors = FALSE) %>%
  # Headline model of each target in the fully nested, unique-well Table 5.
  filter((target_contaminant %in% c("Na", "Cl") & predictor_tier == "EC_only_baseline") |
         (target_contaminant == "TDS" & predictor_tier == "Tier1_Field") |
         (target_contaminant %in% c("B", "F") & predictor_tier == "Tier3_Full")) %>%
  mutate(cutoff = round(cutoff_mean, 2))
stopifnot(nrow(best_wlo) == 5,
          with(best_wlo, all(tp_count + tn_count + fp_count + fn_count == 81)))
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
# Fill now encodes the COUNT. Previously fill was a flat correct/error colour,
# so a cell holding 0 carried exactly the same visual weight as one holding 68.
# Error cells are marked by an outline instead.
pS3 <- ggplot(cm_long, aes(x = predicted, y = actual)) +
  geom_tile(aes(fill = count), colour = "white", linewidth = 0.5) +
  geom_tile(data = subset(cm_long, is_err), fill = NA, colour = "#D55E00",
            linewidth = 0.8) +
  geom_text(aes(label = count, colour = count > max(count) * 0.55),
            size = gs(8), fontface = "bold") +
  scale_colour_manual(values = c(`TRUE` = "white", `FALSE` = "grey15"),
                      guide = "none") +
  scale_fill_viridis_c(option = "mako", direction = -1, name = "Wells",
                       guide = guide_colourbar(barwidth = unit(0.3, "cm"),
                                               barheight = unit(2.4, "cm"))) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Predicted (at well-level operating cutoff)",
       y = "Observed",
       caption = "Orange outline = misclassified cell") +
  nature_theme() +
  theme(axis.text = element_text(size = 7),
        plot.caption = element_text(size = 6.5, colour = "grey35", hjust = 0))
save_fig(pS3, "FigS3_confusion_matrices", width = W2, height = 2.4)

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
# Restructured from 15 facets in a single row (each ~11 mm wide at print, with
# "with coordinates"/"without coordinates" repeated 30 times) to a 2 x 5 grid
# with tier on the x axis and the paired with/without values joined by a line,
# so the ablation effect is read directly. EC-only logistic is excluded: it
# never uses coordinates, so its "with"/"without" values are identical by
# construction and contributed 30 flat, uninformative points.
coord_pair <- coord %>%
  filter(algorithm != "EcOnlyLogistic") %>%
  mutate(algorithm = factor(algorithm,
                            levels = c("LogisticRegression", "RandomForest",
                                       "GradientBoostedTrees"),
                            labels = c("Logistic Reg.", "Random Forest",
                                       "Grad. Boosted")),
         cv_mode = factor(cv_mode,
                          levels = c("Stratified_Nested_CV", "Spatial_Group_CV"),
                          labels = c("Stratified nested CV", "Spatial-group CV")),
         coords = factor(coords, levels = c("with coordinates",
                                            "without coordinates"),
                         labels = c("with", "without")))
# Explicit numeric offsets rather than position_dodge(): geom_line and
# geom_point dodge on different groupings, which left every connector floating
# beside the pair it was meant to join.
ALGO_OFF <- c("Logistic Reg." = -0.24, "Random Forest" = 0, "Grad. Boosted" = 0.24)
COORD_OFF <- c("with" = -0.06, "without" = 0.06)
coord_pair <- coord_pair %>%
  mutate(tier_i = as.integer(factor(base_tier, levels = c("Tier 1", "Tier 2", "Tier 3"))),
         x_num  = tier_i + ALGO_OFF[as.character(algorithm)] +
                  COORD_OFF[as.character(coords)])
pS4 <- ggplot(coord_pair, aes(x = x_num, y = pr_auc, colour = algorithm)) +
  geom_line(aes(group = interaction(algorithm, tier_i)),
            linewidth = 0.35, alpha = 0.65) +
  geom_point(aes(shape = coords), size = 1.5, alpha = 0.95) +
  scale_colour_manual(values = ALGO_COL, name = "Algorithm") +
  scale_shape_manual(values = c("with" = 16, "without" = 1),
                     name = "Coordinates") +
  scale_x_continuous(breaks = 1:3, labels = c("Tier 1", "Tier 2", "Tier 3"),
                     limits = c(0.55, 3.45)) +
  facet_grid(cv_mode ~ target_contaminant,
             labeller = labeller(target_contaminant = pm_labeller(ION_PM))) +
  labs(x = "Predictor tier", y = "PR-AUC (pooled OOF)") +
  nature_theme() +
  theme(axis.text.x = element_text(size = 6.5),
        legend.position = "bottom", legend.box = "horizontal")
save_fig(pS4, "FigS4_coordinate_ablation", width = W2, height = 4.0)

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
  geom_histogram(bins = 12, fill = "#0072B2", colour = "white", linewidth = 0.3) +
  geom_vline(xintercept = 37.5, linetype = "dashed", colour = "#D55E00",
             linewidth = 0.5) +
  # hjust = 1 keeps the annotation inside the panel; it previously ran off the
  # right edge.
  annotate("text", x = 36, y = Inf, label = "Screening threshold 37.5 mg/L",
           hjust = 1, vjust = 1.6, size = gs(6), colour = "#D55E00") +
  labs(x = expression(NO[3]^"-" * " (mg/L)"), y = "Wells") +
  nature_theme()
pS5b <- ggplot(no3, aes(x = lon, y = lat, colour = bdl)) +
  geom_point(aes(size = value), alpha = 0.8) +
  scale_colour_manual(values = c("TRUE" = "#D55E00", "FALSE" = "#0072B2"),
                      name = "Below detection\nlimit", labels = c("FALSE" = "No", "TRUE" = "Yes")) +
  scale_size_continuous(range = c(0.8, 3.4), name = expression(NO[3]^"-" * " (mg/L)")) +
  scale_x_continuous(labels = deg_lon) +
  scale_y_continuous(labels = deg_lat) +
  coord_fixed(ratio = 1 / cos(LAT0 * pi / 180)) +
  labs(x = NULL, y = NULL) +
  nature_theme()
pS5 <- pS5a + pS5b + plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 9, face = "bold"))
save_fig(pS5, "FigS5_NO3_descriptive", width = W2, height = 2.9)

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
                       labels = c("F1", "F1.5", "F2", "F3")),
         target_contaminant = factor(target_contaminant, levels = TARGET_ORDER))
# beta is ordinal, so an ordered (viridis) ramp is used rather than four
# unrelated hues; it is also colour-vision safe.
pS6 <- ggplot(fb_long, aes(x = probability_cutoff, y = score, colour = beta)) +
  geom_line(linewidth = 0.55) +
  scale_colour_viridis_d(option = "viridis", end = 0.9, name = NULL,
                         labels = parse(text = c("F[1]~(beta==1)",
                                                 "F[1.5]~(beta==1.5)",
                                                 "F[2]~(beta==2)",
                                                 "F[3]~(beta==3)"))) +
  scale_x_continuous(breaks = c(0, 0.5, 1), labels = c("0", "0.5", "1")) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Probability cutoff", y = expression(F[beta]~score)) +
  nature_theme() +
  theme(legend.position = "bottom")
save_fig(pS6, "FigS6_beta_sensitivity", width = W2, height = 2.6)

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
RULE_COL <- c("Pooled (post-hoc)" = "#0072B2",
              "Nested (fold-local cutoffs)" = "#D55E00")
pS7a <- ggplot(op_plot, aes(x = resulting_sensitivity, y = resulting_specificity,
                            colour = rule, shape = algorithm)) +
  geom_point(size = 1.6, alpha = 0.9) +
  scale_colour_manual(values = RULE_COL, name = NULL) +
  scale_shape_manual(values = c("EcOnlyLogistic" = 4, "LogisticRegression" = 16,
                                "RandomForest" = 17, "GradientBoostedTrees" = 15),
                     labels = c("EcOnlyLogistic" = "EC-only logistic",
                                "LogisticRegression" = "Logistic Reg.",
                                "RandomForest" = "Random Forest",
                                "GradientBoostedTrees" = "Grad. Boosted"),
                     name = NULL) +
  # The default data-driven breaks collapsed into an unreadable run of digits
  # ("0.8250.8580.8750.9000.926") in every panel.
  scale_x_continuous(breaks = scales::pretty_breaks(n = 3)) +
  facet_wrap(~ target_contaminant, nrow = 1, ncol = 5, labeller = pm_labeller(ION_PM)) +
  labs(x = "Sensitivity at operating cutoff", y = "Specificity at operating cutoff") +
  nature_theme() +
  theme(legend.position = "bottom", legend.box = "vertical",
        legend.spacing.y = unit(0.1, "lines"))
# Panel B previously mapped colour to TARGET while panel A mapped the same
# red/blue to RULE - one figure, one palette, two meanings. Target now sits on
# the x axis so a single colour legend serves both panels.
pS7b <- ggplot(op_plot, aes(x = target_contaminant, y = optimized_probability_cutoff,
                            colour = rule)) +
  geom_point(size = 1.6, alpha = 0.85,
             position = position_dodge(width = 0.5)) +
  scale_colour_manual(values = RULE_COL, name = NULL, guide = "none") +
  scale_x_discrete(labels = pm_scale(ION_PM)) +
  labs(x = "Target", y = "Probability cutoff") +
  nature_theme()
pS7 <- pS7a + pS7b + plot_layout(widths = c(2.1, 1)) +
  plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 9, face = "bold"))
save_fig(pS7, "FigS7_nested_vs_pooled_cutoffs", width = W2, height = 3.0)

dbDisconnect(con)
cat("\n=== All revised figures written to", FIG_DIR, "===\n")
