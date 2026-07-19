/// All classification metrics for fold-level evaluation.

#[derive(Debug, Clone, Default)]
pub struct MetricBundle {
    pub roc_auc: f64,
    pub pr_auc: f64,
    pub balanced_accuracy: f64,
    pub recall_sensitivity: f64,
    pub specificity: f64,
    pub f1_score: f64,
    pub f2_score: f64,
    pub brier_score: f64,
    pub calibration_slope: f64,
    pub calibration_intercept: f64,
    pub n_pos: usize,
    pub n_neg: usize,
}

pub fn compute_all_metrics(y_true: &[i32], y_prob: &[f64]) -> MetricBundle {
    let n = y_true.len();
    if n == 0 {
        return MetricBundle::default();
    }

    let n_pos = y_true.iter().filter(|&&v| v == 1).count();
    let n_neg = n - n_pos;

    let roc_auc = compute_roc_auc(y_true, y_prob);
    let pr_auc = compute_pr_auc(y_true, y_prob);

    // Default threshold 0.5
    let (tp, tn, fp, fn_) = confusion_at_threshold(y_true, y_prob, 0.5);
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let spec = if tn + fp > 0 {
        tn as f64 / (tn + fp) as f64
    } else {
        0.0
    };
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let balanced_acc = (recall + spec) / 2.0;
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let f2 = if 4.0 * precision + recall > 0.0 {
        5.0 * precision * recall / (4.0 * precision + recall)
    } else {
        0.0
    };
    let brier = compute_brier(y_true, y_prob);
    let (cal_slope, cal_intercept) = compute_calibration(y_true, y_prob);

    MetricBundle {
        roc_auc,
        pr_auc,
        balanced_accuracy: balanced_acc,
        recall_sensitivity: recall,
        specificity: spec,
        f1_score: f1,
        f2_score: f2,
        brier_score: brier,
        calibration_slope: cal_slope,
        calibration_intercept: cal_intercept,
        n_pos,
        n_neg,
    }
}

/// Area under the ROC curve (trapezoidal rule).
pub fn compute_roc_auc(y_true: &[i32], y_prob: &[f64]) -> f64 {
    let n = y_true.len();
    let n_pos = y_true.iter().filter(|&&v| v == 1).count();
    let n_neg = n - n_pos;
    if n_pos == 0 || n_neg == 0 {
        return 0.5;
    }

    // Sort by descending probability
    let mut pairs: Vec<(f64, i32)> = y_prob.iter().copied().zip(y_true.iter().copied()).collect();
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut auc = 0.0;
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut prev_tp = 0usize;
    let mut prev_fp = 0usize;
    let mut prev_prob = f64::INFINITY;

    for &(prob, label) in &pairs {
        if (prob - prev_prob).abs() > 1e-10 {
            // Trapezoid
            auc += (fp as f64 - prev_fp as f64) * (tp as f64 + prev_tp as f64) / 2.0;
            prev_tp = tp;
            prev_fp = fp;
            prev_prob = prob;
        }
        if label == 1 {
            tp += 1;
        } else {
            fp += 1;
        }
    }
    auc += (fp as f64 - prev_fp as f64) * (tp as f64 + prev_tp as f64) / 2.0;
    auc / (n_pos as f64 * n_neg as f64)
}

/// Area under the Precision-Recall curve (interpolated trapezoid).
///
/// Tie-breaking: when two predictions share the same probability, the negative
/// sample (label=0) is ranked BEFORE the positive (label=1).  This is the
/// pessimistic (worst-case) convention and prevents artificially high PR-AUC
/// when all predictions are equal — e.g. for dummy classifiers.
pub fn compute_pr_auc(y_true: &[i32], y_prob: &[f64]) -> f64 {
    let n_pos = y_true.iter().filter(|&&v| v == 1).count();
    if n_pos == 0 {
        return 0.0;
    }

    let mut pairs: Vec<(f64, i32)> = y_prob.iter().copied().zip(y_true.iter().copied()).collect();
    pairs.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1)) // tie: label 0 before label 1 (pessimistic)
    });

    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut auc = 0.0;
    let mut prev_recall = 0.0f64;
    let mut prev_precision = 1.0f64;

    for &(_, label) in &pairs {
        if label == 1 {
            tp += 1;
        } else {
            fp += 1;
        }
        let recall = tp as f64 / n_pos as f64;
        let precision = tp as f64 / (tp + fp) as f64;
        // Trapezoid
        auc += (recall - prev_recall) * (precision + prev_precision) / 2.0;
        prev_recall = recall;
        prev_precision = precision;
    }
    auc
}

pub fn confusion_at_threshold(
    y_true: &[i32],
    y_prob: &[f64],
    threshold: f64,
) -> (usize, usize, usize, usize) {
    let mut tp = 0usize;
    let mut tn = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    for (&y, &p) in y_true.iter().zip(y_prob.iter()) {
        let pred = if p >= threshold { 1 } else { 0 };
        match (y, pred) {
            (1, 1) => tp += 1,
            (0, 0) => tn += 1,
            (0, 1) => fp += 1,
            (1, 0) => fn_ += 1,
            _ => {}
        }
    }
    (tp, tn, fp, fn_)
}

pub fn compute_brier(y_true: &[i32], y_prob: &[f64]) -> f64 {
    let n = y_true.len() as f64;
    if n == 0.0 {
        return 1.0;
    }
    y_true
        .iter()
        .zip(y_prob.iter())
        .map(|(&y, &p)| (y as f64 - p).powi(2))
        .sum::<f64>()
        / n
}

/// Logistic calibration regression: y ~ logistic(intercept + slope * logit(p_hat))
///
/// This is the Van Calster et al. (2016) convention: fit a logistic regression of
/// the binary outcome on the logit of the predicted probability.  A perfectly
/// calibrated model has slope≈1 and intercept≈0.  Using OLS with binary labels
/// (linear probability model) compresses the slope systematically and is incorrect.
pub fn compute_calibration(y_true: &[i32], y_prob: &[f64]) -> (f64, f64) {
    let n = y_true.len();
    if n < 4 {
        return (1.0, 0.0);
    }

    let logit_p: Vec<f64> = y_prob
        .iter()
        .map(|&p| crate::utils::logit(p.clamp(1e-6, 1.0 - 1e-6)))
        .collect();

    let mut slope = 1.0f64;
    let mut intercept = 0.0f64;
    let lr = 0.05;
    let nf = n as f64;

    for _ in 0..1000 {
        let mut d_slope = 0.0f64;
        let mut d_intercept = 0.0f64;
        for (&yi, &xi) in y_true.iter().zip(logit_p.iter()) {
            let pred = crate::utils::sigmoid(intercept + slope * xi);
            let err = pred - yi as f64;
            d_slope += err * xi;
            d_intercept += err;
        }
        slope -= lr * d_slope / nf;
        intercept -= lr * d_intercept / nf;
    }
    (slope, intercept)
}

#[allow(dead_code)]
fn simple_linear_regression(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    if n < 2.0 {
        return (1.0, 0.0);
    }
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let sxx: f64 = x.iter().map(|xi| (xi - mx).powi(2)).sum();
    let sxy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mx) * (yi - my))
        .sum();
    if sxx.abs() < 1e-10 {
        return (1.0, 0.0);
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    (slope, intercept)
}

// ── Threshold sensitivity analysis ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThresholdPoint {
    pub cutoff: f64,
    pub sensitivity: f64,
    pub specificity: f64,
    pub precision: f64,
    pub f1: f64,
    pub f2: f64,
    pub fn_count: usize,
    pub fp_count: usize,
}

pub fn threshold_sweep(y_true: &[i32], y_prob: &[f64]) -> Vec<ThresholdPoint> {
    let steps: Vec<f64> = (1..=99).map(|i| i as f64 / 100.0).collect();
    let n_pos = y_true.iter().filter(|&&v| v == 1).count();
    let n_neg = y_true.len() - n_pos;

    steps
        .iter()
        .map(|&cut| {
            let (tp, tn, fp, fn_) = confusion_at_threshold(y_true, y_prob, cut);
            let sens = if n_pos > 0 {
                tp as f64 / n_pos as f64
            } else {
                0.0
            };
            let spec = if n_neg > 0 {
                tn as f64 / n_neg as f64
            } else {
                0.0
            };
            let prec = if tp + fp > 0 {
                tp as f64 / (tp + fp) as f64
            } else {
                0.0
            };
            let f1 = if prec + sens > 0.0 {
                2.0 * prec * sens / (prec + sens)
            } else {
                0.0
            };
            let f2 = if 4.0 * prec + sens > 0.0 {
                5.0 * prec * sens / (4.0 * prec + sens)
            } else {
                0.0
            };
            ThresholdPoint {
                cutoff: cut,
                sensitivity: sens,
                specificity: spec,
                precision: prec,
                f1,
                f2,
                fn_count: fn_,
                fp_count: fp,
            }
        })
        .collect()
}
