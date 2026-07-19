/// All preprocessing MUST occur inside CV folds only.
/// This module provides stateless transformation objects trained on fold-train data
/// and applied to fold-test data.

#[derive(Debug, Clone)]
pub struct MedianImputer {
    pub medians: Vec<f64>,
}

impl MedianImputer {
    pub fn fit(x: &[Vec<f64>]) -> Self {
        if x.is_empty() {
            return Self { medians: vec![] };
        }
        let n_features = x[0].len();
        let medians = (0..n_features)
            .map(|j| {
                let mut vals: Vec<f64> = x
                    .iter()
                    .filter_map(|row| {
                        let v = row[j];
                        if v.is_finite() {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .collect();
                if vals.is_empty() {
                    return 0.0;
                }
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let n = vals.len();
                if n & 1 == 0 {
                    (vals[n / 2 - 1] + vals[n / 2]) / 2.0
                } else {
                    vals[n / 2]
                }
            })
            .collect();
        Self { medians }
    }

    pub fn transform(&self, x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        x.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &v)| {
                        if v.is_finite() {
                            v
                        } else {
                            *self.medians.get(j).unwrap_or(&0.0)
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct StandardScaler {
    pub means: Vec<f64>,
    pub stds: Vec<f64>,
}

impl StandardScaler {
    pub fn fit(x: &[Vec<f64>]) -> Self {
        if x.is_empty() {
            return Self {
                means: vec![],
                stds: vec![],
            };
        }
        let n_features = x[0].len();
        let mut means = vec![0.0f64; n_features];
        let mut stds = vec![1.0f64; n_features];

        for j in 0..n_features {
            let vals: Vec<f64> = x.iter().map(|r| r[j]).collect();
            let m = vals.iter().sum::<f64>() / vals.len() as f64;
            means[j] = m;
            let var = vals.iter().map(|v| (v - m).powi(2)).sum::<f64>()
                / (vals.len() as f64 - 1.0).max(1.0);
            stds[j] = var.sqrt().max(1e-8);
        }
        Self { means, stds }
    }

    pub fn transform(&self, x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        x.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &v)| (v - self.means[j]) / self.stds[j])
                    .collect()
            })
            .collect()
    }
}

/// Full preprocessing pipeline: impute then scale.
#[derive(Debug, Clone)]
pub struct FoldPreprocessor {
    pub imputer: MedianImputer,
    pub scaler: StandardScaler,
}

impl FoldPreprocessor {
    pub fn fit(x_train: &[Vec<f64>]) -> Self {
        let imputer = MedianImputer::fit(x_train);
        let imputed = imputer.transform(x_train);
        let scaler = StandardScaler::fit(&imputed);
        Self { imputer, scaler }
    }

    pub fn transform(&self, x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let imputed = self.imputer.transform(x);
        self.scaler.transform(&imputed)
    }
}

/// Compute positive-class weight for class weighting inside fold.
/// weight_pos = n_neg / n_pos (balanced), floored at 1.0.
pub fn compute_class_weight(y: &[i32]) -> f64 {
    let n_pos = y.iter().filter(|&&v| v == 1).count();
    let n_neg = y.iter().filter(|&&v| v == 0).count();
    if n_pos == 0 || n_neg == 0 {
        return 1.0;
    }
    (n_neg as f64 / n_pos as f64).max(1.0)
}
