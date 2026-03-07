//! Rule: Flag features whose geometry falls outside the layer's declared bounding box.

use geo::{BoundingRect, Coord, Rect};

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

/// Flags features with geometry extending beyond the layer's declared extent.
pub struct ExtentBounds;

impl Default for ExtentBounds {
    fn default() -> Self {
        Self
    }
}

impl Rule for ExtentBounds {
    fn id(&self) -> &str {
        "data/extent-bounds"
    }

    fn name(&self) -> &str {
        "Extent Bounds"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            // Need a declared bounding box to check against.
            let bounds = match layer.bounds {
                Some(b) => Rect::new(Coord { x: b[0], y: b[1] }, Coord { x: b[2], y: b[3] }),
                None => continue,
            };

            for (idx, feature) in layer.features.iter().enumerate() {
                let geom = match &feature.geometry {
                    Some(g) => g,
                    None => continue,
                };

                let feature_rect = match geom.bounding_rect() {
                    Some(r) => r,
                    None => continue,
                };

                // Check if feature bbox extends outside declared layer bounds.
                if feature_rect.min().x < bounds.min().x
                    || feature_rect.min().y < bounds.min().y
                    || feature_rect.max().x > bounds.max().x
                    || feature_rect.max().y > bounds.max().y
                {
                    let feature_label = feature
                        .id
                        .clone()
                        .unwrap_or_else(|| format!("#{idx}"));

                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Feature {feature_label} in layer '{}' extends outside declared bounds",
                            layer.name
                        ),
                        location: Some(SpatialLocation::Feature {
                            id: feature_label,
                        }),
                        geometry: Some(geom.clone()),
                        metric: None,
                        suggestion: Some(
                            "Update the layer extent or correct the feature geometry".into(),
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
        factory: || Box::new(ExtentBounds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap;

    #[test]
    fn detects_out_of_bounds_feature() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![Feature {
                id: Some("1".into()),
                geometry: Some(geo::Geometry::Point(geo::Point::new(200.0, 100.0))),
                properties: HashMap::new(),
            }],
            bounds: Some([-180.0, -90.0, 180.0, 90.0]),
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ExtentBounds;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("outside declared bounds"));
        assert!(findings[0].geometry.is_some());
    }

    #[test]
    fn no_findings_within_bounds() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![Feature {
                id: Some("1".into()),
                geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                properties: HashMap::new(),
            }],
            bounds: Some([-180.0, -90.0, 180.0, 90.0]),
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ExtentBounds;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_when_no_bounds_declared() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![Feature {
                id: Some("1".into()),
                geometry: Some(geo::Geometry::Point(geo::Point::new(200.0, 100.0))),
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

        let rule = ExtentBounds;
        assert!(rule.check(&ctx).is_empty());
    }
}
