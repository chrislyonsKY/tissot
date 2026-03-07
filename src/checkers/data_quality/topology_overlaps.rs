//! Rule: Detect overlapping polygons in a layer.
//!
//! Uses an R-tree spatial index for efficient intersection queries.
//! Overlaps may indicate digitization errors or conflicting boundaries.

use geo::Geometry;

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

/// Detects overlapping polygons using R-tree spatial indexing.
pub struct TopologyOverlaps;

impl Default for TopologyOverlaps {
    fn default() -> Self {
        Self
    }
}

/// Check if a geometry is a polygon type (Polygon or MultiPolygon).
fn is_polygon_geometry(geom: &Geometry) -> bool {
    matches!(geom, Geometry::Polygon(_) | Geometry::MultiPolygon(_))
}

impl Rule for TopologyOverlaps {
    fn id(&self) -> &str {
        "data/topology-overlaps"
    }

    fn name(&self) -> &str {
        "Topology Overlaps"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["topology", "polygon"]
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let findings = Vec::new();

        for layer in ctx.layers {
            // Collect polygon features for this layer.
            let polygons: Vec<(usize, &Geometry)> = layer
                .features
                .iter()
                .enumerate()
                .filter_map(|(idx, f)| {
                    f.geometry
                        .as_ref()
                        .filter(|g| is_polygon_geometry(g))
                        .map(|g| (idx, g))
                })
                .collect();

            if polygons.len() < 2 {
                continue;
            }

            // R-tree based overlap detection: build spatial index from envelopes,
            // for each polygon find candidates with overlapping bounding boxes,
            // compute actual polygon intersection to detect overlapping regions.
            todo!("R-tree overlap detection: build rstar index, query intersecting envelopes, compute pairwise polygon intersections");

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
        0.7
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(TopologyOverlaps),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap;

    #[test]
    fn rule_metadata() {
        let rule = TopologyOverlaps;
        assert_eq!(rule.id(), "data/topology-overlaps");
        assert_eq!(rule.domain(), Domain::DataQuality);
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.can_fix());
        assert_eq!(rule.tags(), &["topology", "polygon"]);
    }

    #[test]
    fn skips_non_polygon_layers() {
        let layer = Layer {
            name: "points".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 1.0))),
                    properties: HashMap::new(),
                },
            ],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = TopologyOverlaps;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_single_polygon() {
        let poly = geo::Polygon::new(
            geo::LineString::from(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)]),
            vec![],
        );
        let layer = Layer {
            name: "single".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![Feature {
                id: Some("1".into()),
                geometry: Some(geo::Geometry::Polygon(poly)),
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

        let rule = TopologyOverlaps;
        assert!(rule.check(&ctx).is_empty());
    }
}
