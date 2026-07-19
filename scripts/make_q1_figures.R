suppressPackageStartupMessages({
  library(ggplot2)
  library(dplyr)
  library(tidyr)
  library(readr)
  library(scales)
  library(patchwork)
  library(viridis)
  library(ggrepel)
})

theme_nature <- function(base_size = 10, base_family = "Arial") {
  theme_minimal(base_size = base_size, base_family = base_family) +
    theme(
      text = element_text(colour = "black"),
      plot.title = element_text(face = "bold", size = base_size + 2, hjust = 0),
      plot.subtitle = element_text(size = base_size, colour = "grey20"),
      axis.title = element_text(face = "bold"),
      axis.text = element_text(colour = "black"),
      panel.grid.major = element_line(linewidth = 0.18, colour = "grey88"),
      panel.grid.minor = element_blank(),
      strip.text = element_text(face = "bold", colour = "black"),
      strip.background = element_rect(fill = "grey94", colour = NA),
      legend.title = element_text(face = "bold"),
      legend.key.height = unit(3.5, "mm"),
      legend.key.width = unit(4.5, "mm"),
      plot.margin = margin(5, 6, 5, 5)
    )
}

target_levels <- c("Na", "Cl", "TDS", "B", "F", "NO3")
tier_levels <- c(
  "Tier1_Field", "Tier2_Reduced", "Tier3_Full",
  "Tier2_Reduced_TDS_EC_inclusive", "Tier2_Reduced_TDS_EC_strict"
)
priority_levels <- c("Low", "Moderate", "High", "Very_High")
priority_cols <- c(
  Low = "#D9D9D9",
  Moderate = "#7FB3D5",
  High = "#E69F00",
  Very_High = "#D55E00"
)
cv_cols <- c(Stratified_Nested_CV = "#0072B2", Spatial_Group_CV = "#D55E00")

root <- normalizePath(".", winslash = "/", mustWork = TRUE)
in_dir <- file.path(root, "outputs", "r_exports")
gis_dir <- file.path(root, "outputs", "gis_exports")
out_dir <- file.path(root, "figures", "q1_risk_alert")
dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

num_cols <- function(df, cols) {
  for (cc in intersect(cols, names(df))) df[[cc]] <- as.numeric(df[[cc]])
  df
}

save_figure <- function(plot, name, width, height) {
  pdf_path <- file.path(out_dir, paste0(name, ".pdf"))
  tif_path <- file.path(out_dir, paste0(name, ".tiff"))
  png_path <- file.path(out_dir, paste0(name, ".png"))
  ggsave(pdf_path, plot, width = width, height = height, units = "in", device = cairo_pdf, bg = "white")
  ggsave(
    tif_path, plot, width = width, height = height, units = "in",
    dpi = 600, compression = "lzw", bg = "white"
  )
  ggsave(png_path, plot, width = width, height = height, units = "in", dpi = 300, bg = "white")
  tibble(file = basename(pdf_path), width_in = width, height_in = height) |>
    bind_rows(tibble(file = basename(tif_path), width_in = width, height_in = height)) |>
    bind_rows(tibble(file = basename(png_path), width_in = width, height_in = height))
}

prevalence <- read_csv(file.path(in_dir, "target_prevalence_summary.csv"), show_col_types = FALSE) |>
  num_cols(c("n_samples", "n_positive", "n_negative", "prevalence", "threshold_value")) |>
  mutate(
    target_contaminant = factor(target_contaminant, levels = target_levels),
    ml_status = recode(ml_status, modelled = "Modelled", descriptive_only = "Descriptive only")
  )

perf <- read_csv(file.path(in_dir, "model_performance_summary.csv"), show_col_types = FALSE) |>
  num_cols(c(
    "mean_roc_auc", "mean_pr_auc", "mean_bal_acc", "mean_recall", "mean_spec",
    "mean_f1", "mean_f2", "mean_brier", "n_folds", "min_pr_auc", "max_pr_auc"
  )) |>
  mutate(
    target_contaminant = factor(target_contaminant, levels = target_levels),
    predictor_tier = factor(predictor_tier, levels = tier_levels),
    cv_mode = factor(cv_mode, levels = c("Stratified_Nested_CV", "Spatial_Group_CV")),
    model_class = if_else(grepl("^Dummy", algorithm), "Baseline", "Model")
  )

priority <- read_csv(file.path(in_dir, "screening_priority_table.csv"), show_col_types = FALSE) |>
  num_cols(c(
    "predicted_probability_median", "predicted_probability_lower_ci",
    "predicted_probability_upper_ci", "latitude", "longitude"
  )) |>
  mutate(
    target_contaminant = factor(target_contaminant, levels = target_levels),
    screening_priority_class = factor(screening_priority_class, levels = priority_levels)
  )

gis_prob <- read_csv(file.path(gis_dir, "target_specific_screening_probabilities.csv"), show_col_types = FALSE) |>
  num_cols(c("predicted_probability_median", "longitude", "latitude", "spatial_cluster_id")) |>
  mutate(
    target_contaminant = factor(target_contaminant, levels = target_levels),
    screening_priority_class = factor(screening_priority_class, levels = priority_levels)
  )

calib <- read_csv(file.path(in_dir, "calibration_inputs.csv"), show_col_types = FALSE) |>
  num_cols(c("true_label", "predicted_probability")) |>
  mutate(target_contaminant = factor(target_contaminant, levels = target_levels))

shap <- read_csv(file.path(in_dir, "shap_values_long.csv"), show_col_types = FALSE) |>
  num_cols(c("shap_value", "feature_cleaned_value", "repeat_index", "fold_index")) |>
  mutate(target_contaminant = factor(target_contaminant, levels = target_levels))

best_runs <- perf |>
  filter(model_class == "Model") |>
  group_by(target_contaminant) |>
  arrange(desc(mean_pr_auc), mean_brier, .by_group = TRUE) |>
  slice(1) |>
  ungroup() |>
  select(target_contaminant, run_id, predictor_tier, algorithm, cv_mode)

manifest <- list()

# Figure 1: workflow schematic
workflow_nodes <- tibble(
  x = c(1, 2.35, 3.7, 5.05, 6.4, 7.75),
  y = c(1, 1, 1, 1, 1, 1),
  label = c(
    "Raw groundwater\nchemistry + coordinates",
    "75% WHO/Ghana\nrisk-alert labels",
    "Target-wise\nleakage control",
    "Fold-internal\npreprocessing",
    "Nested + spatial\ncross-validation",
    "Priority exports\n+ explainability"
  )
)
workflow_edges <- tibble(
  x = workflow_nodes$x[-nrow(workflow_nodes)] + 0.48,
  xend = workflow_nodes$x[-1] - 0.48,
  y = 1,
  yend = 1
)
fig1 <- ggplot() +
  geom_segment(
    data = workflow_edges,
    aes(x = x, xend = xend, y = y, yend = yend),
    arrow = arrow(length = unit(2.2, "mm")),
    linewidth = 0.35,
    colour = "grey25"
  ) +
  geom_label(
    data = workflow_nodes,
    aes(x = x, y = y, label = label),
    label.size = 0.25,
    label.r = unit(1.5, "mm"),
    fill = "white",
    size = 2.25,
    lineheight = 0.92
  ) +
  annotate(
    "text", x = 4.35, y = 0.55,
    label = "All scaling, imputation, class weighting, tuning, predictions and SHAP are fold-specific",
    size = 2.4, fontface = "italic", colour = "grey25"
  ) +
  coord_cartesian(xlim = c(0.35, 8.4), ylim = c(0.35, 1.35), clip = "off") +
  labs(title = "Leakage-controlled virtual screening architecture") +
  theme_void(base_family = "Arial", base_size = 8) +
  theme(plot.title = element_text(face = "bold", size = 10, hjust = 0))
manifest[[length(manifest) + 1]] <- save_figure(fig1, "Figure_1_workflow_architecture", 7.2, 2.0)

# Figure 2: prevalence and eligibility
fig2 <- prevalence |>
  mutate(
    target_contaminant = reorder(target_contaminant, prevalence),
    label = paste0(n_positive, "/", n_samples)
  ) |>
  ggplot(aes(x = target_contaminant, y = prevalence, fill = ml_status)) +
  geom_col(width = 0.68, colour = "black", linewidth = 0.2) +
  geom_text(aes(label = label), hjust = -0.08, size = 2.4) +
  coord_flip(ylim = c(0, max(prevalence$prevalence) * 1.18)) +
  scale_y_continuous(labels = percent_format(accuracy = 1)) +
  scale_fill_manual(values = c("Modelled" = "#0072B2", "Descriptive only" = "grey65")) +
  labs(
    title = "Risk-alert prevalence at 75% of drinking-water guideline values",
    x = NULL, y = "Wells above risk-alert threshold", fill = NULL
  ) +
  theme_nature()
manifest[[length(manifest) + 1]] <- save_figure(fig2, "Figure_2_risk_alert_prevalence", 3.6, 3.2)

# Figure 3: performance heatmap
heat <- perf |>
  filter(model_class == "Model", !is.na(predictor_tier)) |>
  group_by(target_contaminant, predictor_tier, cv_mode) |>
  summarise(best_pr_auc = max(mean_pr_auc, na.rm = TRUE), .groups = "drop") |>
  filter(!is.infinite(best_pr_auc))

fig3 <- ggplot(heat, aes(x = predictor_tier, y = target_contaminant, fill = best_pr_auc)) +
  geom_tile(colour = "white", linewidth = 0.35) +
  geom_text(aes(label = sprintf("%.2f", best_pr_auc)), size = 2.2, colour = "black") +
  facet_wrap(~cv_mode, ncol = 1) +
  scale_fill_viridis_c(option = "C", limits = c(0, 1), name = "Best PR-AUC") +
  scale_x_discrete(labels = c(
    Tier1_Field = "Tier 1\nfield",
    Tier2_Reduced = "Tier 2\nreduced",
    Tier3_Full = "Tier 3\nfull",
    Tier2_Reduced_TDS_EC_inclusive = "TDS EC\ninclusive",
    Tier2_Reduced_TDS_EC_strict = "TDS EC\nstrict"
  )) +
  labs(
    title = "Risk-alert screenability across targets and predictor tiers",
    x = NULL, y = NULL
  ) +
  theme_nature() +
  theme(axis.text.x = element_text(angle = 0, hjust = 0.5))
manifest[[length(manifest) + 1]] <- save_figure(fig3, "Figure_3_performance_heatmap", 7.2, 5.8)

# Figure 4: spatial validation penalty
penalty <- perf |>
  filter(model_class == "Model") |>
  group_by(target_contaminant, predictor_tier, algorithm, cv_mode) |>
  summarise(pr_auc = max(mean_pr_auc, na.rm = TRUE), .groups = "drop") |>
  pivot_wider(names_from = cv_mode, values_from = pr_auc) |>
  filter(!is.na(Stratified_Nested_CV), !is.na(Spatial_Group_CV)) |>
  mutate(delta_spatial_minus_stratified = Spatial_Group_CV - Stratified_Nested_CV) |>
  group_by(target_contaminant) |>
  arrange(delta_spatial_minus_stratified, .by_group = TRUE) |>
  slice(1) |>
  ungroup()

fig4a <- penalty |>
  ggplot(aes(x = target_contaminant, y = delta_spatial_minus_stratified, fill = delta_spatial_minus_stratified >= 0)) +
  geom_hline(yintercept = 0, linewidth = 0.3, colour = "grey35") +
  geom_col(width = 0.65, colour = "black", linewidth = 0.2) +
  scale_fill_manual(values = c("TRUE" = "#0072B2", "FALSE" = "#D55E00"), guide = "none") +
  scale_y_continuous(labels = number_format(accuracy = 0.01)) +
  labs(title = "Worst observed spatial validation penalty", x = NULL, y = "Spatial PR-AUC minus stratified PR-AUC") +
  theme_nature()

paired <- perf |>
  filter(model_class == "Model") |>
  group_by(target_contaminant, cv_mode) |>
  arrange(desc(mean_pr_auc), mean_brier, .by_group = TRUE) |>
  slice(1) |>
  ungroup()

fig4b <- ggplot(paired, aes(x = cv_mode, y = mean_pr_auc, group = target_contaminant)) +
  geom_line(colour = "grey55", linewidth = 0.35) +
  geom_point(aes(colour = target_contaminant), size = 2) +
  scale_colour_viridis_d(option = "D", end = 0.88, name = "Target") +
  scale_x_discrete(labels = c(Stratified_Nested_CV = "Stratified", Spatial_Group_CV = "Spatial")) +
  coord_cartesian(ylim = c(0, 1)) +
  labs(title = "Best model by validation mode", x = NULL, y = "PR-AUC") +
  theme_nature()

fig4 <- fig4a + fig4b + plot_layout(widths = c(1, 1))
manifest[[length(manifest) + 1]] <- save_figure(fig4, "Figure_4_spatial_validation_penalty", 7.2, 3.2)

# Figure 5: calibration curves
best_calib <- best_runs |>
  inner_join(calib, by = c("run_id", "target_contaminant")) |>
  mutate(bin = pmin(10, pmax(1, ceiling(predicted_probability * 10)))) |>
  group_by(target_contaminant, bin) |>
  summarise(
    mean_pred = mean(predicted_probability, na.rm = TRUE),
    observed = mean(true_label, na.rm = TRUE),
    n = n(),
    .groups = "drop"
  )

fig5 <- ggplot(best_calib, aes(x = mean_pred, y = observed)) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed", linewidth = 0.3, colour = "grey40") +
  geom_line(aes(colour = target_contaminant), linewidth = 0.45) +
  geom_point(aes(size = n, colour = target_contaminant), alpha = 0.9) +
  facet_wrap(~target_contaminant, ncol = 3) +
  scale_colour_viridis_d(option = "D", end = 0.88, guide = "none") +
  scale_size_continuous(range = c(1.2, 3.6), guide = "none") +
  coord_equal(xlim = c(0, 1), ylim = c(0, 1)) +
  labs(
    title = "Calibration of selected risk-alert screening models",
    x = "Mean predicted probability", y = "Observed risk-alert fraction"
  ) +
  theme_nature()
manifest[[length(manifest) + 1]] <- save_figure(fig5, "Figure_5_calibration_reliability", 7.2, 4.8)

# Figure 6: SHAP behaviour diagnostics
reorder_within <- function(x, by, within, fun = mean, sep = "___", ...) {
  new_x <- paste(x, within, sep = sep)
  stats::reorder(new_x, by, FUN = fun)
}
scale_x_reordered <- function(..., sep = "___") {
  reg <- paste0(sep, ".+$")
  scale_x_discrete(labels = function(x) gsub(reg, "", x), ...)
}
shap_top <- best_runs |>
  inner_join(shap, by = c("run_id", "target_contaminant")) |>
  group_by(target_contaminant, feature_name) |>
  summarise(mean_abs_shap = mean(abs(shap_value), na.rm = TRUE), .groups = "drop") |>
  group_by(target_contaminant) |>
  slice_max(mean_abs_shap, n = 6, with_ties = FALSE) |>
  ungroup() |>
  mutate(feature_name_reordered = reorder_within(feature_name, mean_abs_shap, target_contaminant))

fig6 <- ggplot(shap_top, aes(x = feature_name_reordered, y = mean_abs_shap, fill = target_contaminant)) +
  geom_col(width = 0.72, colour = "black", linewidth = 0.18) +
  coord_flip() +
  facet_wrap(~target_contaminant, scales = "free_y") +
  scale_x_reordered() +
  scale_fill_viridis_d(option = "D", end = 0.88, guide = "none") +
  labs(
    title = "Top fold-aggregated SHAP contributors in selected models",
    subtitle = "Interpreted as model-behaviour diagnostics, not causal effects",
    x = NULL, y = "Mean absolute SHAP value"
  ) +
  theme_nature()
manifest[[length(manifest) + 1]] <- save_figure(fig6, "Figure_6_shap_diagnostics", 7.2, 4.5)

# Figure 7: screening-priority map
labels_map <- gis_prob |>
  group_by(sample_id) |>
  slice_max(predicted_probability_median, n = 1, with_ties = FALSE) |>
  ungroup() |>
  slice_max(predicted_probability_median, n = 8, with_ties = FALSE)

fig7 <- ggplot(gis_prob, aes(x = longitude, y = latitude)) +
  geom_point(
    aes(fill = screening_priority_class, size = predicted_probability_median),
    shape = 21, colour = "black", stroke = 0.18, alpha = 0.88
  ) +
  geom_text_repel(
    data = labels_map,
    aes(label = sample_id),
    size = 2.1,
    min.segment.length = 0,
    segment.size = 0.18,
    box.padding = 0.18,
    seed = 20260531
  ) +
  facet_wrap(~target_contaminant, ncol = 3) +
  scale_fill_manual(values = priority_cols, drop = FALSE, name = "Priority") +
  scale_size_continuous(range = c(1.2, 4.1), limits = c(0, 1), guide = "none") +
  coord_equal() +
  labs(
    title = "Well-level screening-priority map",
    subtitle = "Map shows model-derived priority for confirmatory testing, not measured contamination",
    x = "Longitude", y = "Latitude"
  ) +
  theme_nature()
manifest[[length(manifest) + 1]] <- save_figure(fig7, "Figure_7_screening_priority_map", 7.2, 5.6)

figure_manifest <- bind_rows(manifest)
write_csv(figure_manifest, file.path(out_dir, "figure_manifest.csv"))

message("Figures written to: ", out_dir)
