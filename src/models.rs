use crate::utils::sigmoid;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ── Model Trait ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub trait Model: Send {
    fn algorithm_name(&self) -> &str;
    /// Fit on training data; class_weight_pos balances positive class.
    fn fit(&mut self, x: &[Vec<f64>], y: &[i32], class_weight_pos: f64, seed: u64);
    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64>;
    /// Feature importances (length = n_features, sum ≈ 1.0).
    fn feature_importances(&self, n_features: usize) -> Vec<f64>;
    /// JSON string of selected hyperparameters.
    fn hyperparams_json(&self) -> String;
    /// Whether tree-based path-SHAP is supported.
    fn supports_tree_shap(&self) -> bool {
        false
    }
    /// Compute per-sample path-based feature attributions (Saabas method).
    fn tree_shap(&self, _x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        vec![]
    }
    /// Base value (mean training prediction in log-odds space).
    fn shap_base_value(&self) -> f64 {
        0.0
    }
}

// ── Dummy: Majority Class ────────────────────────────────────────────────────

pub struct DummyMajority {
    majority_prob: f64,
}

impl DummyMajority {
    pub fn new() -> Self {
        Self { majority_prob: 0.5 }
    }
}

impl Model for DummyMajority {
    fn algorithm_name(&self) -> &str {
        "DummyMajority"
    }

    fn fit(&mut self, _x: &[Vec<f64>], y: &[i32], _w: f64, _seed: u64) {
        let n_pos = y.iter().filter(|&&v| v == 1).count();
        let n_neg = y.len() - n_pos;
        self.majority_prob = if n_neg >= n_pos { 0.0 } else { 1.0 };
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        vec![self.majority_prob; x.len()]
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        vec![0.0; n_features]
    }

    fn hyperparams_json(&self) -> String {
        "{\"strategy\":\"majority\"}".to_string()
    }
}

// ── Dummy: Stratified ────────────────────────────────────────────────────────

pub struct DummyStratified {
    prevalence: f64,
}

impl DummyStratified {
    pub fn new() -> Self {
        Self { prevalence: 0.5 }
    }
}

impl Model for DummyStratified {
    fn algorithm_name(&self) -> &str {
        "DummyStratified"
    }

    fn fit(&mut self, _x: &[Vec<f64>], y: &[i32], _w: f64, _seed: u64) {
        let n_pos = y.iter().filter(|&&v| v == 1).count() as f64;
        self.prevalence = n_pos / y.len() as f64;
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        vec![self.prevalence; x.len()]
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        vec![0.0; n_features]
    }

    fn hyperparams_json(&self) -> String {
        "{\"strategy\":\"stratified\"}".to_string()
    }
}

// ── Logistic Regression ──────────────────────────────────────────────────────

pub struct LogisticRegression {
    weights: Vec<f64>,
    bias: f64,
    pub c: f64, // regularisation strength (1/lambda)
    max_iter: usize,
    lr: f64,
    base_val: f64,
}

impl LogisticRegression {
    pub fn new(c: f64) -> Self {
        Self {
            weights: vec![],
            bias: 0.0,
            c,
            max_iter: 1000,
            lr: 0.05,
            base_val: 0.0,
        }
    }
}

impl Model for LogisticRegression {
    fn algorithm_name(&self) -> &str {
        "LogisticRegression"
    }

    fn fit(&mut self, x: &[Vec<f64>], y: &[i32], class_weight_pos: f64, _seed: u64) {
        if x.is_empty() {
            return;
        }
        let n = x.len();
        let p = x[0].len();
        self.weights = vec![0.0; p];
        self.bias = 0.0;

        let sample_weights: Vec<f64> = y
            .iter()
            .map(|&yi| if yi == 1 { class_weight_pos } else { 1.0 })
            .collect();
        let total_w: f64 = sample_weights.iter().sum::<f64>();

        let lambda = 1.0 / (self.c * n as f64);

        for _ in 0..self.max_iter {
            let mut dw = vec![0.0; p];
            let mut db = 0.0;

            for i in 0..n {
                let z: f64 = self.bias
                    + self
                        .weights
                        .iter()
                        .zip(&x[i])
                        .map(|(w, xi)| w * xi)
                        .sum::<f64>();
                let pred = sigmoid(z);
                let err = (pred - y[i] as f64) * sample_weights[i] / total_w;
                db += err;
                for (gradient, value) in dw.iter_mut().zip(&x[i]) {
                    *gradient += err * value;
                }
            }

            // L2 regularisation gradient
            for (gradient, weight) in dw.iter_mut().zip(&self.weights) {
                *gradient += lambda * weight;
            }

            self.bias -= self.lr * db;
            for (weight, gradient) in self.weights.iter_mut().zip(&dw) {
                *weight -= self.lr * gradient;
            }
        }

        // Store base value as mean prediction logit over training data
        let mean_pred: f64 = x
            .iter()
            .map(|xi| {
                sigmoid(self.bias + self.weights.iter().zip(xi).map(|(w, v)| w * v).sum::<f64>())
            })
            .sum::<f64>()
            / n as f64;
        self.base_val = if mean_pred > 0.0 && mean_pred < 1.0 {
            (mean_pred / (1.0 - mean_pred)).ln()
        } else {
            0.0
        };
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        x.iter()
            .map(|xi| {
                let z = self.bias + self.weights.iter().zip(xi).map(|(w, v)| w * v).sum::<f64>();
                sigmoid(z)
            })
            .collect()
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        if self.weights.is_empty() {
            return vec![0.0; n_features];
        }
        let abs_sum: f64 = self.weights.iter().map(|w| w.abs()).sum::<f64>();
        if abs_sum == 0.0 {
            return vec![0.0; n_features];
        }
        self.weights.iter().map(|w| w.abs() / abs_sum).collect()
    }

    fn hyperparams_json(&self) -> String {
        format!("{{\"C\":{},\"max_iter\":{}}}", self.c, self.max_iter)
    }

    fn shap_base_value(&self) -> f64 {
        self.base_val
    }
}

// ── Decision Tree (CART, for classification and regression) ──────────────────

#[derive(Debug, Clone)]
pub enum TreeNode {
    Leaf {
        prob: f64,
        n: usize,
        n_pos: usize,
        #[allow(dead_code)]
        log_odds: f64,
    },
    Split {
        feature_idx: usize,
        threshold: f64,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
        prob: f64,
        n: usize,
        n_pos: usize,
        n_left: usize,
        n_right: usize,
    },
}

impl TreeNode {
    pub fn predict_prob(&self, x: &[f64]) -> f64 {
        match self {
            TreeNode::Leaf { prob, .. } => *prob,
            TreeNode::Split {
                feature_idx,
                threshold,
                left,
                right,
                ..
            } => {
                if x[*feature_idx] <= *threshold {
                    left.predict_prob(x)
                } else {
                    right.predict_prob(x)
                }
            }
        }
    }

    pub fn predict_value(&self, x: &[f64]) -> f64 {
        self.predict_prob(x)
    }

    pub fn node_prob(&self) -> f64 {
        match self {
            TreeNode::Leaf { prob, .. } | TreeNode::Split { prob, .. } => *prob,
        }
    }

    pub fn node_n(&self) -> usize {
        match self {
            TreeNode::Leaf { n, .. } | TreeNode::Split { n, .. } => *n,
        }
    }
}

fn gini_impurity(n_pos: f64, n_total: f64) -> f64 {
    if n_total == 0.0 {
        return 0.0;
    }
    let p = n_pos / n_total;
    1.0 - p * p - (1.0 - p) * (1.0 - p)
}

fn build_tree(
    x: &[Vec<f64>],
    y: &[f64], // float labels (0.0 or 1.0, or pseudo-residuals for GBT)
    weights: &[f64],
    depth: usize,
    max_depth: usize,
    min_samples_leaf: usize,
    feature_indices: &[usize],
) -> TreeNode {
    let n = x.len();
    let total_w: f64 = weights.iter().sum();
    let pos_w: f64 = y
        .iter()
        .zip(weights)
        .map(|(&yi, &wi)| if yi > 0.5 { wi } else { 0.0 })
        .sum();
    let prob = if total_w > 0.0 { pos_w / total_w } else { 0.5 };
    let n_pos = y.iter().filter(|&&v| v > 0.5).count();
    let log_odds_val = if prob > 0.0 && prob < 1.0 {
        (prob / (1.0 - prob)).ln()
    } else {
        0.0
    };

    if depth >= max_depth || n <= min_samples_leaf * 2 {
        return TreeNode::Leaf {
            prob,
            n,
            n_pos,
            log_odds: log_odds_val,
        };
    }

    // Find best split
    let mut best_gain = 1e-10;
    let mut best_feat = 0;
    let mut best_thresh = 0.0;

    let parent_gini = gini_impurity(pos_w, total_w);

    for &fi in feature_indices {
        // Collect (value, label, weight) and sort by feature value
        let mut triples: Vec<(f64, f64, f64)> = x
            .iter()
            .zip(y.iter())
            .zip(weights.iter())
            .map(|((xi, &yi), &wi)| (xi[fi], yi, wi))
            .collect();
        triples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut left_pos = 0.0f64;
        let mut left_w = 0.0f64;

        for k in 0..triples.len() - 1 {
            left_w += triples[k].2;
            left_pos += if triples[k].1 > 0.5 {
                triples[k].2
            } else {
                0.0
            };

            if (triples[k].0 - triples[k + 1].0).abs() < 1e-10 {
                continue;
            }

            let right_w = total_w - left_w;
            if left_w < min_samples_leaf as f64 || right_w < min_samples_leaf as f64 {
                continue;
            }

            let right_pos = pos_w - left_pos;
            let g_left = gini_impurity(left_pos, left_w);
            let g_right = gini_impurity(right_pos, right_w);
            let gain = parent_gini - (left_w / total_w) * g_left - (right_w / total_w) * g_right;

            if gain > best_gain {
                best_gain = gain;
                best_feat = fi;
                best_thresh = (triples[k].0 + triples[k + 1].0) / 2.0;
            }
        }
    }

    if best_gain <= 1e-10 {
        return TreeNode::Leaf {
            prob,
            n,
            n_pos,
            log_odds: log_odds_val,
        };
    }

    // Partition
    let (left_mask, right_mask): (Vec<bool>, Vec<bool>) = x
        .iter()
        .map(|xi| xi[best_feat] <= best_thresh)
        .partition(|&v| v);
    let _ = left_mask;
    let _ = right_mask;

    let left_indices: Vec<usize> = x
        .iter()
        .enumerate()
        .filter(|(_, xi)| xi[best_feat] <= best_thresh)
        .map(|(i, _)| i)
        .collect();
    let right_indices: Vec<usize> = x
        .iter()
        .enumerate()
        .filter(|(_, xi)| xi[best_feat] > best_thresh)
        .map(|(i, _)| i)
        .collect();

    if left_indices.is_empty() || right_indices.is_empty() {
        return TreeNode::Leaf {
            prob,
            n,
            n_pos,
            log_odds: log_odds_val,
        };
    }

    let x_left: Vec<Vec<f64>> = left_indices.iter().map(|&i| x[i].clone()).collect();
    let y_left: Vec<f64> = left_indices.iter().map(|&i| y[i]).collect();
    let w_left: Vec<f64> = left_indices.iter().map(|&i| weights[i]).collect();

    let x_right: Vec<Vec<f64>> = right_indices.iter().map(|&i| x[i].clone()).collect();
    let y_right: Vec<f64> = right_indices.iter().map(|&i| y[i]).collect();
    let w_right: Vec<f64> = right_indices.iter().map(|&i| weights[i]).collect();

    let n_left = left_indices.len();
    let n_right = right_indices.len();

    let left = build_tree(
        &x_left,
        &y_left,
        &w_left,
        depth + 1,
        max_depth,
        min_samples_leaf,
        feature_indices,
    );
    let right = build_tree(
        &x_right,
        &y_right,
        &w_right,
        depth + 1,
        max_depth,
        min_samples_leaf,
        feature_indices,
    );

    TreeNode::Split {
        feature_idx: best_feat,
        threshold: best_thresh,
        left: Box::new(left),
        right: Box::new(right),
        prob,
        n,
        n_pos,
        n_left,
        n_right,
    }
}

/// Path-based feature attribution (Saabas method).
fn path_shap(node: &TreeNode, x: &[f64], contrib: &mut Vec<f64>) {
    match node {
        TreeNode::Leaf { .. } => {}
        TreeNode::Split {
            feature_idx,
            threshold,
            left,
            right,
            prob: _node_prob,
            ..
        } => {
            let child = if x[*feature_idx] <= *threshold {
                left
            } else {
                right
            };
            let other = if x[*feature_idx] <= *threshold {
                right
            } else {
                left
            };
            let child_prob = child.node_prob();
            let other_prob = other.node_prob();
            let child_n = child.node_n() as f64;
            let other_n = other.node_n() as f64;
            let total_n = child_n + other_n;
            // Expected value at this node vs child
            let expected = (child_n * child_prob + other_n * other_prob) / total_n;
            let contribution = child_prob - expected;
            contrib[*feature_idx] += contribution;
            path_shap(child, x, contrib);
        }
    }
}

/// Compute feature importance as total weighted gain over splits.
fn tree_feature_importance(node: &TreeNode, n_features: usize) -> Vec<f64> {
    let mut imp = vec![0.0f64; n_features];
    accumulate_importance(node, &mut imp);
    let sum: f64 = imp.iter().sum();
    if sum > 0.0 {
        imp.iter_mut().for_each(|v| *v /= sum);
    }
    imp
}

fn accumulate_importance(node: &TreeNode, imp: &mut Vec<f64>) {
    if let TreeNode::Split {
        feature_idx,
        left,
        right,
        n,
        n_left,
        n_right,
        n_pos,
        prob,
        ..
    } = node
    {
        let n_total = *n as f64;
        let n_l = *n_left as f64;
        let n_r = *n_right as f64;
        let pos_w = *n_pos as f64;
        let g_parent = gini_impurity(pos_w, n_total);
        let left_pos = match left.as_ref() {
            TreeNode::Leaf { n_pos, .. } | TreeNode::Split { n_pos, .. } => *n_pos as f64,
        };
        let g_left = gini_impurity(left_pos, n_l);
        let right_pos = pos_w - left_pos;
        let g_right = gini_impurity(right_pos, n_r);
        let gain = n_total * (g_parent - (n_l / n_total) * g_left - (n_r / n_total) * g_right);
        let _ = prob;
        if *feature_idx < imp.len() {
            imp[*feature_idx] += gain;
        }
        accumulate_importance(left, imp);
        accumulate_importance(right, imp);
    }
}

// ── Random Forest ─────────────────────────────────────────────────────────────

pub struct RandomForest {
    trees: Vec<TreeNode>,
    pub n_estimators: usize,
    pub max_depth: usize,
    pub max_features: Option<usize>,
    pub min_samples_leaf: usize,
    base_val: f64,
    n_features: usize,
}

impl RandomForest {
    pub fn new(n_estimators: usize, max_depth: usize) -> Self {
        Self {
            trees: vec![],
            n_estimators,
            max_depth,
            max_features: None,
            min_samples_leaf: 2,
            base_val: 0.0,
            n_features: 0,
        }
    }
}

impl Model for RandomForest {
    fn algorithm_name(&self) -> &str {
        "RandomForest"
    }

    fn fit(&mut self, x: &[Vec<f64>], y: &[i32], class_weight_pos: f64, seed: u64) {
        if x.is_empty() {
            return;
        }
        let n = x.len();
        self.n_features = x[0].len();
        let max_feat = self
            .max_features
            .unwrap_or_else(|| ((self.n_features as f64).sqrt() as usize).max(1));

        let y_f: Vec<f64> = y.iter().map(|&v| v as f64).collect();
        let weights: Vec<f64> = y
            .iter()
            .map(|&v| if v == 1 { class_weight_pos } else { 1.0 })
            .collect();

        let mean_pred = y_f.iter().sum::<f64>() / n as f64;
        self.base_val = if mean_pred > 0.0 && mean_pred < 1.0 {
            (mean_pred / (1.0 - mean_pred)).ln()
        } else {
            0.0
        };

        self.trees.clear();

        for t in 0..self.n_estimators {
            let tree_seed = seed.wrapping_add(t as u64 * 9999991);
            let mut tree_rng = ChaCha8Rng::seed_from_u64(tree_seed);

            // Bootstrap sample with replacement
            let boot_idx: Vec<usize> = (0..n).map(|_| tree_rng.gen_range(0..n)).collect();
            let x_boot: Vec<Vec<f64>> = boot_idx.iter().map(|&i| x[i].clone()).collect();
            let y_boot: Vec<f64> = boot_idx.iter().map(|&i| y_f[i]).collect();
            let w_boot: Vec<f64> = boot_idx.iter().map(|&i| weights[i]).collect();

            // Random feature subset — use tree_rng, not the outer rng, so each
            // tree's feature subset is reproducible regardless of n_estimators.
            let mut feat_idx: Vec<usize> = (0..self.n_features).collect();
            feat_idx.shuffle(&mut tree_rng);
            feat_idx.truncate(max_feat);

            let tree = build_tree(
                &x_boot,
                &y_boot,
                &w_boot,
                0,
                self.max_depth,
                self.min_samples_leaf,
                &feat_idx,
            );
            self.trees.push(tree);
        }
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        if self.trees.is_empty() {
            return vec![0.5; x.len()];
        }
        x.iter()
            .map(|xi| {
                let mean_p = self.trees.iter().map(|t| t.predict_prob(xi)).sum::<f64>()
                    / self.trees.len() as f64;
                mean_p.clamp(1e-10, 1.0 - 1e-10)
            })
            .collect()
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        if self.trees.is_empty() {
            return vec![0.0; n_features];
        }
        let n = n_features.max(self.n_features);
        let mut avg = vec![0.0f64; n];
        for tree in &self.trees {
            let imp = tree_feature_importance(tree, n);
            avg.iter_mut().zip(&imp).for_each(|(a, &b)| *a += b);
        }
        let t = self.trees.len() as f64;
        avg.iter_mut().for_each(|v| *v /= t);
        avg
    }

    fn hyperparams_json(&self) -> String {
        format!(
            "{{\"n_estimators\":{},\"max_depth\":{}}}",
            self.n_estimators, self.max_depth
        )
    }

    fn supports_tree_shap(&self) -> bool {
        true
    }

    fn tree_shap(&self, x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n_feat = self.n_features;
        x.iter()
            .map(|xi| {
                let mut contrib = vec![0.0f64; n_feat];
                for tree in &self.trees {
                    let mut c = vec![0.0f64; n_feat];
                    path_shap(tree, xi, &mut c);
                    contrib.iter_mut().zip(&c).for_each(|(a, &b)| *a += b);
                }
                let t = self.trees.len() as f64;
                contrib.iter_mut().for_each(|v| *v /= t);
                contrib
            })
            .collect()
    }

    fn shap_base_value(&self) -> f64 {
        self.base_val
    }
}

// ── Regression Tree (for GBT residual fitting) ───────────────────────────────

fn build_regression_tree(
    x: &[Vec<f64>],
    residuals: &[f64],
    depth: usize,
    max_depth: usize,
    min_samples_leaf: usize,
    feature_indices: &[usize],
) -> TreeNode {
    let n = x.len();
    let mean_val: f64 = residuals.iter().sum::<f64>() / n as f64;

    if depth >= max_depth || n <= min_samples_leaf * 2 {
        return TreeNode::Leaf {
            prob: mean_val, // store raw mean residual
            n,
            n_pos: 0,
            log_odds: mean_val,
        };
    }

    // Find best split by variance reduction
    let parent_var: f64 = {
        let m = mean_val;
        residuals.iter().map(|&r| (r - m).powi(2)).sum::<f64>() / n as f64
    };

    let mut best_gain = 1e-10;
    let mut best_feat = 0;
    let mut best_thresh = 0.0;

    // Pre-compute totals once per feature loop to enable correct incremental tracking.
    let total_sum: f64 = residuals.iter().sum();
    let total_sum2: f64 = residuals.iter().map(|&r| r.powi(2)).sum();

    for &fi in feature_indices {
        let mut triples: Vec<(f64, f64)> = x
            .iter()
            .zip(residuals.iter())
            .map(|(xi, &r)| (xi[fi], r))
            .collect();
        triples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut left_sum = 0.0f64;
        let mut left_sum2 = 0.0f64;
        let mut left_n = 0usize;

        for k in 0..triples.len() - 1 {
            left_sum += triples[k].1;
            left_sum2 += triples[k].1.powi(2);
            left_n += 1;
            let right_n = n - left_n;

            if (triples[k].0 - triples[k + 1].0).abs() < 1e-10 {
                continue;
            }
            if left_n < min_samples_leaf || right_n < min_samples_leaf {
                continue;
            }

            // right_sum2 tracked incrementally from total — correct because triples
            // is sorted by feature value, so left_sum2 accumulates the sorted-left
            // squared residuals exactly.
            let right_sum = total_sum - left_sum;
            let right_sum2 = total_sum2 - left_sum2;
            let left_mean = left_sum / left_n as f64;
            let right_mean = right_sum / right_n as f64;

            let left_var = (left_sum2 / left_n as f64 - left_mean.powi(2)).max(0.0);
            let right_var = (right_sum2 / right_n as f64 - right_mean.powi(2)).max(0.0);

            let child_var = (left_n as f64 * left_var + right_n as f64 * right_var) / n as f64;
            let gain = parent_var - child_var;

            if gain > best_gain {
                best_gain = gain;
                best_feat = fi;
                best_thresh = (triples[k].0 + triples[k + 1].0) / 2.0;
            }
        }
    }

    if best_gain <= 1e-10 {
        return TreeNode::Leaf {
            prob: mean_val,
            n,
            n_pos: 0,
            log_odds: mean_val,
        };
    }

    let left_indices: Vec<usize> = x
        .iter()
        .enumerate()
        .filter(|(_, xi)| xi[best_feat] <= best_thresh)
        .map(|(i, _)| i)
        .collect();
    let right_indices: Vec<usize> = x
        .iter()
        .enumerate()
        .filter(|(_, xi)| xi[best_feat] > best_thresh)
        .map(|(i, _)| i)
        .collect();

    if left_indices.is_empty() || right_indices.is_empty() {
        return TreeNode::Leaf {
            prob: mean_val,
            n,
            n_pos: 0,
            log_odds: mean_val,
        };
    }

    let x_l: Vec<Vec<f64>> = left_indices.iter().map(|&i| x[i].clone()).collect();
    let r_l: Vec<f64> = left_indices.iter().map(|&i| residuals[i]).collect();
    let x_r: Vec<Vec<f64>> = right_indices.iter().map(|&i| x[i].clone()).collect();
    let r_r: Vec<f64> = right_indices.iter().map(|&i| residuals[i]).collect();

    let n_left = left_indices.len();
    let n_right = right_indices.len();

    let left = build_regression_tree(
        &x_l,
        &r_l,
        depth + 1,
        max_depth,
        min_samples_leaf,
        feature_indices,
    );
    let right = build_regression_tree(
        &x_r,
        &r_r,
        depth + 1,
        max_depth,
        min_samples_leaf,
        feature_indices,
    );

    TreeNode::Split {
        feature_idx: best_feat,
        threshold: best_thresh,
        left: Box::new(left),
        right: Box::new(right),
        prob: mean_val,
        n,
        n_pos: 0,
        n_left,
        n_right,
    }
}

// ── Gradient Boosted Trees ────────────────────────────────────────────────────

pub struct GradientBoostedTrees {
    trees: Vec<TreeNode>,
    pub learning_rate: f64,
    pub n_estimators: usize,
    pub max_depth: usize,
    initial_log_odds: f64,
    n_features: usize,
    min_samples_leaf: usize,
}

impl GradientBoostedTrees {
    pub fn new(n_estimators: usize, learning_rate: f64, max_depth: usize) -> Self {
        Self {
            trees: vec![],
            learning_rate,
            n_estimators,
            max_depth,
            initial_log_odds: 0.0,
            n_features: 0,
            min_samples_leaf: 2,
        }
    }
}

impl Model for GradientBoostedTrees {
    fn algorithm_name(&self) -> &str {
        "GradientBoostedTrees"
    }

    fn fit(&mut self, x: &[Vec<f64>], y: &[i32], class_weight_pos: f64, seed: u64) {
        if x.is_empty() {
            return;
        }
        let n = x.len();
        self.n_features = x[0].len();
        let all_features: Vec<usize> = (0..self.n_features).collect();

        let n_pos: f64 = y.iter().filter(|&&v| v == 1).count() as f64;
        let mean_y = n_pos / n as f64;
        self.initial_log_odds = crate::utils::logit(mean_y.clamp(1e-6, 1.0 - 1e-6));

        let mut f = vec![self.initial_log_odds; n];
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        self.trees.clear();

        for t in 0..self.n_estimators {
            // Negative gradient of logistic loss (with class weighting)
            let pseudo_residuals: Vec<f64> = y
                .iter()
                .zip(f.iter())
                .map(|(&yi, &fi)| {
                    let p = sigmoid(fi);
                    let w = if yi == 1 { class_weight_pos } else { 1.0 };
                    w * (yi as f64 - p)
                })
                .collect();

            // Random feature subset for diversity
            let max_feat = ((self.n_features as f64).sqrt() as usize)
                .max(1)
                .min(self.n_features);
            let mut feat_idx = all_features.clone();
            feat_idx.shuffle(&mut rng);
            feat_idx.truncate(max_feat);

            let tree = build_regression_tree(
                x,
                &pseudo_residuals,
                0,
                self.max_depth,
                self.min_samples_leaf,
                &feat_idx,
            );

            // Update log-odds
            for i in 0..n {
                let leaf_val = tree.predict_value(&x[i]);
                f[i] += self.learning_rate * leaf_val;
            }

            let _ = t;
            self.trees.push(tree);
        }
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        x.iter()
            .map(|xi| {
                let log_odds = self.initial_log_odds
                    + self
                        .trees
                        .iter()
                        .map(|t| self.learning_rate * t.predict_value(xi))
                        .sum::<f64>();
                sigmoid(log_odds).clamp(1e-10, 1.0 - 1e-10)
            })
            .collect()
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        if self.trees.is_empty() {
            return vec![0.0; n_features];
        }
        let n = n_features.max(self.n_features);
        let mut avg = vec![0.0f64; n];
        for tree in &self.trees {
            let imp = tree_feature_importance(tree, n);
            avg.iter_mut().zip(&imp).for_each(|(a, &b)| *a += b);
        }
        let t = self.trees.len() as f64;
        avg.iter_mut().for_each(|v| *v /= t);
        avg
    }

    fn hyperparams_json(&self) -> String {
        format!(
            "{{\"n_estimators\":{},\"learning_rate\":{},\"max_depth\":{}}}",
            self.n_estimators, self.learning_rate, self.max_depth
        )
    }

    fn supports_tree_shap(&self) -> bool {
        true
    }

    fn tree_shap(&self, x: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n_feat = self.n_features;
        x.iter()
            .map(|xi| {
                let mut contrib = vec![0.0f64; n_feat];
                for tree in &self.trees {
                    let mut c = vec![0.0f64; n_feat];
                    path_shap(tree, xi, &mut c);
                    contrib
                        .iter_mut()
                        .zip(&c)
                        .for_each(|(a, &b)| *a += self.learning_rate * b);
                }
                contrib
            })
            .collect()
    }

    fn shap_base_value(&self) -> f64 {
        self.initial_log_odds
    }
}

// ── EC-only Logistic Regression ───────────────────────────────────────────────

/// Wrapper that uses only the EC feature (or first field feature index).
pub struct EcOnlyLogistic {
    inner: LogisticRegression,
    ec_feature_idx: Option<usize>,
}

impl EcOnlyLogistic {
    pub fn new(feature_names: &[String]) -> Self {
        let idx = feature_names.iter().position(|f| f == "EC");
        Self {
            inner: LogisticRegression::new(1.0),
            ec_feature_idx: idx,
        }
    }
}

impl Model for EcOnlyLogistic {
    fn algorithm_name(&self) -> &str {
        "EcOnlyLogistic"
    }

    fn fit(&mut self, x: &[Vec<f64>], y: &[i32], class_weight_pos: f64, seed: u64) {
        if let Some(idx) = self.ec_feature_idx {
            let x_ec: Vec<Vec<f64>> = x.iter().map(|row| vec![row[idx]]).collect();
            self.inner.fit(&x_ec, y, class_weight_pos, seed);
        }
    }

    fn predict_proba(&self, x: &[Vec<f64>]) -> Vec<f64> {
        if let Some(idx) = self.ec_feature_idx {
            let x_ec: Vec<Vec<f64>> = x.iter().map(|row| vec![row[idx]]).collect();
            self.inner.predict_proba(&x_ec)
        } else {
            vec![0.5; x.len()]
        }
    }

    fn feature_importances(&self, n_features: usize) -> Vec<f64> {
        let mut imp = vec![0.0f64; n_features];
        if let Some(idx) = self.ec_feature_idx {
            if idx < n_features {
                imp[idx] = 1.0;
            }
        }
        imp
    }

    fn hyperparams_json(&self) -> String {
        format!(
            "{{\"strategy\":\"EC_only\",\"ec_feature_idx\":{:?}}}",
            self.ec_feature_idx
        )
    }
}

// ── Hyperparameter grid search (inner CV) ─────────────────────────────────────

use crate::cv::{inner_cv_folds, subset_rows, subset_y};
use crate::metrics::compute_pr_auc;

/// Tune regularised logistic regression C over a grid using inner CV.
pub fn tune_logistic_c(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> f64 {
    let c_grid = [0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0];
    let mut best_c = 1.0;
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return 1.0,
    };

    for &c in &c_grid {
        let mut scores = Vec::new();
        for fold in &folds {
            let x_tr = subset_rows(x_train, &fold.train_idx);
            let y_tr = subset_y(y_train, &fold.train_idx);
            let x_val = subset_rows(x_train, &fold.test_idx);
            let y_val = subset_y(y_train, &fold.test_idx);

            let mut lr = LogisticRegression::new(c);
            lr.fit(&x_tr, &y_tr, class_weight_pos, seed);
            let proba = lr.predict_proba(&x_val);
            let score = compute_pr_auc(&y_val, &proba);
            scores.push(score);
        }
        let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;
        if mean_score > best_score {
            best_score = mean_score;
            best_c = c;
        }
    }
    best_c
}

/// Tune random forest (n_estimators, max_depth) via inner CV.
pub fn tune_random_forest(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> (usize, usize) {
    let grids: &[(usize, usize)] = &[(50, 3), (100, 4), (100, 5), (200, 4)];
    let mut best = (100, 4);
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return best,
    };

    for &(n_est, depth) in grids {
        let mut scores = Vec::new();
        for fold in &folds {
            let x_tr = subset_rows(x_train, &fold.train_idx);
            let y_tr = subset_y(y_train, &fold.train_idx);
            let x_val = subset_rows(x_train, &fold.test_idx);
            let y_val = subset_y(y_train, &fold.test_idx);

            let mut rf = RandomForest::new(n_est, depth);
            rf.fit(&x_tr, &y_tr, class_weight_pos, seed);
            let proba = rf.predict_proba(&x_val);
            let score = compute_pr_auc(&y_val, &proba);
            scores.push(score);
        }
        let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;
        if mean_score > best_score {
            best_score = mean_score;
            best = (n_est, depth);
        }
    }
    best
}

/// Tune GBT (n_estimators, learning_rate) via inner CV.
pub fn tune_gbt(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> (usize, f64) {
    let grids: &[(usize, f64)] = &[(50, 0.1), (100, 0.1), (100, 0.05), (50, 0.2)];
    let mut best = (100, 0.1);
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return best,
    };

    for &(n_est, lr) in grids {
        let mut scores = Vec::new();
        for fold in &folds {
            let x_tr = subset_rows(x_train, &fold.train_idx);
            let y_tr = subset_y(y_train, &fold.train_idx);
            let x_val = subset_rows(x_train, &fold.test_idx);
            let y_val = subset_y(y_train, &fold.test_idx);

            let mut gbt = GradientBoostedTrees::new(n_est, lr, 3);
            gbt.fit(&x_tr, &y_tr, class_weight_pos, seed);
            let proba = gbt.predict_proba(&x_val);
            let score = compute_pr_auc(&y_val, &proba);
            scores.push(score);
        }
        let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;
        if mean_score > best_score {
            best_score = mean_score;
            best = (n_est, lr);
        }
    }
    best
}

// MetricBundle is used via crate::metrics::MetricBundle directly
