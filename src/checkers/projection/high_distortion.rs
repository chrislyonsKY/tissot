/// Rule: Flag high projection distortion via X-Ray analysis.
use crate::core::rule::{CheckContext, Domain, Finding, Rule, Severity, SpatialLocation};
use crate::xray;

/// Flags when projection distortion exceeds the configured threshold.
pub struct HighDistortion;

impl Default for HighDistortion {
    fn default() -> Self {
        Self
    }
}

impl Rule for HighDistortion {
    fn id(&self) -> &str {
        "projection/high-distortion"
    }

    fn name(&self) -> &str {
        "High Projection Distortion"
    }

    fn domain(&self) -> Domain {
        Domain::Projection
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let threshold = ctx.config.check.max_distortion_pct;

        for layer in ctx.layers {
            let crs = match &layer.crs {
                Some(c) => c.clone(),
                None => continue, // Missing CRS handled by another rule.
            };

            // Skip geographic CRS — no projection distortion.
            if crs == "EPSG:4326" || crs == "OGC:CRS84" {
                continue;
            }

            let report = match xray::analyze(layer, ctx.config, ctx.file_path) {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("X-Ray analysis failed for layer '{}': {e}", layer.name);
                    continue;
                }
            };

            if report.summary.max_area_distortion_pct > threshold {
                let severity = if report.summary.max_area_distortion_pct > threshold * 2.0 {
                    Severity::Error
                } else {
                    Severity::Warning
                };

                let suggestion = if let Some(rec) = report.recommendations.first() {
                    format!(
                        "Consider reprojecting to {} ({}). Run `tissot xray {}` for details.",
                        rec.crs, rec.name, ctx.file_path
                    )
                } else {
                    format!(
                        "Run `tissot xray {}` for CRS recommendations.",
                        ctx.file_path
                    )
                };

                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity,
                    message: format!(
                        "Layer '{}' has {:.1}% max area distortion (threshold: {:.1}%) in CRS {}",
                        layer.name, report.summary.max_area_distortion_pct, threshold, crs
                    ),
                    location: Some(SpatialLocation::Layer {
                        name: layer.name.clone(),
                    }),
                    geometry: None,
                    metric: Some(report.summary.max_area_distortion_pct),
                    suggestion: Some(suggestion),
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
    crate::core::rule::RuleEntry {
        factory: || Box::new(HighDistortion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::Layer;

    #[test]
    fn no_distortion_on_geographic_crs() {
        let layer = Layer {
            name: "geographic".into(),
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

        let rule = HighDistortion;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_missing_crs() {
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

        let rule = HighDistortion;
        assert!(rule.check(&ctx).is_empty());
    }
}
