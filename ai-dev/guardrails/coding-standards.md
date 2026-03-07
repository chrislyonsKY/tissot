# Coding Standards Guardrails

These rules apply to ALL code generated for Tissot, regardless of which agent is active.
Violations are treated as Critical findings.

## Rust

- All library code must propagate errors with `?` — no `unwrap()`, no `expect()`
- All library code must use `log` crate for output — no `println!` or `eprintln!`
- CLI binary (main.rs) may use `anyhow` — library code must use `thiserror`
- All public types and functions must have `///` doc comments
- All geometry operations must use `geo` crate primitives (Point, LineString, Polygon, etc.)
- All CRS operations must go through the `proj` crate — no hand-rolled transforms
- All config/data serialization must use `serde` with `#[derive(Serialize, Deserialize)]`
- All checker rules must implement the `Rule` trait — no standalone diagnostic functions
- All modules must have `#[cfg(test)] mod tests` with at least one test
- Target: Rust 2024 edition, minimum Rust 1.83
- Clippy must pass with `-D warnings` (deny all warnings)
- Format with `cargo fmt` — no exceptions

## Python Bindings

- Python layer is a THIN wrapper — zero computation logic in Python
- All public Python functions must have corresponding `.pyi` type stubs
- Use `PyResult<T>` return types — map Rust errors to Python exceptions
- Accept/return standard types (str, dict, list) — not custom Rust types across the boundary
- For geometry interop: accept/return WKT, WKB, or GeoJSON strings

## HTML/JavaScript (Visual Reports)

- All reports must be self-contained — no CDN dependencies, no external fetches
- MapLibre GL JS must be bundled inline in report HTML
- Reports must work offline
- Reports must be responsive (work on laptop screens, not just ultrawide monitors)
- Dark theme is the default
- No frameworks (React, Vue, etc.) in reports — vanilla JS + MapLibre only

## Testing

- Unit tests for every module
- Integration tests with real sample geodata files in `tests/fixtures/`
- Test fixtures must include known-good and known-bad data
- X-Ray tests must validate distortion calculations against known analytical solutions
- Performance tests for datasets at 10K, 50K, and 100K features

## Dependencies

- Every new dependency requires a DL- decision record in `ai-dev/decisions/`
- Prefer crates with > 100K downloads and active maintenance
- Prefer crates from the GeoRust ecosystem for geospatial operations
- Minimize transitive dependency count — check with `cargo tree`
