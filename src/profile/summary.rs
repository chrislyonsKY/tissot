//! Profile summary — structural overview of a geospatial dataset.
//!
//! Generates per-layer statistics including feature count, geometry type,
//! CRS, extent, field count, and null geometry count.

use geo::HasDimensions;
use serde::{Deserialize, Serialize};

use crate::core::rule::Layer;

/// Complete profile summary for a geospatial dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    /// Path to the input file.
    pub file_path: String,
    /// Detected format name (e.g., "GeoJSON", "GeoPackage").
    pub format_name: String,
    /// File size in bytes.
    pub file_size_bytes: u64,
    /// Number of layers in the dataset.
    pub layer_count: usize,
    /// Per-layer profile information.
    pub layers: Vec<LayerProfile>,
}

/// Profile information for a single layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerProfile {
    /// Layer name.
    pub name: String,
    /// Number of features in the layer.
    pub feature_count: usize,
    /// Dominant geometry type (e.g., "Point", "Polygon", "Mixed").
    pub geometry_type: String,
    /// CRS identifier if available.
    pub crs: Option<String>,
    /// Bounding box [min_x, min_y, max_x, max_y].
    pub extent: Option<[f64; 4]>,
    /// Number of attribute fields.
    pub field_count: usize,
    /// Number of features with null or empty geometry.
    pub null_geometry_count: usize,
}

/// Detect the dominant geometry type from a layer's features.
fn detect_geometry_type(layer: &Layer) -> String {
    use std::collections::HashMap;
    let mut type_counts: HashMap<&str, usize> = HashMap::new();

    for feature in &layer.features {
        if let Some(ref geom) = feature.geometry {
            let name = match geom {
                geo::Geometry::Point(_) => "Point",
                geo::Geometry::Line(_) => "Line",
                geo::Geometry::LineString(_) => "LineString",
                geo::Geometry::Polygon(_) => "Polygon",
                geo::Geometry::MultiPoint(_) => "MultiPoint",
                geo::Geometry::MultiLineString(_) => "MultiLineString",
                geo::Geometry::MultiPolygon(_) => "MultiPolygon",
                geo::Geometry::GeometryCollection(_) => "GeometryCollection",
                geo::Geometry::Triangle(_) => "Triangle",
                geo::Geometry::Rect(_) => "Rect",
            };
            *type_counts.entry(name).or_insert(0) += 1;
        }
    }

    if type_counts.is_empty() {
        return "None".to_string();
    }
    if type_counts.len() == 1 {
        return type_counts.keys().next().unwrap().to_string();
    }

    // If mixed, report the most common type with "Mixed" prefix.
    let dominant = type_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(name, _)| *name)
        .unwrap_or("Unknown");

    format!("Mixed (primarily {dominant})")
}

/// Count features with null or empty geometry.
fn count_null_geometries(layer: &Layer) -> usize {
    layer
        .features
        .iter()
        .filter(|f| match &f.geometry {
            None => true,
            Some(geom) => geom.is_empty(),
        })
        .count()
}

/// Collect the set of unique attribute field names across all features.
fn collect_field_names(layer: &Layer) -> usize {
    let mut fields: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for feature in &layer.features {
        for key in feature.properties.keys() {
            fields.insert(key.as_str());
        }
    }
    fields.len()
}

/// Generate a profile summary for the given layers and input file path.
///
/// Reads file metadata from disk for the file size.
pub fn generate_profile(layers: &[Layer], input_path: &str, format_name: &str) -> ProfileSummary {
    let file_size_bytes = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);

    let layer_profiles: Vec<LayerProfile> = layers
        .iter()
        .map(|layer| LayerProfile {
            name: layer.name.clone(),
            feature_count: layer.features.len(),
            geometry_type: detect_geometry_type(layer),
            crs: layer.crs.clone(),
            extent: layer.bounds,
            field_count: collect_field_names(layer),
            null_geometry_count: count_null_geometries(layer),
        })
        .collect();

    ProfileSummary {
        file_path: input_path.to_string(),
        format_name: format_name.to_string(),
        file_size_bytes,
        layer_count: layers.len(),
        layers: layer_profiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rule::Feature;
    use std::collections::HashMap;

    #[test]
    fn profile_with_single_layer() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), serde_json::json!("test"));
        props.insert("value".to_string(), serde_json::json!(42));

        let layer = Layer {
            name: "test_layer".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: props.clone(),
                },
                Feature {
                    id: Some("2".into()),
                    geometry: None,
                    properties: props,
                },
            ],
            bounds: Some([-180.0, -90.0, 180.0, 90.0]),
        };

        let profile = generate_profile(&[layer], "/nonexistent/test.geojson", "GeoJSON");
        assert_eq!(profile.layer_count, 1);
        assert_eq!(profile.layers[0].feature_count, 2);
        assert_eq!(profile.layers[0].geometry_type, "Point");
        assert_eq!(profile.layers[0].field_count, 2);
        assert_eq!(profile.layers[0].null_geometry_count, 1);
    }

    #[test]
    fn detect_geometry_type_point() {
        let layer = Layer {
            name: "pts".into(),
            crs: None,
            features: vec![Feature {
                id: None,
                geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                properties: HashMap::new(),
            }],
            bounds: None,
        };
        assert_eq!(detect_geometry_type(&layer), "Point");
    }

    #[test]
    fn detect_geometry_type_none() {
        let layer = Layer {
            name: "empty".into(),
            crs: None,
            features: vec![],
            bounds: None,
        };
        assert_eq!(detect_geometry_type(&layer), "None");
    }
}
