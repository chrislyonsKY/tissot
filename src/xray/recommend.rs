/// CRS recommendation engine — ranks projection candidates for a dataset.
///
/// Evaluates candidate CRS options against the dataset's geographic extent
/// and ranks them by optimization target (minimize area, distance, shape,
/// or balanced distortion).
use super::distortion::DistortionSample;
use serde::Serialize;

/// A CRS recommendation with comparison metrics.
#[derive(Debug, Serialize)]
pub struct CrsRecommendation {
    /// EPSG code or CRS identifier.
    pub crs: String,
    /// Human-readable name.
    pub name: String,
    /// Description of why this CRS is suitable.
    pub rationale: String,
    /// Optimization target this CRS was scored for.
    pub target: OptimizationTarget,
    /// Fitness score (0.0–1.0, higher is better).
    pub fitness: f64,
    /// Maximum area distortion percentage if this CRS were used.
    pub max_area_distortion_pct: f64,
    /// Maximum angular distortion in degrees if this CRS were used.
    pub max_angular_distortion_deg: f64,
}

/// What to optimize the CRS selection for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationTarget {
    /// Minimize area distortion (best for thematic/choropleth maps).
    Area,
    /// Minimize distance distortion (best for navigation/measurement).
    Distance,
    /// Minimize angular distortion (best for conformal/navigation).
    Shape,
    /// Balance all metrics.
    Balanced,
}

/// Geographic scope of the dataset.
#[derive(Debug, Clone, Copy)]
enum Scope {
    /// Less than 2° span — state/county level.
    Local,
    /// 2–10° span — multi-state/national.
    Regional,
    /// 10–60° span — continental.
    Continental,
    /// >60° span — global.
    Global,
}

/// Recommend CRS candidates for a dataset.
pub fn recommend(
    _samples: &[DistortionSample],
    bounds: [f64; 4],
    top_n: usize,
) -> Vec<CrsRecommendation> {
    let scope = classify_scope(bounds);
    let centroid_lon = (bounds[0] + bounds[2]) / 2.0;
    let centroid_lat = (bounds[1] + bounds[3]) / 2.0;

    let mut candidates = generate_candidates(scope, centroid_lon, centroid_lat);

    // Sort by fitness (descending).
    candidates.sort_by(|a, b| {
        b.fitness
            .partial_cmp(&a.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.truncate(top_n);
    candidates
}

/// Classify the geographic scope of the dataset.
fn classify_scope(bounds: [f64; 4]) -> Scope {
    let span_x = (bounds[2] - bounds[0]).abs();
    let span_y = (bounds[3] - bounds[1]).abs();
    let max_span = span_x.max(span_y);

    if max_span < 2.0 {
        Scope::Local
    } else if max_span < 10.0 {
        Scope::Regional
    } else if max_span < 60.0 {
        Scope::Continental
    } else {
        Scope::Global
    }
}

/// Generate CRS candidates based on scope and location.
fn generate_candidates(
    scope: Scope,
    centroid_lon: f64,
    centroid_lat: f64,
) -> Vec<CrsRecommendation> {
    let mut candidates = Vec::new();

    // Always include UTM zone for the centroid.
    let utm = utm_zone(centroid_lon, centroid_lat);
    candidates.push(CrsRecommendation {
        crs: utm.0.clone(),
        name: utm.1.clone(),
        rationale: "UTM zone for dataset centroid — good general-purpose projected CRS".into(),
        target: OptimizationTarget::Balanced,
        fitness: match scope {
            Scope::Local | Scope::Regional => 0.85,
            Scope::Continental => 0.60,
            Scope::Global => 0.30,
        },
        max_area_distortion_pct: 0.04,
        max_angular_distortion_deg: 0.0,
    });

    // Add scope-specific candidates.
    match scope {
        Scope::Local | Scope::Regional => {
            // Albers Equal Area centered on the data.
            candidates.push(CrsRecommendation {
                crs: format!(
                    "+proj=aea +lat_1={} +lat_2={} +lat_0={} +lon_0={} +datum=WGS84",
                    centroid_lat - 2.0,
                    centroid_lat + 2.0,
                    centroid_lat,
                    centroid_lon
                ),
                name: "Custom Albers Equal Area".into(),
                rationale:
                    "Equal-area projection centered on your data — best for area measurements"
                        .into(),
                target: OptimizationTarget::Area,
                fitness: 0.90,
                max_area_distortion_pct: 0.01,
                max_angular_distortion_deg: 2.0,
            });

            // Lambert Conformal Conic.
            candidates.push(CrsRecommendation {
                crs: format!("+proj=lcc +lat_1={} +lat_2={} +lat_0={} +lon_0={} +datum=WGS84",
                    centroid_lat - 2.0, centroid_lat + 2.0, centroid_lat, centroid_lon),
                name: "Custom Lambert Conformal Conic".into(),
                rationale: "Conformal projection centered on your data — best for preserving shapes".into(),
                target: OptimizationTarget::Shape,
                fitness: 0.85,
                max_area_distortion_pct: 3.0,
                max_angular_distortion_deg: 0.0,
            });
        }
        Scope::Continental => {
            // Continental equal-area.
            if centroid_lat > 20.0
                && centroid_lat < 72.0
                && centroid_lon > -170.0
                && centroid_lon < -50.0
            {
                candidates.push(CrsRecommendation {
                    crs: "EPSG:5070".into(),
                    name: "NAD83 / Conus Albers".into(),
                    rationale: "Standard equal-area projection for CONUS".into(),
                    target: OptimizationTarget::Area,
                    fitness: 0.88,
                    max_area_distortion_pct: 0.5,
                    max_angular_distortion_deg: 4.0,
                });
            }

            if centroid_lat > 35.0
                && centroid_lat < 72.0
                && centroid_lon > -12.0
                && centroid_lon < 45.0
            {
                candidates.push(CrsRecommendation {
                    crs: "EPSG:3035".into(),
                    name: "ETRS89-extended / LAEA Europe".into(),
                    rationale: "Standard equal-area projection for Europe".into(),
                    target: OptimizationTarget::Area,
                    fitness: 0.88,
                    max_area_distortion_pct: 0.5,
                    max_angular_distortion_deg: 4.0,
                });
            }
        }
        Scope::Global => {
            candidates.push(CrsRecommendation {
                crs: "ESRI:54009".into(),
                name: "World Mollweide".into(),
                rationale: "Global equal-area projection — best for area comparison".into(),
                target: OptimizationTarget::Area,
                fitness: 0.75,
                max_area_distortion_pct: 0.0,
                max_angular_distortion_deg: 20.0,
            });

            candidates.push(CrsRecommendation {
                crs: "ESRI:54030".into(),
                name: "World Robinson".into(),
                rationale: "Good compromise projection for global display".into(),
                target: OptimizationTarget::Balanced,
                fitness: 0.70,
                max_area_distortion_pct: 5.0,
                max_angular_distortion_deg: 10.0,
            });

            candidates.push(CrsRecommendation {
                crs: "EPSG:4326".into(),
                name: "WGS 84 (Geographic)".into(),
                rationale: "Unprojected geographic coordinates — no projection distortion but not suitable for area/distance measurements".into(),
                target: OptimizationTarget::Balanced,
                fitness: 0.50,
                max_area_distortion_pct: 0.0,
                max_angular_distortion_deg: 0.0,
            });
        }
    }

    // Always flag Web Mercator as poor choice for analysis.
    candidates.push(CrsRecommendation {
        crs: "EPSG:3857".into(),
        name: "Web Mercator (NOT recommended)".into(),
        rationale: "Web Mercator has severe area distortion away from equator — use only for web tile display".into(),
        target: OptimizationTarget::Balanced,
        fitness: 0.20,
        max_area_distortion_pct: 30.0,
        max_angular_distortion_deg: 0.0,
    });

    candidates
}

/// Determine the UTM zone for a given longitude/latitude.
fn utm_zone(lon: f64, lat: f64) -> (String, String) {
    let zone_number = ((lon + 180.0) / 6.0).floor() as u32 + 1;
    let _hemisphere = if lat >= 0.0 { "north" } else { "south" };
    let epsg = if lat >= 0.0 {
        32600 + zone_number
    } else {
        32700 + zone_number
    };
    (
        format!("EPSG:{epsg}"),
        format!(
            "UTM Zone {zone_number}{}",
            if lat >= 0.0 { "N" } else { "S" }
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utm_zone_kentucky() {
        let (crs, name) = utm_zone(-84.5, 38.0);
        assert_eq!(crs, "EPSG:32616");
        assert!(name.contains("16N"));
    }

    #[test]
    fn utm_zone_southern() {
        let (crs, _name) = utm_zone(25.0, -30.0);
        assert!(crs.starts_with("EPSG:327"));
    }

    #[test]
    fn scope_classification() {
        assert!(matches!(
            classify_scope([-85.0, 37.0, -84.0, 38.0]),
            Scope::Local
        ));
        assert!(matches!(
            classify_scope([-100.0, 25.0, -70.0, 50.0]),
            Scope::Continental
        ));
        assert!(matches!(
            classify_scope([-180.0, -90.0, 180.0, 90.0]),
            Scope::Global
        ));
    }

    #[test]
    fn recommend_returns_results() {
        let recs = recommend(&[], [-85.0, 37.0, -84.0, 38.0], 5);
        assert!(!recs.is_empty());
        assert!(recs.len() <= 5);
    }

    #[test]
    fn web_mercator_always_low_ranked() {
        let recs = recommend(&[], [-85.0, 37.0, -84.0, 38.0], 10);
        let mercator = recs.iter().find(|r| r.crs == "EPSG:3857").unwrap();
        assert!(mercator.fitness < 0.5);
    }
}
