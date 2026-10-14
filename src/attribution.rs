use crate::models::Model;

/// Compute exact SHAP values for any model (Lundberg & Lee, 2017).
///
/// Tree models: exact path-dependent (cover-weighted) TreeSHAP values (Lundberg et al., 2018)
/// of the tree's cover-weighted conditional expectation (node training counts
/// of the fitted trees; no explicit background data set); the
/// decomposition is exactly additive (base value + sum of SHAP values = model
/// output).
///
/// Linear models: exact SHAP values for the standardised logistic regression,
/// phi_j = w_j * z_j with base = bias, so sum_j phi_j = logit(x) - bias
/// exactly (the training background has zero mean on standardised features).
pub struct AttributionResult {
    /// shape: [n_samples × n_features]
    pub values: Vec<Vec<f64>>,
    /// base value in log-odds space
    pub base_value_logit: f64,
    /// base value in probability space
    pub base_value_prob: f64,
    /// mean absolute SHAP per feature
    pub mean_abs_shap: Vec<f64>,
}

pub fn compute_shap(
    model: &dyn Model,
    x: &[Vec<f64>],
    class_weight: f64,
) -> AttributionResult {
    let n_features = if x.is_empty() { 0 } else { x[0].len() };
    let n_samples = x.len();

    let base_value_logit = model.attribution_base_logit();
    let base_value_prob = crate::utils::sigmoid(base_value_logit);

    let values: Vec<Vec<f64>> = if model.supports_tree_attribution() {
        model.tree_attributions(x, class_weight)
    } else if let Some(weights) = model.linear_weights(n_features) {
        // Exact additive logit decomposition on the standardised features:
        // phi_j = w_j * z_j, base = bias.
        x.iter()
            .map(|xi| {
                weights
                    .iter()
                    .zip(xi.iter())
                    .map(|(w, z)| w * z)
                    .collect::<Vec<f64>>()
            })
            .collect()
    } else {
        vec![vec![0.0; n_features]; n_samples]
    };

    // Mean absolute SHAP per feature
    let mean_abs_shap: Vec<f64> = if values.is_empty() {
        vec![0.0; n_features]
    } else {
        (0..n_features)
            .map(|j| values.iter().map(|row| row[j].abs()).sum::<f64>() / n_samples as f64)
            .collect()
    };

    AttributionResult {
        values,
        base_value_logit,
        base_value_prob,
        mean_abs_shap,
    }
}

/// Store attribution values in SQLite for a specific run/fold.
#[allow(clippy::too_many_arguments)]
pub fn store_shap_values(
    conn: &rusqlite::Connection,
    run_id: &str,
    sample_ids: &[String],
    feature_names: &[String],
    attr: &AttributionResult,
    x_raw: &[Vec<f64>],
    cv_mode: &str,
    repeat: usize,
    fold: usize,
) -> anyhow::Result<()> {
    for (i, sid) in sample_ids.iter().enumerate() {
        if i >= attr.values.len() {
            break;
        }
        for (j, feat) in feature_names.iter().enumerate() {
            if j >= attr.values[i].len() {
                break;
            }
            let raw_val = x_raw.get(i).and_then(|row| row.get(j)).copied();
            conn.execute(
                "INSERT INTO shap_values
                 (run_id, sample_id, cv_mode, repeat_index, fold_index,
                  feature_name, shap_value, feature_raw_value, feature_cleaned_value)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    run_id,
                    sid,
                    cv_mode,
                    repeat as i64,
                    fold as i64,
                    feat,
                    attr.values[i][j],
                    raw_val,
                    raw_val,
                ],
            )?;
        }
    }
    Ok(())
}

/// Store fold base values.
pub fn store_fold_base_values(
    conn: &rusqlite::Connection,
    run_id: &str,
    cv_mode: &str,
    repeat: usize,
    fold: usize,
    attr: &AttributionResult,
    train_prevalence: f64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO fold_base_values
         (run_id, cv_mode, repeat_index, fold_index, shap_base_value_logit,
          shap_base_value_prob, train_fold_prevalence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            run_id,
            cv_mode,
            repeat as i64,
            fold as i64,
            attr.base_value_logit,
            attr.base_value_prob,
            train_prevalence,
        ],
    )?;
    Ok(())
}

/// Store fold feature tracking.
pub fn store_fold_feature_tracking(
    conn: &rusqlite::Connection,
    run_id: &str,
    cv_mode: &str,
    repeat: usize,
    fold: usize,
    feature_names: &[String],
    attr: &AttributionResult,
) -> anyhow::Result<()> {
    for (j, feat) in feature_names.iter().enumerate() {
        let mas = attr.mean_abs_shap.get(j).copied().unwrap_or(0.0);
        conn.execute(
            "INSERT INTO fold_feature_tracking
             (run_id, cv_mode, repeat_index, fold_index, feature_name,
              is_selected, mean_absolute_shap, selection_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, 'included_by_leakage_filter')",
            rusqlite::params![run_id, cv_mode, repeat as i64, fold as i64, feat, mas,],
        )?;
    }
    Ok(())
}
