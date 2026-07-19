# Fix Figure 1 (workflow) and Figure 7 (legend) + FigS3 (confusion matrix labels)

suppressPackageStartupMessages({
  library(RSQLite); library(DBI); library(ggplot2); library(dplyr)
  library(tidyr); library(patchwork); library(ggrepel); library(grid)
})

PROJECT_ROOT <- "C:/Users/DicksonAbdul-Wahab/Documents/Dr_Sukari_Manuscript/manuscript_4b/groundwater_virtual_screening"
DB_PATH <- file.path(PROJECT_ROOT, "outputs/groundwater_screening.db")
FIG_DIR <- file.path(PROJECT_ROOT, "manuscript/artifacts/figures")
con     <- dbConnect(SQLite(), DB_PATH)

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

TARGET_ORDER <- c("Na", "Cl", "TDS", "B", "F", "NO3")

# =============================================================================
# FIX FIGURE 1 — Cleaner flow diagram with correct arrow connections
# =============================================================================
cat("Fixing Figure 1...\n")

# Define a proper 4-column pipeline:
# Col A (x=1):  Data inputs
# Col B (x=3.5): Preprocessing
# Col C (x=6):  Model training & validation
# Col D (x=8.5): Outputs

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

# Arrows: from right-edge of source to left-edge of dest (same y, or angled)
# Standard horizontal arrows within same row
horiz_arrows <- data.frame(
  x1 = c(2.0, 2.0,   4.5, 4.5,   7.0, 7.0, 7.0),
  y1 = c(8,   5,     8,   5,     9.5, 7,   4.5),
  x2 = c(2.5, 2.5,   5.0, 5.0,   7.5, 7.5, 7.5),
  y2 = c(8,   5,     8,   5,     9.5, 7,   4.5)
)

# Diagonal/connecting arrows:
# Tier design (3.5,5) → Model Training (6,4.5): bend down
# Leakage (3.5,8) → Repeated Strat (6,9.5): slight angle up
# Also connect Repeated Strat/Spatial → Model Training via vertical connector
connector_arrows <- data.frame(
  x1 = c(3.5, 3.5,  6.0,  6.0),
  y1 = c(8,   5,    9.5,  7),
  x2 = c(3.5, 3.5,  6.0,  6.0),
  y2 = c(8.2, 5.2,  9.7,  7.2),
  type = c("v","v","v","v")  # placeholder, not used
)

# Short connectors: Strat CV & Spatial CV → Model Training (vertical then horizontal)
vert_seg <- data.frame(
  x1 = c(6,    6),
  y1 = c(9.5 - 0.8, 7 - 0.8),
  x2 = c(6,    6),
  y2 = c(4.5 + 0.8, 4.5 + 0.8)
)

# Two input boxes fan into the column
fan_arrows <- data.frame(
  x1 = c(6, 6),
  y1 = c(8.7, 6.2),
  x2 = c(6, 6),
  y2 = c(5.3, 5.3)
)

p1 <- ggplot() +
  # Box fills
  geom_rect(data = bx,
            aes(xmin = x - w/2, xmax = x + w/2,
                ymin = y - h/2, ymax = y + h/2, fill = fill),
            colour = "grey30", linewidth = 0.65, alpha = 0.95) +
  # Box labels
  geom_text(data = bx,
            aes(x = x, y = y, label = label),
            size = 3.1, lineheight = 1.15) +
  # Horizontal arrows between columns (same row)
  geom_segment(data = horiz_arrows,
               aes(x = x1, y = y1, xend = x2, yend = y2),
               arrow = arrow(length = unit(0.2, "cm"), type = "closed"),
               colour = "grey20", linewidth = 0.8) +
  # Vertical connector from box 5 (Stratified CV, y=9.5) to box 7 (Model, y=4.5)
  annotate("segment",
           x = 6.0, y = 8.7, xend = 6.0, yend = 5.3,
           colour = "grey35", linewidth = 0.7,
           arrow = arrow(length = unit(0.2, "cm"), type = "closed")) +
  # Vertical connector from box 6 (Spatial CV, y=7) also feeds model
  annotate("segment",
           x = 6.4, y = 7 - 0.8, xend = 6.4, yend = 5.3,
           colour = "grey35", linewidth = 0.7, linetype = "dashed") +
  annotate("segment",
           x = 6.4, y = 5.3, xend = 6.05, yend = 5.3,
           colour = "grey35", linewidth = 0.7,
           arrow = arrow(length = unit(0.15, "cm"), type = "closed")) +
  # Section labels
  annotate("text", x = 1,   y = 11, label = "INPUT",        size = 4.5, fontface = "bold", colour = "grey30") +
  annotate("text", x = 3.5, y = 11, label = "PREPROCESSING",size = 4.5, fontface = "bold", colour = "grey30") +
  annotate("text", x = 6,   y = 11, label = "MODELLING",     size = 4.5, fontface = "bold", colour = "grey30") +
  annotate("text", x = 8.5, y = 11, label = "OUTPUTS",        size = 4.5, fontface = "bold", colour = "grey30") +
  # Column divider lines
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


# =============================================================================
# FIX FIGURE 7 — Screening priority map with correct legend colours
# =============================================================================
cat("Fixing Figure 7...\n")

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

PRIORITY_COLS <- c(
  "Low"      = "#27ae60",
  "Moderate" = "#e67e22",
  "High"     = "#c0392b"
)

sp_map <- sp_map %>%
  mutate(
    target_contaminant = factor(target_contaminant, levels = TARGET_ORDER),
    priority           = factor(screening_priority_class,
                                levels = c("Low","Moderate","High"))
  )

clust_pts <- cluster_meta %>%
  rename(lon = centroid_longitude, lat = centroid_latitude) %>%
  mutate(label = paste0("C", spatial_cluster_id, "\n(n=", number_of_wells, ")"))

p7 <- ggplot(sp_map, aes(x = lon, y = lat)) +
  geom_point(aes(fill  = priority,
                 size  = predicted_probability_median),
             shape = 21, colour = "white", stroke = 0.4, alpha = 0.88) +
  geom_point(data = clust_pts,
             aes(x = lon, y = lat),
             shape = 3, size = 4.5, colour = "black", stroke = 1.8,
             inherit.aes = FALSE) +
  geom_text_repel(data = clust_pts,
                  aes(x = lon, y = lat, label = label),
                  size = 3.4, colour = "black",
                  box.padding = 0.5, point.padding = 0.3,
                  segment.colour = "grey50",
                  min.segment.length = 0.2,
                  inherit.aes = FALSE) +
  scale_fill_manual(
    values = PRIORITY_COLS,
    name   = "Screening\npriority",
    labels = c("Low","Moderate","High"),
    guide  = guide_legend(
      override.aes = list(size = 5, colour = "white", stroke = 0.4),
      order = 1
    )
  ) +
  scale_size_continuous(
    range  = c(2.5, 8),
    name   = "Median\nscreening\nprobability",
    breaks = c(0.2, 0.5, 0.8),
    labels = c("0.20", "0.50", "0.80"),
    guide  = guide_legend(
      override.aes = list(fill = "grey50", colour = "white"),
      order = 2
    )
  ) +
  labs(x = "Longitude (°E)", y = "Latitude (°N)") +
  facet_wrap(~ target_contaminant, nrow = 2, ncol = 3) +
  nature_theme() +
  theme(
    legend.position  = "right",
    strip.text       = element_text(size = 13, face = "bold"),
    panel.border     = element_rect(colour = "grey60", linewidth = 0.5),
    panel.spacing    = unit(0.4, "lines"),
    axis.text        = element_text(size = 11),
    legend.key       = element_rect(fill = "white", colour = NA),
    legend.background = element_rect(fill = "white", colour = "grey80")
  )

save_fig(p7, "Figure7_screening_priority_map", width = 13, height = 8)


# =============================================================================
# FIX FigS3 — Confusion matrices with fold-count note in title
# =============================================================================
cat("Fixing FigS3...\n")

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

# Find n_pos from FN + sensitivity (FN / (1 - sens))
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

# Select best (highest sensitivity) per target
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
    predicted = ifelse(cell %in% c("TP","FP"),
                       "Predicted\nExceeds", "Predicted\nBelow"),
    actual    = factor(actual, levels = c("Actual\nExceeds","Actual\nBelow")),
    predicted = factor(predicted, levels = c("Predicted\nExceeds","Predicted\nBelow")),
    cell_label = paste0(cell, "\n(n=", count, ")")
  )

# Subtitle with sensitivity info per panel
sens_labels <- best_model %>%
  mutate(
    panel_title = paste0(target_contaminant, " [", algorithm, "]"),
    sens_label  = sprintf("Sens=%.0f%%  Spec=%.0f%%",
                          resulting_sensitivity * 100,
                          resulting_specificity * 100)
  )

cm_long <- cm_long %>%
  left_join(sens_labels %>% select(target_contaminant, algorithm, sens_label),
            by = c("target_contaminant","algorithm"))

pS3 <- ggplot(cm_long, aes(x = predicted, y = actual, fill = cell)) +
  geom_tile(colour = "white", linewidth = 1.2, width = 0.95, height = 0.95) +
  geom_text(aes(label = count), size = 8, fontface = "bold", colour = "white") +
  geom_text(aes(label = cell),  size = 4.5, fontface = "plain", colour = "white",
            nudge_y = -0.22) +
  geom_text(data = sens_labels,
            aes(x = 1.5, y = 2.58, label = sens_label),
            size = 3.6, colour = "grey20", fontface = "italic", inherit.aes = FALSE) +
  scale_fill_manual(
    values = c("TP" = "#27ae60", "TN" = "#2980b9",
               "FP" = "#e67e22", "FN" = "#c0392b"),
    guide = "none"
  ) +
  scale_y_discrete(expand = expansion(add = c(0.6, 0.7))) +
  facet_wrap(~ target_contaminant + algorithm, nrow = 2,
             labeller = label_wrap_gen(multi_line = TRUE)) +
  labs(x = NULL, y = NULL,
       caption = "Counts are cumulative across all outer CV fold test sets (10 repeats × 5 folds).\nGreen = correct; Red/Orange = errors. Threshold rule: sensitivity ≥ 0.90 (Tier 3 Full model).") +
  nature_theme() +
  theme(axis.text.x    = element_text(size = 12, face = "bold"),
        axis.text.y    = element_text(size = 12, face = "bold"),
        strip.text     = element_text(size = 11, face = "bold"),
        panel.border   = element_rect(colour = "grey40", linewidth = 0.7),
        panel.spacing  = unit(0.5, "lines"),
        axis.ticks     = element_blank(),
        plot.caption   = element_text(size = 10, colour = "grey40", hjust = 0))

save_fig(pS3, "FigS3_confusion_matrices", width = 15, height = 8)

dbDisconnect(con)
cat("\nDone. Fixed: Figure1, Figure7, FigS3\n")
