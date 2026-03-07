/// Heatmap generation via Inverse Distance Weighting (IDW) interpolation.
///
/// Takes distortion sample points and interpolates a continuous surface
/// for visualization as a color-coded overlay on the map.
use super::distortion::DistortionSample;
use serde::Serialize;

/// Interpolated heatmap grid.
#[derive(Debug, Serialize)]
pub struct HeatmapGrid {
    /// Grid bounds [min_x, min_y, max_x, max_y].
    pub bounds: [f64; 4],
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
    /// Grid cell values (area distortion %) in row-major order.
    pub values: Vec<f64>,
    /// Cell width in CRS units.
    pub cell_width: f64,
    /// Cell height in CRS units.
    pub cell_height: f64,
}

/// IDW interpolation power parameter.
const IDW_POWER: f64 = 2.0;
/// Default grid resolution.
const DEFAULT_GRID_SIZE: usize = 50;

/// Generate a heatmap grid from distortion samples using IDW interpolation.
pub fn interpolate(samples: &[DistortionSample], bounds: [f64; 4]) -> HeatmapGrid {
    let [min_x, min_y, max_x, max_y] = bounds;
    let cols = DEFAULT_GRID_SIZE;
    let rows = DEFAULT_GRID_SIZE;
    let cell_width = (max_x - min_x) / cols as f64;
    let cell_height = (max_y - min_y) / rows as f64;

    if samples.is_empty() || cell_width <= 0.0 || cell_height <= 0.0 {
        return HeatmapGrid {
            bounds,
            cols,
            rows,
            values: vec![0.0; cols * rows],
            cell_width: cell_width.max(0.001),
            cell_height: cell_height.max(0.001),
        };
    }

    let mut values = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        for col in 0..cols {
            let x = min_x + (col as f64 + 0.5) * cell_width;
            let y = min_y + (row as f64 + 0.5) * cell_height;
            let val = idw_value(x, y, samples);
            values.push(val);
        }
    }

    HeatmapGrid {
        bounds,
        cols,
        rows,
        values,
        cell_width,
        cell_height,
    }
}

/// Compute IDW-interpolated value at a single point.
fn idw_value(x: f64, y: f64, samples: &[DistortionSample]) -> f64 {
    let mut weight_sum = 0.0;
    let mut value_sum = 0.0;

    for s in samples {
        let dx = x - s.lon;
        let dy = y - s.lat;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq < 1e-12 {
            // Exactly on a sample point.
            return s.area_distortion_pct;
        }

        let weight = 1.0 / dist_sq.powf(IDW_POWER / 2.0);
        weight_sum += weight;
        value_sum += weight * s.area_distortion_pct;
    }

    if weight_sum > 0.0 {
        value_sum / weight_sum
    } else {
        0.0
    }
}

impl HeatmapGrid {
    /// Get the value at a grid cell (col, row).
    pub fn get(&self, col: usize, row: usize) -> Option<f64> {
        if col < self.cols && row < self.rows {
            Some(self.values[row * self.cols + col])
        } else {
            None
        }
    }

    /// Convert heatmap to GeoJSON FeatureCollection for MapLibre rendering.
    pub fn to_geojson(&self) -> serde_json::Value {
        let [min_x, min_y, _, _] = self.bounds;
        let mut features = Vec::new();

        for row in 0..self.rows {
            for col in 0..self.cols {
                let value = self.values[row * self.cols + col];
                let x = min_x + col as f64 * self.cell_width;
                let y = min_y + row as f64 * self.cell_height;

                features.push(serde_json::json!({
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [x, y],
                            [x + self.cell_width, y],
                            [x + self.cell_width, y + self.cell_height],
                            [x, y + self.cell_height],
                            [x, y]
                        ]]
                    },
                    "properties": {
                        "distortion_pct": value,
                        "color": distortion_color(value)
                    }
                }));
            }
        }

        serde_json::json!({
            "type": "FeatureCollection",
            "features": features
        })
    }
}

/// Map distortion percentage to a color for visualization.
/// Green (0-2%) → Yellow (2-5%) → Orange (5-10%) → Red (>10%).
fn distortion_color(pct: f64) -> String {
    if pct < 2.0 {
        "#22c55e".to_string() // green
    } else if pct < 5.0 {
        "#eab308".to_string() // yellow
    } else if pct < 10.0 {
        "#f97316".to_string() // orange
    } else {
        "#ef4444".to_string() // red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_samples() -> Vec<DistortionSample> {
        vec![
            DistortionSample {
                lon: -85.0,
                lat: 38.0,
                area_scale_factor: 1.05,
                area_distortion_pct: 5.0,
                angular_distortion_deg: 2.0,
                semimajor: 1.02,
                semiminor: 0.98,
                theta: 0.0,
            },
            DistortionSample {
                lon: -84.0,
                lat: 39.0,
                area_scale_factor: 1.10,
                area_distortion_pct: 10.0,
                angular_distortion_deg: 5.0,
                semimajor: 1.05,
                semiminor: 0.95,
                theta: 0.0,
            },
        ]
    }

    #[test]
    fn interpolate_produces_grid() {
        let samples = make_samples();
        let grid = interpolate(&samples, [-86.0, 37.0, -83.0, 40.0]);
        assert_eq!(grid.cols, 50);
        assert_eq!(grid.rows, 50);
        assert_eq!(grid.values.len(), 50 * 50);
    }

    #[test]
    fn empty_samples_returns_zero_grid() {
        let grid = interpolate(&[], [-86.0, 37.0, -83.0, 40.0]);
        assert!(grid.values.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn idw_exact_match() {
        let samples = make_samples();
        let val = idw_value(-85.0, 38.0, &samples);
        assert!((val - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn heatmap_to_geojson() {
        let samples = make_samples();
        let grid = interpolate(&samples, [-86.0, 37.0, -83.0, 40.0]);
        let geojson = grid.to_geojson();
        assert_eq!(geojson["type"], "FeatureCollection");
        let features = geojson["features"].as_array().unwrap();
        assert_eq!(features.len(), 50 * 50);
    }

    #[test]
    fn distortion_color_ranges() {
        assert_eq!(distortion_color(0.5), "#22c55e");
        assert_eq!(distortion_color(3.0), "#eab308");
        assert_eq!(distortion_color(7.0), "#f97316");
        assert_eq!(distortion_color(15.0), "#ef4444");
    }
}
