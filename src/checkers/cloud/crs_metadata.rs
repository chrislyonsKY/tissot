//! Rule: Validate CRS metadata is present and embedded in the file.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

/// Checks that CRS metadata is properly embedded and readable.
pub struct CrsMetadata;

impl Default for CrsMetadata {
    fn default() -> Self {
        Self
    }
}

impl Rule for CrsMetadata {
    fn id(&self) -> &str {
        "cloud/crs-metadata"
    }

    fn name(&self) -> &str {
        "CRS Metadata"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "crs", "metadata"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let path = ctx.file_path.to_lowercase();

        for layer in ctx.layers {
            if layer.crs.is_none() {
                let message = if path.ends_with(".shp") {
                    format!(
                        "Layer '{}' has no CRS defined. Shapefile may be missing its .prj sidecar file",
                        layer.name
                    )
                } else {
                    format!(
                        "Layer '{}' has no CRS metadata embedded. All downstream spatial operations will assume an arbitrary coordinate system",
                        layer.name
                    )
                };

                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message,
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: None,
                    suggestion: Some(
                        "Define the CRS for this dataset. Use `tissot fix --reproject EPSG:4326` if the data is in WGS 84.".to_string(),
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
        factory: || Box::new(CrsMetadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;

    #[test]
    fn flags_missing_crs() {
        let layer = Layer {
            name: "roads".into(),
            crs: None,
            features: vec![],
            bounds: None,
        };
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "data.gpkg",
        };
        let rule = CrsMetadata;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
    }

    #[test]
    fn no_finding_when_crs_present() {
        let layer = Layer {
            name: "roads".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![],
            bounds: None,
        };
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "data.gpkg",
        };
        let rule = CrsMetadata;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn shapefile_specific_message() {
        let layer = Layer {
            name: "parcels".into(),
            crs: None,
            features: vec![],
            bounds: None,
        };
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "parcels.shp",
        };
        let rule = CrsMetadata;
        let findings = rule.check(&ctx);
        assert!(findings[0].message.contains(".prj"));
    }
}
