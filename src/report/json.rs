use crate::core::error::Result;
/// JSON output renderer — machine-readable reports.
use crate::core::report::ReportData;
use crate::score::ScoreReport;
use crate::xray::XrayReport;

/// Serialize X-Ray report to JSON string.
pub fn xray_json(report: &XrayReport) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(|e| {
        crate::core::error::TissotError::Internal(format!("JSON serialization failed: {e}"))
    })
}

/// Serialize check report to JSON string.
pub fn check_json(report: &ReportData) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(|e| {
        crate::core::error::TissotError::Internal(format!("JSON serialization failed: {e}"))
    })
}

/// Serialize score report to JSON string.
pub fn score_json(report: &ScoreReport) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(|e| {
        crate::core::error::TissotError::Internal(format!("JSON serialization failed: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::report::ReportData;

    #[test]
    fn check_json_roundtrip() {
        let report = ReportData::from_findings("test.geojson".into(), vec![]);
        let json = check_json(&report).unwrap();
        assert!(json.contains("test.geojson"));
    }

    #[test]
    fn score_json_roundtrip() {
        use crate::score::categories::{Category, CategoryScore};

        let report = ScoreReport {
            overall: 95,
            grade: "A".into(),
            categories: vec![CategoryScore {
                category: Category::Projection,
                score: 100,
                weight: 0.25,
                finding_count: 0,
                grade: "A".into(),
            }],
            finding_count: 0,
        };
        let json = score_json(&report).unwrap();
        assert!(json.contains("95"));
    }
}
