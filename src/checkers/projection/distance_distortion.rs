//! Rule: Detect distance distortion by comparing projected vs geodesic distances.
//!
//! Samples pairs of feature centroids and computes the error between
//! Euclidean distance in the projected CRS and geodesic distance on the ellipsoid.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

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

            // Sample centroid pairs, compute projected (Euclidean) vs geodesic distance,
            // and report deviation as percentage error.
            // Requires inverse-projecting points to geographic coords for geodesic calc.
            todo!("Distance distortion: inverse-project centroids to WGS84 via proj, compute geodesic vs Euclidean distance, report max/mean error");

            #[allow(unreachable_code)]
            {
                let _ = &findings;
                let _ = SpatialLocation::Layer {
                    name: layer.name.clone(),
                };
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
}
