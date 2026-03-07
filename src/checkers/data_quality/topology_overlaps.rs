//! Rule: Detect overlapping polygons in a layer.
//!
//! Uses an R-tree spatial index for efficient intersection queries.
//! Overlaps may indicate digitization errors or conflicting boundaries.

use geo::{BooleanOps, BoundingRect, Geometry, Polygon};
use rstar::{AABB, RTree, RTreeObject};

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Detects overlapping polygons using R-tree spatial indexing.
pub struct TopologyOverlaps;

impl Default for TopologyOverlaps {
    fn default() -> Self {
        Self
    }
}

/// Lightweight polygon record for R-tree indexing.
struct IndexedBbox {
    /// Index into the owning `polygons` slice.
    idx: usize,
    min: [f64; 2],
    max: [f64; 2],
}

impl RTreeObject for IndexedBbox {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.min, self.max)
    }
}

/// Extract all `Polygon<f64>` instances from a geometry (handles MultiPolygon).
fn extract_polygons(geom: &Geometry) -> Vec<Polygon<f64>> {
    match geom {
        Geometry::Polygon(p) => vec![p.clone()],
        Geometry::MultiPolygon(mp) => mp.0.clone(),
        _ => vec![],
    }
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
        let mut findings = Vec::new();

        for layer in ctx.layers {
            // Collect (feature_label, polygon) pairs from all polygon-type features.
            let mut polys: Vec<(String, Polygon<f64>)> = Vec::new();
            for feature in &layer.features {
                let geom = match &feature.geometry {
                    Some(g) => g,
                    None => continue,
                };
                let label = feature
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("#{}", polys.len()));
                for poly in extract_polygons(geom) {
                    polys.push((label.clone(), poly));
                }
            }

            if polys.len() < 2 {
                continue;
            }

            // Build R-tree from bounding boxes for efficient candidate filtering.
            let entries: Vec<IndexedBbox> = polys
                .iter()
                .enumerate()
                .filter_map(|(idx, (_, poly))| {
                    poly.bounding_rect().map(|r| IndexedBbox {
                        idx,
                        min: [r.min().x, r.min().y],
                        max: [r.max().x, r.max().y],
                    })
                })
                .collect();

            let tree = RTree::bulk_load(entries);

            // Track reported pairs to avoid duplicate findings.
            let mut reported: std::collections::HashSet<(usize, usize)> =
                std::collections::HashSet::new();

            for (i, (label_a, poly_a)) in polys.iter().enumerate() {
                let bbox_a = match poly_a.bounding_rect() {
                    Some(b) => b,
                    None => continue,
                };

                let query = AABB::from_corners(
                    [bbox_a.min().x, bbox_a.min().y],
                    [bbox_a.max().x, bbox_a.max().y],
                );

                for candidate in tree.locate_in_envelope_intersecting(&query) {
                    let j = candidate.idx;
                    if j <= i {
                        continue;
                    }
                    let pair = (i, j);
                    if reported.contains(&pair) {
                        continue;
                    }

                    let (label_b, poly_b) = &polys[j];

                    // Compute the actual intersection area.
                    // We use BooleanOps::intersection() for accuracy; the area threshold
                    // filters out floating-point noise from boundary-only touches.
                    // 1e-10 sq units is negligible for any realistic polygon dataset.
                    use geo::Area;
                    let intersection = poly_a.intersection(poly_b);
                    let area = intersection.unsigned_area();

                    if area > 1e-10 {
                        reported.insert(pair);

                        findings.push(Finding {
                            rule_id: self.id().to_string(),
                            severity: self.default_severity(),
                            message: format!(
                                "Features {label_a} and {label_b} in layer '{}' overlap (shared area: {area:.4} sq units)",
                                layer.name
                            ),
                            location: Some(SpatialLocation::Layer {
                                name: layer.name.clone(),
                            }),
                            geometry: intersection
                                .bounding_rect()
                                .map(geo::Geometry::Rect),
                            metric: Some(area),
                            suggestion: Some(
                                "Remove overlapping areas. Use `tissot fix --topology` for automated repair.".to_string(),
                            ),
                            fixable: true,
                        });
                    }
                }
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

    #[test]
    fn detects_overlapping_polygons() {
        // poly_a: [0,0]–[2,2], poly_b: [1,0]–[3,2] — they share a 1×2 interior strip.
        let poly_a = geo::Polygon::new(
            geo::LineString::from(vec![
                (0.0, 0.0),
                (2.0, 0.0),
                (2.0, 2.0),
                (0.0, 2.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let poly_b = geo::Polygon::new(
            geo::LineString::from(vec![
                (1.0, 0.0),
                (3.0, 0.0),
                (3.0, 2.0),
                (1.0, 2.0),
                (1.0, 0.0),
            ]),
            vec![],
        );
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(geo::Geometry::Polygon(poly_a)),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(geo::Geometry::Polygon(poly_b)),
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
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("overlap"));
        assert!(findings[0].metric.unwrap() > 0.0);
        assert!(findings[0].fixable);
    }

    #[test]
    fn no_findings_for_adjacent_polygons() {
        // poly_a: [0,0]–[1,1], poly_b: [1,0]–[2,1] — they share only a boundary edge.
        let poly_a = geo::Polygon::new(
            geo::LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let poly_b = geo::Polygon::new(
            geo::LineString::from(vec![
                (1.0, 0.0),
                (2.0, 0.0),
                (2.0, 1.0),
                (1.0, 1.0),
                (1.0, 0.0),
            ]),
            vec![],
        );
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(geo::Geometry::Polygon(poly_a)),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(geo::Geometry::Polygon(poly_b)),
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
}
