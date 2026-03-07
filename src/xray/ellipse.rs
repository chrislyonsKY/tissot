/// Tissot ellipse generation — creates ellipse polygons at sample points.
///
/// Each ellipse represents local distortion: a circle means no distortion,
/// an ellipse shows the direction and magnitude of stretch.
use super::distortion::DistortionSample;
use serde::Serialize;

/// Number of vertices per ellipse polygon.
const ELLIPSE_VERTICES: usize = 72;

/// A Tissot indicatrix ellipse for visualization.
#[derive(Debug, Serialize)]
pub struct TissotEllipse {
    /// Center longitude.
    pub lon: f64,
    /// Center latitude.
    pub lat: f64,
    /// Semimajor axis length (display units).
    pub semimajor: f64,
    /// Semiminor axis length (display units).
    pub semiminor: f64,
    /// Orientation angle in degrees.
    pub angle_deg: f64,
    /// Area distortion percentage at this point.
    pub area_distortion_pct: f64,
    /// Angular distortion in degrees at this point.
    pub angular_distortion_deg: f64,
    /// GeoJSON polygon coordinates for this ellipse.
    pub coordinates: Vec<[f64; 2]>,
}

/// Generate Tissot ellipses at all sample points.
pub fn generate(samples: &[DistortionSample]) -> Vec<TissotEllipse> {
    samples.iter().map(generate_one).collect()
}

/// Generate a single Tissot ellipse at a sample point.
fn generate_one(sample: &DistortionSample) -> TissotEllipse {
    // Scale factor for visual display (degrees).
    // Base radius in degrees — visually appropriate for map rendering.
    let base_radius = 0.3;

    let a = base_radius * sample.semimajor;
    let b = base_radius * sample.semiminor;
    let theta = sample.theta;

    let mut coords = Vec::with_capacity(ELLIPSE_VERTICES + 1);
    let step = 2.0 * std::f64::consts::PI / ELLIPSE_VERTICES as f64;

    for i in 0..=ELLIPSE_VERTICES {
        let t = i as f64 * step;
        // Parametric ellipse, then rotate by theta.
        let ex = a * t.cos();
        let ey = b * t.sin();
        let rx = ex * theta.cos() - ey * theta.sin();
        let ry = ex * theta.sin() + ey * theta.cos();
        // Offset from center, adjusting lon by cos(lat) for display.
        let cos_lat = sample.lat.to_radians().cos().max(0.01);
        coords.push([sample.lon + rx / cos_lat, sample.lat + ry]);
    }

    TissotEllipse {
        lon: sample.lon,
        lat: sample.lat,
        semimajor: sample.semimajor,
        semiminor: sample.semiminor,
        angle_deg: sample.theta.to_degrees(),
        area_distortion_pct: sample.area_distortion_pct,
        angular_distortion_deg: sample.angular_distortion_deg,
        coordinates: coords,
    }
}

impl TissotEllipse {
    /// Convert to GeoJSON Feature.
    pub fn to_geojson_feature(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Polygon",
                "coordinates": [self.coordinates]
            },
            "properties": {
                "area_distortion_pct": self.area_distortion_pct,
                "angular_distortion_deg": self.angular_distortion_deg,
                "semimajor": self.semimajor,
                "semiminor": self.semiminor,
                "angle_deg": self.angle_deg
            }
        })
    }
}

/// Convert all ellipses to a GeoJSON FeatureCollection.
pub fn to_geojson_collection(ellipses: &[TissotEllipse]) -> serde_json::Value {
    let features: Vec<serde_json::Value> =
        ellipses.iter().map(|e| e.to_geojson_feature()).collect();
    serde_json::json!({
        "type": "FeatureCollection",
        "features": features
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DistortionSample {
        DistortionSample {
            lon: -84.5,
            lat: 38.0,
            area_scale_factor: 1.10,
            area_distortion_pct: 10.0,
            angular_distortion_deg: 5.0,
            semimajor: 1.05,
            semiminor: 0.95,
            theta: 0.0,
        }
    }

    #[test]
    fn ellipse_has_correct_vertex_count() {
        let ellipse = generate_one(&sample());
        // 72 vertices + closing point = 73
        assert_eq!(ellipse.coordinates.len(), ELLIPSE_VERTICES + 1);
    }

    #[test]
    fn ellipse_is_closed() {
        let ellipse = generate_one(&sample());
        let first = ellipse.coordinates.first().unwrap();
        let last = ellipse.coordinates.last().unwrap();
        assert!((first[0] - last[0]).abs() < 1e-10);
        assert!((first[1] - last[1]).abs() < 1e-10);
    }

    #[test]
    fn circle_when_no_distortion() {
        let s = DistortionSample {
            lon: 0.0,
            lat: 0.0,
            area_scale_factor: 1.0,
            area_distortion_pct: 0.0,
            angular_distortion_deg: 0.0,
            semimajor: 1.0,
            semiminor: 1.0,
            theta: 0.0,
        };
        let ellipse = generate_one(&s);
        // For a=b (circle), all points should be equidistant from center.
        let distances: Vec<f64> = ellipse
            .coordinates
            .iter()
            .take(ELLIPSE_VERTICES)
            .map(|c| {
                let dx = c[0] - s.lon;
                let dy = c[1] - s.lat;
                (dx * dx + dy * dy).sqrt()
            })
            .collect();
        let max_d = distances.iter().cloned().fold(0.0f64, f64::max);
        let min_d = distances.iter().cloned().fold(f64::MAX, f64::min);
        // Circle should have near-equal radius at all points.
        assert!((max_d - min_d) < 0.01, "max={max_d}, min={min_d}");
    }

    #[test]
    fn geojson_feature_valid() {
        let ellipse = generate_one(&sample());
        let feature = ellipse.to_geojson_feature();
        assert_eq!(feature["type"], "Feature");
        assert_eq!(feature["geometry"]["type"], "Polygon");
    }

    #[test]
    fn geojson_collection_valid() {
        let samples = vec![sample()];
        let ellipses = generate(&samples);
        let collection = to_geojson_collection(&ellipses);
        assert_eq!(collection["type"], "FeatureCollection");
    }
}
