/// GeoPackage reader.
///
/// This module reserves the `.gpkg` path and provides explicit errors until
/// full read/write support is implemented.
use crate::core::error::{Result, TissotError};
use crate::core::rule::Layer;
use std::path::Path;

/// Read a GeoPackage file and return layers.
pub fn read(path: &Path) -> Result<Vec<Layer>> {
    Err(TissotError::UnsupportedFormat(format!(
        "GeoPackage support is not yet available for '{}' (planned: pure-Rust geozero path with optional GDAL fallback)",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    #[test]
    fn returns_explicit_not_supported_error() {
        let path = std::path::Path::new("data.gpkg");
        let err = super::read(path).unwrap_err();
        assert!(
            err.to_string()
                .contains("GeoPackage support is not yet available")
        );
    }
}
