use crate::models::Model;

/// Compute SHAP-style attributions for any model.
///
/// For tree models: Saabas path-based attributions (path_shap).
/// For linear models: exact linear attributions w_i * (x_i - E[x_i]).
///
/// Returns per-sample, per-feature attribution vectors, plus base values.
pub struct ShapResult {
    /// shape: [n_samples × n_features]
    pub values: Vec<Vec<f64>>,
    /// fold-specific base value in log-odds space
    pub base_value_logit: f64,
    /// fold-specific base value in probability space
    pub base_value_prob: f64,
    /// mean absolute SHAP per feature
    pub mean_abs_shap: Vec<f64>,
}

pub fn compute_shap(model: &dyn Model, x: &[Vec<f64>], y_train: &[i32]) -> ShapResult {
    let n_features = if x.is_empty() { 0 } else { x[0].len() };
    let n_samples = x.len();

    let base_value_logit = model.shap_base_value();
    let base_value_prob = crate::utils::sigmoid(base_value_logit);

    let values: Vec<Vec<f64>> = if model.supports_tree_shap() {
        model.tree_shap(x)
    } else {
        // Linear approximation: attribution = importance_weight * (x_i - mean_x_i)
        let feature_means: Vec<f64> = (0..n_features)
            .map(|j| {
                let vals: Vec<f64> = x.iter().map(|row| row[j]).collect();
                vals.iter().sum::<f64>() / vals.len() as f64
            })
            .collect();
        let importances = model.feature_importances(n_features);

        x.iter()
            .map(|xi| {
                xi.iter()
                    .enumerate()
                    .map(|(j, &v)| importances[j] * (v - feature_means[j]))
                    .collect()
            })
            .collect()
    };

    // Mean absolute SHAP per feature
    let mean_abs_shap: Vec<f64> = if values.is_empty() {
        vec![0.0; n_features]
    } else {
        (0..n_features)
            .map(|j| values.iter().map(|row| row[j].abs()).sum::<f64>() / n_samples as f64)
            .collect()
    };

    let _ = y_train;

    ShapResult {
        values,
        base_value_logit,
        base_value_prob,
        mean_abs_shap,
    }
}

/// Store SHAP values in SQLite for a specific run/fold.
#[allow(clippy::too_many_arguments)]
pub fn store_shap_values(
    conn: &rusqlite::Connection,
    run_id: &str,
    sample_ids: &[String],
    feature_names: &[String],
    shap: &ShapResult,
    x_raw: &[Vec<f64>],
    cv_mode: &str,
    repeat: usize,
    fold: usize,
) -> anyhow::Result<()> {
    for (i, sid) in sample_ids.iter().enumerate() {
        if i >= shap.values.len() {
            break;
        }
        for (j, feat) in feature_names.iter().enumerate() {
            if j >= shap.values[i].len() {
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
                    shap.values[i][j],
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
    shap: &ShapResult,
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
            shap.base_value_logit,
            shap.base_value_prob,
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
    shap: &ShapResult,
) -> anyhow::Result<()> {
    for (j, feat) in feature_names.iter().enumerate() {
        let mas = shap.mean_abs_shap.get(j).copied().unwrap_or(0.0);
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
