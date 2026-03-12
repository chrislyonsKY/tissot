//! Integration tests for the scoring engine.

use std::path::PathBuf;
use tissot::checkers::run_checks;
use tissot::core::config::Config;
use tissot::io;
use tissot::score::compute_score;

/// Helper: resolve path to an example dataset file.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("datasets")
        .join(name)
}

/// Helper: load a file, run checks, compute score.
fn score_file(name: &str) -> tissot::score::ScoreReport {
    let path = fixture(name);
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();
    let findings = run_checks(&layers, &config, path.to_str().unwrap(), None);
    compute_score(&findings, &config)
}

// ── Score is in 0-100 range ────────────────────────────────────────────────

#[test]
fn simple_points_score_in_valid_range() {
    let report = score_file("simple_points.geojson");
    assert!(
        report.overall <= 100,
        "score must be <= 100, got {}",
        report.overall
    );
    // Score type is u32, so it's always >= 0
}

#[test]
fn parcels_score_in_valid_range() {
    let report = score_file("parcels_with_issues.geojson");
    assert!(
        report.overall <= 100,
        "score must be <= 100, got {}",
        report.overall
    );
}

// ── Parcels with issues should score lower than clean data ─────────────────

#[test]
fn parcels_score_lower_than_simple_points() {
    let clean_report = score_file("simple_points.geojson");
    let issue_report = score_file("parcels_with_issues.geojson");

    assert!(
        issue_report.overall <= clean_report.overall,
        "parcels_with_issues ({}) should score <= simple_points ({})",
        issue_report.overall,
        clean_report.overall
    );
}

// ── Score categories exist ─────────────────────────────────────────────────

#[test]
fn score_report_has_all_categories() {
    let report = score_file("simple_points.geojson");

    assert_eq!(
        report.categories.len(),
        5,
        "should have 5 score categories"
    );

    let category_names: Vec<String> = report
        .categories
        .iter()
        .map(|c| c.category.to_string())
        .collect();

    assert!(
        category_names.contains(&"Projection".to_string()),
        "should include Projection category"
    );
    assert!(
        category_names.contains(&"Data Integrity".to_string()),
        "should include Data Integrity category"
    );
    assert!(
        category_names.contains(&"Accessibility".to_string()),
        "should include Accessibility category"
    );
    assert!(
        category_names.contains(&"Cloud Readiness".to_string()),
        "should include Cloud Readiness category"
    );
    assert!(
        category_names.contains(&"Classification".to_string()),
        "should include Classification category"
    );
}

// ── Category scores are individually valid ─────────────────────────────────

#[test]
fn category_scores_in_valid_range() {
    let report = score_file("parcels_with_issues.geojson");

    for cat in &report.categories {
        assert!(
            cat.score <= 100,
            "category '{}' score {} must be <= 100",
            cat.category,
            cat.score
        );
        assert!(
            cat.weight > 0.0 && cat.weight <= 1.0,
            "category '{}' weight {} must be in (0, 1]",
            cat.category,
            cat.weight
        );
    }
}

// ── Category weights sum to 1.0 ────────────────────────────────────────────

#[test]
fn category_weights_sum_to_one() {
    let report = score_file("simple_points.geojson");

    let weight_sum: f64 = report.categories.iter().map(|c| c.weight).sum();
    assert!(
        (weight_sum - 1.0).abs() < 0.01,
        "category weights should sum to ~1.0, got {weight_sum}"
    );
}

// ── Grade assignment ───────────────────────────────────────────────────────

#[test]
fn grade_is_valid_letter() {
    let report = score_file("simple_points.geojson");
    let valid_grades = ["A", "B", "C", "D", "F"];
    assert!(
        valid_grades.contains(&report.grade.as_str()),
        "grade should be A/B/C/D/F, got '{}'",
        report.grade
    );
}

#[test]
fn category_grades_are_valid_letters() {
    let report = score_file("parcels_with_issues.geojson");
    let valid_grades = ["A", "B", "C", "D", "F"];
    for cat in &report.categories {
        assert!(
            valid_grades.contains(&cat.grade.as_str()),
            "category '{}' grade should be A/B/C/D/F, got '{}'",
            cat.category,
            cat.grade
        );
    }
}

// ── Finding count matches ──────────────────────────────────────────────────

#[test]
fn finding_count_matches_checker_output() {
    let path = fixture("parcels_with_issues.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();
    let findings = run_checks(&layers, &config, path.to_str().unwrap(), None);
    let report = compute_score(&findings, &config);

    assert_eq!(
        report.finding_count,
        findings.len(),
        "score report finding_count should match actual findings"
    );
}

// ── Perfect score with no findings ─────────────────────────────────────────

#[test]
fn no_findings_yields_perfect_score() {
    let config = Config::default();
    let report = compute_score(&[], &config);
    assert_eq!(report.overall, 100, "no findings should yield score 100");
    assert_eq!(report.grade, "A", "score 100 should be grade A");
}

// ── Score across different datasets ────────────────────────────────────────

#[test]
fn world_cities_scores_well() {
    let report = score_file("world_cities.geojson");
    // Clean point data should score reasonably well
    assert!(
        report.overall >= 40,
        "world_cities should score >= 40, got {}",
        report.overall
    );
}

#[test]
fn empty_dataset_lower_score() {
    let report = score_file("empty.geojson");
    // Empty dataset triggers findings, so it shouldn't get a perfect score
    assert!(
        report.overall < 100,
        "empty dataset should not score 100, got {}",
        report.overall
    );
}
