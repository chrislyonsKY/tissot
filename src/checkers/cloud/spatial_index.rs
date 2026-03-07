//! Rule: Check for spatial index presence in cloud-optimized formats.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity};

/// Checks whether the file format includes a spatial index for efficient partial reads.
pub struct SpatialIndex;

impl Default for SpatialIndex {
    fn default() -> Self {
        Self
    }
}

impl Rule for SpatialIndex {
    fn id(&self) -> &str {
        "cloud/spatial-index"
    }

    fn name(&self) -> &str {
        "Spatial Index"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "index", "performance"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let path = ctx.file_path.to_lowercase();

        // Only applies to formats that support spatial indexes.
        if path.ends_with(".fgb") {
            // FlatGeobuf: would need to parse the header to check for the
            // packed Hilbert R-tree. For now, flag as needing verification.
            todo!("Parse FlatGeobuf header to check for spatial index presence");
        }

        // GeoParquet: check for bbox column / spatial metadata — requires
        // parquet footer parsing.
        if path.ends_with(".parquet") || path.ends_with(".geoparquet") {
            todo!("Parse GeoParquet footer for spatial metadata and bbox column");
        }

        vec![]
    }

    fn score_weight(&self) -> f64 {
        0.8
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(SpatialIndex),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[test]
    fn rule_metadata() {
        let rule = SpatialIndex;
        assert_eq!(rule.id(), "cloud/spatial-index");
        assert_eq!(rule.domain(), Domain::Cloud);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }

    #[test]
    fn skips_non_indexed_formats() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "data.geojson",
        };
        let rule = SpatialIndex;
        assert!(rule.check(&ctx).is_empty());
    }
}
