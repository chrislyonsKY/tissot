/// IO layer — format detection and reading for geospatial data.
///
/// Follows DL-004: geozero-first (pure Rust readers primary, GDAL optional).
pub mod flatgeobuf_reader;
pub mod geojson_reader;
pub mod geopackage_reader;
pub mod shapefile_reader;

use crate::core::error::{Result, TissotError};
use crate::core::rule::Layer;
use std::path::Path;

/// Supported input formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// GeoJSON (.geojson, .json)
    GeoJson,
    /// Shapefile (.shp)
    Shapefile,
    /// FlatGeobuf (.fgb)
    FlatGeobuf,
    /// GeoPackage (.gpkg)
    GeoPackage,
}

/// Detect file format from extension.
pub fn detect_format(path: &Path) -> Result<Format> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "geojson" | "json" => Ok(Format::GeoJson),
        "shp" => Ok(Format::Shapefile),
        "fgb" => Ok(Format::FlatGeobuf),
        "gpkg" => Ok(Format::GeoPackage),
        _ => Err(TissotError::UnsupportedFormat(format!(
            "Unknown file extension: .{ext}"
        ))),
    }
}

/// Read layers from a file, auto-detecting format.
pub fn read_file(path: &Path) -> Result<Vec<Layer>> {
    let format = detect_format(path)?;
    match format {
        Format::GeoJson => geojson_reader::read(path),
        Format::Shapefile => shapefile_reader::read(path),
        Format::FlatGeobuf => flatgeobuf_reader::read(path),
        Format::GeoPackage => geopackage_reader::read(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_geojson() {
        let path = PathBuf::from("data.geojson");
        assert_eq!(detect_format(&path).unwrap(), Format::GeoJson);
    }

    #[test]
    fn detect_json_as_geojson() {
        let path = PathBuf::from("data.json");
        assert_eq!(detect_format(&path).unwrap(), Format::GeoJson);
    }

    #[test]
    fn detect_shapefile() {
        let path = PathBuf::from("data.shp");
        assert_eq!(detect_format(&path).unwrap(), Format::Shapefile);
    }

    #[test]
    fn detect_flatgeobuf() {
        let path = PathBuf::from("data.fgb");
        assert_eq!(detect_format(&path).unwrap(), Format::FlatGeobuf);
    }

    #[test]
    fn detect_geopackage() {
        let path = PathBuf::from("data.gpkg");
        assert_eq!(detect_format(&path).unwrap(), Format::GeoPackage);
    }

    #[test]
    fn detect_unknown() {
        let path = PathBuf::from("data.xyz");
        assert!(detect_format(&path).is_err());
    }
}
