//! Format detection and metadata for geospatial file formats.
//!
//! Identifies format from file extension and provides metadata including
//! whether the format is cloud-optimized and links to the CNG Formats Guide.

use serde::{Deserialize, Serialize};

/// Metadata about a geospatial file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    /// Human-readable format name (e.g., "GeoJSON", "GeoPackage").
    pub name: String,
    /// File extension (e.g., ".geojson", ".gpkg").
    pub extension: String,
    /// Whether this format is cloud-optimized (supports HTTP range requests, etc.).
    pub cloud_optimized: bool,
    /// URL to the relevant CNG (Cloud-Native Geospatial) Formats Guide entry.
    pub cng_guide_url: Option<String>,
}

/// Detect the geospatial format from a file path's extension.
///
/// Returns `None` for unrecognized extensions.
pub fn detect_format(path: &str) -> Option<FormatInfo> {
    let lower = path.to_lowercase();

    if lower.ends_with(".geojson") || lower.ends_with(".json") {
        Some(FormatInfo {
            name: "GeoJSON".into(),
            extension: ".geojson".into(),
            cloud_optimized: false,
            cng_guide_url: None,
        })
    } else if lower.ends_with(".gpkg") {
        Some(FormatInfo {
            name: "GeoPackage".into(),
            extension: ".gpkg".into(),
            cloud_optimized: false,
            cng_guide_url: Some("https://guide.cloudnativegeo.org/geopackage/".into()),
        })
    } else if lower.ends_with(".shp") {
        Some(FormatInfo {
            name: "Shapefile".into(),
            extension: ".shp".into(),
            cloud_optimized: false,
            cng_guide_url: None,
        })
    } else if lower.ends_with(".fgb") {
        Some(FormatInfo {
            name: "FlatGeobuf".into(),
            extension: ".fgb".into(),
            cloud_optimized: true,
            cng_guide_url: Some("https://guide.cloudnativegeo.org/flatgeobuf/".into()),
        })
    } else if lower.ends_with(".parquet") || lower.ends_with(".geoparquet") {
        Some(FormatInfo {
            name: "GeoParquet".into(),
            extension: ".parquet".into(),
            cloud_optimized: true,
            cng_guide_url: Some("https://guide.cloudnativegeo.org/geoparquet/".into()),
        })
    } else if lower.ends_with(".pmtiles") {
        Some(FormatInfo {
            name: "PMTiles".into(),
            extension: ".pmtiles".into(),
            cloud_optimized: true,
            cng_guide_url: Some("https://guide.cloudnativegeo.org/pmtiles/".into()),
        })
    } else if lower.ends_with(".tif") || lower.ends_with(".tiff") {
        Some(FormatInfo {
            name: "GeoTIFF".into(),
            extension: ".tif".into(),
            cloud_optimized: false,
            cng_guide_url: Some(
                "https://guide.cloudnativegeo.org/cloud-optimized-geotiffs/".into(),
            ),
        })
    } else if lower.ends_with(".qgz") || lower.ends_with(".qgs") {
        Some(FormatInfo {
            name: "QGIS Project".into(),
            extension: if lower.ends_with(".qgz") {
                ".qgz".into()
            } else {
                ".qgs".into()
            },
            cloud_optimized: false,
            cng_guide_url: None,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_geojson() {
        let info = detect_format("data/test.geojson").unwrap();
        assert_eq!(info.name, "GeoJSON");
        assert!(!info.cloud_optimized);
    }

    #[test]
    fn detect_geopackage() {
        let info = detect_format("data/test.gpkg").unwrap();
        assert_eq!(info.name, "GeoPackage");
        assert!(!info.cloud_optimized);
        assert!(info.cng_guide_url.is_some());
    }

    #[test]
    fn detect_flatgeobuf() {
        let info = detect_format("/path/to/data.fgb").unwrap();
        assert_eq!(info.name, "FlatGeobuf");
        assert!(info.cloud_optimized);
    }

    #[test]
    fn detect_shapefile() {
        let info = detect_format("roads.shp").unwrap();
        assert_eq!(info.name, "Shapefile");
        assert!(!info.cloud_optimized);
    }

    #[test]
    fn detect_geoparquet() {
        let info = detect_format("data.parquet").unwrap();
        assert_eq!(info.name, "GeoParquet");
        assert!(info.cloud_optimized);
    }

    #[test]
    fn unknown_format() {
        assert!(detect_format("data.xyz").is_none());
    }

    #[test]
    fn case_insensitive() {
        let info = detect_format("DATA.GEOJSON").unwrap();
        assert_eq!(info.name, "GeoJSON");
    }
}
