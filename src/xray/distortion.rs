/// Distortion computation — Jacobian-based projection distortion analysis.
///
/// For each sample point, computes the Jacobian of the map projection,
/// then derives Tissot ellipse parameters: semimajor (a), semiminor (b),
/// angle (θ), area scale factor (h = a × b), and angular distortion (ω).
use crate::core::error::{Result, TissotError};
use crate::core::rule::Layer;
use geo::{Centroid, Geometry};
use serde::Serialize;

/// Distortion metrics at a single sample point.
#[derive(Debug, Clone, Serialize)]
pub struct DistortionSample {
    /// Longitude of sample point (WGS 84).
    pub lon: f64,
    /// Latitude of sample point (WGS 84).
    pub lat: f64,
    /// Area scale factor h = a × b (1.0 = no distortion).
    pub area_scale_factor: f64,
    /// Area distortion as percentage: |h - 1| × 100.
    pub area_distortion_pct: f64,
    /// Angular distortion ω = 2 × arcsin((a-b)/(a+b)) in degrees.
    pub angular_distortion_deg: f64,
    /// Tissot ellipse semimajor axis.
    pub semimajor: f64,
    /// Tissot ellipse semiminor axis.
    pub semiminor: f64,
    /// Tissot ellipse orientation angle in radians.
    pub theta: f64,
}

/// Compute distortion samples at feature centroids.
///
/// For datasets with fewer features than `max_samples`, all centroids are used.
/// For larger datasets, a spatially stratified sample is drawn.
pub fn compute_samples(
    layer: &Layer,
    crs: &str,
    max_samples: usize,
) -> Result<Vec<DistortionSample>> {
    // Collect centroids from all features that have geometry.
    let mut centroids: Vec<(f64, f64)> = Vec::new();
    for feature in &layer.features {
        if let Some(ref geom) = feature.geometry {
            if let Some(centroid) = geometry_centroid(geom) {
                centroids.push(centroid);
            }
        }
    }

    if centroids.is_empty() {
        return Ok(Vec::new());
    }

    // Subsample if necessary (stratified grid sampling).
    let sampled = if centroids.len() > max_samples {
        stratified_sample(&centroids, max_samples)
    } else {
        centroids
    };

    // Compute distortion at each sample point.
    let mut samples = Vec::with_capacity(sampled.len());
    for (lon, lat) in &sampled {
        let sample = compute_distortion_at(*lon, *lat, crs)?;
        samples.push(sample);
    }

    Ok(samples)
}

/// Compute the centroid of a geometry as (lon, lat).
fn geometry_centroid(geom: &Geometry) -> Option<(f64, f64)> {
    let centroid = geom.centroid()?;
    Some((centroid.x(), centroid.y()))
}

/// Spatially stratified subsampling of centroids.
///
/// Divides extent into grid cells and picks one point per cell.
fn stratified_sample(centroids: &[(f64, f64)], target: usize) -> Vec<(f64, f64)> {
    if centroids.is_empty() {
        return Vec::new();
    }

    let min_x = centroids.iter().map(|c| c.0).fold(f64::MAX, f64::min);
    let max_x = centroids.iter().map(|c| c.0).fold(f64::MIN, f64::max);
    let min_y = centroids.iter().map(|c| c.1).fold(f64::MAX, f64::min);
    let max_y = centroids.iter().map(|c| c.1).fold(f64::MIN, f64::max);

    let cols = (target as f64).sqrt().ceil() as usize;
    let rows = cols;
    let cell_w = (max_x - min_x) / cols as f64;
    let cell_h = (max_y - min_y) / rows as f64;

    if cell_w <= 0.0 || cell_h <= 0.0 {
        // All points at same location — just return up to target.
        return centroids.iter().take(target).copied().collect();
    }

    // Pick first centroid falling into each grid cell.
    let mut grid: std::collections::HashMap<(usize, usize), (f64, f64)> =
        std::collections::HashMap::new();

    for &(x, y) in centroids {
        let col = ((x - min_x) / cell_w).floor() as usize;
        let row = ((y - min_y) / cell_h).floor() as usize;
        let col = col.min(cols - 1);
        let row = row.min(rows - 1);
        grid.entry((col, row)).or_insert((x, y));
    }

    grid.into_values().take(target).collect()
}

/// Compute Tissot distortion parameters at a single geographic point.
///
/// Uses a finite-difference approximation of the Jacobian. The projection
/// is defined by the CRS string (e.g., "EPSG:3857").
fn compute_distortion_at(lon: f64, lat: f64, crs: &str) -> Result<DistortionSample> {
    // For CRS that is already geographic (EPSG:4326), distortion is zero by definition.
    if crs == "EPSG:4326" || crs == "OGC:CRS84" {
        return Ok(DistortionSample {
            lon,
            lat,
            area_scale_factor: 1.0,
            area_distortion_pct: 0.0,
            angular_distortion_deg: 0.0,
            semimajor: 1.0,
            semiminor: 1.0,
            theta: 0.0,
        });
    }

    // Build proj pipeline from WGS 84 to target CRS.
    let proj = proj::Proj::new_known_crs("EPSG:4326", crs, None)
        .map_err(|e| TissotError::Projection(format!("Failed to create projection {crs}: {e}")))?;

    // Finite-difference step size (degrees).
    let delta = 0.001;

    // Forward-project the center point and offset points.
    let center = project_point(&proj, lon, lat)?;
    let dx = project_point(&proj, lon + delta, lat)?;
    let dy = project_point(&proj, lon, lat + delta)?;

    // Approximate Jacobian components.
    let dxdlon = (dx.0 - center.0) / delta;
    let dydlon = (dx.1 - center.1) / delta;
    let dxdlat = (dy.0 - center.0) / delta;
    let dydlat = (dy.1 - center.1) / delta;

    // Scale by cos(lat) to account for meridian convergence.
    let cos_lat = lat.to_radians().cos();
    let a11 = dxdlon * cos_lat;
    let a12 = dxdlat;
    let a21 = dydlon * cos_lat;
    let a22 = dydlat;

    // Compute Tissot parameters from Jacobian.
    // h = sqrt((a11² + a21²))  — scale along parallel
    // k = sqrt((a12² + a22²))  — scale along meridian
    let h = (a11 * a11 + a21 * a21).sqrt();
    let k = (a12 * a12 + a22 * a22).sqrt();

    // Area scale factor = |det(J)| / (h_ref * k_ref)
    let det = (a11 * a22 - a12 * a21).abs();

    // Normalize to unit scale on a reference sphere.
    // Earth radius in metres ≈ 6_371_000
    let earth_r = 6_371_000.0;
    let deg_to_m_lon = earth_r * lat.to_radians().cos() * std::f64::consts::PI / 180.0;
    let deg_to_m_lat = earth_r * std::f64::consts::PI / 180.0;
    let ref_det = deg_to_m_lon * deg_to_m_lat;

    let area_scale = if ref_det > 0.0 { det / ref_det } else { 1.0 };

    // Semimajor and semiminor from singular values of J.
    let sum_sq = h * h + k * k;
    let diff = ((h * h - k * k).powi(2) + 4.0 * (a11 * a12 + a21 * a22).powi(2)).sqrt();
    let semimajor = ((sum_sq + diff) / 2.0).sqrt().max(0.001);
    let semiminor = ((sum_sq - diff) / 2.0).sqrt().max(0.001);

    // Angular distortion: ω = 2 × arcsin((a - b) / (a + b)).
    let omega_arg = ((semimajor - semiminor) / (semimajor + semiminor)).clamp(-1.0, 1.0);
    let angular_distortion_deg = 2.0 * omega_arg.asin().to_degrees();

    // Ellipse orientation angle.
    let theta = (2.0 * (a11 * a12 + a21 * a22)).atan2(h * h - k * k) / 2.0;

    let area_distortion_pct = (area_scale - 1.0).abs() * 100.0;

    Ok(DistortionSample {
        lon,
        lat,
        area_scale_factor: area_scale,
        area_distortion_pct,
        angular_distortion_deg: angular_distortion_deg.abs(),
        semimajor,
        semiminor,
        theta,
    })
}

/// Project a geographic point (lon/lat) and return projected coordinates.
fn project_point(proj: &proj::Proj, lon: f64, lat: f64) -> Result<(f64, f64)> {
    let coord = geo::coord! { x: lon, y: lat };
    let result = proj.convert(coord).map_err(|e| {
        TissotError::Projection(format!("Projection failed at ({lon}, {lat}): {e}"))
    })?;
    Ok((result.x, result.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgs84_zero_distortion() {
        let sample = compute_distortion_at(-84.5, 38.0, "EPSG:4326").unwrap();
        assert!((sample.area_distortion_pct).abs() < f64::EPSILON);
        assert!((sample.angular_distortion_deg).abs() < f64::EPSILON);
    }

    #[test]
    fn web_mercator_high_latitude_distortion() {
        // Web Mercator has significant area distortion at high latitudes.
        let sample = compute_distortion_at(-84.5, 60.0, "EPSG:3857").unwrap();
        assert!(
            sample.area_distortion_pct > 1.0,
            "Expected significant distortion at 60°N for Web Mercator, got {}%",
            sample.area_distortion_pct
        );
    }

    #[test]
    fn web_mercator_equator_low_distortion() {
        // Web Mercator should have low distortion near the equator.
        let sample = compute_distortion_at(-84.5, 0.1, "EPSG:3857").unwrap();
        assert!(
            sample.area_distortion_pct < 5.0,
            "Expected low distortion near equator for Web Mercator, got {}%",
            sample.area_distortion_pct
        );
    }

    #[test]
    fn stratified_sample_reduces_count() {
        let centroids: Vec<(f64, f64)> = (0..1000)
            .map(|i| (i as f64 * 0.01 - 5.0, i as f64 * 0.01))
            .collect();
        let sampled = stratified_sample(&centroids, 100);
        assert!(sampled.len() <= 100);
        assert!(!sampled.is_empty());
    }

    #[test]
    fn empty_layer_returns_empty_samples() {
        let layer = Layer {
            name: "empty".into(),
            crs: Some("EPSG:3857".into()),
            features: vec![],
            bounds: None,
        };
        let samples = compute_samples(&layer, "EPSG:3857", 1000).unwrap();
        assert!(samples.is_empty());
    }
}
