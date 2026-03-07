/// Shapefile reader — pure Rust implementation via `shapefile` crate.
use crate::core::error::{Result, TissotError};
use crate::core::rule::{Feature, Layer};
use geo::Geometry;
use shapefile::record::traits::HasXY;
use std::collections::HashMap;
use std::path::Path;

/// Read a Shapefile (.shp) and return layers.
pub fn read(path: &Path) -> Result<Vec<Layer>> {
    let mut reader = shapefile::Reader::from_path(path).map_err(|e| {
        TissotError::Io(std::io::Error::other(format!(
            "Failed to open shapefile: {e}"
        )))
    })?;

    // Try reading the .prj sidecar for CRS info.
    let crs = read_prj(path);

    let mut features = Vec::new();

    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.map_err(|e| {
            TissotError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to read shapefile record: {e}"),
            ))
        })?;

        let geometry = convert_shape(shape);
        let properties = convert_record(record);

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

/// Read the .prj sidecar file for CRS information.
fn read_prj(shp_path: &Path) -> Option<String> {
    let prj_path = shp_path.with_extension("prj");
    let wkt = std::fs::read_to_string(prj_path).ok()?;
    // Try to extract EPSG from the WKT, otherwise return the raw WKT.
    if let Some(epsg) = extract_epsg_from_wkt(&wkt) {
        Some(format!("EPSG:{epsg}"))
    } else {
        Some(wkt.trim().to_string())
    }
}

/// Attempt to extract EPSG code from a WKT CRS definition.
fn extract_epsg_from_wkt(wkt: &str) -> Option<u32> {
    // Look for AUTHORITY["EPSG","XXXX"] pattern.
    let upper = wkt.to_uppercase();
    let idx = upper.find("AUTHORITY[\"EPSG\",")?;
    let after = &wkt[idx + 17..];
    let quote_start = after.find('"')? + 1;
    let remaining = &after[quote_start..];
    let quote_end = remaining.find('"')?;
    remaining[..quote_end].parse::<u32>().ok()
}

/// Convert a shapefile Shape to a geo::Geometry.
fn convert_shape(shape: shapefile::Shape) -> Option<Geometry> {
    match shape {
        shapefile::Shape::Point(p) => Some(Geometry::Point(geo::Point::new(p.x, p.y))),
        shapefile::Shape::PointM(p) => Some(Geometry::Point(geo::Point::new(p.x, p.y))),
        shapefile::Shape::PointZ(p) => Some(Geometry::Point(geo::Point::new(p.x, p.y))),
        shapefile::Shape::Multipoint(mp) => {
            let points: Vec<geo::Point> = mp
                .points()
                .iter()
                .map(|p| geo::Point::new(p.x(), p.y()))
                .collect();
            Some(Geometry::MultiPoint(geo::MultiPoint(points)))
        }
        shapefile::Shape::Polyline(pl) => convert_polyline_parts(pl.parts()),
        shapefile::Shape::PolylineM(pl) => convert_polyline_parts(pl.parts()),
        shapefile::Shape::PolylineZ(pl) => convert_polyline_parts(pl.parts()),
        shapefile::Shape::Polygon(pg) => convert_polygon_rings(pg.rings()),
        shapefile::Shape::PolygonM(pg) => convert_polygon_rings(pg.rings()),
        shapefile::Shape::PolygonZ(pg) => convert_polygon_rings(pg.rings()),
        shapefile::Shape::NullShape => None,
        _ => None,
    }
}

fn convert_polyline_parts<PointType: HasXY>(parts: &[Vec<PointType>]) -> Option<Geometry> {
    let lines: Vec<geo::LineString> = parts
        .iter()
        .map(|part| {
            geo::LineString::from(
                part.iter()
                    .map(|p| geo::coord! { x: p.x(), y: p.y() })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    match lines.len() {
        0 => None,
        1 => lines.into_iter().next().map(Geometry::LineString),
        _ => Some(Geometry::MultiLineString(geo::MultiLineString(lines))),
    }
}

fn convert_polygon_rings<PointType: HasXY>(
    rings: &[shapefile::PolygonRing<PointType>],
) -> Option<Geometry> {
    let mut polygons: Vec<geo::Polygon> = Vec::new();
    let mut current_exterior: Option<geo::LineString> = None;
    let mut holes: Vec<geo::LineString> = Vec::new();

    for ring in rings {
        let coords: Vec<geo::Coord> = ring
            .points()
            .iter()
            .map(|p| geo::coord! { x: p.x(), y: p.y() })
            .collect();
        let ls = geo::LineString::from(coords);

        match ring {
            shapefile::PolygonRing::Outer(_) => {
                if let Some(ext) = current_exterior.take() {
                    polygons.push(geo::Polygon::new(ext, std::mem::take(&mut holes)));
                }
                current_exterior = Some(ls);
            }
            shapefile::PolygonRing::Inner(_) => holes.push(ls),
        }
    }

    if let Some(ext) = current_exterior {
        polygons.push(geo::Polygon::new(ext, holes));
    }

    match polygons.len() {
        0 => None,
        1 => polygons.into_iter().next().map(Geometry::Polygon),
        _ => Some(Geometry::MultiPolygon(geo::MultiPolygon(polygons))),
    }
}

/// Convert a shapefile dBASE record to a properties HashMap.
fn convert_record(record: shapefile::dbase::Record) -> HashMap<String, serde_json::Value> {
    let mut props = HashMap::new();
    for (name, value) in record {
        let json_val = match value {
            shapefile::dbase::FieldValue::Character(Some(s)) => serde_json::Value::String(s),
            shapefile::dbase::FieldValue::Numeric(Some(n)) => {
                serde_json::json!(n)
            }
            shapefile::dbase::FieldValue::Float(Some(n)) => {
                serde_json::json!(n as f64)
            }
            shapefile::dbase::FieldValue::Integer(n) => {
                serde_json::json!(n)
            }
            shapefile::dbase::FieldValue::Double(n) => {
                serde_json::json!(n)
            }
            shapefile::dbase::FieldValue::Logical(Some(b)) => serde_json::Value::Bool(b),
            _ => serde_json::Value::Null,
        };
        if !json_val.is_null() {
            props.insert(name, json_val);
        }
    }
    props
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
    use super::*;

    #[test]
    fn extract_epsg_from_wkt_valid() {
        let wkt = r#"GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["Degree",0.017453292519943295],AUTHORITY["EPSG","4326"]]"#;
        assert_eq!(extract_epsg_from_wkt(wkt), Some(4326));
    }

    #[test]
    fn extract_epsg_from_wkt_missing() {
        let wkt = r#"PROJCS["Unknown"]"#;
        assert_eq!(extract_epsg_from_wkt(wkt), None);
    }

    #[test]
    fn null_shape_returns_none() {
        let shape = shapefile::Shape::NullShape;
        assert!(convert_shape(shape).is_none());
    }
}
