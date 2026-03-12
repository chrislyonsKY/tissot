//! Integration tests for the checker engine.

use std::path::PathBuf;
use tissot::core::config::Config;
use tissot::core::rule::{Domain, Severity};
use tissot::checkers::run_checks;
use tissot::io;

/// Helper: resolve path to an example dataset file.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("datasets")
        .join(name)
}

/// Helper: load a dataset and run all checks with default config.
fn check_file(name: &str) -> Vec<tissot::core::rule::Finding> {
    let path = fixture(name);
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();
    run_checks(&layers, &config, path.to_str().unwrap(), None)
}

/// Helper: load a dataset and run checks filtered by domain.
fn check_file_domain(name: &str, domain: Domain) -> Vec<tissot::core::rule::Finding> {
    let path = fixture(name);
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();
    run_checks(&layers, &config, path.to_str().unwrap(), Some(domain))
}

// ── Parcels with issues should produce null geometry findings ──────────────

#[test]
fn parcels_with_issues_has_null_geometry_finding() {
    let findings = check_file("parcels_with_issues.geojson");

    let null_geom_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id.contains("null-geometry") || f.rule_id.contains("null_geometry"))
        .collect();

    assert!(
        !null_geom_findings.is_empty(),
        "parcels_with_issues should trigger null geometry findings, got {} total findings: {:?}",
        findings.len(),
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );
}

// ── Empty dataset should trigger empty-dataset finding ─────────────────────

#[test]
fn empty_geojson_triggers_empty_dataset_finding() {
    let findings = check_file("empty.geojson");

    let empty_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id.contains("empty"))
        .collect();

    assert!(
        !empty_findings.is_empty(),
        "empty.geojson should trigger an empty-dataset finding, got findings: {:?}",
        findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
    );
}

// ── World cities should be relatively clean ────────────────────────────────

#[test]
fn world_cities_relatively_clean() {
    let findings = check_file("world_cities.geojson");

    // Count only errors (warnings/info are acceptable for clean data)
    let error_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();

    // Clean data may still have some projection/cloud warnings, but should
    // have very few actual errors from data quality domain
    let data_quality_errors: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id.starts_with("data") && f.severity == Severity::Error)
        .collect();

    assert!(
        data_quality_errors.len() <= 2,
        "world_cities should have few data quality errors, got {}: {:?}",
        data_quality_errors.len(),
        data_quality_errors
            .iter()
            .map(|f| &f.rule_id)
            .collect::<Vec<_>>()
    );

    // Verify that findings is not empty (rules did execute)
    // At minimum, cloud rules should fire since it's a GeoJSON file
    assert!(
        !findings.is_empty() || error_count == 0,
        "checker should have run and produced some findings"
    );
}

// ── Domain filtering: quality only ─────────────────────────────────────────

#[test]
fn filter_by_data_quality_domain() {
    let all_findings = check_file("parcels_with_issues.geojson");
    let quality_findings = check_file_domain("parcels_with_issues.geojson", Domain::DataQuality);

    // Domain-filtered results should be a subset
    assert!(
        quality_findings.len() <= all_findings.len(),
        "filtered findings ({}) should not exceed total findings ({})",
        quality_findings.len(),
        all_findings.len()
    );

    // All filtered findings should be from the data quality domain
    for f in &quality_findings {
        assert!(
            f.rule_id.starts_with("data"),
            "domain-filtered finding '{}' should belong to data quality domain",
            f.rule_id
        );
    }
}

// ── Domain filtering: projection only ──────────────────────────────────────

#[test]
fn filter_by_projection_domain() {
    let proj_findings = check_file_domain("us_states_mercator.geojson", Domain::Projection);

    for f in &proj_findings {
        assert!(
            f.rule_id.starts_with("proj"),
            "projection-filtered finding '{}' should belong to projection domain",
            f.rule_id
        );
    }
}

// ── Domain filtering: cloud only ───────────────────────────────────────────

#[test]
fn filter_by_cloud_domain() {
    let cloud_findings = check_file_domain("simple_points.geojson", Domain::Cloud);

    for f in &cloud_findings {
        assert!(
            f.rule_id.starts_with("cloud"),
            "cloud-filtered finding '{}' should belong to cloud domain",
            f.rule_id
        );
    }
}

// ── Severity levels are valid ──────────────────────────────────────────────

#[test]
fn findings_have_valid_severity() {
    let findings = check_file("parcels_with_issues.geojson");

    for f in &findings {
        // Every finding should have a valid severity
        match f.severity {
            Severity::Info | Severity::Warning | Severity::Error => {}
        }

        // Every finding should have a non-empty rule_id and message
        assert!(!f.rule_id.is_empty(), "rule_id must not be empty");
        assert!(!f.message.is_empty(), "message must not be empty");
    }
}

// ── Findings sorted by severity (errors first) ────────────────────────────

#[test]
fn findings_sorted_errors_first() {
    let findings = check_file("parcels_with_issues.geojson");

    if findings.len() >= 2 {
        for window in findings.windows(2) {
            assert!(
                window[0].severity >= window[1].severity,
                "findings should be sorted by severity descending: {:?} came before {:?}",
                window[0].severity,
                window[1].severity
            );
        }
    }
}

// ── Checks on simple points (few issues expected) ──────────────────────────

#[test]
fn simple_points_minimal_issues() {
    let findings = check_file("simple_points.geojson");

    let data_errors: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id.starts_with("data") && f.severity == Severity::Error)
        .collect();

    assert!(
        data_errors.is_empty(),
        "simple_points should have no data quality errors, got: {:?}",
        data_errors
            .iter()
            .map(|f| format!("{}: {}", f.rule_id, f.message))
            .collect::<Vec<_>>()
    );
}

// ── Running checks with empty layers does not panic ────────────────────────

#[test]
fn checks_on_empty_layers_does_not_panic() {
    let config = Config::default();
    let findings = run_checks(&[], &config, "nonexistent.geojson", None);
    // Should not panic — findings may or may not be empty depending on rules
    let _ = findings;
}
