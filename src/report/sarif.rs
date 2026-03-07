/// SARIF output renderer for CI/CD and code scanning integrations.
use crate::core::error::{Result, TissotError};
use crate::core::report::ReportData;

/// Serialize a check report to SARIF v2.1.0.
pub fn check_sarif(report: &ReportData) -> Result<String> {
    let mut rules = Vec::new();
    let mut results = Vec::new();
    let mut seen_rule_ids = std::collections::HashSet::new();

    for finding in &report.findings {
        if seen_rule_ids.insert(finding.rule_id.clone()) {
            rules.push(serde_json::json!({
                "id": finding.rule_id,
                "name": finding.rule_id,
                "shortDescription": { "text": finding.rule_id },
            }));
        }

        results.push(serde_json::json!({
            "ruleId": finding.rule_id,
            "level": sarif_level(finding.severity),
            "message": { "text": finding.message },
            "locations": [
                {
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": report.file_path,
                        }
                    }
                }
            ]
        }));
    }

    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "tissot",
                        "informationUri": "https://github.com/chrislyonsKY/tissot",
                        "rules": rules
                    }
                },
                "results": results
            }
        ]
    });

    serde_json::to_string_pretty(&sarif)
        .map_err(|e| TissotError::Internal(format!("SARIF serialization failed: {e}")))
}

fn sarif_level(severity: crate::core::rule::Severity) -> &'static str {
    match severity {
        crate::core::rule::Severity::Error => "error",
        crate::core::rule::Severity::Warning => "warning",
        crate::core::rule::Severity::Info => "note",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sarif_contains_required_root_fields() {
        let report = ReportData::from_findings("test.geojson".into(), vec![]);
        let output = check_sarif(&report).unwrap();
        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("\"runs\""));
    }
}
