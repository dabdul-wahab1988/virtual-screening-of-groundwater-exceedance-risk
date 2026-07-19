use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub fn sha256_file(path: &str) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

pub fn sha256_str(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

pub fn now_iso8601() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn file_size_bytes(path: &str) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn ensure_dir(path: &str) -> Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        fs::create_dir_all(p)?;
    }
    Ok(())
}

pub fn sigmoid(x: f64) -> f64 {
    if x > 30.0 {
        return 1.0 - 1e-15;
    }
    if x < -30.0 {
        return 1e-15;
    }
    1.0 / (1.0 + (-x).exp())
}

pub fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-12, 1.0 - 1e-12);
    (p / (1.0 - p)).ln()
}

#[allow(dead_code)]
pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

#[allow(dead_code)]
pub fn std_dev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64;
    var.sqrt()
}

pub fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let n = sorted.len() as f64;
    let idx = (p / 100.0) * (n - 1.0);
    let lo = idx.floor() as usize;
    let hi = (idx.ceil() as usize).min(sorted.len() - 1);
    if lo == hi {
        return sorted[lo];
    }
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Parse a value that may be "bdl", "nd", "<X", or a plain float.
/// Returns (cleaned_value, is_bdl, is_parse_error)
pub fn parse_numeric_bdl(raw: &str) -> (Option<f64>, bool, bool) {
    let (value, is_bdl, is_parse_error, _) = parse_numeric_bdl_with_limit(raw, None);
    (value, is_bdl, is_parse_error)
}

/// Parse a value that may be "bdl", "nd", "<X", or a plain float.
///
/// Plain BDL markers are left-censored observations. If a positive finite
/// substitution limit is provided, they are replaced by limit / sqrt(2);
/// otherwise they remain missing.
pub fn parse_numeric_bdl_with_limit(
    raw: &str,
    substitution_limit: Option<f64>,
) -> (Option<f64>, bool, bool, Option<&'static str>) {
    let t = raw.trim();
    if t.is_empty() {
        return (None, false, false, None);
    }
    // Explicit BDL markers
    if t.eq_ignore_ascii_case("bdl")
        || t.eq_ignore_ascii_case("nd")
        || t.eq_ignore_ascii_case("n.d.")
        || t.eq_ignore_ascii_case("<dl")
        || t.eq_ignore_ascii_case("bl")
        || t == "-"
    {
        if let Some(limit) = substitution_limit.filter(|v| v.is_finite() && *v > 0.0) {
            return (
                Some(limit / 2.0_f64.sqrt()),
                true,
                false,
                Some("substituted_as_limit_div_sqrt2"),
            );
        }
        return (None, true, false, Some("bdl_stored_as_missing_no_limit"));
    }
    // "<X" pattern: substitute value / sqrt(2)
    if let Some(rest) = t.strip_prefix('<') {
        let rest = rest.trim();
        if let Ok(v) = rest.parse::<f64>() {
            return (
                Some(v / 2.0_f64.sqrt()),
                true,
                false,
                Some("substituted_as_reported_limit_div_sqrt2"),
            );
        }
        return (None, true, true, Some("failed_bdl_limit_parse"));
    }
    // Plain float
    match t.parse::<f64>() {
        Ok(v) => (Some(v), false, false, None),
        Err(_) => (None, false, true, None),
    }
}

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().atan2((1.0 - a).sqrt())
}
