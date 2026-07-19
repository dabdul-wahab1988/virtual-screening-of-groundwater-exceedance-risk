use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// A single fold: indices of training samples and test samples.
#[derive(Debug, Clone)]
pub struct Fold {
    pub repeat: usize,
    pub fold: usize,
    pub train_idx: Vec<usize>,
    pub test_idx: Vec<usize>,
}

// ── Stratified K-Fold ────────────────────────────────────────────────────────

/// Stratified K-Fold: preserves class proportion in each fold.
pub fn stratified_kfold(y: &[i32], n_folds: usize, shuffle_seed: u64) -> Result<Vec<Fold>> {
    let n = y.len();
    if n < n_folds {
        return Err(anyhow!("n_samples ({}) < n_folds ({})", n, n_folds));
    }

    let pos_idx: Vec<usize> = y
        .iter()
        .enumerate()
        .filter(|(_, &v)| v == 1)
        .map(|(i, _)| i)
        .collect();
    let neg_idx: Vec<usize> = y
        .iter()
        .enumerate()
        .filter(|(_, &v)| v == 0)
        .map(|(i, _)| i)
        .collect();

    if pos_idx.is_empty() || neg_idx.is_empty() {
        return Err(anyhow!(
            "Stratified CV requires at least one positive and one negative sample"
        ));
    }

    let mut rng = ChaCha8Rng::seed_from_u64(shuffle_seed);
    let mut pos_shuffled = pos_idx.clone();
    let mut neg_shuffled = neg_idx.clone();
    pos_shuffled.shuffle(&mut rng);
    neg_shuffled.shuffle(&mut rng);

    let mut folds: Vec<Vec<usize>> = vec![vec![]; n_folds];
    for (i, &idx) in pos_shuffled.iter().enumerate() {
        folds[i % n_folds].push(idx);
    }
    for (i, &idx) in neg_shuffled.iter().enumerate() {
        folds[i % n_folds].push(idx);
    }

    let result = (0..n_folds)
        .map(|k| {
            let test_idx = folds[k].clone();
            let train_idx: Vec<usize> = (0..n_folds)
                .filter(|&j| j != k)
                .flat_map(|j| folds[j].clone())
                .collect();
            Fold {
                repeat: 0,
                fold: k,
                train_idx,
                test_idx,
            }
        })
        .collect();

    Ok(result)
}

/// Repeated stratified K-fold: generates repeats × k_folds folds.
pub fn repeated_stratified_kfold(
    y: &[i32],
    n_folds: usize,
    n_repeats: usize,
    base_seed: u64,
) -> Result<Vec<Fold>> {
    let mut all_folds = Vec::new();
    for r in 0..n_repeats {
        let seed = base_seed.wrapping_add(r as u64 * 1_000_003);
        let folds = stratified_kfold(y, n_folds, seed)?;
        for mut f in folds {
            f.repeat = r;
            all_folds.push(f);
        }
    }
    Ok(all_folds)
}

// ── Spatial Group K-Fold ─────────────────────────────────────────────────────

/// Spatial group K-fold: groups wells by spatial cluster.
/// Tries to use each cluster as a test fold in turn.
pub fn spatial_group_kfold(
    sample_ids: &[String],
    cluster_assignments: &[(String, usize)],
    n_folds: usize,
) -> Result<Vec<Fold>> {
    // Build sample_id -> cluster_id lookup
    let cluster_map: std::collections::HashMap<&str, usize> = cluster_assignments
        .iter()
        .map(|(sid, cid)| (sid.as_str(), *cid))
        .collect();

    let n_clusters: usize = cluster_assignments
        .iter()
        .map(|(_, c)| *c)
        .max()
        .map(|m| m + 1)
        .unwrap_or(1);

    // Group sample indices by cluster
    let mut cluster_indices: Vec<Vec<usize>> = vec![vec![]; n_clusters];
    for (i, sid) in sample_ids.iter().enumerate() {
        let cid = *cluster_map.get(sid.as_str()).unwrap_or(&0);
        cluster_indices[cid].push(i);
    }

    // Merge clusters into n_folds groups (simple round-robin merge)
    let mut fold_groups: Vec<Vec<usize>> = vec![vec![]; n_folds];
    let non_empty_clusters: Vec<&Vec<usize>> =
        cluster_indices.iter().filter(|c| !c.is_empty()).collect();

    for (i, cluster) in non_empty_clusters.iter().enumerate() {
        let fold_idx = i % n_folds;
        fold_groups[fold_idx].extend_from_slice(cluster);
    }

    let folds = (0..n_folds)
        .map(|k| {
            let test_idx = fold_groups[k].clone();
            let train_idx: Vec<usize> = (0..n_folds)
                .filter(|&j| j != k)
                .flat_map(|j| fold_groups[j].clone())
                .collect();
            Fold {
                repeat: 0,
                fold: k,
                train_idx,
                test_idx,
            }
        })
        .filter(|f| !f.test_idx.is_empty() && !f.train_idx.is_empty())
        .collect();

    Ok(folds)
}

/// Validate that a binary-class CV design is usable and matches the declared
/// design. This prevents silent changes such as 4-fold spatial CV being stored
/// as if it were the configured 5-fold run.
pub fn validate_binary_folds(
    folds: &[Fold],
    y: &[i32],
    expected_folds: usize,
    cv_mode: &str,
) -> Result<()> {
    if folds.len() != expected_folds {
        return Err(anyhow!(
            "{} produced {} usable folds, but {} were requested",
            cv_mode,
            folds.len(),
            expected_folds
        ));
    }

    for fold in folds {
        let (train_pos, train_neg) = class_counts(y, &fold.train_idx);
        let (test_pos, test_neg) = class_counts(y, &fold.test_idx);
        if train_pos == 0 || train_neg == 0 || test_pos == 0 || test_neg == 0 {
            return Err(anyhow!(
                "{} repeat {} fold {} is not class-balanced enough: train_pos={}, train_neg={}, test_pos={}, test_neg={}",
                cv_mode,
                fold.repeat,
                fold.fold,
                train_pos,
                train_neg,
                test_pos,
                test_neg
            ));
        }
    }

    Ok(())
}

fn class_counts(y: &[i32], idx: &[usize]) -> (usize, usize) {
    let pos = idx.iter().filter(|&&i| y[i] == 1).count();
    let neg = idx.len().saturating_sub(pos);
    (pos, neg)
}

// ── Inner CV for hyperparameter tuning ──────────────────────────────────────

/// Simple inner stratified K-fold for a training-fold subset.
pub fn inner_cv_folds(y_train: &[i32], n_inner: usize, seed: u64) -> Result<Vec<Fold>> {
    if y_train.len() < n_inner * 2 {
        return Err(anyhow!(
            "Too few training samples for {} inner folds",
            n_inner
        ));
    }
    stratified_kfold(y_train, n_inner, seed)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract a sub-matrix given row indices.
pub fn subset_rows(x: &[Vec<f64>], idx: &[usize]) -> Vec<Vec<f64>> {
    idx.iter().map(|&i| x[i].clone()).collect()
}

/// Extract a sub-vector given row indices.
pub fn subset_y(y: &[i32], idx: &[usize]) -> Vec<i32> {
    idx.iter().map(|&i| y[i]).collect()
}

pub fn subset_ids(ids: &[String], idx: &[usize]) -> Vec<String> {
    idx.iter().map(|&i| ids[i].clone()).collect()
}
