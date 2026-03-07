/// Projection X-Ray engine — the hero feature.
///
/// Computes per-feature distortion metrics, generates heatmaps,
/// renders Tissot ellipses, and recommends CRS candidates.
pub mod distortion;
pub mod ellipse;
pub mod heatmap;
pub mod recommend;

use crate::core::config::Config;
use crate::core::error::Result;
use crate::core::rule::Layer;
use serde::Serialize;

/// Complete X-Ray analysis result.
#[derive(Debug, Serialize)]
pub struct XrayReport {
    /// Source file path.
    pub file_path: String,
    /// CRS of the input data.
    pub source_crs: String,
    /// Distortion sample points with metrics.
    pub samples: Vec<distortion::DistortionSample>,
    /// Summary statistics.
    pub summary: DistortionSummary,
    /// Heatmap grid for visualization.
    pub heatmap: heatmap::HeatmapGrid,
    /// Tissot ellipse polygons (GeoJSON-ready).
    pub ellipses: Vec<ellipse::TissotEllipse>,
    /// CRS recommendations ranked by fitness.
    pub recommendations: Vec<recommend::CrsRecommendation>,
}

/// Summary statistics for distortion analysis.
#[derive(Debug, Serialize)]
pub struct DistortionSummary {
    /// Number of sample points.
    pub sample_count: usize,
    /// Maximum area distortion percentage.
    pub max_area_distortion_pct: f64,
    /// Mean area distortion percentage.
    pub mean_area_distortion_pct: f64,
    /// Median area distortion percentage.
    pub median_area_distortion_pct: f64,
    /// Maximum angular distortion in degrees.
    pub max_angular_distortion_deg: f64,
    /// Mean angular distortion in degrees.
    pub mean_angular_distortion_deg: f64,
}

/// Run the full X-Ray analysis on a layer.
pub fn analyze(layer: &Layer, config: &Config, file_path: &str) -> Result<XrayReport> {
    let source_crs = layer.crs.clone().unwrap_or_else(|| "EPSG:4326".to_string());

    // 1. Sample distortion at feature centroids
    let samples = distortion::compute_samples(layer, &source_crs, config.xray.max_samples)?;

    // 2. Compute summary statistics
    let summary = compute_summary(&samples);

    // 3. Generate heatmap grid via IDW interpolation
    let bounds = layer.bounds.unwrap_or([-180.0, -90.0, 180.0, 90.0]);
    let heatmap = heatmap::interpolate(&samples, bounds);

    // 4. Generate Tissot ellipses at sample points
    let ellipses = ellipse::generate(&samples);

    // 5. Recommend optimal CRS candidates
    let recommendations = recommend::recommend(&samples, bounds, config.xray.top_recommendations);

    Ok(XrayReport {
        file_path: file_path.to_string(),
        source_crs,
        samples,
        summary,
        heatmap,
        ellipses,
        recommendations,
    })
}

/// Compute summary statistics from distortion samples.
fn compute_summary(samples: &[distortion::DistortionSample]) -> DistortionSummary {
    if samples.is_empty() {
        return DistortionSummary {
            sample_count: 0,
            max_area_distortion_pct: 0.0,
            mean_area_distortion_pct: 0.0,
            median_area_distortion_pct: 0.0,
            max_angular_distortion_deg: 0.0,
            mean_angular_distortion_deg: 0.0,
        };
    }

    let mut area_distortions: Vec<f64> = samples.iter().map(|s| s.area_distortion_pct).collect();
    let angular_distortions: Vec<f64> = samples.iter().map(|s| s.angular_distortion_deg).collect();

    area_distortions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let max_area = area_distortions.last().copied().unwrap_or(0.0);
    let mean_area = area_distortions.iter().sum::<f64>() / area_distortions.len() as f64;
    let median_area = if area_distortions.len() % 2 == 0 {
        let mid = area_distortions.len() / 2;
        (area_distortions[mid - 1] + area_distortions[mid]) / 2.0
    } else {
        area_distortions[area_distortions.len() / 2]
    };

    let max_angular = angular_distortions.iter().cloned().fold(0.0f64, f64::max);
    let mean_angular = angular_distortions.iter().sum::<f64>() / angular_distortions.len() as f64;

    DistortionSummary {
        sample_count: samples.len(),
        max_area_distortion_pct: max_area,
        mean_area_distortion_pct: mean_area,
        median_area_distortion_pct: median_area,
        max_angular_distortion_deg: max_angular,
        mean_angular_distortion_deg: mean_angular,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xray::distortion::DistortionSample;

    #[test]
    fn summary_empty() {
        let summary = compute_summary(&[]);
        assert_eq!(summary.sample_count, 0);
        assert_eq!(summary.max_area_distortion_pct, 0.0);
    }

    #[test]
    fn summary_basic() {
        let samples = vec![
            DistortionSample {
                lon: -84.0,
                lat: 38.0,
                area_scale_factor: 1.10,
                area_distortion_pct: 10.0,
                angular_distortion_deg: 5.0,
                semimajor: 1.05,
                semiminor: 0.95,
                theta: 0.0,
            },
            DistortionSample {
                lon: -85.0,
                lat: 39.0,
                area_scale_factor: 1.05,
                area_distortion_pct: 5.0,
                angular_distortion_deg: 2.0,
                semimajor: 1.02,
                semiminor: 0.98,
                theta: 0.0,
            },
        ];
        let summary = compute_summary(&samples);
        assert_eq!(summary.sample_count, 2);
        assert!((summary.max_area_distortion_pct - 10.0).abs() < f64::EPSILON);
        assert!((summary.mean_area_distortion_pct - 7.5).abs() < f64::EPSILON);
    }
}
