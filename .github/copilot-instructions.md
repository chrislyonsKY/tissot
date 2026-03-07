# Copilot Instructions — Tissot

Tissot is a visual-first geospatial diagnostics engine written in Rust with Python bindings (PyO3).
See your distortion. Named after Tissot's indicatrix.

## Project Context

- **Language**: Rust (2024 edition, 1.83+) with Python bindings via PyO3/maturin
- **Purpose**: Projection distortion analysis, cartographic linting, spatial diffing, data quality checks, cloud-native format validation
- **Philosophy**: Visual-first output (browser maps default), zero-config to start, autofix capability
- **License**: MIT OR Apache-2.0
- **Repo**: https://github.com/chrislyonsKY/tissot

## Architecture — Six Engines + Supporting Modules

### Core (`src/core/`)
- `types.rs` — Domain, Severity, Finding, CheckContext, Layer, Feature, CrsInfo, Config, BoundingBox, Schema
- `rule.rs` — The `Rule` trait (central abstraction). All checkers implement this. Requires Send + Sync for Rayon parallelism.
- `registry.rs` — Collects, stores, and filters rules by domain/tags/config
- `config.rs` — Loads .tissot.yml, merges with env vars and CLI flags. Zero-config default.

### X-Ray Engine (`src/xray/`) — HERO FEATURE
- `distortion.rs` — Jacobian-based Tissot parameter computation (semimajor, semiminor, area scale, angular distortion)
- `heatmap.rs` — IDW interpolation of distortion values into a continuous grid
- `ellipse.rs` — Generates Tissot ellipse polygons as GeoJSON-serializable geo::Polygon
- `recommend.rs` — CRS recommendation engine: evaluates candidates against actual data distortion
- `sampling.rs` — Stratified grid sampling for large datasets (≤1K: all, ≤50K: 500, >50K: 1000)

### Checkers (`src/checkers/`)
Five diagnostic domains, each containing rules implementing the Rule trait:

**data_quality/** — Structural integrity
- `null_geometry.rs` — "data/null-geometry" (Error)
- `duplicate_geometry.rs` — "data/duplicate-geometry" (Warning)
- `schema_validation.rs` — "data/schema-validate" (Error)
- `extent_bounds.rs` — "data/extent-bounds" (Warning)
- `topology_gaps.rs` — "data/topology-gaps" (Warning, fixable)
- `topology_overlaps.rs` — "data/topology-overlaps" (Warning, fixable)
- `self_intersection.rs` — "data/self-intersection" (Error)

**projection/** — CRS suitability
- `area_distortion.rs` — "proj/area-distortion" (Warning >5%, Error >10%, fixable)
- `distance_distortion.rs` — "proj/distance-distortion" (Warning)
- `datum_mismatch.rs` — "proj/datum-mismatch" (Error)

**cloud/** — Cloud-native format validation (aligned with CNG Formats Guide)
- `format_recommendation.rs` — "cloud/format-recommendation" (Info)
- `crs_metadata.rs` — "cloud/crs-metadata" (Error)
- `multi_file_integrity.rs` — "cloud/multi-file-integrity" (Error)
- `spatial_index.rs` — "cloud/spatial-index" (Warning) [Phase 2]
- `compression.rs` — "cloud/compression" (Info) [Phase 2]
- `file_size.rs` — "cloud/file-size" (Warning) [Phase 2]

**cartography/** — Visual/perceptual map quality [Phase 3]
**diff/** — Change detection between versions [Phase 2]

### Score Engine (`src/score/`)
- `calculator.rs` — Weighted average: Projection 0.25, DataQuality 0.30, Accessibility 0.20, CloudReadiness 0.20, Classification 0.05. Per-category: 100 minus penalties (Error: -15, Warning: -5, Info: -1). Letter grade A-F.
- `categories.rs` — ScoreCategory struct with name, weight, severity counts, computed score
- `badge.rs` — SVG badge generation: "Tissot Score: 87/100 — B" with color coding

### Profile (`src/profile/`)
- `summary.rs` — ProfileSummary: file path, format, size, layer count, per-layer stats (features, geometry type, CRS, extent, fields, null count)
- `format_info.rs` — Format detection from extension, cloud-optimized flag, CNG guide URL

### Explain (`src/explain/`)
- `crs_database.rs` — Curated EPSG lookup table (20+ entries: 4326, 3857, 3089, 2205, UTM zones, Albers, Lambert, etc.) with preservation properties and plain-English descriptions
- `properties.rs` — explain_crs() returns CrsExplanation with projection family, preservation properties, warnings, recommended use

### Fix Engine (`src/fix/`) [Phase 2]
- `reproject.rs`, `topology.rs`, `symbology.rs`, `schema.rs`

### IO Layer (`src/io/`) — Geozero-first (DL-004)
- Primary: geozero + shapefile + flatgeobuf crates (pure Rust, Wasm-compatible)
- Optional: gdal crate behind `--features gdal` flag
- `wasm.rs` — Byte-array IO for browser target (DL-005)

### Report (`src/report/`)
- `visual/server.rs` — Local axum server, opens browser automatically
- `visual/xray_map.rs`, `findings_map.rs`, `score_dashboard.rs`, `diff_slider.rs`, `watch_dashboard.rs`, `combined_report.rs`, `profile_card.rs`, `benchmark_card.rs`
- `terminal.rs` — Rich terminal output (secondary to visual)
- `json.rs` — Machine-readable JSON
- `sarif.rs` — SARIF for CI/CD (Phase 2)

## CLI Commands (Phase 1)

```
tissot xray <file>              # Projection distortion → browser map
tissot check <file>             # Diagnostic linting → browser findings map
tissot check <file> --domain cloud  # Cloud optimization rules only
tissot score <file>             # Quality score → browser dashboard
tissot profile <file>           # Dataset summary → terminal
tissot explain <epsg|file>      # CRS reference → terminal
tissot --terminal               # Any command: suppress browser, terminal only
tissot --json                   # Any command: machine-readable JSON output
```

## Key Crates

| Crate | Purpose |
|-------|---------|
| geo, geo-types | Geometry primitives and algorithms |
| proj | CRS transforms (bundled_proj feature) |
| geozero | Zero-copy format IO |
| shapefile | Pure Rust .shp reader |
| flatgeobuf | Pure Rust .fgb reader |
| gdal | Optional GDAL fallback |
| clap 4 | CLI with derive API |
| serde, serde_json | Serialization |
| serde_yaml | Config file parsing |
| thiserror | Library error types |
| anyhow | CLI error handling (main.rs only) |
| log, env_logger | Logging (never println! in library) |
| rstar | R-tree spatial indexing |
| axum | Local web server for visual reports |
| askama | HTML template engine |
| wasm-bindgen | Wasm↔JS bridge (DL-005) |
| rayon | Parallel rule execution |

## Code Conventions — ALWAYS FOLLOW

### Rust
- Edition 2024, minimum Rust 1.83
- `thiserror` for library errors, `anyhow` for CLI binary ONLY
- Propagate errors with `?` — NEVER `unwrap()` or `expect()` in library code
- `log` crate for output — NEVER `println!` in library code
- All geometry via `geo` crate types — NEVER raw coordinate tuples
- All CRS operations via `proj` crate — NEVER hand-rolled transforms
- `serde` with `#[derive(Serialize, Deserialize)]` on all public data structs
- All checker rules implement the `Rule` trait — no standalone diagnostic functions
- Every module has `#[cfg(test)] mod tests`
- `cargo fmt` and `cargo clippy -- -D warnings` must pass

### Findings
- ALWAYS include `geometry: Some(...)` when the finding has a spatial location
- ALWAYS include a `suggestion` with an actionable fix recommendation
- Set `fixable: true` only when `tissot fix` can resolve the issue
- Link to CNG Formats Guide URLs in cloud rule suggestions

### Visual Reports
- Browser is the PRIMARY output — `--terminal` is the opt-out
- MapLibre GL JS for all map rendering
- Self-contained HTML — NO CDN dependencies, must work offline
- Dark theme default

### Python Bindings
- Thin wrapper only — ALL computation stays in Rust
- Accept/return WKT, WKB, GeoJSON strings for geometry interop
- Maintain .pyi type stubs for every public function

## What NOT To Do

- Don't make terminal the default output — visual map is always default
- Don't require config for first run — zero-config must work
- Don't use `unwrap()` or `expect()` in library code
- Don't put computation logic in Python bindings
- Don't hardcode CRS/EPSG codes — use config or auto-detection
- Don't use CDN-hosted assets in visual reports
- Don't assume WGS 84 — always read CRS from data source
- Don't add dependencies without justification
- Don't generate boring reports — every visual output should make someone want to screenshot it