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
    /// Whether tree-based exact interventional TreeSHAP is supported.
    fn supports_tree_attribution(&self) -> bool {
        false
    }
    /// Compute per-sample exact interventional TreeSHAP values (Lundberg et
    /// al., 2018) with the training-fold data as the background distribution.
    /// `class_weight` is the positive-class weight used at fit time (needed
    /// for the weighted background covers of classification trees).
    fn tree_attributions(&self, _x: &[Vec<f64>], _class_weight: f64) -> Vec<Vec<f64>> {
        vec![]
    }
    /// Raw linear coefficients (standardised-feature space), if the model is
    /// linear. Length must equal n_features (zeros for unused features).
    /// Used for the exact additive logit decomposition.
    fn linear_weights(&self, _n_features: usize) -> Option<Vec<f64>> {
        None
    }
    /// Base value (log-odds space) for the additive decomposition.
    fn attribution_base_logit(&self) -> f64 {
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
}

impl LogisticRegression {
    pub fn new(c: f64) -> Self {
        Self {
            weights: vec![],
            bias: 0.0,
            c,
            max_iter: 1000,
            lr: 0.05,
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

    fn linear_weights(&self, _n_features: usize) -> Option<Vec<f64>> {
        Some(self.weights.clone())
    }

    fn attribution_base_logit(&self) -> f64 {
        self.bias
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

    pub fn node_n_pos(&self) -> usize {
        match self {
            TreeNode::Leaf { n_pos, .. } | TreeNode::Split { n_pos, .. } => *n_pos,
        }
    }
}

// ── Exact interventional TreeSHAP (Lundberg et al., 2018) ────────────────────
//
// Exact Shapley values for a single tree with the training-fold data as the
// background distribution. For a path P of split features along the sample's
// route, features off the path contribute zero, and the contribution of path
// feature j is
//   phi_j = sum_{T subset P \ {j}} W(|T|) * [v(T u {j}) - v(T)]
// with W(t) = sum_{k=0}^{M-|P|} C(M-|P|, k) * (t+k)! (M-t-k-1)! / M!,
// where M is the total number of features and v(T) is the expected node value
// when the features in T are fixed to the sample's values and the rest are
// marginalised over the background. Background covers are the node sample
// counts (weighted by the positive-class weight for classification trees, so
// that the decomposition is exactly additive: sum_j phi_j = f(x) - E[f]).
//
// Complexity is O(2^|P| * depth) per sample per tree; with max_depth <= 4 this
// is negligible for the 81-sample dataset.

fn node_cover(node: &TreeNode, weighted: bool, class_weight: f64) -> f64 {
    let n = node.node_n() as f64;
    if weighted {
        let n_pos = node.node_n_pos() as f64;
        (n - n_pos) + class_weight * n_pos
    } else {
        n
    }
}

/// Expected value over the background given the fixed-feature subset mask
/// (mask bits index the unique features on the sample's path).
fn tree_shap_expectation(
    node: &TreeNode,
    path: &[(usize, &TreeNode, &TreeNode)],
    mask: usize,
    weighted: bool,
    class_weight: f64,
) -> f64 {
    match node {
        TreeNode::Leaf { prob, .. } => *prob,
        TreeNode::Split {
            left,
            right,
            ..
        } => {
            // Locate this split node in the path (by node identity).
            let mut fixed = false;
            let mut follows_left = false;
            for (bit, child, split_ref) in path.iter() {
                if std::ptr::eq(*split_ref, node) {
                    if mask & (1 << bit) != 0 {
                        fixed = true;
                    }
                    follows_left = std::ptr::eq(*child, left.as_ref());
                    break;
                }
            }
            if fixed {
                if follows_left {
                    tree_shap_expectation(left, path, mask, weighted, class_weight)
                } else {
                    tree_shap_expectation(right, path, mask, weighted, class_weight)
                }
            } else {
                let c = node_cover(node, weighted, class_weight);
                let cl = node_cover(left, weighted, class_weight);
                let cr = node_cover(right, weighted, class_weight);
                if c <= 0.0 {
                    return node.node_prob();
                }
                (cl * tree_shap_expectation(left, path, mask, weighted, class_weight)
                    + cr * tree_shap_expectation(right, path, mask, weighted, class_weight))
                    / c
            }
        }
    }
}

/// Exact interventional TreeSHAP values for one tree and one sample.
fn exact_tree_shap_values(
    tree: &TreeNode,
    x: &[f64],
    n_features: usize,
    weighted: bool,
    class_weight: f64,
) -> Vec<f64> {
    let mut contrib = vec![0.0f64; n_features];

    // Collect the sample's path; the same feature may appear at several
    // depths, so subsets are defined over the UNIQUE features on the path.
    let mut raw_path: Vec<(usize, &TreeNode, &TreeNode)> = Vec::new(); // (feature, child, split node)
    let mut cur = tree;
    loop {
        match cur {
            TreeNode::Leaf { .. } => break,
            TreeNode::Split {
                feature_idx,
                threshold,
                left,
                right,
                ..
            } => {
                let child = if x[*feature_idx] <= *threshold {
                    left.as_ref()
                } else {
                    right.as_ref()
                };
                raw_path.push((*feature_idx, child, cur));
                cur = child;
            }
        }
    }

    // Unique features in path order, with bit indices.
    let mut unique: Vec<usize> = Vec::new();
    let mut path: Vec<(usize, &TreeNode, &TreeNode)> = Vec::new();
    for (fj, child, split_ref) in raw_path {
        let bit = match unique.iter().position(|&u| u == fj) {
            Some(b) => b,
            None => {
                unique.push(fj);
                unique.len() - 1
            }
        };
        path.push((bit, child, split_ref));
    }

    let p = unique.len();
    if p == 0 || n_features == 0 {
        return contrib;
    }
    let m = n_features;

    // Precompute W(t), t = 0..p-1.
    let mut fact = vec![1.0f64; m + 1];
    for i in 1..=m {
        fact[i] = fact[i - 1] * i as f64;
    }
    let mut binom = vec![1.0f64; (m - p) + 1];
    for k in 1..=m - p {
        // C(m-p, k) via recurrence
        binom[k] = binom[k - 1] * (m - p - k + 1) as f64 / k as f64;
    }
    let mut wts = vec![0.0f64; p];
    for t in 0..p {
        let mut s = 0.0;
        for k in 0..=(m - p) {
            s += binom[k] * fact[t + k] * fact[m - t - k - 1] / fact[m];
        }
        wts[t] = s;
    }

    // Evaluate v(T) for every subset T of the unique path features.
    let n_subsets = 1usize << p;
    let mut v = vec![0.0f64; n_subsets];
    for mask in 0..n_subsets {
        v[mask] = tree_shap_expectation(tree, &path, mask, weighted, class_weight);
    }

    // Contributions.
    for (bit, &fj) in unique.iter().enumerate() {
        let b = 1usize << bit;
        let mut s = 0.0;
        for mask in 0..n_subsets {
            if mask & b != 0 {
                continue;
            }
            let t = mask.count_ones() as usize;
            s += wts[t] * (v[mask | b] - v[mask]);
        }
        contrib[fj] += s;
    }
    contrib
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
        let _ = mean_pred;

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

        // SHAP base value: background expectation E[f(x)] = mean of the root
        // (weighted) probabilities across trees.
        let t = self.trees.len() as f64;
        let mean_root: f64 = if t > 0.0 {
            self.trees.iter().map(|tr| tr.node_prob()).sum::<f64>() / t
        } else {
            0.5
        };
        self.base_val = crate::utils::logit(mean_root.clamp(1e-6, 1.0 - 1e-6));
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

    fn supports_tree_attribution(&self) -> bool {
        true
    }

    fn tree_attributions(&self, x: &[Vec<f64>], class_weight: f64) -> Vec<Vec<f64>> {
        let n_feat = self.n_features;
        let n_trees = self.trees.len() as f64;
        x.iter()
            .map(|xi| {
                let mut contrib = vec![0.0f64; n_feat];
                for tree in &self.trees {
                    let c = exact_tree_shap_values(tree, xi, n_feat, true, class_weight);
                    contrib.iter_mut().zip(&c).for_each(|(a, &b)| *a += b);
                }
                if n_trees > 0.0 {
                    contrib.iter_mut().for_each(|v| *v /= n_trees);
                }
                contrib
            })
            .collect()
    }

    fn attribution_base_logit(&self) -> f64 {
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
    /// lr * sum over trees of the root mean residual: the exact SHAP base
    /// value on the log-odds scale (E[logit] = initial + lr*sum(root means)).
    base_extra: f64,
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
            base_extra: 0.0,
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

        // SHAP base value on the log-odds scale: E[logit(x)] over the
        // training background = initial + lr * sum_t (mean residual of tree t),
        // where each tree's root stores its mean residual.
        self.base_extra = self.learning_rate
            * self
                .trees
                .iter()
                .map(|t| t.node_prob())
                .sum::<f64>();
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

    fn supports_tree_attribution(&self) -> bool {
        true
    }

    fn tree_attributions(&self, x: &[Vec<f64>], _class_weight: f64) -> Vec<Vec<f64>> {
        let n_feat = self.n_features;
        x.iter()
            .map(|xi| {
                let mut contrib = vec![0.0f64; n_feat];
                for tree in &self.trees {
                    // Regression trees on pseudo-residuals: unweighted covers.
                    let c = exact_tree_shap_values(tree, xi, n_feat, false, 1.0);
                    contrib
                        .iter_mut()
                        .zip(&c)
                        .for_each(|(a, &b)| *a += self.learning_rate * b);
                }
                contrib
            })
            .collect()
    }

    fn attribution_base_logit(&self) -> f64 {
        self.initial_log_odds + self.base_extra
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

    fn linear_weights(&self, n_features: usize) -> Option<Vec<f64>> {
        // Single coefficient on the EC feature, padded to the full feature set.
        let inner_w = self.inner.linear_weights(n_features)?;
        let mut w_full = vec![0.0; n_features];
        if let Some(idx) = self.ec_feature_idx {
            if idx < n_features {
                if let Some(&w) = inner_w.first() {
                    w_full[idx] = w;
                }
            }
        }
        Some(w_full)
    }

    fn attribution_base_logit(&self) -> f64 {
        self.inner.attribution_base_logit()
    }
}

// ── Hyperparameter grid search (inner CV) ─────────────────────────────────────

use crate::cv::{inner_cv_folds, subset_rows, subset_y};
use crate::metrics::compute_pr_auc;

/// Fit a model on each inner training fold and pool the validation-fold
/// predictions (y_true, probability). These internal-validation predictions
/// are used for the fully nested operational-cutoff selection so that the
/// cutoff never touches outer held-out data.
pub fn inner_cv_pooled_predictions(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
    make_model: &mut dyn FnMut() -> Box<dyn Model>,
) -> Vec<(i32, f64)> {
    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let mut pooled = Vec::new();
    for fold in &folds {
        let x_tr = subset_rows(x_train, &fold.train_idx);
        let y_tr = subset_y(y_train, &fold.train_idx);
        let x_val = subset_rows(x_train, &fold.test_idx);
        let y_val = subset_y(y_train, &fold.test_idx);
        let mut m = make_model();
        m.fit(&x_tr, &y_tr, class_weight_pos, seed);
        let proba = m.predict_proba(&x_val);
        pooled.extend(y_val.iter().copied().zip(proba));
    }
    pooled
}

/// Tune regularised logistic regression C over a grid using inner CV.
/// Returns (best C, pooled inner-validation predictions under best C).
pub fn tune_logistic_c(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> (f64, Vec<(i32, f64)>) {
    let c_grid = [0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0];
    let mut best_c = 1.0;
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return (1.0, vec![]),
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

    let pooled = inner_cv_pooled_predictions(
        x_train,
        y_train,
        class_weight_pos,
        n_inner,
        seed,
        &mut || Box::new(LogisticRegression::new(best_c)),
    );
    (best_c, pooled)
}

/// Tune random forest (n_estimators, max_depth) via inner CV.
/// Returns (best params, pooled inner-validation predictions under best params).
pub fn tune_random_forest(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> ((usize, usize), Vec<(i32, f64)>) {
    // Grid restricted to shallow trees (max_depth <= 4) for the small-sample
    // setting: with ~65 training rows and up to 21 predictors, deeper trees
    // are prone to overfitting and unstable fold-level behaviour.
    let grids: &[(usize, usize)] = &[(50, 3), (100, 3), (100, 4), (200, 4)];
    let mut best = (100, 4);
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return (best, vec![]),
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

    let (n_est, depth) = best;
    let pooled = inner_cv_pooled_predictions(
        x_train,
        y_train,
        class_weight_pos,
        n_inner,
        seed,
        &mut || Box::new(RandomForest::new(n_est, depth)),
    );
    (best, pooled)
}

/// Tune GBT (n_estimators, learning_rate) via inner CV.
/// Returns (best params, pooled inner-validation predictions under best params).
pub fn tune_gbt(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
) -> ((usize, f64), Vec<(i32, f64)>) {
    let grids: &[(usize, f64)] = &[(50, 0.1), (100, 0.1), (100, 0.05), (50, 0.2)];
    let mut best = (100, 0.1);
    let mut best_score = -1.0;

    let folds = match inner_cv_folds(y_train, n_inner, seed) {
        Ok(f) => f,
        Err(_) => return (best, vec![]),
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

    let (n_est, lr) = best;
    let pooled = inner_cv_pooled_predictions(
        x_train,
        y_train,
        class_weight_pos,
        n_inner,
        seed,
        &mut || Box::new(GradientBoostedTrees::new(n_est, lr, 3)),
    );
    (best, pooled)
}

/// Pooled inner-validation predictions for a fixed-hyperparameter model
/// (used for EC-only logistic regression, which is not tuned).
pub fn fixed_model_inner_predictions(
    x_train: &[Vec<f64>],
    y_train: &[i32],
    class_weight_pos: f64,
    n_inner: usize,
    seed: u64,
    feature_names: &[String],
) -> Vec<(i32, f64)> {
    inner_cv_pooled_predictions(
        x_train,
        y_train,
        class_weight_pos,
        n_inner,
        seed,
        &mut || Box::new(EcOnlyLogistic::new(feature_names)),
    )
}

// MetricBundle is used via crate::metrics::MetricBundle directly

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pos_of(node: &TreeNode) -> usize {
        match node {
            TreeNode::Leaf { n_pos, .. } | TreeNode::Split { n_pos, .. } => *n_pos,
        }
    }

    /// Brute-force Shapley values with the same interventional background
    /// definition: features fixed to the sample's values follow the sample's
    /// branch; the rest take the cover-weighted average of the children.
    fn brute_force_shapley(tree: &TreeNode, x: &[f64], n_features: usize) -> Vec<f64> {
        let m = n_features;
        let mut fact = vec![1.0f64; m + 1];
        for i in 1..=m {
            fact[i] = fact[i - 1] * i as f64;
        }

        fn v(tree: &TreeNode, x: &[f64], mask: usize) -> f64 {
            match tree {
                TreeNode::Leaf { prob, .. } => *prob,
                TreeNode::Split {
                    feature_idx,
                    threshold,
                    left,
                    right,
                    n,
                    n_left,
                    n_right,
                    ..
                } => {
                    if mask & (1 << feature_idx) != 0 {
                        if x[*feature_idx] <= *threshold {
                            v(left, x, mask)
                        } else {
                            v(right, x, mask)
                        }
                    } else {
                        let n_tot = *n as f64;
                        if n_tot <= 0.0 {
                            return tree.node_prob();
                        }
                        (*n_left as f64 * v(left, x, mask)
                            + *n_right as f64 * v(right, x, mask))
                            / n_tot
                    }
                }
            }
        }

        let mut phi = vec![0.0f64; m];
        for j in 0..m {
            let mut s = 0.0;
            for mask in 0..(1usize << m) {
                if mask & (1 << j) != 0 {
                    continue;
                }
                let t = mask.count_ones() as usize;
                let w = fact[t] * fact[m - t - 1] / fact[m];
                s += w * (v(tree, x, mask | (1 << j)) - v(tree, x, mask));
            }
            phi[j] = s;
        }
        phi
    }

    fn make_tree() -> TreeNode {
        // Root split on feature 0 at 0.5
        //   left: split on feature 1 at 0.5
        //     left: leaf 0.2 (n=3, 1 pos)
        //     right: leaf 0.6 (n=2, 1 pos)
        //   right: leaf 0.9 (n=5, 4 pos)
        TreeNode::Split {
            feature_idx: 0,
            threshold: 0.5,
            left: Box::new(TreeNode::Split {
                feature_idx: 1,
                threshold: 0.5,
                left: Box::new(TreeNode::Leaf {
                    prob: 0.2,
                    n: 3,
                    n_pos: 1,
                    log_odds: -1.386,
                }),
                right: Box::new(TreeNode::Leaf {
                    prob: 0.6,
                    n: 2,
                    n_pos: 1,
                    log_odds: 0.405,
                }),
                prob: 0.36,
                n: 5,
                n_pos: 2,
                n_left: 3,
                n_right: 2,
            }),
            right: Box::new(TreeNode::Leaf {
                prob: 0.9,
                n: 5,
                n_pos: 4,
                log_odds: 2.197,
            }),
            prob: 0.63,
            n: 10,
            n_pos: 6,
            n_left: 5,
            n_right: 5,
        }
    }

    #[test]
    fn treeshap_matches_bruteforce_shapley() {
        let tree = make_tree();
        let x = vec![0.2, 0.9, 0.7]; // path: f0 left, f1 right; f2 off path
        let n_features = 3;
        let got = exact_tree_shap_values(&tree, &x, n_features, false, 1.0);
        let want = brute_force_shapley(&tree, &x, n_features);
        for j in 0..n_features {
            assert!(
                (got[j] - want[j]).abs() < 1e-9,
                "feature {}: TreeSHAP {} vs brute force {}",
                j,
                got[j],
                want[j]
            );
        }
        // Additivity: sum of SHAP + base = f(x).
        let f_x = tree.predict_prob(&x);
        let base = tree.node_prob();
        let sum_contrib: f64 = got.iter().sum();
        assert!(
            (base + sum_contrib - f_x).abs() < 1e-9,
            "additivity violated: base {} + sum {} vs f(x) {}",
            base,
            sum_contrib,
            f_x
        );
    }

    #[test]
    fn treeshap_matches_bruteforce_with_repeated_feature() {
        // Root splits on f0; the left child splits on f0 AGAIN at 0.2.
        let tree = TreeNode::Split {
            feature_idx: 0,
            threshold: 0.5,
            left: Box::new(TreeNode::Split {
                feature_idx: 0,
                threshold: 0.2,
                left: Box::new(TreeNode::Leaf {
                    prob: 0.1,
                    n: 2,
                    n_pos: 0,
                    log_odds: -2.197,
                }),
                right: Box::new(TreeNode::Leaf {
                    prob: 0.4,
                    n: 3,
                    n_pos: 1,
                    log_odds: -0.405,
                }),
                prob: 0.28,
                n: 5,
                n_pos: 1,
                n_left: 2,
                n_right: 3,
            }),
            right: Box::new(TreeNode::Leaf {
                prob: 0.8,
                n: 5,
                n_pos: 4,
                log_odds: 1.386,
            }),
            prob: 0.54,
            n: 10,
            n_pos: 5,
            n_left: 5,
            n_right: 5,
        };
        let x = vec![0.1, 0.5, 0.5]; // f0 -> left at root, left at child
        let n_features = 3;
        let got = exact_tree_shap_values(&tree, &x, n_features, false, 1.0);
        let want = brute_force_shapley(&tree, &x, n_features);
        for j in 0..n_features {
            assert!(
                (got[j] - want[j]).abs() < 1e-9,
                "feature {}: TreeSHAP {} vs brute force {}",
                j,
                got[j],
                want[j]
            );
        }
        let f_x = tree.predict_prob(&x);
        let base = tree.node_prob();
        let sum_contrib: f64 = got.iter().sum();
        assert!(
            (base + sum_contrib - f_x).abs() < 1e-9,
            "repeated-feature additivity violated: {} + {} vs {}",
            base,
            sum_contrib,
            f_x
        );
    }

    #[test]
    fn treeshap_additivity_weighted_covers() {
        let tree = make_tree();
        let x = vec![0.2, 0.9, 0.7];
        let n_features = 3;
        let w = 3.0;
        let got = exact_tree_shap_values(&tree, &x, n_features, true, w);
        let f_x = tree.predict_prob(&x);
        // Weighted background expectation at the root:
        // E_w[f] = (c_l * v_l + c_r * v_r) / c_root with c = n_neg + w*n_pos.
        fn e_w(node: &TreeNode, w: f64) -> f64 {
            match node {
                TreeNode::Leaf { prob, .. } => *prob,
                TreeNode::Split {
                    left, right, n, n_pos, ..
                } => {
                    let c_tot = (*n - *n_pos) as f64 + w * *n_pos as f64;
                    if c_tot <= 0.0 {
                        return node.node_prob();
                    }
                    (node_cover(left, true, w) * e_w(left, w)
                        + node_cover(right, true, w) * e_w(right, w))
                        / c_tot
                }
            }
        }
        let base = e_w(&tree, w);
        let sum_contrib: f64 = got.iter().sum();
        assert!(
            (base + sum_contrib - f_x).abs() < 1e-9,
            "weighted additivity violated: base {} + sum {} vs f(x) {}",
            base,
            sum_contrib,
            f_x
        );
        let _ = pos_of;
    }
}

