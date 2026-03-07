/// Rule: Detect features with null/missing geometry.
use crate::core::rule::{CheckContext, Domain, Feature, Finding, Rule, Severity, SpatialLocation};

/// Flags features that have null or missing geometry.
pub struct NullGeometry;

impl Default for NullGeometry {
    fn default() -> Self {
        Self
    }
}

impl Rule for NullGeometry {
    fn id(&self) -> &str {
        "data_quality/null-geometry"
    }

    fn name(&self) -> &str {
        "Null Geometry"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            for (idx, feature) in layer.features.iter().enumerate() {
                if feature.geometry.is_none() {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Feature {} has null geometry in layer '{}'",
                            feature_label(feature, idx),
                            layer.name
                        ),
                        location: Some(SpatialLocation::Feature {
                            id: feature_label(feature, idx),
                        }),
                        geometry: None,
                        metric: None,
                        suggestion: Some(
                            "Remove null-geometry features or add valid geometry".into(),
                        ),
                        fixable: false,
                    });
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.8
    }
}

/// Get feature label from id or index.
fn feature_label(feature: &Feature, idx: usize) -> String {
    feature.id.clone().unwrap_or_else(|| format!("#{idx}"))
}

inventory::submit! {
    crate::core::rule::RuleEntry {
        factory: || Box::new(NullGeometry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;
    use std::collections::HashMap;

    #[test]
    fn detects_null_geometry() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: None,
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
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

        let rule = NullGeometry;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].message.contains("null geometry"));
    }

    #[test]
    fn no_findings_when_all_valid() {
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

        let rule = NullGeometry;
        assert!(rule.check(&ctx).is_empty());
    }
}
