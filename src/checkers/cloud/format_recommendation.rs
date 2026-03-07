//! Rule: Recommend cloud-optimized formats when legacy formats are detected.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity};

/// Flags datasets using non-cloud-optimized formats and suggests alternatives.
pub struct FormatRecommendation;

impl Default for FormatRecommendation {
    fn default() -> Self {
        Self
    }
}

impl Rule for FormatRecommendation {
    fn id(&self) -> &str {
        "cloud/format-recommendation"
    }

    fn name(&self) -> &str {
        "Format Recommendation"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "format"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let path = ctx.file_path.to_lowercase();

        // Already cloud-optimized formats — no finding.
        if path.ends_with(".fgb")
            || path.ends_with(".parquet")
            || path.ends_with(".geoparquet")
            || path.ends_with(".pmtiles")
        {
            return vec![];
        }

        let (format_name, suggestion) = if path.ends_with(".shp") {
            ("Shapefile", "Convert to FlatGeobuf (streamable, spatially indexed) or GeoParquet (columnar, compressed). Shapefile has a 2GB limit and requires multiple sidecar files. See: https://guide.cloudnativegeo.org/")
        } else if path.ends_with(".gpkg") {
            ("GeoPackage", "Convert to FlatGeobuf or GeoParquet for cloud-native access. GeoPackage (SQLite) requires full download for any read. See: https://guide.cloudnativegeo.org/geopackage/")
        } else if path.ends_with(".geojson") || path.ends_with(".json") {
            let file_size = std::fs::metadata(ctx.file_path)
                .map(|m| m.len())
                .unwrap_or(0);
            let threshold = 10 * 1024 * 1024; // 10 MB
            if file_size < threshold {
                return vec![];
            }
            ("GeoJSON (large)", "Large GeoJSON files are slow to parse and not streamable. Convert to FlatGeobuf or GeoParquet. See: https://guide.cloudnativegeo.org/")
        } else {
            return vec![];
        };

        vec![Finding {
            rule_id: self.id().to_string(),
            severity: self.default_severity(),
            message: format!(
                "Dataset is in {format_name} format, which is not cloud-optimized"
            ),
            location: None,
            geometry: None,
            metric: None,
            suggestion: Some(suggestion.to_string()),
            fixable: false,
        }]
    }

    fn score_weight(&self) -> f64 {
        0.5
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(FormatRecommendation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[test]
    fn flags_shapefile() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "data/roads.shp",
        };
        let rule = FormatRecommendation;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("Shapefile"));
    }

    #[test]
    fn skips_flatgeobuf() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "data/roads.fgb",
        };
        let rule = FormatRecommendation;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn skips_small_geojson() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "examples/datasets/simple_points.geojson",
        };
        let rule = FormatRecommendation;
        // Small GeoJSON should not be flagged.
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn rule_metadata() {
        let rule = FormatRecommendation;
        assert_eq!(rule.id(), "cloud/format-recommendation");
        assert_eq!(rule.domain(), Domain::Cloud);
    }
}
