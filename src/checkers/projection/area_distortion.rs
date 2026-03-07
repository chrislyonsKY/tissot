//! Rule: Flag high area distortion by wrapping the X-Ray distortion engine.
//!
//! Configurable threshold via `check.max_distortion_pct` (default: 5.0%).
//! Reports Warning above 5%, Error above 10%. Fixable via reprojection.

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Flags layers with excessive area distortion by invoking the X-Ray engine.
///
/// This rule wraps `crate::xray::analyze` to evaluate area scale factor
/// deviation across sample points in the dataset.
pub struct AreaDistortion {
    /// Maximum acceptable area distortion percentage before Warning.
    pub warning_threshold_pct: f64,
    /// Maximum acceptable area distortion percentage before Error.
    pub error_threshold_pct: f64,
}

impl Default for AreaDistortion {
    fn default() -> Self {
        Self {
            warning_threshold_pct: 5.0,
            error_threshold_pct: 10.0,
        }
    }
}

impl Rule for AreaDistortion {
    fn id(&self) -> &str {
        "proj/area-distortion"
    }

    fn name(&self) -> &str {
        "Area Distortion"
    }

    fn domain(&self) -> Domain {
        Domain::Projection
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["projection", "distortion", "area"]
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let warning_threshold = ctx
            .config
            .check
            .max_distortion_pct
            .min(self.warning_threshold_pct);

        for layer in ctx.layers {
            let crs = match &layer.crs {
                Some(c) => c.clone(),
                None => continue,
            };

            // Skip geographic CRS — no projection distortion to measure.
            if crs == "EPSG:4326" || crs == "OGC:CRS84" {
                continue;
            }

            // Run X-Ray analysis on the layer.
            let report = match crate::xray::analyze(layer, ctx.config, ctx.file_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("X-Ray analysis failed for layer '{}': {e}", layer.name);
                    continue;
                }
            };

            let max_distortion = report.summary.max_area_distortion_pct;
            let mean_distortion = report.summary.mean_area_distortion_pct;

            if max_distortion > warning_threshold {
                let severity = if max_distortion > self.error_threshold_pct {
                    Severity::Error
                } else {
                    Severity::Warning
                };

                let suggestion = if let Some(rec) = report.recommendations.first() {
                    format!(
                        "Consider reprojecting to {} ({}). Max area distortion: {:.1}%, mean: {:.1}%. Run `tissot fix --reproject`.",
                        rec.crs, rec.name, max_distortion, mean_distortion
                    )
                } else {
                    format!(
                        "Run `tissot xray {file}` for CRS recommendations. Max area distortion: {:.1}%.",
                        max_distortion,
                        file = ctx.file_path
                    )
                };

                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity,
                    message: format!(
                        "Layer '{}' has {:.1}% max area distortion (mean: {:.1}%) in CRS {crs}",
                        layer.name, max_distortion, mean_distortion
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: Some(max_distortion),
                    suggestion: Some(suggestion),
                    fixable: true,
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
    RuleEntry {
        factory: || Box::new(AreaDistortion::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata() {
        let rule = AreaDistortion::default();
        assert_eq!(rule.id(), "proj/area-distortion");
        assert_eq!(rule.domain(), Domain::Projection);
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.can_fix());
        assert!(rule.tags().contains(&"area"));
    }

    #[test]
    fn default_thresholds() {
        let rule = AreaDistortion::default();
        assert!((rule.warning_threshold_pct - 5.0).abs() < f64::EPSILON);
        assert!((rule.error_threshold_pct - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn skips_geographic_crs() {
        use crate::core::config::Config;
        use crate::core::rule::Layer;

        let layer = Layer {
            name: "wgs84".into(),
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

        let rule = AreaDistortion::default();
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_missing_crs() {
        use crate::core::config::Config;
        use crate::core::rule::Layer;

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

        let rule = AreaDistortion::default();
        assert!(rule.check(&ctx).is_empty());
    }
}
