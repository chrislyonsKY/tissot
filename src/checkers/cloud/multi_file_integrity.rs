//! Rule: Validate multi-file format integrity (sidecar files).

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity};

/// Checks that all required sidecar files are present for multi-file formats.
pub struct MultiFileIntegrity;

impl Default for MultiFileIntegrity {
    fn default() -> Self {
        Self
    }
}

impl Rule for MultiFileIntegrity {
    fn id(&self) -> &str {
        "cloud/multi-file-integrity"
    }

    fn name(&self) -> &str {
        "Multi-File Integrity"
    }

    fn domain(&self) -> Domain {
        Domain::Cloud
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn tags(&self) -> &[&str] {
        &["cloud", "integrity", "shapefile"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let path = ctx.file_path;
        if !path.to_lowercase().ends_with(".shp") {
            return vec![];
        }

        let base = path.trim_end_matches(".shp").trim_end_matches(".SHP");
        let mut findings = Vec::new();

        // Required companions.
        let required = [(".shx", "spatial index"), (".dbf", "attribute table")];
        for (ext, desc) in &required {
            let companion = format!("{base}{ext}");
            if !std::path::Path::new(&companion).exists() {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Shapefile is missing {ext} ({desc}) file. The .shp file cannot be read without it"
                    ),
                    location: None,
                    geometry: None,
                    metric: None,
                    suggestion: Some(format!(
                        "Ensure the {ext} file is alongside the .shp file, or convert to a single-file format like FlatGeobuf or GeoParquet"
                    )),
                    fixable: false,
                });
            }
        }

        // Optional but recommended.
        let recommended = [
            (".prj", "CRS/projection definition"),
            (".cpg", "character encoding"),
        ];
        for (ext, desc) in &recommended {
            let companion = format!("{base}{ext}");
            if !std::path::Path::new(&companion).exists() {
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "Shapefile is missing {ext} ({desc}) file"
                    ),
                    location: None,
                    geometry: None,
                    metric: None,
                    suggestion: Some(format!(
                        "Add the {ext} file for {desc}, or convert to GeoParquet/FlatGeobuf which embed all metadata in a single file"
                    )),
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
    RuleEntry {
        factory: || Box::new(MultiFileIntegrity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[test]
    fn skips_non_shapefile() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "data.geojson",
        };
        let rule = MultiFileIntegrity;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn flags_missing_companions() {
        let config = Config::default();
        let ctx = CheckContext {
            layers: &[],
            config: &config,
            file_path: "/tmp/nonexistent_tissot_test.shp",
        };
        let rule = MultiFileIntegrity;
        let findings = rule.check(&ctx);
        // Should flag .shx, .dbf (Error) and .prj, .cpg (Warning).
        assert!(findings.len() >= 2);
        assert!(findings.iter().any(|f| f.message.contains(".shx")));
    }

    #[test]
    fn rule_metadata() {
        let rule = MultiFileIntegrity;
        assert_eq!(rule.id(), "cloud/multi-file-integrity");
        assert_eq!(rule.domain(), Domain::Cloud);
    }
}
