/// Fix engine — applies automatic remediations for findings.
use crate::core::config::Config;
use crate::core::error::{Result, TissotError};
use crate::core::rule::Layer;
use serde::Serialize;
use std::path::Path;

/// Summary of a fix operation.
#[derive(Debug, Serialize)]
pub struct FixReport {
    /// Input path.
    pub input: String,
    /// Output path.
    pub output: String,
    /// Number of updated features.
    pub updated_features: usize,
    /// Human-readable actions applied.
    pub actions: Vec<String>,
}

/// Reproject all layer geometries to a target CRS and write GeoJSON output.
pub fn reproject_file(
    input: &Path,
    layers: &[Layer],
    source_crs: &str,
    target_crs: &str,
    in_place: bool,
    _config: &Config,
) -> Result<FixReport> {
    let output = if in_place {
        input.to_path_buf()
    } else {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        input.with_file_name(format!("{stem}_fixed.geojson"))
    };

    let proj = proj::Proj::new_known_crs(source_crs, target_crs, None).map_err(|e| {
        TissotError::Projection(format!(
            "Failed to create projection {source_crs} -> {target_crs}: {e}"
        ))
    })?;

    let mut updated = 0usize;
    let mut out_features = Vec::new();

    for layer in layers {
        for f in &layer.features {
            let mut geojson_feature = geojson::Feature {
                bbox: None,
                geometry: None,
                id: f.id.clone().map(geojson::feature::Id::String),
                properties: Some(f.properties.clone().into_iter().collect()),
                foreign_members: None,
            };

            if let Some(ref geom) = f.geometry {
                let mapped = map_geometry(geom, &proj)?;
                let gjson = geojson::Geometry::from(&mapped);
                geojson_feature.geometry = Some(gjson);
                updated += 1;
            }

            out_features.push(geojson_feature);
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: out_features,
        foreign_members: None,
    };
    let gjson = geojson::GeoJson::from(fc);
    std::fs::write(&output, gjson.to_string())?;

    Ok(FixReport {
        input: input.display().to_string(),
        output: output.display().to_string(),
        updated_features: updated,
        actions: vec![
            format!("Reprojected coordinates from {source_crs} to {target_crs}"),
            "Wrote GeoJSON output".to_string(),
        ],
    })
}

/// Topology cleanup: remove null geometries and deduplicate exact geometry representations.
pub fn heal_topology_file(input: &Path, layers: &[Layer], in_place: bool) -> Result<FixReport> {
    let output = if in_place {
        input.to_path_buf()
    } else {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        input.with_file_name(format!("{stem}_topology_fixed.geojson"))
    };

    let mut out_features = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut kept = 0usize;

    for layer in layers {
        for f in &layer.features {
            let Some(ref geom) = f.geometry else {
                continue;
            };
            let key = format!("{geom:?}");
            if !seen.insert(key) {
                continue;
            }

            let mut gf = geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::from(geom)),
                id: f.id.clone().map(geojson::feature::Id::String),
                properties: Some(f.properties.clone().into_iter().collect()),
                foreign_members: None,
            };
            if gf.properties.is_none() {
                gf.properties = Some(serde_json::Map::new());
            }
            out_features.push(gf);
            kept += 1;
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: out_features,
        foreign_members: None,
    };
    let gjson = geojson::GeoJson::from(fc);
    std::fs::write(&output, gjson.to_string())?;

    Ok(FixReport {
        input: input.display().to_string(),
        output: output.display().to_string(),
        updated_features: kept,
        actions: vec![
            "Removed null geometries".to_string(),
            "Removed duplicate geometries".to_string(),
        ],
    })
}

fn map_geometry(geom: &geo::Geometry, proj: &proj::Proj) -> Result<geo::Geometry> {
    use geo::MapCoords;
    let mapped = geom.map_coords(|coord| {
        if let Ok(out) = proj.convert(coord) {
            geo::Coord { x: out.x, y: out.y }
        } else {
            coord
        }
    });
    Ok(mapped)
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_public_fix_entrypoints() {
        let _reproject = super::reproject_file;
        let _heal = super::heal_topology_file;
    }
}
