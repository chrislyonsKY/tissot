//! Rule: Check if point/label features are too dense (likely to overlap).
//!
//! Uses rstar spatial indexing to efficiently find clusters of nearby points.
//! When features are packed into a small area, labels will overlap and become
//! unreadable on a map.

use geo::{BoundingRect, Coord, Geometry};
use rstar::{RTree, primitives::GeomWithData};

use crate::core::rule::{
    CheckContext, Domain, Feature, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Default search radius in coordinate units for clustering detection.
/// For WGS 84 data this is roughly 0.001 degrees (~111 meters at equator).
const DEFAULT_SEARCH_RADIUS: f64 = 0.001;

/// Minimum number of neighbors within the search radius to flag as dense.
const DEFAULT_DENSITY_THRESHOLD: usize = 5;

/// Checks if point features are too densely packed, causing label overlap.
pub struct LabelDensity;

impl Default for LabelDensity {
    fn default() -> Self {
        Self
    }
}

/// Extract a representative point coordinate from a geometry.
fn centroid_coord(geom: &Geometry) -> Option<Coord> {
    match geom {
        Geometry::Point(p) => Some(p.0),
        Geometry::MultiPoint(mp) => {
            if mp.0.is_empty() {
                None
            } else {
                Some(mp.0[0].0)
            }
        }
        other => {
            // For polygons/lines, use the center of the bounding box.
            let rect = other.bounding_rect()?;
            Some(rect.center())
        }
    }
}

/// Label for a feature used in finding messages.
fn feature_label(feature: &Feature, idx: usize) -> String {
    feature.id.clone().unwrap_or_else(|| format!("#{idx}"))
}

impl Rule for LabelDensity {
    fn id(&self) -> &str {
        "cartography/label-density"
    }

    fn name(&self) -> &str {
        "Label Density"
    }

    fn domain(&self) -> Domain {
        Domain::Cartography
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["cartography", "labels", "readability"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            // Build spatial index of all feature centroids.
            let points: Vec<GeomWithData<[f64; 2], usize>> = layer
                .features
                .iter()
                .enumerate()
                .filter_map(|(idx, feature)| {
                    let geom = feature.geometry.as_ref()?;
                    let coord = centroid_coord(geom)?;
                    Some(GeomWithData::new([coord.x, coord.y], idx))
                })
                .collect();

            if points.is_empty() {
                continue;
            }

            let tree = RTree::bulk_load(points);

            // Track which features have already been reported to avoid duplicates.
            let mut reported: std::collections::HashSet<usize> = std::collections::HashSet::new();

            for (idx, feature) in layer.features.iter().enumerate() {
                if reported.contains(&idx) {
                    continue;
                }

                let geom = match &feature.geometry {
                    Some(g) => g,
                    None => continue,
                };

                let coord = match centroid_coord(geom) {
                    Some(c) => c,
                    None => continue,
                };

                // Count neighbors within the search radius using the spatial index.
                let envelope = rstar::AABB::from_corners(
                    [
                        coord.x - DEFAULT_SEARCH_RADIUS,
                        coord.y - DEFAULT_SEARCH_RADIUS,
                    ],
                    [
                        coord.x + DEFAULT_SEARCH_RADIUS,
                        coord.y + DEFAULT_SEARCH_RADIUS,
                    ],
                );

                let neighbors: Vec<&GeomWithData<[f64; 2], usize>> =
                    tree.locate_in_envelope(&envelope).collect();

                // Subtract 1 because the point itself is included.
                let neighbor_count = neighbors.len().saturating_sub(1);

                if neighbor_count >= DEFAULT_DENSITY_THRESHOLD {
                    // Mark all neighbors as reported to reduce noise.
                    for neighbor in &neighbors {
                        reported.insert(neighbor.data);
                    }

                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Feature {} in layer '{}' has {} neighbors within {:.4} units — labels will likely overlap",
                            feature_label(feature, idx),
                            layer.name,
                            neighbor_count,
                            DEFAULT_SEARCH_RADIUS,
                        ),
                        location: Some(SpatialLocation::BoundingBox {
                            min_x: coord.x - DEFAULT_SEARCH_RADIUS,
                            min_y: coord.y - DEFAULT_SEARCH_RADIUS,
                            max_x: coord.x + DEFAULT_SEARCH_RADIUS,
                            max_y: coord.y + DEFAULT_SEARCH_RADIUS,
                        }),
                        geometry: Some(geom.clone()),
                        metric: Some(neighbor_count as f64),
                        suggestion: Some(
                            "Reduce label density by filtering features at this zoom level, \
                             using label collision detection, or clustering nearby points"
                                .to_string(),
                        ),
                        fixable: false,
                    });
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.5
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(LabelDensity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;
    use std::collections::HashMap;

    fn make_point_feature(x: f64, y: f64) -> Feature {
        Feature {
            id: None,
            geometry: Some(Geometry::Point(geo::Point::new(x, y))),
            properties: HashMap::new(),
        }
    }

    #[test]
    fn flags_dense_cluster() {
        // Create a tight cluster of 8 points within a small area.
        let features: Vec<Feature> = (0..8)
            .map(|i| make_point_feature(10.0 + (i as f64) * 0.0001, 20.0 + (i as f64) * 0.0001))
            .collect();

        let layer = Layer {
            name: "cities".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = LabelDensity;
        let findings = rule.check(&ctx);
        assert!(
            !findings.is_empty(),
            "Should flag dense cluster of 8 points"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn no_finding_for_spread_out_points() {
        // Points spread far apart — no density issue.
        let features: Vec<Feature> = (0..5)
            .map(|i| make_point_feature(i as f64 * 10.0, i as f64 * 10.0))
            .collect();

        let layer = Layer {
            name: "cities".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = LabelDensity;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn rule_metadata() {
        let rule = LabelDensity;
        assert_eq!(rule.id(), "cartography/label-density");
        assert_eq!(rule.domain(), Domain::Cartography);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }

    #[test]
    fn handles_empty_layer() {
        let layer = Layer {
            name: "empty".into(),
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

        let rule = LabelDensity;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn handles_null_geometries() {
        let features = vec![
            Feature {
                id: Some("1".into()),
                geometry: None,
                properties: HashMap::new(),
            },
            make_point_feature(0.0, 0.0),
        ];

        let layer = Layer {
            name: "mixed".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = LabelDensity;
        // Should not panic on null geometries.
        let _ = rule.check(&ctx);
    }
}
