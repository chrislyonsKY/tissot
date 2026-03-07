# Tissot Architecture

> System design for a visual-first geospatial diagnostics engine.

## Design Philosophy

1. **Visual-first** — The primary output is an interactive map in the browser. Every diagnostic has a spatial location. Terminal text is for CI/CD and scripting. Humans get maps.
2. **Zero-config to start, deep config to tune** — Running `tissot xray data.gpkg` requires no setup. `.tissot.yml` exists for power users who want to customize rules, thresholds, and output.
3. **Show, then tell, then fix** — X-Ray shows distortion. Check tells you what's wrong. Fix resolves it. Score summarizes it. This is the user journey.
4. **Performance is a feature** — Rust core with Rayon parallelism. X-Ray on a 500MB GeoPackage in seconds.
5. **Offline-capable** — All visual reports are self-contained HTML. No CDN, no internet required.

## System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                              CLI                                  │
│  tissot xray | tissot check | tissot diff | tissot fix | tissot  │
│  score | tissot watch                                             │
└───────────────────────────────┬──────────────────────────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                  │
     ┌────────▼───────┐ ┌──────▼──────┐ ┌────────▼────────┐
     │   IO Layer      │ │  Engines    │ │ Visual Server   │
     │                 │ │             │ │                  │
     │ Reads:          │ │ • X-Ray     │ │ axum server      │
     │ .gpkg .geojson  │ │ • Checker   │ │ MapLibre GL JS   │
     │ .shp .qgz .aprx│ │ • Fix       │ │ Self-contained   │
     │                 │ │ • Score     │ │ HTML reports     │
     │ Writes (fix):   │ │ • Diff      │ │                  │
     │ .gpkg .geojson  │ │             │ │ Endpoints:       │
     └─────────────────┘ └─────────────┘ │ /xray            │
                                          │ /findings        │
                                          │ /diff            │
                                          │ /score           │
                                          │ /watch (SSE)     │
                                          └──────────────────┘
```

## X-Ray Engine (`src/xray/`)

The hero feature. Computes projection distortion on the user's actual data and renders it visually.

### Pipeline

```
Input file → Read features + CRS
    │
    ▼
Sample feature centroids (or all, if < threshold)
    │
    ▼
For each sample point:
  1. Compute Jacobian of projection at that point
  2. Derive Tissot parameters: semimajor (a), semiminor (b), angle (θ)
  3. Compute area scale factor: h = a × b
  4. Compute angular distortion: ω = 2 × arcsin((a-b)/(a+b))
  5. Compute area error vs equal-area reference
  6. Compute distance error vs geodesic reference
    │
    ▼
Generate distortion heatmap (interpolated surface from sample points)
    │
    ▼
Generate Tissot ellipses at sample locations (scaled to actual distortion)
    │
    ▼
Evaluate candidate CRS list against same metrics
    │
    ▼
Rank candidates by optimization target (area | distance | shape | balanced)
    │
    ▼
Render interactive MapLibre report:
  - Base layer: user's features
  - Overlay: distortion heatmap (red = high, green = low)
  - Overlay: Tissot ellipses at sample points
  - Panel: metrics table (max/mean/median error)
  - Panel: CRS recommendation ranked list
  - Toggle: switch between current CRS and recommended CRS
```

### Key Design Decisions

- **Sample-based for large datasets**: For datasets > 5,000 features, sample centroids spatially (using stratified grid sampling) to keep computation under 5 seconds.
- **Ellipse rendering**: Tissot ellipses are generated as GeoJSON polygons (72-point approximation) and rendered as a MapLibre layer. They are scaled relative to the local distortion — a circle means no distortion, an ellipse shows stretch direction.
- **CRS candidate generation**: Based on dataset geographic extent and centroid. Uses a curated database of CRS options per region (US State Plane, UTM, continental equal-area, etc.).
- **Comparison mode**: `tissot xray data.gpkg --compare 3089` renders both CRS side by side with a sync'd map view.

## Checker Engine (`src/checkers/`)

Rule-based diagnostics. Each rule implements the `Rule` trait.

### Rule Trait

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn domain(&self) -> Domain;
    fn default_severity(&self) -> Severity;
    fn check(&self, ctx: &CheckContext) -> Vec<Finding>;
    fn tags(&self) -> &[&str] { &[] }

    /// Can this rule autofix its findings?
    fn can_fix(&self) -> bool { false }

    /// Apply autofix for this rule's findings
    fn fix(&self, _ctx: &mut FixContext) -> Result<Vec<FixAction>> {
        Err(Error::NoAutofix(self.id().to_string()))
    }

    /// Weight for score calculation (0.0 to 1.0)
    fn score_weight(&self) -> f64 { 1.0 }
}
```

### Finding Struct

```rust
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: Option<SpatialLocation>,
    pub geometry: Option<geo::Geometry>,  // For rendering on map
    pub metric: Option<f64>,
    pub suggestion: Option<String>,
    pub fixable: bool,                    // Can tissot fix this?
}
```

Every finding optionally carries a `geometry` field so the visual report can place it on the map.

### Domains

| Domain | Rules cover | Phase |
|---|---|---|
| `projection` | CRS distortion, datum mismatch, EPSG validity | 1 |
| `data_quality` | Topology, schema, null geometry, duplicates, extent | 1 |
| `cartography` | Color contrast, label overlap, classification, color-only | 2 |
| `diff` | Geometry changes, feature add/remove, attribute changes | 2 |

## Fix Engine (`src/fix/`)

Autofix builds on the checker engine. Rules that support `can_fix() = true` implement the `fix()` method.

### Fix Actions

```rust
pub enum FixAction {
    Reproject { from: CrsInfo, to: CrsInfo, output_path: PathBuf },
    HealTopologyGap { feature_ids: Vec<String>, method: HealMethod },
    HealTopologyOverlap { feature_ids: Vec<String>, method: HealMethod },
    RewriteSymbology { layer: String, changes: Vec<SymbologyChange> },
    NormalizeSchema { changes: Vec<SchemaChange> },
    RemoveDuplicates { feature_ids: Vec<String> },
}

pub enum HealMethod {
    SnapToNeighbor { tolerance: f64 },
    InsertFillPolygon,
    AdjustBoundary,
}
```

### Fix Workflow

```
tissot fix data.gpkg --reproject
    │
    ▼
Run projection checker → collect findings
    │
    ▼
Generate CRS recommendation (X-Ray engine)
    │
    ▼
Preview: "Will reproject 12,847 features from EPSG:3857 → EPSG:3089"
    │
    ▼
User confirms (--yes to skip)
    │
    ▼
Execute reproject → write new file (data_fixed.gpkg)
    │
    ▼
Re-run checker on output → verify fix resolved findings
```

Fix always writes to a NEW file (never modifies in-place) unless `--in-place` is explicitly passed.

## Score Engine (`src/score/`)

Aggregates checker findings into a Lighthouse-style quality score.

### Scoring Algorithm

```
Overall Score = weighted average of category scores

Categories:
  - Projection Quality (weight: 0.25)
    Based on: max area distortion, distance distortion, CRS appropriateness
  - Data Integrity (weight: 0.30)
    Based on: topology errors, null geometries, duplicates, schema validity
  - Accessibility (weight: 0.25)
    Based on: color contrast, color-only encoding, label readability
  - Classification Quality (weight: 0.20)
    Based on: GVF, class balance, appropriate number of classes

Per-category score:
  Start at 100
  Subtract points per finding based on severity:
    Error: -15 points each (capped at -60)
    Warning: -5 points each (capped at -30)
    Info: -1 point each (capped at -10)
  Floor at 0

Letter grade:
  A: 90-100  |  B: 75-89  |  C: 60-74  |  D: 40-59  |  F: 0-39
```

### Badge Generation

`tissot score data.gpkg --badge badge.svg` generates a shields.io-style SVG badge:

```
[Tissot Score: 87/100 — B]  (green)
[Tissot Score: 42/100 — D]  (orange)
[Tissot Score: 18/100 — F]  (red)
```

For embedding in README files, PRs, and metadata.

## Visual Report Server (`src/report/visual/`)

A lightweight local web server (axum) that serves interactive reports.

### Design

- Spins up on a random available port (e.g., `http://localhost:48721`)
- Opens default browser automatically
- Serves self-contained HTML with embedded MapLibre GL JS
- Shuts down on Ctrl+C (or after configurable timeout)
- For watch mode: uses Server-Sent Events (SSE) to push updates to the browser

### Report Types

| Endpoint | Command | Content |
|---|---|---|
| `/xray` | `tissot xray` | Distortion heatmap + ellipses + recommendations |
| `/findings` | `tissot check` | All findings plotted on data map |
| `/diff` | `tissot diff` | Before/after slider with change highlights |
| `/score` | `tissot score` | Score dashboard with category breakdown + gauges |
| `/watch` | `tissot watch` | Live dashboard with SSE-streamed finding updates |

### MapLibre Integration

All maps use MapLibre GL JS (open-source fork of Mapbox GL JS). Features:
- WebGL rendering for smooth interaction with large datasets
- GeoJSON source layers for findings, distortion, ellipses
- Custom layers for heatmap (distortion) and line (ellipse) rendering
- Popup on click showing finding details
- Layer toggle controls

### Offline Requirement

All HTML reports bundle MapLibre JS + CSS inline. Map tiles for basemaps use a fallback strategy:
1. Try loading tiles from configured tile server
2. If offline, render features-only on a blank canvas with graticule grid
3. Optionally accept a local MBTiles file as basemap source

## IO Layer (`src/io/`)

### Geozero-First Strategy (DL-004)

The IO layer uses **pure-Rust crates as the primary path**. GDAL is an optional feature flag (`--features gdal`) for formats that lack mature pure-Rust readers or for write operations. This enables Wasm compilation of the core library (DL-005).

### Format Support

| Phase | Format | Primary Reader (pure Rust) | Fallback (GDAL feature) | Writer |
|---|---|---|---|---|
| 1 | GeoJSON | `geozero` + `serde_json` | — | `serde_json` |
| 1 | GeoPackage (.gpkg) | `geozero` (via `geozero-gpkg`) | `gdal` crate | `gdal` (feature-gated) |
| 1 | Shapefile (.shp) | `shapefile` crate | `gdal` crate | — |
| 1 | FlatGeobuf (.fgb) | `flatgeobuf` crate | — | `flatgeobuf` |
| 2 | GeoParquet | `geoarrow-rs` / `parquet` crate | — | `parquet` |
| 2 | QGIS Project (.qgz/.qgs) | Custom XML parser + `zip` crate | — | — |
| 3 | ArcGIS Project (.aprx/.mapx) | Custom XML parser + `zip` crate | — | — |

### Wasm IO Path

In the Wasm build, file data arrives as `&[u8]` from the browser's `FileReader` API. The `io::wasm` module wraps geozero to parse byte arrays without filesystem access:

```rust
// src/io/wasm.rs — #[cfg(target_arch = "wasm32")]
pub fn parse_bytes(data: &[u8], format: &str) -> Result<Vec<Layer>> {
    match format {
        "geojson" => geojson::read_from_bytes(data),
        "gpkg" => gpkg::read_from_bytes(data),
        "shp" => shapefile::read_from_bytes(data),
        "fgb" => flatgeobuf::read_from_bytes(data),
        _ => Err(TissotError::UnsupportedFormat(format.to_string())),
    }
}
```

### DataSource Trait

```rust
pub trait DataSource {
    fn layers(&self) -> Result<Vec<Layer>>;
    fn project_metadata(&self) -> Option<ProjectFile>;
    fn write_layer(&self, layer: &Layer, path: &Path) -> Result<()>;
}
```

## Watch Mode

```
tissot watch ./data/ [--port 8080]
    │
    ▼
Start axum server → open browser to /watch dashboard
    │
    ▼
Start filesystem watcher (notify crate)
    │
    ▼
On file change:
  1. Determine which files changed
  2. Re-run relevant checks (not all checks — only affected domains)
  3. Recompute score
  4. Push update to browser via SSE
  5. Browser updates findings map + score in real-time
```

## Performance Targets

| Operation | Dataset Size | Target Time |
|---|---|---|
| `tissot xray` | 10K features | < 2 seconds |
| `tissot xray` | 100K features | < 10 seconds |
| `tissot check` | 10K features | < 3 seconds |
| `tissot check` | 100K features | < 15 seconds |
| `tissot diff` | 10K features × 2 | < 5 seconds |
| `tissot score` | Full check + compute | < 5 seconds |
| `tissot watch` | Re-check on change | < 3 seconds |

Strategy: Rayon parallelism for independent rules, R-tree spatial indexing (rstar) for neighbor queries, stratified sampling for X-Ray on large datasets.

## WebAssembly Target (DL-005)

The core engine compiles to Wasm for browser-based execution with zero installation.

### Build Matrix

| Target | CLI | Server | File IO | Core Engine | PyO3 |
|---|---|---|---|---|---|
| Native (x86_64/ARM64) | ✅ | ✅ | ✅ | ✅ | ✅ |
| wasm32-unknown-unknown | ❌ | ❌ | Byte arrays only | ✅ | ❌ |

### Conditional Compilation

```rust
// Native-only code
#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod server;

// Wasm-only code
#[cfg(target_arch = "wasm32")]
mod wasm;

// Shared code — the core engine
mod xray;        // ✅ Wasm-safe
mod checkers;    // ✅ Wasm-safe
mod score;       // ✅ Wasm-safe
mod core;        // ✅ Wasm-safe
```

### Browser Demo

A static GitHub Pages site provides "Try Tissot" — drag-and-drop a file, get an X-Ray report in the browser. No install, no server, no signup. This is the primary adoption funnel and conference demo vehicle.

```
User drags file → FileReader API → Uint8Array → Wasm (tissot_xray_from_bytes)
  → JSON report → MapLibre GL JS renders heatmap + ellipses
```

## WebGPU Heatmap Rendering (DL-006, Phase 2)

In Phase 2, the distortion heatmap interpolation moves from CPU to GPU via WebGPU compute shaders, enabling real-time interactive visualization.

### Pipeline

```
Sample points (from Wasm X-Ray engine)
  → Upload to GPU Storage Buffer
  → Dispatch WGSL compute shader (IDW interpolation)
  → Each GPU thread computes one output cell
  → Write to output texture
  → Map texture as MapLibre custom layer
  → Re-dispatch on viewport change (pan/zoom)
```

### Fallback

WebGPU detection with graceful fallback to CPU-precomputed GeoJSON grid:

1. **WebGPU available** → real-time GPU heatmap, updates on pan/zoom
2. **WebGPU unavailable** → static pre-baked heatmap (Phase 1 behavior)

This ensures the tool works everywhere while providing a dramatically better experience on modern browsers.
