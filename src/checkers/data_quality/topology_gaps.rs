//! Rule: Detect gaps between adjacent polygons in a layer.
//!
//! Uses an R-tree spatial index for efficient neighbor queries.
//! Gaps indicate missing coverage between polygon boundaries.

use geo::Geometry;

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

/// Detects topological gaps between adjacent polygons using R-tree spatial indexing.
pub struct TopologyGaps;

impl Default for TopologyGaps {
    fn default() -> Self {
        Self
    }
}

/// Check if a geometry is a polygon type (Polygon or MultiPolygon).
fn is_polygon_geometry(geom: &Geometry) -> bool {
    matches!(geom, Geometry::Polygon(_) | Geometry::MultiPolygon(_))
}

impl Rule for TopologyGaps {
    fn id(&self) -> &str {
        "data/topology-gaps"
    }

    fn name(&self) -> &str {
        "Topology Gaps"
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
        let mut findings = Vec::new();

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

            // R-tree based gap detection: build spatial index, find adjacent polygons,
            // compute difference to detect gap regions.
            // This requires computing the union boundary and finding uncovered areas.
            todo!("R-tree gap detection: build rstar index from polygon envelopes, query neighbors, compute gap geometries");

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
        factory: || Box::new(TopologyGaps),
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
        let rule = TopologyGaps;
        assert_eq!(rule.id(), "data/topology-gaps");
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

        let rule = TopologyGaps;
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

        let rule = TopologyGaps;
        assert!(rule.check(&ctx).is_empty());
    }
}
