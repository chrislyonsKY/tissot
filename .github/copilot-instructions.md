# Copilot Instructions — Tissot

Tissot is a visual-first geospatial diagnostics engine written in Rust with Python bindings (PyO3).

## Project Context

- **Language**: Rust (2024 edition, 1.83+) with Python bindings via PyO3/maturin
- **Purpose**: Projection distortion analysis, cartographic linting, spatial diffing, data quality checks
- **Key crates**: geo, proj, geozero, gdal, clap, axum, askama, serde, thiserror, anyhow (CLI only)
- **Philosophy**: Visual-first output (browser maps), zero-config to start, autofix capability

## Code Conventions

- Library code: use `thiserror` for errors, propagate with `?`, never `unwrap()` or `expect()`
- CLI binary: may use `anyhow` for error handling
- Logging: use `log` crate, never `println!` in library code
- Geometry: always use `geo` crate types (Point, LineString, Polygon)
- CRS: always use `proj` crate for coordinate transforms
- Serialization: `serde` with derive macros
- Testing: every module has `#[cfg(test)] mod tests`
- Formatting: `cargo fmt`, clippy with `-D warnings`

## Architecture

Diagnostic rules implement the `Rule` trait (see src/core/rule.rs). The trait requires:
- `id()`, `name()`, `domain()`, `default_severity()`
- `check(&self, ctx: &CheckContext) -> Vec<Finding>`
- Optional: `can_fix()` and `fix()` for autofix support
- Optional: `score_weight()` for quality score calculation

Six subsystems: X-Ray engine, Checker engine, Fix engine, Score engine, Visual report server, IO layer.

## What NOT To Do

- Don't use raw coordinate tuples — use geo crate types
- Don't hardcode CRS/EPSG codes — use config or auto-detection
- Don't add CDN dependencies to HTML reports — everything must work offline
- Don't put computation logic in Python bindings — Rust only
- Don't add dependencies without justification
