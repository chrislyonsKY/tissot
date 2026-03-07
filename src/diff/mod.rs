/// Diff engine — compares two geospatial datasets.
use crate::core::rule::Layer;
use serde::Serialize;

/// Result of dataset comparison.
#[derive(Debug, Serialize)]
pub struct DiffReport {
    /// Left file path.
    pub left_file: String,
    /// Right file path.
    pub right_file: String,
    /// Left feature count.
    pub left_features: usize,
    /// Right feature count.
    pub right_features: usize,
    /// Approximate added feature count.
    pub added: usize,
    /// Approximate removed feature count.
    pub removed: usize,
    /// Bounding box delta if available.
    pub extent_changed: bool,
}

/// Compute a lightweight structural diff between two layer sets.
pub fn compare(left_file: &str, right_file: &str, left: &[Layer], right: &[Layer]) -> DiffReport {
    let left_features = left.iter().map(|l| l.features.len()).sum::<usize>();
    let right_features = right.iter().map(|l| l.features.len()).sum::<usize>();

    let added = right_features.saturating_sub(left_features);
    let removed = left_features.saturating_sub(right_features);

    let left_bounds = union_bounds(left);
    let right_bounds = union_bounds(right);
    let extent_changed = left_bounds != right_bounds;

    DiffReport {
        left_file: left_file.to_string(),
        right_file: right_file.to_string(),
        left_features,
        right_features,
        added,
        removed,
        extent_changed,
    }
}

/// Convert a layer set to a GeoJSON FeatureCollection for visual diff rendering.
pub fn layers_to_geojson(layers: &[Layer]) -> serde_json::Value {
    let mut features = Vec::new();

    for layer in layers {
        for feature in &layer.features {
            let Some(ref geometry) = feature.geometry else {
                continue;
            };

            let mut properties = serde_json::Map::new();
            properties.insert(
                "layer".to_string(),
                serde_json::Value::String(layer.name.clone()),
            );
            if let Some(ref id) = feature.id {
                properties.insert("id".to_string(), serde_json::Value::String(id.clone()));
            }
            for (k, v) in &feature.properties {
                properties.insert(k.clone(), v.clone());
            }

            features.push(geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::from(geometry)),
                id: feature.id.clone().map(geojson::feature::Id::String),
                properties: Some(properties),
                foreign_members: None,
            });
        }
    }

    serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    })
}

fn union_bounds(layers: &[Layer]) -> Option<[f64; 4]> {
    let mut out: Option<[f64; 4]> = None;
    for layer in layers {
        if let Some(b) = layer.bounds {
            out = Some(match out {
                None => b,
                Some(cur) => [
                    cur[0].min(b[0]),
                    cur[1].min(b[1]),
                    cur[2].max(b[2]),
                    cur[3].max(b[3]),
                ],
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_counts() {
        let l = Layer {
            name: "a".to_string(),
            crs: Some("EPSG:4326".to_string()),
            features: vec![],
            bounds: None,
        };
        let report = compare("l", "r", std::slice::from_ref(&l), std::slice::from_ref(&l));
        assert_eq!(report.added, 0);
        assert_eq!(report.removed, 0);
    }

    #[test]
    fn geojson_export_contains_features() {
        let layer = Layer {
            name: "a".to_string(),
            crs: Some("EPSG:4326".to_string()),
            features: vec![crate::core::rule::Feature {
                id: Some("f-1".to_string()),
                geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 2.0))),
                properties: std::collections::HashMap::new(),
            }],
            bounds: Some([1.0, 2.0, 1.0, 2.0]),
        };
        let geojson = layers_to_geojson(&[layer]);
        assert_eq!(geojson["type"], "FeatureCollection");
        assert_eq!(geojson["features"].as_array().map(|f| f.len()), Some(1));
    }
}
