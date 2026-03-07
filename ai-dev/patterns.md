# Tissot Patterns & Anti-Patterns

> Code patterns that work well in this project, and mistakes to avoid.

## Patterns

### Rule Registration

All rules self-register via the `inventory` crate at compile time. No central file needs updating when a new rule is added.

```rust
// In src/checkers/data_quality/topology_gaps.rs
use inventory;
use crate::core::Rule;

pub struct TopologyGaps { /* ... */ }
impl Rule for TopologyGaps { /* ... */ }

inventory::submit! {
    Box::new(TopologyGaps::default()) as Box<dyn Rule>
}
```

### Finding with Geometry

Always attach geometry to findings when possible. This is what makes the visual reports work.

```rust
// ✅ Finding with location for map rendering
Finding {
    rule_id: "data/topology-gaps".into(),
    severity: Severity::Warning,
    message: format!("Gap of {:.1} m² between features {} and {}", area, id_a, id_b),
    location: Some(SpatialLocation::BoundingBox(gap_bbox)),
    geometry: Some(gap_polygon.into()),  // This renders on the map!
    metric: Some(area),
    suggestion: Some("Run `tissot fix --topology` to auto-heal".into()),
    fixable: true,
}
```

### Config Cascade

CLI flags override env vars override `.tissot.yml` override defaults. Implemented via the `config` module's merge strategy.

```rust
let config = Config::default()
    .merge_file(".tissot.yml")?    // optional, may not exist
    .merge_env("TISSOT_")?         // TISSOT_SAMPLE_SIZE=1000
    .merge_cli(&cli_args);         // --sample-size 1000
```

### Visual Report Data Flow

Reports receive a `ReportData` struct, not raw findings. This struct is pre-processed for map rendering.

```rust
pub struct ReportData {
    pub findings: Vec<Finding>,
    pub features_geojson: String,     // User's data as GeoJSON for MapLibre
    pub findings_geojson: String,     // Findings as GeoJSON layer
    pub heatmap_geojson: Option<String>,  // For X-Ray
    pub ellipses_geojson: Option<String>, // For X-Ray
    pub metadata: ReportMetadata,
}
```

## Anti-Patterns

### Swallowing Errors

```rust
// ❌ WRONG — silent failure
if let Ok(layer) = read_layer(path) {
    process(layer);
}

// ✅ CORRECT — propagate or log
let layer = read_layer(path)?;
process(layer);
```

### God Functions

```rust
// ❌ WRONG — one function doing everything
fn analyze_projection(path: &str) -> Report {
    // 200 lines of reading, computing, formatting...
}

// ✅ CORRECT — composed pipeline
fn analyze_projection(path: &str, config: &XrayConfig) -> Result<XrayReport> {
    let layers = io::read(path)?;
    let samples = xray::sample_points(&layers, config.sample_size)?;
    let distortion = xray::compute_distortion(&samples, &layers[0].crs)?;
    let heatmap = xray::generate_heatmap(&distortion, config.heatmap_resolution)?;
    let recommendations = xray::recommend_crs(&layers, &distortion, config)?;
    Ok(XrayReport { distortion, heatmap, recommendations })
}
```

### Geometry Without CRS

```rust
// ❌ WRONG — geometry floating without CRS context
fn compute_area(polygon: &Polygon) -> f64 {
    polygon.unsigned_area()  // In what units? What CRS?
}

// ✅ CORRECT — CRS-aware
fn compute_area(polygon: &Polygon, crs: &CrsInfo) -> Result<f64> {
    if crs.is_geographic() {
        // Use geodesic area calculation
        Ok(polygon.geodesic_area_unsigned())
    } else {
        // Projected CRS — Cartesian area is meaningful
        Ok(polygon.unsigned_area())
    }
}
```
