//! Rule: Check internal compression for cloud-optimized formats.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity};

/// Checks whether the dataset uses appropriate internal compression.
pub struct Compression;

impl Default for Compression {
    fn default() -> Self {
        Self
    }
}

impl Rule for Compression {
    fn id(&self) -> &str {
        "cloud/compression"
    }

    fn name(&self) -> &str {
        "Compression"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "compression", "performance"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let path = ctx.file_path.to_lowercase();

        // GeoParquet: check metadata for compression codec.
        if path.ends_with(".parquet") || path.ends_with(".geoparquet") {
            todo!("Parse GeoParquet metadata for compression codec (snappy, zstd, gzip)");
        }

        // Large uncompressed GeoJSON — suggest conversion.
        if path.ends_with(".geojson") || path.ends_with(".json") {
            let file_size = std::fs::metadata(ctx.file_path)
                .map(|m| m.len())
                .unwrap_or(0);
            let threshold = 10 * 1024 * 1024; // 10 MB
            if file_size > threshold {
                return vec![Finding {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    message: format!(
                        "GeoJSON file is {}MB with no internal compression. Consider converting to FlatGeobuf or GeoParquet",
                        file_size / (1024 * 1024)
                    ),
                    location: None,
                    geometry: None,
                    metric: Some(file_size as f64),
                    suggestion: Some(
                        "Convert to FlatGeobuf (streamable) or GeoParquet (compressed, columnar). See: https://guide.cloudnativegeo.org/".to_string()
                    ),
                    fixable: false,
                }];
            }
        }

        vec![]
    }

    fn score_weight(&self) -> f64 {
        0.3
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(Compression),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[test]
    fn rule_metadata() {
        let rule = Compression;
        assert_eq!(rule.id(), "cloud/compression");
        assert_eq!(rule.domain(), Domain::Cloud);
        assert_eq!(rule.default_severity(), Severity::Info);
    }

    #[test]
    fn skips_small_geojson() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "examples/datasets/simple_points.geojson",
        };
        let rule = Compression;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_non_applicable_formats() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "data.shp",
        };
        let rule = Compression;
        assert!(rule.check(&ctx).is_empty());
    }
}
