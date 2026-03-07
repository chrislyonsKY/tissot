/// Rule: Detect layers with missing or undefined CRS.
use crate::core::rule::{CheckContext, Domain, Finding, Rule, Severity, SpatialLocation};

/// Flags layers that have no CRS defined.
pub struct MissingCrs;

impl Default for MissingCrs {
    fn default() -> Self {
        Self
    }
}

impl Rule for MissingCrs {
    fn id(&self) -> &str {
        "projection/missing-crs"
    }

    fn name(&self) -> &str {
        "Missing CRS"
    }

    fn domain(&self) -> Domain {
        Domain::Projection
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            if layer.crs.is_none() {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!(
                        "Layer '{}' has no coordinate reference system (CRS) defined",
                        layer.name
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: None,
                    suggestion: Some(
                        "Define a CRS for the layer. If coordinates are lon/lat, use EPSG:4326."
                            .into(),
                    ),
                    fixable: false,
                });
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        1.0
    }
}

inventory::submit! {
    crate::core::rule::RuleEntry {
        factory: || Box::new(MissingCrs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;

    #[test]
    fn detects_missing_crs() {
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

        let rule = MissingCrs;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
    }

    #[test]
    fn no_finding_when_crs_present() {
        let layer = Layer {
            name: "valid".into(),
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

        let rule = MissingCrs;
        assert!(rule.check(&ctx).is_empty());
    }
}
