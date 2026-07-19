use crate::utils::haversine_km;
use anyhow::Result;
use rusqlite::Connection;

/// K-means clustering on (latitude, longitude) pairs.
/// Returns cluster assignments (0-based) for each sample.
pub fn kmeans_cluster(coords: &[(f64, f64)], k: usize, max_iter: usize, _seed: u64) -> Vec<usize> {
    let n = coords.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let k = k.min(n);

    // Initialise centroids using KMeans++ style (deterministic via seed)
    let mut centroids: Vec<(f64, f64)> = {
        // Use evenly-spaced indices as starting centroids (simple and deterministic)
        let step = n / k;
        (0..k).map(|i| coords[(i * step) % n]).collect()
    };

    let mut assignments = vec![0usize; n];

    for _iter in 0..max_iter {
        // Assignment step
        let mut changed = false;
        for (i, &(lat, lon)) in coords.iter().enumerate() {
            let best = (0..k)
                .min_by(|&a, &b| {
                    let da = haversine_km(lat, lon, centroids[a].0, centroids[a].1);
                    let db = haversine_km(lat, lon, centroids[b].0, centroids[b].1);
                    da.partial_cmp(&db).unwrap()
                })
                .unwrap_or(0);
            if assignments[i] != best {
                assignments[i] = best;
                changed = true;
            }
        }
        if !changed {
            break;
        }

        // Update centroids
        for (c, centroid) in centroids.iter_mut().enumerate().take(k) {
            let members: Vec<(f64, f64)> = coords
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == c)
                .map(|(_, &p)| p)
                .collect();
            if !members.is_empty() {
                let mean_lat = members.iter().map(|p| p.0).sum::<f64>() / members.len() as f64;
                let mean_lon = members.iter().map(|p| p.1).sum::<f64>() / members.len() as f64;
                *centroid = (mean_lat, mean_lon);
            }
        }
    }
    assignments
}

/// Determine optimal k via silhouette-like inertia elbow (simple inertia).
pub fn choose_k(coords: &[(f64, f64)], max_k: usize) -> usize {
    let n = coords.len();
    if n <= 3 {
        return 2_usize.min(n);
    }
    let max_k = max_k.min(n / 2).max(2);

    let mut inertias = vec![0.0f64; max_k + 1];
    for (k, inertia_slot) in inertias.iter_mut().enumerate().take(max_k + 1).skip(2) {
        let assignments = kmeans_cluster(coords, k, 100, 42);
        let centroids: Vec<(f64, f64)> = (0..k)
            .map(|c| {
                let members: Vec<(f64, f64)> = coords
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == c)
                    .map(|(_, &p)| p)
                    .collect();
                if members.is_empty() {
                    return (0.0, 0.0);
                }
                let ml = members.iter().map(|p| p.0).sum::<f64>() / members.len() as f64;
                let mn = members.iter().map(|p| p.1).sum::<f64>() / members.len() as f64;
                (ml, mn)
            })
            .collect();

        let inertia: f64 = coords
            .iter()
            .enumerate()
            .map(|(i, &(lat, lon))| {
                let c = assignments[i];
                haversine_km(lat, lon, centroids[c].0, centroids[c].1).powi(2)
            })
            .sum();
        *inertia_slot = inertia;
    }

    // Simple elbow: choose k where improvement drops below 20% of total range
    let first = inertias[2];
    let last = inertias[max_k];
    if first <= last || (first - last).abs() < 1e-6 {
        return 3;
    }
    for k in 2..max_k {
        let improvement = (inertias[k] - inertias[k + 1]) / (first - last);
        if improvement < 0.10 {
            return k;
        }
    }
    max_k
}

/// Build spatial clusters and store in SQLite.
pub fn build_and_store_spatial_clusters(
    conn: &Connection,
    lat_col: &str,
    lon_col: &str,
    max_clusters: usize,
    forced_k: Option<usize>,
) -> Result<()> {
    // Load coordinates
    struct SampleCoord {
        sample_id: String,
        lat: f64,
        lon: f64,
    }

    let mut stmt = conn.prepare(
        "SELECT s.sample_id, lat.cleaned_value_real, lon.cleaned_value_real
         FROM raw_samples s
         JOIN cleaned_measurements lat ON lat.sample_id = s.sample_id AND lat.variable_name = ?1
         JOIN cleaned_measurements lon ON lon.sample_id = s.sample_id AND lon.variable_name = ?2
         ORDER BY s.original_row_index",
    )?;

    let rows: Vec<SampleCoord> = stmt
        .query_map(rusqlite::params![lat_col, lon_col], |row| {
            Ok(SampleCoord {
                sample_id: row.get(0)?,
                lat: row.get(1)?,
                lon: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    if rows.is_empty() {
        log::warn!("No coordinate data found for spatial clustering");
        return Ok(());
    }

    let coords: Vec<(f64, f64)> = rows.iter().map(|r| (r.lat, r.lon)).collect();
    let k = forced_k.unwrap_or_else(|| choose_k(&coords, max_clusters));

    log::info!("Spatial clustering: k={} for {} wells", k, coords.len());

    let assignments = kmeans_cluster(&coords, k, 200, 42);

    // Compute cluster metadata
    for c in 0..k {
        let members: Vec<(f64, f64)> = coords
            .iter()
            .enumerate()
            .filter(|(i, _)| assignments[*i] == c)
            .map(|(_, &p)| p)
            .collect();
        if members.is_empty() {
            continue;
        }
        let centroid_lat = members.iter().map(|p| p.0).sum::<f64>() / members.len() as f64;
        let centroid_lon = members.iter().map(|p| p.1).sum::<f64>() / members.len() as f64;

        // Max intra-cluster distance
        let max_dist: f64 = members
            .iter()
            .flat_map(|&(a_lat, a_lon)| {
                members
                    .iter()
                    .map(move |&(b_lat, b_lon)| haversine_km(a_lat, a_lon, b_lat, b_lon))
            })
            .fold(0.0_f64, f64::max);

        conn.execute(
            "INSERT OR REPLACE INTO spatial_cluster_metadata
             (spatial_cluster_id, centroid_latitude, centroid_longitude,
              number_of_wells, max_intra_cluster_distance_km)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                c as i64,
                centroid_lat,
                centroid_lon,
                members.len() as i64,
                max_dist
            ],
        )?;
    }

    // Store assignments
    for (i, sc) in rows.iter().enumerate() {
        conn.execute(
            "INSERT OR REPLACE INTO sample_spatial_assignment
             (sample_id, latitude, longitude, spatial_cluster_id)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![sc.sample_id, sc.lat, sc.lon, assignments[i] as i64],
        )?;
    }

    log::info!(
        "Stored spatial cluster assignments for {} samples",
        rows.len()
    );
    Ok(())
}

/// Load sample-to-cluster assignments from SQLite.
pub fn load_cluster_assignments(conn: &Connection) -> Result<Vec<(String, usize)>> {
    let mut stmt = conn.prepare(
        "SELECT sample_id, spatial_cluster_id FROM sample_spatial_assignment ORDER BY sample_id",
    )?;
    let pairs = stmt
        .query_map([], |row| {
            let sid: String = row.get(0)?;
            let cid: i64 = row.get(1)?;
            Ok((sid, cid as usize))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(pairs)
}
