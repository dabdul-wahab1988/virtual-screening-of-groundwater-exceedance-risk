use crate::utils::haversine_km;
use anyhow::Result;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rusqlite::Connection;

const EARTH_RADIUS_KM: f64 = 6371.0;

/// Project (lat, lon) pairs onto a local equirectangular plane centred on the
/// dataset centroid. Distances are in kilometres. For a basin of < 100 km
/// extent, the planar approximation error is negligible (sub-kilometre), and
/// Euclidean k-means on the projected coordinates is mathematically coherent:
/// the centroid update (coordinate mean) is the true least-squares centre of
/// the projected cluster, unlike mean lat/lon under great-circle distance.
pub fn project_equirectangular(coords: &[(f64, f64)]) -> (Vec<(f64, f64)>, f64, f64) {
    let n = coords.len();
    if n == 0 {
        return (vec![], 0.0, 0.0);
    }
    let lat0 = coords.iter().map(|c| c.0).sum::<f64>() / n as f64;
    let lon0 = coords.iter().map(|c| c.1).sum::<f64>() / n as f64;
    let cos_lat0 = lat0.to_radians().cos().max(1e-6);
    let projected: Vec<(f64, f64)> = coords
        .iter()
        .map(|&(lat, lon)| {
            let x = (lon - lon0).to_radians() * cos_lat0 * EARTH_RADIUS_KM;
            let y = (lat - lat0).to_radians() * EARTH_RADIUS_KM;
            (x, y)
        })
        .collect();
    (projected, lat0, lon0)
}

/// Inverse equirectangular projection back to (lat, lon).
#[allow(dead_code)]
pub fn unproject(lat0: f64, lon0: f64, x_km: f64, y_km: f64) -> (f64, f64) {
    let cos_lat0 = lat0.to_radians().cos().max(1e-6);
    let lon = lon0 + (x_km / EARTH_RADIUS_KM / cos_lat0).to_degrees();
    let lat = lat0 + (y_km / EARTH_RADIUS_KM).to_degrees();
    (lat, lon)
}

/// Euclidean k-means on projected coordinates with deterministic k-means++
/// initialisation (seeded). Returns cluster assignments (0-based).
pub fn kmeans_cluster_projected(
    projected: &[(f64, f64)],
    k: usize,
    max_iter: usize,
    seed: u64,
) -> Vec<usize> {
    let n = projected.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let k = k.min(n);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // k-means++ initialisation (deterministic under the seed).
    let mut centroids: Vec<(f64, f64)> = Vec::with_capacity(k);
    let first = rng.gen_range(0..n);
    centroids.push(projected[first]);
    while centroids.len() < k {
        // D^2 weighting: sample next centroid with probability ~ squared distance.
        let mut total = 0.0f64;
        let mut d2: Vec<f64> = projected
            .iter()
            .map(|p| {
                let d = centroids
                    .iter()
                    .map(|c| squared_dist(*p, *c))
                    .fold(f64::INFINITY, f64::min);
                total += d;
                d
            })
            .collect();
        if total <= 1e-12 {
            // All remaining points coincide with centroids; pick deterministically.
            let idx = centroids.len() % n;
            centroids.push(projected[idx]);
            continue;
        }
        let mut target = rng.gen_range(0.0..total);
        let mut chosen = n - 1;
        for (i, d) in d2.iter_mut().enumerate() {
            target -= *d;
            if target <= 0.0 {
                chosen = i;
                break;
            }
        }
        centroids.push(projected[chosen]);
        d2.clear();
    }

    let mut assignments = vec![0usize; n];
    for _iter in 0..max_iter {
        let mut changed = false;
        for (i, p) in projected.iter().enumerate() {
            let best = (0..k)
                .min_by(|&a, &b| {
                    squared_dist(*p, centroids[a])
                        .partial_cmp(&squared_dist(*p, centroids[b]))
                        .unwrap()
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
        for (c, centroid) in centroids.iter_mut().enumerate().take(k) {
            let members: Vec<(f64, f64)> = projected
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == c)
                .map(|(_, &p)| p)
                .collect();
            if !members.is_empty() {
                let mx = members.iter().map(|p| p.0).sum::<f64>() / members.len() as f64;
                let my = members.iter().map(|p| p.1).sum::<f64>() / members.len() as f64;
                *centroid = (mx, my);
            }
        }
    }
    assignments
}

fn squared_dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}

/// Determine optimal k via the inertia-elbow heuristic on projected Euclidean
/// distances: stop at the first k where the marginal inertia improvement falls
/// below 10% of the total improvement range (k from 2 to max_k, capped).
pub fn choose_k_projected(projected: &[(f64, f64)], max_k: usize, seed: u64) -> usize {
    let n = projected.len();
    if n <= 3 {
        return 2_usize.min(n).max(1);
    }
    let max_k = max_k.min(n / 2).max(2);

    let mut inertias = vec![0.0f64; max_k + 1];
    for k in 2..=max_k {
        let assignments = kmeans_cluster_projected(projected, k, 200, seed);
        let mut centroids: Vec<(f64, f64)> = (0..k)
            .map(|c| {
                let members: Vec<(f64, f64)> = projected
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == c)
                    .map(|(_, &p)| p)
                    .collect();
                if members.is_empty() {
                    return (0.0, 0.0);
                }
                let mx = members.iter().map(|p| p.0).sum::<f64>() / members.len() as f64;
                let my = members.iter().map(|p| p.1).sum::<f64>() / members.len() as f64;
                (mx, my)
            })
            .collect();
        let inertia: f64 = projected
            .iter()
            .enumerate()
            .map(|(i, p)| squared_dist(*p, centroids[assignments[i]]))
            .sum();
        inertias[k] = inertia;
        centroids.clear();
    }

    let first = inertias[2];
    let last = inertias[max_k];
    if first <= last || (first - last).abs() < 1e-6 {
        return 3.min(max_k);
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
    let (projected, _lat0, _lon0) = project_equirectangular(&coords);
    let seed = 42u64;
    let k = forced_k.unwrap_or_else(|| choose_k_projected(&projected, max_clusters, seed));

    log::info!(
        "Spatial clustering: k={} for {} wells (equirectangular projection, Euclidean k-means, seed {})",
        k,
        coords.len(),
        seed
    );

    let assignments = kmeans_cluster_projected(&projected, k, 200, seed);

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

        // Max intra-cluster distance (great-circle km, for reporting)
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
              number_of_wells, max_intra_cluster_distance_km, clustering_method, projection)
             VALUES (?1, ?2, ?3, ?4, ?5, 'kmeans_projected_equirectangular', 'local_equirectangular_km')",
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
