/// PyO3 Python bindings for Tissot.
///
/// Thin wrapper — all computation happens in Rust. Functions accept file paths
/// and option strings, returning JSON strings that Python can `json.loads()`.
use pyo3::prelude::*;
use std::path::Path;

use crate::checkers;
use crate::core::config::Config;
use crate::core::error::TissotError;
use crate::core::rule::Domain;
use crate::diff;
use crate::fix;
use crate::io as tissot_io;
use crate::score;
use crate::xray;

/// Convert a TissotError into a Python exception.
fn to_py_err(e: TissotError) -> PyErr {
    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
}

/// Parse a domain string into a Domain enum.
fn parse_domain(s: &str) -> Option<Domain> {
    match s.to_lowercase().as_str() {
        "projection" | "proj" | "crs" => Some(Domain::Projection),
        "quality" | "data_quality" | "data-quality" => Some(Domain::DataQuality),
        "cartography" | "carto" => Some(Domain::Cartography),
        "diff" => Some(Domain::Diff),
        "cloud" | "cloud-native" => Some(Domain::Cloud),
        _ => None,
    }
}

/// Run Projection X-Ray analysis on a geospatial file.
///
/// Returns a JSON string containing the full XrayReport with distortion
/// samples, heatmap grid, Tissot ellipses, and CRS recommendations.
///
/// Args:
///     file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
///
/// Returns:
///     JSON string of the XrayReport.
///
/// Raises:
///     RuntimeError: If the file cannot be read or analysis fails.
#[pyfunction]
fn xray(file_path: &str) -> PyResult<String> {
    let path = Path::new(file_path);
    let config = Config::default();
    let layers = tissot_io::read_file(path).map_err(to_py_err)?;

    let layer = layers
        .first()
        .ok_or_else(|| to_py_err(TissotError::Internal("No layers found in file".into())))?;

    let report = xray::analyze(layer, &config, file_path).map_err(to_py_err)?;

    serde_json::to_string(&report)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Run diagnostic checks on a geospatial file.
///
/// Returns a JSON string containing an array of Finding objects with
/// rule IDs, severity levels, messages, and spatial locations.
///
/// Args:
///     file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
///     domain: Optional domain filter — one of "projection", "quality",
///         "cartography", "diff", "cloud". If None, all domains are checked.
///
/// Returns:
///     JSON string of the findings array.
///
/// Raises:
///     RuntimeError: If the file cannot be read or checks fail.
#[pyfunction]
#[pyo3(signature = (file_path, domain=None))]
fn check(file_path: &str, domain: Option<&str>) -> PyResult<String> {
    let path = Path::new(file_path);
    let config = Config::default();
    let layers = tissot_io::read_file(path).map_err(to_py_err)?;

    let domain_filter = domain.and_then(parse_domain);

    let findings = checkers::run_checks(&layers, &config, file_path, domain_filter);

    serde_json::to_string(&findings)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Compute a quality score (0-100) for a geospatial file.
///
/// Runs all diagnostic checks and aggregates the results into a
/// Lighthouse-style score with category breakdown and letter grade.
///
/// Args:
///     file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
///
/// Returns:
///     JSON string of the ScoreReport with overall score, grade,
///     category scores, and finding count.
///
/// Raises:
///     RuntimeError: If the file cannot be read or scoring fails.
#[pyfunction]
fn score(file_path: &str) -> PyResult<String> {
    let path = Path::new(file_path);
    let config = Config::default();
    let layers = tissot_io::read_file(path).map_err(to_py_err)?;

    let findings = checkers::run_checks(&layers, &config, file_path, None);
    let report = score::compute_score(&findings, &config);

    serde_json::to_string(&report)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Apply automatic fixes to a geospatial file.
///
/// Supports reprojection to a target CRS and topology healing
/// (null geometry removal, duplicate geometry deduplication).
/// Writes a new file with the "_fixed" suffix by default.
///
/// Args:
///     file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
///     reproject: Optional target CRS string (e.g. "EPSG:3857").
///         If provided, reprojects all geometries.
///     topology: If True, removes null and duplicate geometries.
///
/// Returns:
///     JSON string of the FixReport with input/output paths,
///     updated feature count, and actions applied.
///
/// Raises:
///     RuntimeError: If the file cannot be read or fix operations fail.
#[pyfunction]
#[pyo3(signature = (file_path, reproject=None, topology=false))]
fn fix(file_path: &str, reproject: Option<&str>, topology: bool) -> PyResult<String> {
    let path = Path::new(file_path);
    let config = Config::default();
    let layers = tissot_io::read_file(path).map_err(to_py_err)?;

    let report = if let Some(target_crs) = reproject {
        let source_crs = layers
            .first()
            .and_then(|l| l.crs.clone())
            .unwrap_or_else(|| "EPSG:4326".to_string());

        fix::reproject_file(path, &layers, &source_crs, target_crs, false, &config)
            .map_err(to_py_err)?
    } else if topology {
        fix::heal_topology_file(path, &layers, false).map_err(to_py_err)?
    } else {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "At least one fix option must be specified: reproject or topology",
        ));
    };

    serde_json::to_string(&report)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Compare two geospatial files and compute a structural diff.
///
/// Computes feature count differences, extent changes, and produces
/// a lightweight comparison report.
///
/// Args:
///     left: Path to the first (baseline) geospatial file.
///     right: Path to the second (comparison) geospatial file.
///
/// Returns:
///     JSON string of the DiffReport with feature counts,
///     added/removed counts, and extent change flag.
///
/// Raises:
///     RuntimeError: If either file cannot be read.
#[pyfunction]
fn diff(left: &str, right: &str) -> PyResult<String> {
    let left_path = Path::new(left);
    let right_path = Path::new(right);

    let left_layers = tissot_io::read_file(left_path).map_err(to_py_err)?;
    let right_layers = tissot_io::read_file(right_path).map_err(to_py_err)?;

    let report = diff::compare(left, right, &left_layers, &right_layers);

    serde_json::to_string(&report)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Tissot Python module — geospatial diagnostics from Rust.
#[pymodule]
pub fn _tissot(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(xray, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(score, m)?)?;
    m.add_function(wrap_pyfunction!(fix, m)?)?;
    m.add_function(wrap_pyfunction!(diff, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_domain_variants() {
        assert_eq!(parse_domain("projection"), Some(Domain::Projection));
        assert_eq!(parse_domain("proj"), Some(Domain::Projection));
        assert_eq!(parse_domain("crs"), Some(Domain::Projection));
        assert_eq!(parse_domain("quality"), Some(Domain::DataQuality));
        assert_eq!(parse_domain("data_quality"), Some(Domain::DataQuality));
        assert_eq!(parse_domain("data-quality"), Some(Domain::DataQuality));
        assert_eq!(parse_domain("cartography"), Some(Domain::Cartography));
        assert_eq!(parse_domain("carto"), Some(Domain::Cartography));
        assert_eq!(parse_domain("diff"), Some(Domain::Diff));
        assert_eq!(parse_domain("cloud"), Some(Domain::Cloud));
        assert_eq!(parse_domain("cloud-native"), Some(Domain::Cloud));
        assert_eq!(parse_domain("unknown"), None);
    }
}
