use super::rule::{Domain, Finding, Severity};
/// Report data aggregation — collects findings into structured output.
use serde::Serialize;

/// Aggregated report data for visual/terminal/JSON output.
#[derive(Debug, Serialize)]
pub struct ReportData {
    /// Path to the analyzed file.
    pub file_path: String,
    /// All findings from all rules.
    pub findings: Vec<Finding>,
    /// Summary statistics.
    pub summary: ReportSummary,
}

/// Summary statistics for a diagnostic run.
#[derive(Debug, Serialize)]
pub struct ReportSummary {
    /// Total number of findings.
    pub total: usize,
    /// Count by severity.
    pub errors: usize,
    /// Count of warnings.
    pub warnings: usize,
    /// Count of info.
    pub info: usize,
    /// Count by domain.
    pub by_domain: std::collections::HashMap<String, usize>,
}

impl ReportData {
    /// Build a report from a list of findings.
    pub fn from_findings(file_path: String, findings: Vec<Finding>) -> Self {
        let mut by_domain: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut errors = 0usize;
        let mut warnings = 0usize;
        let mut info = 0usize;

        for f in &findings {
            match f.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => info += 1,
            }
            // Extract domain from rule_id (e.g., "data_quality/null-geometry" → "data_quality")
            if let Some(domain) = f.rule_id.split('/').next() {
                *by_domain.entry(domain.to_string()).or_insert(0) += 1;
            }
        }

        let total = findings.len();

        Self {
            file_path,
            findings,
            summary: ReportSummary {
                total,
                errors,
                warnings,
                info,
                by_domain,
            },
        }
    }

    /// Filter findings by domain.
    pub fn filter_domain(&self, domain: Domain) -> Vec<&Finding> {
        let prefix = domain.to_string();
        self.findings
            .iter()
            .filter(|f| f.rule_id.starts_with(&prefix))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rule::Finding;

    #[test]
    fn report_from_findings() {
        let findings = vec![
            Finding {
                rule_id: "data_quality/null-geometry".into(),
                severity: Severity::Error,
                message: "Null geometry found".into(),
                location: None,
                geometry: None,
                metric: None,
                suggestion: None,
                fixable: false,
            },
            Finding {
                rule_id: "projection/missing-crs".into(),
                severity: Severity::Warning,
                message: "No CRS defined".into(),
                location: None,
                geometry: None,
                metric: None,
                suggestion: None,
                fixable: false,
            },
        ];

        let report = ReportData::from_findings("test.geojson".into(), findings);
        assert_eq!(report.summary.total, 2);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.info, 0);
    }
}
