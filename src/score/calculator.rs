use super::categories::{Category, CategoryScore};
/// Score computation — 0-100 quality rating from diagnostic findings.
use crate::core::config::Config;
use crate::core::rule::{Finding, Severity};
use serde::Serialize;

/// Complete score report.
#[derive(Debug, Serialize)]
pub struct ScoreReport {
    /// Overall score (0-100).
    pub overall: u32,
    /// Letter grade.
    pub grade: String,
    /// Scores per category.
    pub categories: Vec<CategoryScore>,
    /// Total number of findings.
    pub finding_count: usize,
}

/// Compute the quality score from findings.
pub fn compute_score(findings: &[Finding], config: &Config) -> ScoreReport {
    let categories = vec![
        compute_category(
            Category::Projection,
            findings,
            config.score.projection_weight,
        ),
        compute_category(
            Category::DataIntegrity,
            findings,
            config.score.data_integrity_weight,
        ),
        compute_category(
            Category::Accessibility,
            findings,
            config.score.accessibility_weight,
        ),
        compute_category(
            Category::Classification,
            findings,
            config.score.classification_weight,
        ),
    ];

    let overall = categories
        .iter()
        .map(|c| c.score as f64 * c.weight)
        .sum::<f64>()
        .round() as u32;

    let grade = letter_grade(overall);

    ScoreReport {
        overall,
        grade,
        categories,
        finding_count: findings.len(),
    }
}

/// Compute score for a single category.
fn compute_category(category: Category, findings: &[Finding], weight: f64) -> CategoryScore {
    let prefix = category.rule_prefix();
    let relevant: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.rule_id.starts_with(prefix))
        .collect();

    let mut score: i32 = 100;

    // Count by severity.
    let mut error_deductions = 0i32;
    let mut warning_deductions = 0i32;
    let mut info_deductions = 0i32;

    for f in &relevant {
        match f.severity {
            Severity::Error => error_deductions += 15,
            Severity::Warning => warning_deductions += 5,
            Severity::Info => info_deductions += 1,
        }
    }

    // Cap deductions per severity level.
    score -= error_deductions.min(60);
    score -= warning_deductions.min(30);
    score -= info_deductions.min(10);

    let score = score.max(0) as u32;

    CategoryScore {
        category,
        score,
        weight,
        finding_count: relevant.len(),
        grade: letter_grade(score),
    }
}

/// Convert numeric score to letter grade.
fn letter_grade(score: u32) -> String {
    match score {
        90..=100 => "A".to_string(),
        75..=89 => "B".to_string(),
        60..=74 => "C".to_string(),
        40..=59 => "D".to_string(),
        _ => "F".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rule::Finding;

    fn make_finding(rule_id: &str, severity: Severity) -> Finding {
        Finding {
            rule_id: rule_id.into(),
            severity,
            message: "test".into(),
            location: None,
            geometry: None,
            metric: None,
            suggestion: None,
            fixable: false,
        }
    }

    #[test]
    fn perfect_score_no_findings() {
        let config = Config::default();
        let report = compute_score(&[], &config);
        assert_eq!(report.overall, 100);
        assert_eq!(report.grade, "A");
    }

    #[test]
    fn score_decreases_with_errors() {
        let config = Config::default();
        let findings = vec![
            make_finding("projection/high-distortion", Severity::Error),
            make_finding("data_quality/null-geometry", Severity::Error),
        ];
        let report = compute_score(&findings, &config);
        assert!(report.overall < 100);
    }

    #[test]
    fn grade_thresholds() {
        assert_eq!(letter_grade(95), "A");
        assert_eq!(letter_grade(80), "B");
        assert_eq!(letter_grade(65), "C");
        assert_eq!(letter_grade(45), "D");
        assert_eq!(letter_grade(20), "F");
    }

    #[test]
    fn deductions_are_capped() {
        let config = Config::default();
        // 10 errors in one category = 150 deduction points, but capped at 60.
        let findings: Vec<Finding> = (0..10)
            .map(|_| make_finding("data_quality/null-geometry", Severity::Error))
            .collect();
        let report = compute_score(&findings, &config);
        // Data integrity category should be 100 - 60 = 40, not negative.
        let di_cat = report
            .categories
            .iter()
            .find(|c| c.category == Category::DataIntegrity)
            .unwrap();
        assert_eq!(di_cat.score, 40);
    }
}
