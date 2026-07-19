# Minor revisions:
#   Fig2  — legend moved outside the plot area (right side)
#   Fig4  — Panel B legend moved to lower-left
#   Fig1  — all text sizes +15%

suppressPackageStartupMessages({
  library(RSQLite); library(DBI); library(ggplot2); library(dplyr)
  library(tidyr); library(patchwork); library(grid)
})

PROJECT_ROOT <- "C:/Users/DicksonAbdul-Wahab/Documents/Dr_Sukari_Manuscript/manuscript_4b/groundwater_virtual_screening"
DB_PATH <- file.path(PROJECT_ROOT, "outputs/groundwater_screening.db")
FIG_DIR <- file.path(PROJECT_ROOT, "manuscript/artifacts/figures")
con     <- dbConnect(SQLite(), DB_PATH)
TARGET_ORDER <- c("Na", "Cl", "TDS", "B", "F", "NO3")

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
  message("Saved: ", path)
}

# =============================================================================
# FIX 1 — Figure 2: legend outside the plot area (right)
# =============================================================================
cat("Figure 2 – legend outside...\n")

elig <- dbGetQuery(con, "SELECT * FROM target_eligibility")
tdef <- dbGetQuery(con, "SELECT target_contaminant, threshold_value, threshold_unit FROM target_definitions")

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
            aes(x = target_contaminant, y = -4,
                label = threshold_label),
            inherit.aes = FALSE, size = 3.8, colour = "grey30", fontface = "italic") +
  annotate("text", x = 0.4, y = -4, label = "Threshold:",
           size = 3.8, colour = "grey30", fontface = "bold", hjust = 1) +
  scale_fill_manual(
    values = c("Exceeds threshold" = "#d62728",
               "Below threshold"   = "#aec7e8"),
    name = NULL
  ) +
  scale_y_continuous(limits = c(-7, 90), breaks = c(0, 20, 40, 60, 80),
                     expand = c(0, 0)) +
  labs(x = "Groundwater quality parameter",
       y = "Number of wells (n = 81)") +
  nature_theme() +
  theme(
    # Legend outside, positioned to the right of the panel
    legend.position  = "right",
    legend.direction = "vertical",
    legend.margin    = margin(0, 0, 0, 8),
    axis.text.x      = element_text(size = 14, face = "bold"),
    axis.ticks.x     = element_blank()
  )

# Slightly wider to accommodate outside legend
save_fig(p2, "Figure2_exceedance_prevalence", width = 9, height = 5.5)


# =============================================================================
# FIX 2 — Figure 4: Panel B legend at lower-left
# =============================================================================
cat("Figure 4 – Panel B legend lower-left...\n")

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
  pivot_wider(names_from = cv_mode,
              values_from = c(pr_auc, recall, f2_score)) %>%
  rename(
    pr_auc_strat   = `pr_auc_Stratified_Nested_CV`,
    pr_auc_spatial = `pr_auc_Spatial_Group_CV`
  ) %>%
  filter(!is.na(pr_auc_strat) & !is.na(pr_auc_spatial)) %>%
  mutate(
    penalty            = pr_auc_strat - pr_auc_spatial,
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    algorithm          = factor(algorithm,
                                levels = c("LogisticRegression","RandomForest","GradientBoostedTrees"),
                                labels = c("Logistic Reg.","Random Forest","Grad. Boosted"))
  )

# Panel A — unchanged
pA <- ggplot(perf_wide, aes(x = pr_auc_strat, y = pr_auc_spatial,
                             colour = target_contaminant, shape = algorithm)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed",
              colour = "grey50", linewidth = 0.8) +
  geom_point(size = 3.5, alpha = 0.85, stroke = 0.5) +
  scale_colour_brewer(palette = "Set1", name = "Target") +
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
  theme(legend.position = "right",
        legend.margin   = margin(0, 0, 0, 0))

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

# Panel B — legend moved to lower-left
pB <- ggplot(pen_summary,
             aes(x = target_contaminant, y = mean_penalty, fill = tier_label)) +
  geom_col(position = position_dodge(0.75), width = 0.65,
           colour = "white", linewidth = 0.3) +
  geom_errorbar(aes(ymin = mean_penalty - se_penalty,
                    ymax = mean_penalty + se_penalty),
                position = position_dodge(0.75), width = 0.25, linewidth = 0.6) +
  geom_hline(yintercept = 0, linetype = "solid",
             colour = "grey30", linewidth = 0.6) +
  scale_fill_manual(
    values = c("T1" = "#3498db", "T2" = "#2ecc71", "T3" = "#e74c3c"),
    name   = "Predictor\ntier"
  ) +
  scale_y_continuous(labels = function(x) sprintf("%.2f", x)) +
  labs(x = "Target parameter",
       y = expression(Delta * "PR-AUC (Stratified − Spatial CV)")) +
  nature_theme() +
  theme(
    legend.position  = c(0.12, 0.18),   # lower-left
    legend.margin    = margin(4, 6, 4, 6),
    axis.text.x      = element_text(size = 14, face = "bold")
  )

p4 <- pA + pB +
  plot_annotation(tag_levels = "A") &
  theme(plot.tag = element_text(size = 14, face = "bold"))

save_fig(p4, "Figure4_spatial_cv_penalty", width = 13, height = 5.5)


# =============================================================================
# FIX 3 — Figure 1: all text sizes +15%
# =============================================================================
cat("Figure 1 – text sizes +15%...\n")

# Scale factor
S <- 1.15
base_box  <- round(3.1 * S, 2)   # 3.57
base_head <- round(4.5 * S, 2)   # 5.18

bx <- data.frame(
  id    = 1:10,
  x     = c( 1,    1,    3.5,  3.5,  6,    6,    6,   8.5,  8.5,  8.5),
  y     = c( 8,    5,    8,    5,    9.5,  7,    4.5, 9.5,  7,    4.5),
  w     = c( 2.0, 2.0,  2.0,  2.0,  2.0,  2.0,  2.0, 2.0,  2.0,  2.0),
  h     = c( 1.6, 1.6,  1.6,  1.6,  1.6,  1.6,  1.6, 1.6,  1.6,  1.6),
  label = c(
    "Raw Groundwater\nData (n = 81)\npH, EC, TDS, Ions\nCoordinates",
    "Target Definition\n6 binary outcomes\n(75% of WHO\ndrinking-water threshold)",
    "Leakage Control\nTarget-wise exclusion\nof direct + derived\npredictor variables",
    "Predictor Tier Design\nTier 1: Field (5 vars)\nTier 2: Reduced (12 vars)\nTier 3: Full (20+ vars)",
    "Repeated Stratified\nNested CV\n10 repeats × 5 folds\n(random splits)",
    "Spatial Block CV\n4 geographic clusters\n(spatial independence\ntest)",
    "Model Training\nLogistic Regression\nRandom Forest\nGradient Boosted Trees",
    "Performance Metrics\nPR-AUC · Recall\nF2-score · Brier\nCalibration slope",
    "SHAP Explainability\nFold-level values\nHydrochemical\nalignment check",
    "Screening Priority\nMap (3-class)\nSQLite audit trail\nGIS-ready export"
  ),
  fill  = c("input","input","preproc","preproc","cv","cv","model","output","output","output"),
  stringsAsFactors = FALSE
)

fill_map <- c(
  input   = "#D6EAF8",
  preproc = "#D5F5E3",
  cv      = "#E8DAEF",
  model   = "#FDEBD0",
  output  = "#D5D8DC"
)

horiz_arrows <- data.frame(
  x1 = c(2.0, 2.0,   4.5, 4.5,   7.0, 7.0, 7.0),
  y1 = c(8,   5,     8,   5,     9.5, 7,   4.5),
  x2 = c(2.5, 2.5,   5.0, 5.0,   7.5, 7.5, 7.5),
  y2 = c(8,   5,     8,   5,     9.5, 7,   4.5)
)

p1 <- ggplot() +
  geom_rect(data = bx,
            aes(xmin = x - w/2, xmax = x + w/2,
                ymin = y - h/2, ymax = y + h/2, fill = fill),
            colour = "grey30", linewidth = 0.65, alpha = 0.95) +
  geom_text(data = bx,
            aes(x = x, y = y, label = label),
            size = base_box, lineheight = 1.15) +
  geom_segment(data = horiz_arrows,
               aes(x = x1, y = y1, xend = x2, yend = y2),
               arrow = arrow(length = unit(0.2, "cm"), type = "closed"),
               colour = "grey20", linewidth = 0.8) +
  # Strat CV → Model Training (solid, vertical drop)
  annotate("segment",
           x = 6.0, y = 8.7, xend = 6.0, yend = 5.3,
           colour = "grey35", linewidth = 0.7,
           arrow = arrow(length = unit(0.2, "cm"), type = "closed")) +
  # Spatial CV → Model Training (dashed side connector)
  annotate("segment",
           x = 6.4, y = 7 - 0.8, xend = 6.4, yend = 5.3,
           colour = "grey35", linewidth = 0.7, linetype = "dashed") +
  annotate("segment",
           x = 6.4, y = 5.3, xend = 6.05, yend = 5.3,
           colour = "grey35", linewidth = 0.7,
           arrow = arrow(length = unit(0.15, "cm"), type = "closed")) +
  # Column headers (×S)
  annotate("text", x = 1,   y = 11, label = "INPUT",
           size = base_head, fontface = "bold", colour = "grey30") +
  annotate("text", x = 3.5, y = 11, label = "PREPROCESSING",
           size = base_head, fontface = "bold", colour = "grey30") +
  annotate("text", x = 6,   y = 11, label = "MODELLING",
           size = base_head, fontface = "bold", colour = "grey30") +
  annotate("text", x = 8.5, y = 11, label = "OUTPUTS",
           size = base_head, fontface = "bold", colour = "grey30") +
  # Divider lines
  annotate("segment", x = 2.25, y = 3.3, xend = 2.25, yend = 10.7,
           colour = "grey70", linewidth = 0.4, linetype = "dotted") +
  annotate("segment", x = 4.75, y = 3.3, xend = 4.75, yend = 10.7,
           colour = "grey70", linewidth = 0.4, linetype = "dotted") +
  annotate("segment", x = 7.25, y = 3.3, xend = 7.25, yend = 10.7,
           colour = "grey70", linewidth = 0.4, linetype = "dotted") +
  scale_fill_manual(values = fill_map, guide = "none") +
  scale_x_continuous(limits = c(-0.1, 9.6), expand = c(0, 0)) +
  scale_y_continuous(limits = c(3.2, 11.5), expand = c(0, 0)) +
  theme_void() +
  theme(plot.background  = element_rect(fill = "white", colour = NA),
        panel.background = element_rect(fill = "white", colour = NA),
        plot.margin      = margin(5, 5, 5, 5))

save_fig(p1, "Figure1_workflow_schematic", width = 12, height = 7)

dbDisconnect(con)
cat("\nDone. Revised: Figure1, Figure2, Figure4\n")
