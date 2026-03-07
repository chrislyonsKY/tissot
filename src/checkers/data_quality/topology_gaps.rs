//! Rule: Detect gaps between adjacent polygons in a layer.
//!
//! Uses an R-tree spatial index for efficient neighbor queries.
//! Gaps indicate missing coverage between polygon boundaries.

use geo::{BoundingRect, Distance, Euclidean, Geometry, Polygon};
use rstar::{AABB, RTree, RTreeObject};

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Detects topological gaps between adjacent polygons using R-tree spatial indexing.
pub struct TopologyGaps;

impl Default for TopologyGaps {
    fn default() -> Self {
        Self
    }
}

/// Lightweight record for R-tree indexing of polygon bounding boxes.
struct IndexedBbox {
    /// Index into the owning `polys` slice.
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

/// Extract all `Polygon<f64>` instances from a geometry.
fn extract_polygons(geom: &Geometry) -> Vec<Polygon<f64>> {
    match geom {
        Geometry::Polygon(p) => vec![p.clone()],
        Geometry::MultiPolygon(mp) => mp.0.clone(),
        _ => vec![],
    }
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

            // Compute a gap tolerance as a fraction of the layer's overall extent.
            // Using a relative tolerance (0.01% of the longest axis) makes the
            // threshold proportional to the CRS units and dataset scale: it is roughly
            // 1 cm for a 100 m plot, 10 m for a 100 km county dataset, or 0.01° for
            // a global dataset in WGS 84. The 1e-8 clamp prevents division-by-zero
            // issues on degenerate single-point extents.
            let (min_x, min_y, max_x, max_y) =
                polys.iter().filter_map(|(_, p)| p.bounding_rect()).fold(
                    (f64::MAX, f64::MAX, f64::MIN, f64::MIN),
                    |(ax, ay, bx, by), r| {
                        (
                            ax.min(r.min().x),
                            ay.min(r.min().y),
                            bx.max(r.max().x),
                            by.max(r.max().y),
                        )
                    },
                );
            let extent = (max_x - min_x).max(max_y - min_y);
            let gap_tolerance = (extent * 1e-4).max(1e-8);

            // Build R-tree with bounding boxes expanded by `gap_tolerance` on each side
            // so that nearby polygons appear as bbox-intersecting candidates.
            let entries: Vec<IndexedBbox> = polys
                .iter()
                .enumerate()
                .filter_map(|(idx, (_, poly))| {
                    poly.bounding_rect().map(|r| IndexedBbox {
                        idx,
                        min: [r.min().x - gap_tolerance, r.min().y - gap_tolerance],
                        max: [r.max().x + gap_tolerance, r.max().y + gap_tolerance],
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

                // Expand the query envelope by gap_tolerance to find near-neighbors.
                let query = AABB::from_corners(
                    [
                        bbox_a.min().x - gap_tolerance,
                        bbox_a.min().y - gap_tolerance,
                    ],
                    [
                        bbox_a.max().x + gap_tolerance,
                        bbox_a.max().y + gap_tolerance,
                    ],
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

                    // The R-tree already filtered to bbox-intersecting candidates after
                    // expansion by `gap_tolerance`, so only genuinely nearby polygons
                    // reach this exact distance call. Exact polygon distance is only
                    // computed for this small candidate set.
                    let dist =
                        Euclidean::distance(poly_a as &Polygon<f64>, poly_b as &Polygon<f64>);

                    // A gap exists when polygons are close but don't actually touch.
                    // dist == 0.0 means they touch or overlap (handled by topology-overlaps).
                    if dist > 0.0 && dist <= gap_tolerance {
                        reported.insert(pair);

                        findings.push(Finding {
                            rule_id: self.id().to_string(),
                            severity: self.default_severity(),
                            message: format!(
                                "Gap of {dist:.6} units between features {label_a} and {label_b} in layer '{}'",
                                layer.name
                            ),
                            location: Some(SpatialLocation::Layer {
                                name: layer.name.clone(),
                            }),
                            geometry: None,
                            metric: Some(dist),
                            suggestion: Some(
                                "Close the gap by snapping polygon vertices. Use `tissot fix --topology` for automated repair.".to_string(),
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

    #[test]
    fn detects_small_gap_between_polygons() {
        // poly_a: [0,0]–[1,1], poly_b: [1.00001,0]–[2.00001,1]
        // The gap is 0.00001 units — within the tolerance (fraction of 2-unit extent).
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
        let gap = 0.00001_f64;
        let poly_b = geo::Polygon::new(
            geo::LineString::from(vec![
                (1.0 + gap, 0.0),
                (2.0 + gap, 0.0),
                (2.0 + gap, 1.0),
                (1.0 + gap, 1.0),
                (1.0 + gap, 0.0),
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

        let rule = TopologyGaps;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1, "Expected one gap finding");
        assert!(findings[0].message.contains("Gap"));
        assert!(findings[0].metric.unwrap() > 0.0);
        assert!(findings[0].fixable);
    }

    #[test]
    fn no_findings_for_touching_polygons() {
        // poly_a: [0,0]–[1,1], poly_b: [1,0]–[2,1] — they share exactly one edge.
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

        let rule = TopologyGaps;
        // Touching polygons (dist == 0) should not be reported as gaps.
        assert!(rule.check(&ctx).is_empty());
    }
}
