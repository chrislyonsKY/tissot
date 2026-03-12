//! Rule: Detect self-intersecting geometry rings.
//!
//! Invalid geometries with self-intersecting rings cause problems in spatial
//! operations like buffering, intersection, and area calculations.

use geo::Geometry;

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Flags geometries with self-intersecting rings (invalid geometry).
pub struct SelfIntersection;

impl Default for SelfIntersection {
    fn default() -> Self {
        Self
    }
}

/// Check whether a geometry has self-intersecting rings.
///
/// Examines each ring of polygon geometries for segments that cross each other.
/// Returns `true` if the geometry is invalid (has self-intersections).
fn has_self_intersection(geom: &Geometry) -> bool {
    match geom {
        Geometry::Polygon(poly) => {
            ring_self_intersects(poly.exterior())
                || poly.interiors().iter().any(ring_self_intersects)
        }
        Geometry::MultiPolygon(mp) => mp.0.iter().any(|poly| {
            ring_self_intersects(poly.exterior())
                || poly.interiors().iter().any(ring_self_intersects)
        }),
        _ => false,
    }
}

/// Check if a ring (LineString) has self-intersecting segments.
///
/// Tests all non-adjacent segment pairs for intersection using a brute-force
/// O(n²) sweep. For production use, this should be replaced with a sweep-line
/// algorithm for O(n log n) performance.
fn ring_self_intersects(ring: &geo::LineString) -> bool {
    let coords: Vec<_> = ring.0.clone();
    let n = coords.len();
    if n < 4 {
        return false;
    }

    for i in 0..n - 1 {
        let a1 = coords[i];
        let a2 = coords[i + 1];

        // Start j at i+2 to skip adjacent segments (they share a vertex).
        for j in (i + 2)..n - 1 {
            // Skip the pair of first and last segments (they share the closing vertex).
            if i == 0 && j == n - 2 {
                continue;
            }
            let b1 = coords[j];
            let b2 = coords[j + 1];

            if segments_intersect_properly(a1, a2, b1, b2) {
                return true;
            }
        }
    }
    false
}

/// Test whether two line segments (a1-a2) and (b1-b2) properly intersect
/// (cross each other, not just touch at endpoints).
fn segments_intersect_properly(
    a1: geo::Coord,
    a2: geo::Coord,
    b1: geo::Coord,
    b2: geo::Coord,
) -> bool {
    let d1 = cross_product_sign(b1, b2, a1);
    let d2 = cross_product_sign(b1, b2, a2);
    let d3 = cross_product_sign(a1, a2, b1);
    let d4 = cross_product_sign(a1, a2, b2);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    false
}

/// Compute the cross product of vectors (p2-p1) and (p3-p1).
fn cross_product_sign(p1: geo::Coord, p2: geo::Coord, p3: geo::Coord) -> f64 {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
}

impl Rule for SelfIntersection {
    fn id(&self) -> &str {
        "data/self-intersection"
    }

    fn name(&self) -> &str {
        "Self Intersection"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn tags(&self) -> &[&str] {
        &["topology", "geometry"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            for (idx, feature) in layer.features.iter().enumerate() {
                let geom = match &feature.geometry {
                    Some(g) => g,
                    None => continue,
                };

                if has_self_intersection(geom) {
                    let feature_label = feature.id.clone().unwrap_or_else(|| format!("#{idx}"));

                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Feature {feature_label} in layer '{}' has self-intersecting geometry",
                            layer.name
                        ),
                        location: Some(SpatialLocation::Feature {
                            id: feature_label,
                        }),
                        geometry: Some(geom.clone()),
                        metric: None,
                        suggestion: Some(
                            "Fix the geometry by removing self-intersections. Use `ST_MakeValid` or `tissot fix --topology`".into(),
                        ),
                        fixable: false,
                    });
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.9
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(SelfIntersection),
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
        let rule = SelfIntersection;
        assert_eq!(rule.id(), "data/self-intersection");
        assert_eq!(rule.domain(), Domain::DataQuality);
        assert_eq!(rule.default_severity(), Severity::Error);
        assert_eq!(rule.tags(), &["topology", "geometry"]);
    }

    #[test]
    fn detects_self_intersecting_polygon() {
        // A bowtie polygon: crosses itself.
        let poly = geo::Polygon::new(
            geo::LineString::from(vec![
                (0.0, 0.0),
                (2.0, 2.0),
                (2.0, 0.0),
                (0.0, 2.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let layer = Layer {
            name: "test".into(),
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

        let rule = SelfIntersection;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("self-intersecting"));
        assert!(findings[0].geometry.is_some());
    }

    #[test]
    fn no_findings_for_valid_polygon() {
        let poly = geo::Polygon::new(
            geo::LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let layer = Layer {
            name: "test".into(),
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

        let rule = SelfIntersection;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_point_geometries() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
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

        let rule = SelfIntersection;
        assert!(rule.check(&ctx).is_empty());
    }
}
