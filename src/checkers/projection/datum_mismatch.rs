//! Rule: Detect CRS/datum mismatches across layers in a multi-layer dataset.
//!
//! When multiple layers reference different coordinate reference systems or datums,
//! overlaying them without transformation will produce incorrect spatial relationships.

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Flags layers that reference different CRS or datums within the same dataset.
pub struct DatumMismatch;

impl Default for DatumMismatch {
    fn default() -> Self {
        Self
    }
}

impl Rule for DatumMismatch {
    fn id(&self) -> &str {
        "proj/datum-mismatch"
    }

    fn name(&self) -> &str {
        "Datum Mismatch"
    }

    fn domain(&self) -> Domain {
        Domain::Projection
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn tags(&self) -> &[&str] {
        &["projection", "datum", "crs"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Collect layers that have a defined CRS.
        let layers_with_crs: Vec<(&str, &str)> = ctx
            .layers
            .iter()
            .filter_map(|l| l.crs.as_deref().map(|c| (l.name.as_str(), c)))
            .collect();

        if layers_with_crs.len() < 2 {
            return findings;
        }

        // Compare all pairs for CRS mismatches.
        let reference_crs = layers_with_crs[0].1;
        let reference_name = layers_with_crs[0].0;

        for &(layer_name, layer_crs) in &layers_with_crs[1..] {
            if layer_crs != reference_crs {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!(
                        "CRS mismatch: layer '{}' uses {} but layer '{}' uses {}",
                        reference_name, reference_crs, layer_name, layer_crs
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer_name.to_string(),
                    }),
                    geometry: None,
                    metric: None,
                    suggestion: Some(
                        "Reproject all layers to a common CRS. Run `tissot fix --reproject` to align to a recommended CRS.".to_string()
                    ),
                    fixable: true,
                });
            }
        }

        findings
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn score_weight(&self) -> f64 {
        1.0
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(DatumMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;

    #[test]
    fn rule_metadata() {
        let rule = DatumMismatch;
        assert_eq!(rule.id(), "proj/datum-mismatch");
        assert_eq!(rule.domain(), Domain::Projection);
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.can_fix());
    }

    #[test]
    fn detects_crs_mismatch() {
        let layers = vec![
            Layer {
                name: "parcels".into(),
                crs: Some("EPSG:4326".into()),
                features: vec![],
                bounds: None,
            },
            Layer {
                name: "roads".into(),
                crs: Some("EPSG:3857".into()),
                features: vec![],
                bounds: None,
            },
        ];

        let config = Config::default();
        let ctx = CheckContext {
            layers: &layers,
            config: &config,
            file_path: "test.gpkg",
        };

        let rule = DatumMismatch;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("CRS mismatch"));
        assert!(findings[0].message.contains("4326"));
        assert!(findings[0].message.contains("3857"));
    }

    #[test]
    fn no_findings_when_crs_matches() {
        let layers = vec![
            Layer {
                name: "parcels".into(),
                crs: Some("EPSG:4326".into()),
                features: vec![],
                bounds: None,
            },
            Layer {
                name: "roads".into(),
                crs: Some("EPSG:4326".into()),
                features: vec![],
                bounds: None,
            },
        ];

        let config = Config::default();
        let ctx = CheckContext {
            layers: &layers,
            config: &config,
            file_path: "test.gpkg",
        };

        let rule = DatumMismatch;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_single_layer() {
        let layers = vec![Layer {
            name: "only".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![],
            bounds: None,
        }];

        let config = Config::default();
        let ctx = CheckContext {
            layers: &layers,
            config: &config,
            file_path: "test.gpkg",
        };

        let rule = DatumMismatch;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn detects_multiple_mismatches() {
        let layers = vec![
            Layer {
                name: "a".into(),
                crs: Some("EPSG:4326".into()),
                features: vec![],
                bounds: None,
            },
            Layer {
                name: "b".into(),
                crs: Some("EPSG:3857".into()),
                features: vec![],
                bounds: None,
            },
            Layer {
                name: "c".into(),
                crs: Some("EPSG:3089".into()),
                features: vec![],
                bounds: None,
            },
        ];

        let config = Config::default();
        let ctx = CheckContext {
            layers: &layers,
            config: &config,
            file_path: "test.gpkg",
        };

        let rule = DatumMismatch;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 2);
    }
}
