//! Rule: Detect distance distortion by comparing projected vs geodesic distances.
//!
//! Samples pairs of feature centroids and computes the error between
//! Euclidean distance in the projected CRS and geodesic distance on the ellipsoid.

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Flags layers where projected distances deviate significantly from geodesic distances.
pub struct DistanceDistortion;

impl Default for DistanceDistortion {
    fn default() -> Self {
        Self
    }
}

impl Rule for DistanceDistortion {
    fn id(&self) -> &str {
        "proj/distance-distortion"
    }

    fn name(&self) -> &str {
        "Distance Distortion"
    }

    fn domain(&self) -> Domain {
        Domain::Projection
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["projection", "distortion", "distance"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            let crs = match &layer.crs {
                Some(c) => c.clone(),
                None => continue,
            };

            // Skip geographic CRS — distances in degree units aren't "distorted"
            // in the projection sense.
            if crs == "EPSG:4326" || crs == "OGC:CRS84" {
                continue;
            }

            // Need at least 2 features to compute pairwise distances.
            let centroids: Vec<(usize, geo::Point)> = layer
                .features
                .iter()
                .enumerate()
                .filter_map(|(idx, f)| {
                    f.geometry.as_ref().and_then(|g| {
                        use geo::Centroid;
                        g.centroid().map(|c| (idx, c))
                    })
                })
                .collect();

            if centroids.len() < 2 {
                continue;
            }

            // Build inverse projection: from the layer's CRS back to WGS 84.
            let inv_proj = match proj::Proj::new_known_crs(&crs, "EPSG:4326", None) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!(
                        "Distance distortion: cannot build inverse projection for {crs}: {e}"
                    );
                    continue;
                }
            };

            // Cap the number of centroids to bound the O(n²) pairwise comparison cost.
            // With 30 centroids, we get at most 435 pairs — enough for a representative
            // sample while keeping per-layer overhead under a millisecond for typical data.
            const MAX_CENTROIDS: usize = 30;
            let sample: Vec<(usize, geo::Point)> =
                centroids.into_iter().take(MAX_CENTROIDS).collect();

            // Inverse-project sampled centroids to WGS 84.
            let wgs84: Vec<Option<geo::Point>> = sample
                .iter()
                .map(|(_, pt)| {
                    let coord = geo::coord! { x: pt.x(), y: pt.y() };
                    inv_proj
                        .convert(coord)
                        .ok()
                        .map(|c| geo::Point::new(c.x, c.y))
                })
                .collect();

            // Compute pairwise projected (Euclidean) vs geodesic distances.
            // O(n²) over MAX_CENTROIDS — bounded by the cap above.
            let mut errors: Vec<f64> = Vec::new();
            for i in 0..sample.len() {
                for j in (i + 1)..sample.len() {
                    let (_, pt_a) = &sample[i];
                    let (_, pt_b) = &sample[j];

                    // Euclidean distance in the projected CRS units.
                    let dx = pt_a.x() - pt_b.x();
                    let dy = pt_a.y() - pt_b.y();
                    let euclidean = (dx * dx + dy * dy).sqrt();

                    // Skip co-located points: distortion ratio is undefined and
                    // the threshold of 1.0 CRS unit is appropriate for meter-based CRS
                    // (geographic CRS is already excluded above).
                    if euclidean < 1.0 {
                        continue;
                    }

                    // Geodesic distance via haversine on the WGS 84 ellipsoid.
                    let (wgs_a, wgs_b) = match (&wgs84[i], &wgs84[j]) {
                        (Some(a), Some(b)) => (a, b),
                        _ => continue,
                    };

                    use geo::{Distance, Haversine};
                    let geodesic = Haversine::distance(*wgs_a, *wgs_b);

                    if geodesic < 1.0 {
                        continue;
                    }

                    let error_pct = ((euclidean - geodesic) / geodesic).abs() * 100.0;
                    errors.push(error_pct);
                }
            }

            if errors.is_empty() {
                continue;
            }

            let max_error = errors.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;

            // Report if distance distortion exceeds the warning threshold.
            let warning_threshold = ctx.config.check.max_distortion_pct;
            if max_error > warning_threshold {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!(
                        "Layer '{}' has {max_error:.1}% max distance distortion (mean: {mean_error:.1}%) in CRS {crs}",
                        layer.name
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: Some(max_error),
                    suggestion: Some(format!(
                        "Run `tissot xray {}` to visualise distortion and get CRS recommendations.",
                        ctx.file_path
                    )),
                    fixable: false,
                });
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.8
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(DistanceDistortion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata() {
        let rule = DistanceDistortion;
        assert_eq!(rule.id(), "proj/distance-distortion");
        assert_eq!(rule.domain(), Domain::Projection);
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(!rule.can_fix());
        assert!(rule.tags().contains(&"distance"));
    }

    #[test]
    fn skips_geographic_crs() {
        use crate::core::config::Config;
        use crate::core::rule::Layer;

        let layer = Layer {
            name: "wgs84".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = DistanceDistortion;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_missing_crs() {
        use crate::core::config::Config;
        use crate::core::rule::Layer;

        let layer = Layer {
            name: "nocrs".into(),
            crs: None,
            features: vec![],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = DistanceDistortion;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_single_feature() {
        use crate::core::config::Config;
        use crate::core::rule::{Feature, Layer};
        use std::collections::HashMap;

        let layer = Layer {
            name: "single".into(),
            crs: Some("EPSG:3857".into()),
            features: vec![Feature {
                id: Some("1".into()),
                geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                properties: HashMap::new(),
            }],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = DistanceDistortion;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn detects_web_mercator_high_latitude_distance_distortion() {
        // Web Mercator (EPSG:3857) significantly stretches distances at high latitudes.
        // Place two points near 60°N in Web Mercator projected coordinates.
        // EPSG:3857 at ~60°N: x ≈ lon * 111_319, y ≈ (R * ln(tan(π/4 + lat/2)))
        // Two points separated by ~100km horizontally at 60°N.
        use crate::core::config::Config;
        use crate::core::rule::{Feature, Layer};
        use std::collections::HashMap;

        // Approximate projected coords for points near 60°N, separated by ~1° longitude.
        // At 60°N in Web Mercator: y ≈ 8_399_737
        let x1 = -9_392_582.0_f64; // ~-84.4° lon
        let x2 = -9_281_263.0_f64; // ~-83.4° lon
        let y = 8_399_737.0_f64; // ~60°N

        let layer = Layer {
            name: "high_lat".into(),
            crs: Some("EPSG:3857".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(x1, y))),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(x2, y))),
                    properties: HashMap::new(),
                },
            ],
            bounds: None,
        };

        // Use a very low distortion threshold so the rule fires.
        let mut config = Config::default();
        config.check.max_distortion_pct = 0.1;
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = DistanceDistortion;
        let findings = rule.check(&ctx);
        assert!(
            !findings.is_empty(),
            "Expected distance distortion finding for Web Mercator at 60°N"
        );
        assert!(findings[0].metric.unwrap() > 0.0);
    }
}
