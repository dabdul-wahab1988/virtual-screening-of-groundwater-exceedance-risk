use crate::io::LeakageRulesConfig;
use anyhow::Result;
use rusqlite::Connection;

/// Returns true if `feature_name` is forbidden for `target` under the given rules.
pub fn is_forbidden(
    feature_name: &str,
    target: &str,
    rules: &LeakageRulesConfig,
    special_variant: Option<&str>,
) -> bool {
    let fname_lower = feature_name.to_lowercase();

    // Global forbidden patterns
    for pat in &rules.global_forbidden_patterns {
        if fname_lower.contains(&pat.to_lowercase()) {
            return true;
        }
    }

    if let Some(target_rule) = rules.targets.get(target) {
        // Forbidden exact
        for f in &target_rule.forbidden_exact {
            if feature_name == f.as_str() {
                return true;
            }
        }
        // Forbidden contains
        for f in &target_rule.forbidden_contains {
            if fname_lower.contains(&f.to_lowercase()) {
                return true;
            }
        }
        // Special model overrides
        if let Some(variant) = special_variant {
            if let Some(specials) = &target_rule.special_models {
                if let Some(spec) = specials.get(variant) {
                    // If this variant explicitly allows the feature, override the contains block
                    for allowed in &spec.allow {
                        if feature_name == allowed.as_str() {
                            return false;
                        }
                    }
                    // Extra forbidden_exact in special variant
                    for f in &spec.forbidden_exact {
                        if feature_name == f.as_str() {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn store_leakage_decisions(
    conn: &Connection,
    target: &str,
    tier_name: &str,
    feature_name: &str,
    action: &str,
    reason: &str,
    rule_source: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO leakage_rules_applied
         (target_contaminant, tier_name, feature_name, action, reason, rule_source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![target, tier_name, feature_name, action, reason, rule_source],
    )?;
    Ok(())
}

/// Returns (included_features, excluded_features) for a target/tier pair.
///
/// `storage_tier` is the name written to the DB (may differ from `tier` for
/// special TDS variants, preventing the strict model from overwriting the
/// standard-model audit record).
pub fn apply_leakage_filter(
    candidate_features: &[String],
    target: &str,
    _tier: &str,
    storage_tier: &str,
    rules: &LeakageRulesConfig,
    special_variant: Option<&str>,
    conn: &Connection,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut included = Vec::new();
    let mut excluded = Vec::new();

    for feat in candidate_features {
        if is_forbidden(feat, target, rules, special_variant) {
            excluded.push(feat.clone());
            store_leakage_decisions(
                conn,
                target,
                storage_tier,
                feat,
                "excluded",
                &format!("Leakage rule applied for target '{}'", target),
                "leakage_rules.yaml",
            )?;
        } else {
            included.push(feat.clone());
            store_leakage_decisions(
                conn,
                target,
                storage_tier,
                feat,
                "included",
                "Passed all leakage checks",
                "leakage_rules.yaml",
            )?;
        }
    }
    Ok((included, excluded))
}

/// Store Tier definitions
pub fn store_predictor_tiers(conn: &Connection) -> Result<()> {
    let tiers = [
        (
            "Tier1_Field",
            "Field-screening model — lowest cost",
            "pH, temperature, EC, and spatial coordinates only",
        ),
        (
            "Tier2_Reduced",
            "Reduced-chemistry model — moderate cost",
            "Common major ions excluding target and circular proxies",
        ),
        (
            "Tier3_Full",
            "Full-chemistry model — upper-bound performance",
            "Full chemistry plus derived ratios, after leakage filtering",
        ),
    ];
    for (name, desc, usage) in &tiers {
        conn.execute(
            "INSERT OR IGNORE INTO predictor_tiers (tier_name, tier_description, intended_use)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![name, desc, usage],
        )?;
    }
    Ok(())
}
