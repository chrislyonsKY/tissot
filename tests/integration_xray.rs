//! Integration tests for the X-Ray projection analysis engine.

use std::path::PathBuf;
use tissot::core::config::Config;
use tissot::io;
use tissot::xray;

/// Helper: resolve path to an example dataset file.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("datasets")
        .join(name)
}

// ── X-Ray analysis on us_states_mercator ───────────────────────────────────

#[test]
fn xray_us_states_mercator_produces_report() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // Report should reference the correct file
    assert!(
        report.file_path.contains("us_states_mercator"),
        "report file_path should reference the input file"
    );

    // Source CRS should be set
    assert!(
        !report.source_crs.is_empty(),
        "source CRS should not be empty"
    );
}

// ── Distortion samples generated ───────────────────────────────────────────

#[test]
fn xray_generates_distortion_samples() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    assert!(
        !report.samples.is_empty(),
        "should generate distortion samples"
    );
    assert_eq!(
        report.summary.sample_count,
        report.samples.len(),
        "summary sample_count should match actual samples"
    );
}

// ── Distortion sample values are reasonable ────────────────────────────────

#[test]
fn xray_sample_values_are_reasonable() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    for sample in &report.samples {
        // Latitude and longitude should be finite
        assert!(sample.lat.is_finite(), "sample lat should be finite");
        assert!(sample.lon.is_finite(), "sample lon should be finite");

        // Area scale factor should be positive
        assert!(
            sample.area_scale_factor > 0.0,
            "area_scale_factor should be positive, got {}",
            sample.area_scale_factor
        );

        // Angular distortion should be non-negative
        assert!(
            sample.angular_distortion_deg >= 0.0,
            "angular distortion should be >= 0, got {}",
            sample.angular_distortion_deg
        );

        // Semi-axes should be positive
        assert!(
            sample.semimajor > 0.0,
            "semimajor should be positive, got {}",
            sample.semimajor
        );
        assert!(
            sample.semiminor > 0.0,
            "semiminor should be positive, got {}",
            sample.semiminor
        );
    }
}

// ── Summary statistics are consistent ──────────────────────────────────────

#[test]
fn xray_summary_statistics_consistent() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();
    let summary = &report.summary;

    // Max should be >= mean
    assert!(
        summary.max_area_distortion_pct >= summary.mean_area_distortion_pct,
        "max ({}) should be >= mean ({})",
        summary.max_area_distortion_pct,
        summary.mean_area_distortion_pct
    );

    // Max angular should be >= mean angular
    assert!(
        summary.max_angular_distortion_deg >= summary.mean_angular_distortion_deg,
        "max angular ({}) should be >= mean angular ({})",
        summary.max_angular_distortion_deg,
        summary.mean_angular_distortion_deg
    );
}

// ── Heatmap grid is generated ──────────────────────────────────────────────

#[test]
fn xray_generates_heatmap() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // Heatmap should have values
    assert!(
        !report.heatmap.values.is_empty(),
        "heatmap should have values"
    );

    // Grid dimensions should be positive
    assert!(report.heatmap.cols > 0, "heatmap should have columns");
    assert!(report.heatmap.rows > 0, "heatmap should have rows");

    // Values count should equal cols * rows
    assert_eq!(
        report.heatmap.values.len(),
        report.heatmap.cols * report.heatmap.rows,
        "heatmap values count should equal cols * rows"
    );
}

// ── Ellipses generated ─────────────────────────────────────────────────────

#[test]
fn xray_generates_ellipses() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // Should generate ellipses matching sample count
    assert_eq!(
        report.ellipses.len(),
        report.samples.len(),
        "should have one ellipse per sample"
    );

    for ellipse in &report.ellipses {
        // Each ellipse should have coordinates (polygon vertices)
        assert!(
            !ellipse.coordinates.is_empty(),
            "ellipse should have coordinate vertices"
        );
        // Center coordinates should be finite
        assert!(ellipse.lon.is_finite());
        assert!(ellipse.lat.is_finite());
        // Semi-axes should be positive
        assert!(ellipse.semimajor > 0.0);
        assert!(ellipse.semiminor > 0.0);
    }
}

// ── Recommendations generated ──────────────────────────────────────────────

#[test]
fn xray_generates_recommendations() {
    let path = fixture("us_states_mercator.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // Should have recommendations (up to top_recommendations)
    assert!(
        !report.recommendations.is_empty(),
        "should generate CRS recommendations"
    );

    assert!(
        report.recommendations.len() <= config.xray.top_recommendations,
        "should not exceed top_recommendations ({}), got {}",
        config.xray.top_recommendations,
        report.recommendations.len()
    );

    for rec in &report.recommendations {
        // Each recommendation should have a CRS identifier
        assert!(
            !rec.crs.is_empty(),
            "recommendation should have a CRS identifier"
        );
        // Should have a human-readable name
        assert!(
            !rec.name.is_empty(),
            "recommendation should have a name"
        );
        // Fitness score should be in [0, 1]
        assert!(
            rec.fitness >= 0.0 && rec.fitness <= 1.0,
            "fitness should be in [0,1], got {}",
            rec.fitness
        );
    }
}

// ── X-Ray on simple points ─────────────────────────────────────────────────

#[test]
fn xray_simple_points() {
    let path = fixture("simple_points.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // Should succeed even with just 2 points
    assert_eq!(report.source_crs, "EPSG:4326");
    // Samples should be generated
    assert!(
        !report.samples.is_empty(),
        "should generate samples even for 2-point dataset"
    );
}

// ── X-Ray on world cities (global extent) ──────────────────────────────────

#[test]
fn xray_world_cities_global_extent() {
    let path = fixture("world_cities.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    // With global extent and WGS 84, there should be notable distortion
    // if the checker evaluates Mercator-like properties
    assert!(
        report.summary.sample_count > 0,
        "should have samples from 15 cities"
    );

    // Heatmap should cover global extent
    assert!(!report.heatmap.values.is_empty());
}

// ── X-Ray report is serializable ───────────────────────────────────────────

#[test]
fn xray_report_serializes_to_json() {
    let path = fixture("simple_points.geojson");
    let layers = io::read_file(&path).unwrap();
    let config = Config::default();

    let report = xray::analyze(&layers[0], &config, path.to_str().unwrap()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty(), "serialized JSON should not be empty");

    // Should be valid JSON that can be parsed back
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object(), "serialized report should be a JSON object");
    assert!(
        parsed.get("source_crs").is_some(),
        "JSON should contain source_crs field"
    );
    assert!(
        parsed.get("samples").is_some(),
        "JSON should contain samples field"
    );
}
