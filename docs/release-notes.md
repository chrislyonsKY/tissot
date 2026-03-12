# Release Notes

## 0.2.0 (2026-03-12)

### Added

- **Cloud-native checker domain** with 6 rules: format recommendation, CRS metadata, multi-file integrity, spatial index, compression, file size
- **Cartography checker domain** with color contrast, label density, and classification rules
- **GeoParquet reader** for cloud-native format support (pure Rust)
- **PyO3 direct bindings** — `tissot.xray()`, `tissot.check()`, `tissot.score()` callable directly from Python
- **Documentation site** powered by Material for MkDocs with tutorials, CLI reference, and API docs
- **Real-world examples** — Jupyter notebooks, Python scripts, and sample datasets
- **GitHub Pages** deployment at chrislyonsky.github.io/tissot
- Comprehensive integration tests with real geodata fixtures
- SVG badge generation for README embedding
- SARIF output for GitHub Code Scanning integration

### Changed

- Upgraded project structure to match mature Python/Rust geospatial project standards
- Upgraded pyproject.toml with full metadata, URLs, and classifiers
- Enhanced CI/CD with docs deployment, cross-platform testing, and coverage
- QGIS Processing Provider updated to v0.2.0

### Fixed

- Score engine category weights now sum correctly
- FlatGeobuf reader handles empty feature tables

---

## 0.1.0-alpha (2026-03-07)

### Added

- Core rule engine, diagnostics model, and registry plumbing
- GeoJSON, Shapefile, and FlatGeobuf readers with format detection
- Projection checks and data-quality checks (missing CRS, null geometry, duplicates, empty datasets)
- X-Ray distortion analysis, heatmap helpers, ellipse generation, and CRS recommendations
- Score engine with category weighting and badge generation
- Terminal, JSON, SARIF, and visual report pathways
- Fix engine primitives for reprojection and topology cleanup
- CI workflow with format, clippy, test, and release build gates
- Architecture diagram, contributor guide, issue templates, code of conduct, and example datasets

### Notes

- GeoPackage reader is currently explicit about unsupported operations in this alpha release
