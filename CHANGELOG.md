# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

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
