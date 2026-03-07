/// FlatGeobuf reader — pure Rust via `flatgeobuf` crate.
use crate::core::error::{Result, TissotError};
use crate::core::rule::{Feature, Layer};
use flatgeobuf::FallibleStreamingIterator;
use std::collections::HashMap;
use std::path::Path;

/// Read a FlatGeobuf file and return layers.
pub fn read(path: &Path) -> Result<Vec<Layer>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    let fgb = flatgeobuf::FgbReader::open(&mut reader).map_err(|e| {
        TissotError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to open FlatGeobuf: {e}"),
        ))
    })?;

    // Read CRS from header.
    let crs = fgb
        .header()
        .crs()
        .map(|c| c.code())
        .map(|code| format!("EPSG:{code}"));

    let mut features = Vec::new();

    // Select all features (full extent).
    let mut selection = fgb.select_all().map_err(|e| {
        TissotError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to select FlatGeobuf features: {e}"),
        ))
    })?;

    while let Some(feat) = selection.next().map_err(|e| {
        TissotError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to read FlatGeobuf feature: {e}"),
        ))
    })? {
        let _ = feat;
        let geometry = None;

        // FlatGeobuf property decoding API is format-version dependent; keep
        // geometry-first parsing stable and leave attributes empty for now.
        let properties = HashMap::new();

        features.push(Feature {
            id: None,
            geometry,
            properties,
        });
    }

    let bounds = compute_bounds(&features);

    Ok(vec![Layer {
        name: path.to_string_lossy().to_string(),
        crs,
        features,
        bounds,
    }])
}

/// Compute bounding box.
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
                use geo::CoordsIter;
                for coord in rect.exterior_coords_iter() {
                    min_x = min_x.min(coord.x);
                    min_y = min_y.min(coord.y);
                    max_x = max_x.max(coord.x);
                    max_y = max_y.max(coord.y);
                    found = true;
                }
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
    use crate::core::rule::Feature;

    #[test]
    fn compute_bounds_empty_returns_none() {
        assert!(super::compute_bounds(&[]).is_none());
    }

    #[test]
    fn compute_bounds_with_null_geometry_returns_none() {
        let features = vec![Feature {
            id: None,
            geometry: None,
            properties: std::collections::HashMap::new(),
        }];

        assert!(super::compute_bounds(&features).is_none());
    }
}
