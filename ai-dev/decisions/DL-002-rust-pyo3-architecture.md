# DL-002: Rust Core with PyO3 Python Bindings

**Date:** 2026-03-07
**Status:** Accepted
**Author:** Chris Lyons

## Context

Tissot performs computationally intensive geospatial operations (distortion calculation, spatial indexing, topology analysis) and needs to be accessible from both CLI and the Python GIS ecosystem (QGIS, ArcPy, Jupyter).

## Decision

Build the core engine in Rust with Python bindings via PyO3 and maturin. The CLI binary and library are both Rust. Python is a thin wrapper only.

## Alternatives Considered

- **Pure Python** — Rejected: too slow for distortion computation on large datasets. Would need C extensions anyway for performance, at which point you're writing two languages.
- **Pure Rust (no Python)** — Rejected: cuts off the QGIS/ArcPy/Jupyter ecosystem, which is the primary integration path for GIS professionals.
- **C/C++ core with Python bindings** — Rejected: Rust provides memory safety, modern tooling (cargo, crates.io), and the GeoRust ecosystem. C++ offers no advantage here.
- **Go** — Rejected: weak geospatial ecosystem, no equivalent to GeoRust. Poor Python interop story.

## Consequences

- **Positive**: Performance-critical code (Jacobian computation, spatial indexing, topology analysis) runs at native speed. Python users get the same performance via compiled wheels.
- **Positive**: The GeoRust ecosystem (geo, proj, geozero, gdal crates) provides mature, well-tested primitives.
- **Positive**: maturin simplifies wheel building and PyPI distribution.
- **Negative**: GDAL and PROJ system dependencies complicate Windows builds. Mitigate with bundled-proj feature and pre-built wheels via CI.
- **Negative**: Rust learning curve for contributors. Mitigate with comprehensive CLAUDE.md, agent configs, and the Rule trait keeping most contributions within a well-defined interface.
- **Trade-off**: The Rule trait API is the contribution boundary. New rules are self-contained Rust files implementing a trait — approachable even for Rust beginners.

## References

- PyO3: https://github.com/PyO3/pyo3
- maturin: https://github.com/PyO3/maturin
- GeoRust ecosystem: https://georust.org/
- pyo3-geoarrow precedent: https://crates.io/crates/pyo3-geoarrow
