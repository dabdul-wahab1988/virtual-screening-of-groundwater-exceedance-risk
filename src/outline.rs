use crate::{
    manifest::get_file_id,
    utils::{now_iso8601, sha256_str},
};
use anyhow::Result;
use rusqlite::Connection;

pub fn store_outline(conn: &Connection, full_text: &str) -> Result<()> {
    let hash = sha256_str(full_text);
    let file_id = get_file_id(conn, "manuscript_outline")?;

    // Detect a rough title from the first non-empty lines
    let title = full_text
        .lines()
        .skip(1) // skip line 1 header
        .find(|l| !l.trim().is_empty())
        .unwrap_or("(title not detected)")
        .trim()
        .to_string();

    conn.execute(
        "INSERT OR REPLACE INTO manuscript_outline
         (file_id, title_detected, full_text, imported_at, sha256_hash)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![file_id, title, full_text, now_iso8601(), hash],
    )?;
    Ok(())
}

/// Returns the targets explicitly listed in the outline.
/// We scan for the list: Na / Cl / TDS / B / F / NO3.
#[allow(dead_code)]
pub fn extract_targets_from_outline(outline_text: &str) -> Vec<String> {
    let candidates = ["Na", "Cl", "TDS", "B", "F", "NO3"];
    candidates
        .iter()
        .filter(|&&t| outline_text.contains(t))
        .map(|t| t.to_string())
        .collect()
}
