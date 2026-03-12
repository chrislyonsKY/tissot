//! Integration tests for the IO layer — reading all supported formats.

use std::path::{Path, PathBuf};
use tissot::io;

/// Helper: resolve path to an example dataset file.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("datasets")
        .join(name)
}

// ── Reading simple_points.geojson ──────────────────────────────────────────

#[test]
fn read_simple_points_geojson() {
    let layers = io::read_file(&fixture("simple_points.geojson")).unwrap();
    assert_eq!(layers.len(), 1, "should produce exactly one layer");

    let layer = &layers[0];
    assert_eq!(layer.features.len(), 2, "simple_points has 2 features");
    assert_eq!(layer.crs, Some("EPSG:4326".to_string()));

    // Both features should have Point geometry
    for feat in &layer.features {
        assert!(
            feat.geometry.is_some(),
            "every feature should have geometry"
        );
    }

    // Verify bounds are computed
    let bounds = layer.bounds.unwrap();
    assert!(bounds[0] <= -84.49, "min_x should be <= -84.49");
    assert!(bounds[2] >= -84.49, "max_x should be >= -84.49");
}

// ── Reading empty.geojson ──────────────────────────────────────────────────

#[test]
fn read_empty_geojson() {
    let layers = io::read_file(&fixture("empty.geojson")).unwrap();
    assert_eq!(layers.len(), 1, "should still produce one layer");

    let layer = &layers[0];
    assert_eq!(layer.features.len(), 0, "empty dataset has zero features");
    assert!(layer.bounds.is_none(), "no features means no bounds");
}

// ── Reading world_cities.geojson ───────────────────────────────────────────

#[test]
fn read_world_cities_geojson() {
    let layers = io::read_file(&fixture("world_cities.geojson")).unwrap();
    let layer = &layers[0];

    assert_eq!(layer.features.len(), 15, "world_cities has 15 features");
    assert_eq!(layer.crs, Some("EPSG:4326".to_string()));

    // All features should have Point geometry
    for feat in &layer.features {
        assert!(feat.geometry.is_some());
        match feat.geometry.as_ref().unwrap() {
            geo::Geometry::Point(_) => {}
            other => panic!("expected Point, got {:?}", other),
        }
    }

    // Verify properties exist
    let first = &layer.features[0];
    assert!(
        first.properties.contains_key("name"),
        "features should have a name property"
    );
    assert!(
        first.properties.contains_key("population"),
        "features should have a population property"
    );
}

// ── Reading kentucky_roads.geojson ─────────────────────────────────────────

#[test]
fn read_kentucky_roads_geojson_line_geometries() {
    let layers = io::read_file(&fixture("kentucky_roads.geojson")).unwrap();
    let layer = &layers[0];

    assert_eq!(layer.features.len(), 5, "kentucky_roads has 5 features");

    // All features should have LineString geometry
    for feat in &layer.features {
        assert!(feat.geometry.is_some());
        match feat.geometry.as_ref().unwrap() {
            geo::Geometry::LineString(_) => {}
            other => panic!("expected LineString, got {:?}", other),
        }
    }

    // Bounds should cover roughly western-to-eastern Kentucky
    let bounds = layer.bounds.unwrap();
    assert!(bounds[0] < -88.0, "min_x should extend into western KY");
    assert!(bounds[2] > -83.0, "max_x should extend into eastern KY");
}

// ── Reading parcels_with_issues.geojson ────────────────────────────────────

#[test]
fn read_parcels_with_issues_mixed_content() {
    let layers = io::read_file(&fixture("parcels_with_issues.geojson")).unwrap();
    let layer = &layers[0];

    assert_eq!(
        layer.features.len(),
        10,
        "parcels_with_issues has 10 features"
    );

    // Should contain at least one feature with null geometry (P004)
    let null_geom_count = layer
        .features
        .iter()
        .filter(|f| f.geometry.is_none())
        .count();
    assert!(
        null_geom_count >= 1,
        "should have at least one null geometry feature, found {null_geom_count}"
    );

    // Most features should be Polygon
    let polygon_count = layer
        .features
        .iter()
        .filter(|f| matches!(f.geometry.as_ref(), Some(geo::Geometry::Polygon(_))))
        .count();
    assert!(polygon_count >= 8, "most features should be polygons");
}

// ── Reading us_states_mercator.geojson ─────────────────────────────────────

#[test]
fn read_us_states_mercator_geojson() {
    let layers = io::read_file(&fixture("us_states_mercator.geojson")).unwrap();
    let layer = &layers[0];

    assert_eq!(layer.features.len(), 5, "us_states_mercator has 5 features");

    // CRS is always EPSG:4326 per GeoJSON spec enforcement in the reader
    assert_eq!(layer.crs, Some("EPSG:4326".to_string()));

    // Verify features have polygon geometry
    for feat in &layer.features {
        assert!(feat.geometry.is_some());
        match feat.geometry.as_ref().unwrap() {
            geo::Geometry::Polygon(_) => {}
            other => panic!("expected Polygon, got {:?}", other),
        }
    }

    // Coordinates are in Web Mercator (large values), bounds should reflect that
    let bounds = layer.bounds.unwrap();
    assert!(
        bounds[0].abs() > 1_000_000.0,
        "Web Mercator coordinates should be large numbers"
    );
}

// ── Error handling: nonexistent file ───────────────────────────────────────

#[test]
fn read_nonexistent_file_returns_error() {
    let result = io::read_file(Path::new("/nonexistent/path/data.geojson"));
    assert!(result.is_err(), "reading a nonexistent file should fail");
}

// ── Error handling: unsupported format ─────────────────────────────────────

#[test]
fn read_unsupported_format_returns_error() {
    let result = io::read_file(Path::new("data.xlsx"));
    assert!(result.is_err(), "unsupported format should fail");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unsupported format") || err_msg.contains("Unknown file extension"),
        "error message should mention unsupported format, got: {err_msg}"
    );
}

// ── Format detection ───────────────────────────────────────────────────────

#[test]
fn detect_format_for_known_extensions() {
    assert_eq!(
        io::detect_format(Path::new("foo.geojson")).unwrap(),
        io::Format::GeoJson
    );
    assert_eq!(
        io::detect_format(Path::new("foo.json")).unwrap(),
        io::Format::GeoJson
    );
    assert_eq!(
        io::detect_format(Path::new("foo.shp")).unwrap(),
        io::Format::Shapefile
    );
    assert_eq!(
        io::detect_format(Path::new("foo.fgb")).unwrap(),
        io::Format::FlatGeobuf
    );
    assert_eq!(
        io::detect_format(Path::new("foo.gpkg")).unwrap(),
        io::Format::GeoPackage
    );
}

#[test]
fn detect_format_unknown_extension_errors() {
    assert!(io::detect_format(Path::new("data.csv")).is_err());
    assert!(io::detect_format(Path::new("data.txt")).is_err());
    assert!(io::detect_format(Path::new("noext")).is_err());
}
