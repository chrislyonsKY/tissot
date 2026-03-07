# DL-004: Geozero-First IO with Optional GDAL

**Date:** 2026-03-07
**Status:** Accepted
**Author:** Chris Lyons

## Context

The original architecture listed GDAL as the primary IO driver for all formats. However, GDAL introduces significant build complexity: it's a massive C++ library with dozens of transitive dependencies, platform-specific build configurations, and binary bloat that makes WebAssembly compilation effectively impossible. The deep-research paper on next-gen GIS architecture makes a compelling case for bypassing GDAL entirely in favor of pure-Rust IO crates from the GeoRust ecosystem.

## Decision

Adopt a **geozero-first** IO strategy. Use pure-Rust crates as the primary path for all supported formats. GDAL is an optional feature flag (`--features gdal`) for formats that lack mature pure-Rust readers.

### Format Priority

| Format | Primary Reader | Fallback (GDAL) | Notes |
|---|---|---|---|
| GeoJSON | `geozero` + `serde_json` | — | Pure Rust, zero-copy capable |
| GeoPackage | `geozero` (via `geozero-gpkg`) | `gdal` crate | geozero's GPKG support is read-only; GDAL needed for write in fix engine |
| Shapefile | `shapefile` crate | `gdal` crate | Pure Rust reader exists and is mature |
| FlatGeobuf | `flatgeobuf` crate | — | Pure Rust, excellent streaming support |
| GeoParquet | `geoarrow-rs` / `parquet` crate | — | Pure Rust, cloud-native format |
| QGIS Project (.qgz) | Custom XML parser + `zip` crate | — | No GDAL involvement |
| ArcGIS Project (.aprx) | Custom XML parser + `zip` crate | — | No GDAL involvement |

### Cargo Feature Flags

```toml
[features]
default = []
gdal = ["dep:gdal"]        # Enables GDAL-backed readers for additional formats
full = ["gdal"]             # All optional features
```

## Alternatives Considered

- **GDAL-only IO** — Rejected: prevents Wasm compilation, adds ~50MB to binary size, creates platform-specific build nightmares (especially Windows), and introduces a C++ FFI boundary that complicates error handling.
- **Pure geozero only, no GDAL at all** — Considered but too restrictive for Phase 1. GeoPackage write support (needed for `tissot fix`) isn't available in pure Rust yet. GDAL as an optional fallback covers this gap without penalizing the default build.

## Consequences

- **Positive**: The default `cargo install tissot` produces a self-contained binary with zero C/C++ dependencies. Dramatically simpler installation.
- **Positive**: The core library compiles to WebAssembly, enabling browser-based execution of the X-Ray engine without a local install.
- **Positive**: FlatGeobuf and GeoParquet support come nearly for free and position Tissot for cloud-native workflows.
- **Negative**: Some GDAL-only formats (FileGDB, KML, WFS) are unavailable without the feature flag. Acceptable for MVP — these are niche formats for Tissot's use case.
- **Trade-off**: `tissot fix --reproject` writing to GeoPackage requires GDAL in Phase 1. In Phase 2, evaluate writing via `rusqlite` + manual GPKG schema construction to eliminate this last GDAL dependency.

## References

- geozero: https://github.com/georust/geozero
- shapefile crate: https://crates.io/crates/shapefile
- flatgeobuf: https://github.com/flatgeobuf/flatgeobuf
- geoarrow-rs: https://github.com/geoarrow/geoarrow-rs
- Deep-research paper: "Architecting the Next-Generation Decentralized Spatial Engine" (2026), pages 4-5, on bypassing GDAL for pure-Rust IO
