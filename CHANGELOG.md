# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [0.2.0] - 2026-03-12

### Added

- **Cartography checker domain** with 3 rules: color contrast, label density, classification count.
- **Cloud-native checker domain** with 6 rules: format recommendation, CRS metadata, multi-file integrity, spatial index, compression, file size.
- **GeoParquet reader** for cloud-native format support.
- **PyO3 direct bindings** — `tissot.xray()`, `tissot.check()`, `tissot.score()` callable directly from Python without subprocess.
- **Documentation site** powered by Material for MkDocs with 5 tutorials, CLI reference, API reference, and architecture docs.
- **GitHub Pages** deployment at chrislyonsky.github.io/tissot.
- **Real-world examples** — 2 Jupyter notebooks, 6 Python scripts, and 6 sample datasets (US states, world cities, parcels with issues, Kentucky roads).
- Comprehensive integration tests covering IO, checker, score, and X-Ray engines.
- Cross-platform CI (Ubuntu, macOS, Windows) with Python wheel verification and docs build.
- SVG badge generation for README embedding.
- SARIF output for GitHub Code Scanning integration.
- Branding assets directory.

### Changed

- Upgraded from alpha (0.1.0) to beta (0.2.0) status.
- Upgraded pyproject.toml with full metadata, project URLs, and expanded classifiers.
- Upgraded Cargo.toml with homepage, documentation URLs.
- Enhanced CI/CD with cross-platform matrix, docs build, and Python wheel verification.
- QGIS Processing Provider updated to v0.2.0.
- Project structure now follows mature geospatial project patterns (docs/, examples/, branding/).

### Fixed

- Score engine category weights now properly validated.
- FlatGeobuf reader handles empty feature tables gracefully.

---

## [0.1.0-alpha] - 2026-03-07

### Added

- Core rule engine, diagnostics model, and registry plumbing.
- GeoJSON, Shapefile, and FlatGeobuf readers with format detection.
- Projection checks and data-quality checks (missing CRS, null geometry, duplicates, empty datasets).
- X-Ray distortion analysis, heatmap helpers, ellipse generation, and CRS recommendations.
- Score engine with category weighting and badge generation.
- Terminal, JSON, SARIF, and visual report pathways.
- Fix engine primitives for reprojection and topology cleanup.
- CI workflow with format, clippy, test, and release build gates.
- Architecture diagram, contributor guide, issue templates, code of conduct, and example datasets.

### Notes

- GeoPackage reader is currently explicit about unsupported operations in this alpha release.
