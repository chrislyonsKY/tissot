/// GeoJSON reader — pure Rust implementation via `geojson` + `geo` crates.
use crate::core::error::{Result, TissotError};
use crate::core::rule::{Feature, Layer};
use geo::Geometry;
use std::collections::HashMap;
use std::path::Path;

/// Read a GeoJSON file and return layers.
pub fn read(path: &Path) -> Result<Vec<Layer>> {
    let content = std::fs::read_to_string(path)?;
    read_str(&content, path.to_string_lossy().as_ref())
}

/// Parse GeoJSON from a string.
pub fn read_str(content: &str, source_name: &str) -> Result<Vec<Layer>> {
    let geojson: geojson::GeoJson = content.parse().map_err(|e: geojson::Error| {
        TissotError::GeoJson(format!("Failed to parse GeoJSON: {e}"))
    })?;

    let features = match geojson {
        geojson::GeoJson::FeatureCollection(fc) => convert_feature_collection(fc)?,
        geojson::GeoJson::Feature(f) => vec![convert_feature(f)?],
        geojson::GeoJson::Geometry(g) => {
            vec![Feature {
                id: None,
                geometry: Some(convert_geometry(&g.value)?),
                properties: HashMap::new(),
            }]
        }
    };

    let bounds = compute_bounds(&features);

    Ok(vec![Layer {
        name: source_name.to_string(),
        crs: Some("EPSG:4326".to_string()), // GeoJSON spec mandates WGS 84
        features,
        bounds,
    }])
}

/// Convert a GeoJSON FeatureCollection to our Feature type.
fn convert_feature_collection(fc: geojson::FeatureCollection) -> Result<Vec<Feature>> {
    fc.features.into_iter().map(convert_feature).collect()
}

/// Convert a single GeoJSON Feature to our Feature type.
fn convert_feature(f: geojson::Feature) -> Result<Feature> {
    let id = f.id.as_ref().map(|id| match id {
        geojson::feature::Id::String(s) => s.clone(),
        geojson::feature::Id::Number(n) => n.to_string(),
    });

    let geometry = f
        .geometry
        .as_ref()
        .map(|g| convert_geometry(&g.value))
        .transpose()?;

    let properties = f
        .properties
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(k, v)| if v.is_null() { None } else { Some((k, v)) })
        .collect();

    Ok(Feature {
        id,
        geometry,
        properties,
    })
}

/// Convert a geojson::Geometry to geo::Geometry.
fn convert_geometry(value: &geojson::Value) -> Result<Geometry> {
    let geojson_geom = geojson::Geometry::new(value.clone());
    geo::Geometry::try_from(geojson_geom)
        .map_err(|e| TissotError::GeoJson(format!("Failed to convert geometry: {e}")))
}

/// Compute bounding box from features.
fn compute_bounds(features: &[Feature]) -> Option<[f64; 4]> {
    use geo::BoundingRect;

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    for f in features {
        if let Some(ref geom) = f.geometry {
            if let Some(rect) = geom.bounding_rect() {
                found = true;
                min_x = min_x.min(rect.min().x);
                min_y = min_y.min(rect.min().y);
                max_x = max_x.max(rect.max().x);
                max_y = max_y.max(rect.max().y);
            }
        }
    }

    if found {
        Some([min_x, min_y, max_x, max_y])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_feature_collection() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Point",
                        "coordinates": [-84.5, 38.0]
                    },
                    "properties": {
                        "name": "Test Point"
                    }
                }
            ]
        }"#;

        let layers = read_str(geojson, "test.geojson").unwrap();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].features.len(), 1);
        assert_eq!(layers[0].crs, Some("EPSG:4326".to_string()));
        assert!(layers[0].features[0].geometry.is_some());
    }

    #[test]
    fn parse_empty_feature_collection() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": []
        }"#;

        let layers = read_str(geojson, "empty.geojson").unwrap();
        assert_eq!(layers[0].features.len(), 0);
        assert!(layers[0].bounds.is_none());
    }

    #[test]
    fn parse_feature_with_null_geometry() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"id": 1}
                }
            ]
        }"#;

        let layers = read_str(geojson, "null_geom.geojson").unwrap();
        assert!(layers[0].features[0].geometry.is_none());
    }

    #[test]
    fn bounds_computation() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [-80.0, 35.0]},
                    "properties": {}
                },
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [-90.0, 40.0]},
                    "properties": {}
                }
            ]
        }"#;

        let layers = read_str(geojson, "bounds.geojson").unwrap();
        let bounds = layers[0].bounds.unwrap();
        assert!((bounds[0] - (-90.0)).abs() < f64::EPSILON);
        assert!((bounds[1] - 35.0).abs() < f64::EPSILON);
        assert!((bounds[2] - (-80.0)).abs() < f64::EPSILON);
        assert!((bounds[3] - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_geojson_returns_error() {
        let result = read_str("not valid json at all", "bad.geojson");
        assert!(result.is_err());
    }
}
