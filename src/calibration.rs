use crate::metrics::compute_calibration;

/// Platt scaling: fit a logistic on top of model log-odds to recalibrate.
/// Returns calibrated probabilities.
#[allow(dead_code)]
pub fn platt_scale(y_true: &[i32], y_prob: &[f64]) -> Vec<f64> {
    use crate::utils::{logit, sigmoid};

    let logits: Vec<f64> = y_prob
        .iter()
        .map(|&p| logit(p.clamp(1e-6, 1.0 - 1e-6)))
        .collect();

    // Fit a = slope, b = intercept via gradient descent on cross-entropy
    let mut a = 1.0f64;
    let mut b = 0.0f64;
    let lr = 0.01;
    let n = y_true.len() as f64;

    for _ in 0..500 {
        let mut da = 0.0;
        let mut db = 0.0;
        for (i, (&yi, &xi)) in y_true.iter().zip(logits.iter()).enumerate() {
            let p = sigmoid(a * xi + b);
            let err = p - yi as f64;
            da += err * xi / n;
            db += err / n;
            let _ = i;
        }
        a -= lr * da;
        b -= lr * db;
    }

    y_prob
        .iter()
        .zip(logits.iter())
        .map(|(_, &xi)| sigmoid(a * xi + b).clamp(1e-6, 1.0 - 1e-6))
        .collect()
}

/// Hosmer-Lemeshow test statistic (chi-squared, 10 bins).
/// Returns (statistic, degrees_of_freedom).
#[allow(dead_code)]
pub fn hosmer_lemeshow(y_true: &[i32], y_prob: &[f64]) -> (f64, usize) {
    let n_bins = 10;
    let n = y_true.len();
    if n < n_bins {
        return (0.0, n_bins - 2);
    }

    let mut pairs: Vec<(f64, i32)> = y_prob.iter().copied().zip(y_true.iter().copied()).collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let bin_size = n / n_bins;
    let mut hl = 0.0f64;

    for b in 0..n_bins {
        let start = b * bin_size;
        let end = if b == n_bins - 1 { n } else { start + bin_size };
        let bin = &pairs[start..end];
        let nb = bin.len() as f64;
        let obs_pos: f64 = bin.iter().filter(|p| p.1 == 1).count() as f64;
        let exp_pos: f64 = bin.iter().map(|p| p.0).sum::<f64>();
        if exp_pos > 0.0 && nb - exp_pos > 0.0 {
            hl += (obs_pos - exp_pos).powi(2) / exp_pos
                + ((nb - obs_pos) - (nb - exp_pos)).powi(2) / (nb - exp_pos);
        }
    }
    (hl, n_bins - 2)
}

/// Compute calibration slope and intercept (logit regression).
#[allow(dead_code)]
pub fn calibration_stats(y_true: &[i32], y_prob: &[f64]) -> (f64, f64) {
    compute_calibration(y_true, y_prob)
}
