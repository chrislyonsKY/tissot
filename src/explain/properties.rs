//! CRS explanation engine — turn EPSG codes into human-readable descriptions.
//!
//! Combines the curated database with projection family classification
//! to produce actionable explanations for any CRS.

use serde::{Deserialize, Serialize};

use super::crs_database;

/// A human-readable explanation of a coordinate reference system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrsExplanation {
    /// EPSG code.
    pub epsg: u32,
    /// CRS name.
    pub name: String,
    /// Projection family (e.g., "Transverse Mercator", "Lambert Conformal Conic").
    pub projection_family: String,
    /// What metric properties are preserved.
    pub preserves: PreservationProperties,
    /// Plain-English description of distortion characteristics.
    pub distortion_characteristics: String,
    /// What this CRS is recommended for.
    pub recommended_use: String,
    /// Warnings about common misuse (e.g., "Do not use for area calculations").
    pub warnings: Vec<String>,
}

/// Metric properties preserved by a CRS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationProperties {
    /// Preserves area (equal-area).
    pub area: bool,
    /// Preserves shape locally (conformal).
    pub shape: bool,
    /// Preserves distance along certain lines (equidistant).
    pub distance: bool,
    /// Preserves direction from a center point (azimuthal).
    pub direction: bool,
}

/// Explain a CRS given its identifier string (e.g., "EPSG:4326").
///
/// Parses the EPSG code, looks it up in the curated database, and
/// generates a comprehensive explanation. For unknown codes, returns
/// a generic explanation based on code classification.
pub fn explain_crs(crs_identifier: &str) -> Option<CrsExplanation> {
    let epsg = parse_epsg(crs_identifier)?;

    if let Some(entry) = crs_database::lookup(epsg) {
        Some(from_database_entry(&entry))
    } else {
        Some(generic_explanation(epsg))
    }
}

/// Parse an EPSG code from a CRS identifier string.
///
/// Accepts formats: "EPSG:4326", "epsg:4326", "4326".
fn parse_epsg(crs: &str) -> Option<u32> {
    let trimmed = crs.trim();

    // Try "EPSG:NNNNN" format.
    if let Some(code_str) = trimmed
        .strip_prefix("EPSG:")
        .or_else(|| trimmed.strip_prefix("epsg:"))
    {
        return code_str.parse().ok();
    }

    // Try bare number.
    trimmed.parse().ok()
}

/// Convert a curated database entry into a full explanation.
fn from_database_entry(entry: &crs_database::CrsEntry) -> CrsExplanation {
    let projection_family = classify_projection_family(entry.epsg, &entry.name);
    let distortion_characteristics = describe_distortion(&entry.name, &projection_family, entry);
    let warnings = generate_warnings(entry);

    CrsExplanation {
        epsg: entry.epsg,
        name: entry.name.clone(),
        projection_family,
        preserves: PreservationProperties {
            area: entry.preserves_area,
            shape: entry.preserves_shape,
            distance: entry.preserves_distance,
            direction: entry.preserves_direction,
        },
        distortion_characteristics,
        recommended_use: entry.suitable_for.clone(),
        warnings,
    }
}

/// Generate a generic explanation for EPSG codes not in the curated database.
fn generic_explanation(epsg: u32) -> CrsExplanation {
    let (name, family, region) = classify_by_code_range(epsg);

    CrsExplanation {
        epsg,
        name,
        projection_family: family,
        preserves: PreservationProperties {
            area: false,
            shape: false,
            distance: false,
            direction: false,
        },
        distortion_characteristics: "Properties unknown — not in curated database. Run `tissot xray` to measure actual distortion.".into(),
        recommended_use: format!("Region: {region}. Use `tissot xray` to evaluate suitability."),
        warnings: vec!["This EPSG code is not in the curated database. Distortion properties are estimated.".into()],
    }
}

/// Classify the projection family based on EPSG code and name heuristics.
fn classify_projection_family(epsg: u32, name: &str) -> String {
    let lower = name.to_lowercase();

    if lower.contains("utm") || lower.contains("transverse mercator") || lower.contains("tm2") {
        "Transverse Mercator".into()
    } else if lower.contains("mercator") || epsg == 3857 {
        "Mercator".into()
    } else if lower.contains("lambert") && lower.contains("conformal") {
        "Lambert Conformal Conic".into()
    } else if lower.contains("albers") {
        "Albers Equal-Area Conic".into()
    } else if lower.contains("laea") || lower.contains("lambert azimuthal") {
        "Lambert Azimuthal Equal-Area".into()
    } else if lower.contains("robinson") {
        "Robinson (compromise)".into()
    } else if lower.contains("cylindrical equal area") {
        "Cylindrical Equal-Area".into()
    } else if epsg == 4326 || epsg == 4269 || epsg == 4267 {
        "Geographic (unprojected)".into()
    } else if (32600..32661).contains(&epsg) || (32700..32761).contains(&epsg) {
        "Transverse Mercator".into()
    } else {
        "Unknown".into()
    }
}

/// Generate a plain-English distortion description.
fn describe_distortion(name: &str, family: &str, entry: &crs_database::CrsEntry) -> String {
    let mut parts = Vec::new();

    if entry.preserves_area {
        parts.push("Preserves area — suitable for area measurements and density mapping");
    }
    if entry.preserves_shape {
        parts.push("Preserves local shape (conformal) — angles are correct locally");
    }

    if !entry.preserves_area && !entry.preserves_shape {
        if family.contains("Geographic") {
            parts.push("Geographic coordinates (degrees) — not a projection, no metric properties preserved");
        } else if family.contains("Robinson") {
            parts.push("Compromise projection — neither area nor shape is exactly preserved, but both are reasonable");
        } else {
            parts.push("Neither area nor shape is exactly preserved");
        }
    }

    if name.to_lowercase().contains("web mercator") || name.to_lowercase().contains("pseudo") {
        parts.push("Area distortion increases dramatically with latitude — Greenland appears as large as Africa");
    }

    parts.join(". ") + "."
}

/// Generate warnings for common CRS misuse.
fn generate_warnings(entry: &crs_database::CrsEntry) -> Vec<String> {
    let mut warnings = Vec::new();

    if entry.epsg == 3857 {
        warnings.push(
            "Do not use for area calculations — area distortion exceeds 400% near the poles."
                .into(),
        );
        warnings.push("Coordinates are in meters but do not represent true ground distances at most latitudes.".into());
    }

    if entry.epsg == 4326 || entry.epsg == 4269 || entry.epsg == 4267 {
        warnings.push("Geographic CRS — coordinates are in degrees, not meters. Do not use for distance or area calculations without projecting first.".into());
    }

    if entry.epsg == 4267 {
        warnings.push("NAD27 is a legacy datum. Positional offsets from NAD83/WGS84 can exceed 100 meters. Convert to NAD83 for modern use.".into());
    }

    if !entry.preserves_area && !entry.preserves_shape {
        warnings.push("This CRS does not preserve area or shape. Consider a more appropriate projection for analytical work.".into());
    }

    warnings
}

/// Classify a CRS by its EPSG code range.
fn classify_by_code_range(epsg: u32) -> (String, String, String) {
    match epsg {
        4000..=4999 => (
            format!("Geographic CRS (EPSG:{epsg})"),
            "Geographic (unprojected)".into(),
            "Varies by datum".into(),
        ),
        2000..=2999 => (
            format!("Projected CRS (EPSG:{epsg})"),
            "Projected (family unknown)".into(),
            "Regional".into(),
        ),
        32600..=32660 => {
            let zone = epsg - 32600;
            (
                format!("WGS84 / UTM Zone {zone}N"),
                "Transverse Mercator".into(),
                format!("UTM Zone {zone} Northern Hemisphere"),
            )
        }
        32700..=32760 => {
            let zone = epsg - 32700;
            (
                format!("WGS84 / UTM Zone {zone}S"),
                "Transverse Mercator".into(),
                format!("UTM Zone {zone} Southern Hemisphere"),
            )
        }
        _ => (
            format!("CRS EPSG:{epsg}"),
            "Unknown".into(),
            "Unknown".into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_wgs84() {
        let expl = explain_crs("EPSG:4326").unwrap();
        assert_eq!(expl.epsg, 4326);
        assert_eq!(expl.name, "WGS 84");
        assert!(expl.projection_family.contains("Geographic"));
        assert!(!expl.warnings.is_empty());
    }

    #[test]
    fn explain_web_mercator() {
        let expl = explain_crs("EPSG:3857").unwrap();
        assert!(expl.preserves.shape);
        assert!(!expl.preserves.area);
        assert!(expl.warnings.iter().any(|w| w.contains("area")));
    }

    #[test]
    fn explain_albers_conus() {
        let expl = explain_crs("EPSG:5070").unwrap();
        assert!(expl.preserves.area);
        assert!(expl.distortion_characteristics.contains("area"));
    }

    #[test]
    fn explain_unknown_code() {
        let expl = explain_crs("EPSG:99999").unwrap();
        assert_eq!(expl.epsg, 99999);
        assert!(
            expl.warnings
                .iter()
                .any(|w| w.contains("not in the curated database"))
        );
    }

    #[test]
    fn parse_epsg_formats() {
        assert_eq!(parse_epsg("EPSG:4326"), Some(4326));
        assert_eq!(parse_epsg("epsg:3857"), Some(3857));
        assert_eq!(parse_epsg("5070"), Some(5070));
        assert_eq!(parse_epsg("not-a-crs"), None);
    }

    #[test]
    fn explain_utm_zone() {
        let expl = explain_crs("EPSG:32617").unwrap();
        assert!(expl.projection_family.contains("Transverse Mercator"));
    }

    #[test]
    fn generic_utm_explanation() {
        let expl = explain_crs("EPSG:32650").unwrap();
        assert!(expl.name.contains("UTM"));
        assert!(expl.projection_family.contains("Transverse Mercator"));
    }
}
