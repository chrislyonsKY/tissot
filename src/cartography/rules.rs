/// Basic cartography rules for Phase 3 bootstrap.
use crate::core::rule::{CheckContext, Domain, Finding, Rule, Severity, SpatialLocation};

/// Flags datasets that appear to rely only on raw default styling metadata.
pub struct MissingSymbologyHints;

impl Rule for MissingSymbologyHints {
    fn id(&self) -> &str {
        "cartography/missing-symbology-hints"
    }

    fn name(&self) -> &str {
        "Missing Symbology Hints"
    }

    fn domain(&self) -> Domain {
        Domain::Cartography
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for layer in ctx.layers {
            if layer.features.iter().all(|f| f.properties.is_empty()) {
                out.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!(
                        "Layer '{}' has no style/classification hint attributes; cartographic checks are limited",
                        layer.name
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: None,
                    suggestion: Some("Include style or class fields to enable deeper cartographic linting".to_string()),
                    fixable: false,
                });
            }
        }
        out
    }
}

inventory::submit! {
    crate::core::rule::RuleEntry {
        factory: || Box::new(MissingSymbologyHints),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_id_stable() {
        assert_eq!(
            MissingSymbologyHints.id(),
            "cartography/missing-symbology-hints"
        );
    }
}
