/// Rule: Detect empty datasets (no features).
use crate::core::rule::{CheckContext, Domain, Finding, Rule, Severity, SpatialLocation};

/// Flags layers that contain zero features.
pub struct EmptyDataset;

impl Default for EmptyDataset {
    fn default() -> Self {
        Self
    }
}

impl Rule for EmptyDataset {
    fn id(&self) -> &str {
        "data_quality/empty-dataset"
    }

    fn name(&self) -> &str {
        "Empty Dataset"
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
            if layer.features.is_empty() {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!("Layer '{}' contains no features", layer.name),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: Some(0.0),
                    suggestion: Some(
                        "Verify the data source is correct and contains features".into(),
                    ),
                    fixable: false,
                });
            }
        }

        findings
    }
}

inventory::submit! {
    crate::core::rule::RuleEntry {
        factory: || Box::new(EmptyDataset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;

    #[test]
    fn detects_empty_layer() {
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
            file_path: "empty.geojson",
        };

        let rule = EmptyDataset;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("no features"));
    }

    #[test]
    fn no_finding_when_has_features() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![crate::core::rule::Feature {
                id: None,
                geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                properties: std::collections::HashMap::new(),
            }],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = EmptyDataset;
        assert!(rule.check(&ctx).is_empty());
    }
}
