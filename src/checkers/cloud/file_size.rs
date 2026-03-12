//! Rule: Flag files that are too large or too small for cloud optimization.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity};

/// Flags files outside the efficient size range for cloud-native access.
pub struct FileSize;

impl Default for FileSize {
    fn default() -> Self {
        Self
    }
}

impl Rule for FileSize {
    fn id(&self) -> &str {
        "cloud/file-size"
    }

    fn name(&self) -> &str {
        "File Size"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "size", "performance"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let file_size = match std::fs::metadata(ctx.file_path) {
            Ok(m) => m.len(),
            Err(_) => return vec![],
        };

        let path = ctx.file_path.to_lowercase();
        let two_gb = 2 * 1024 * 1024 * 1024u64;
        let one_mb = 1024 * 1024u64;

        // Shapefile > 2GB: hard limit.
        if path.ends_with(".shp") && file_size > two_gb {
            return vec![Finding {
                rule_id: self.id().to_string(),
                severity: Severity::Error,
                message: "File exceeds Shapefile's 2GB limit. Data may be truncated".to_string(),
                location: None,
                geometry: None,
                metric: Some(file_size as f64),
                suggestion: Some(
                    "Convert to GeoParquet or FlatGeobuf which have no size limits".to_string(),
                ),
                fixable: false,
            }];
        }

        // Any file > 2GB: suggest partitioning.
        if file_size > two_gb {
            return vec![Finding {
                rule_id: self.id().to_string(),
                severity: self.default_severity(),
                message: format!(
                    "File is {:.1}GB. Consider partitioning for efficient cloud access",
                    file_size as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                location: None,
                geometry: None,
                metric: Some(file_size as f64),
                suggestion: Some(
                    "Consider spatial partitioning or use a multi-file GeoParquet dataset"
                        .to_string(),
                ),
                fixable: false,
            }];
        }

        // GeoParquet < 1MB: overhead may not be worth it.
        if (path.ends_with(".parquet") || path.ends_with(".geoparquet")) && file_size < one_mb {
            return vec![Finding {
                rule_id: self.id().to_string(),
                severity: Severity::Info,
                message: "GeoParquet file is very small. Parquet's columnar overhead may not provide benefits at this size".to_string(),
                location: None,
                geometry: None,
                metric: Some(file_size as f64),
                suggestion: Some(
                    "GeoJSON may be simpler for datasets this small".to_string(),
                ),
                fixable: false,
            }];
        }

        vec![]
    }

    fn score_weight(&self) -> f64 {
        0.5
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(FileSize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[test]
    fn rule_metadata() {
        let rule = FileSize;
        assert_eq!(rule.id(), "cloud/file-size");
        assert_eq!(rule.domain(), Domain::Cloud);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }

    #[test]
    fn no_finding_for_normal_file() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "examples/datasets/simple_points.geojson",
        };
        let rule = FileSize;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn no_finding_for_missing_file() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "/nonexistent/file.shp",
        };
        let rule = FileSize;
        assert!(rule.check(&ctx).is_empty());
    }
}
