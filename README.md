<p align="center">
  <img src="assets/banner.svg" alt="Tissot — See Your Distortion" width="800"/>
</p>

<p align="center">
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.83%2B-orange?logo=rust" alt="Rust"></a>
  <a href="https://www.python.org/"><img src="https://img.shields.io/badge/python-3.9--3.13-blue?logo=python&logoColor=white" alt="Python"></a>
  <a href="https://crates.io/crates/tissot"><img src="https://img.shields.io/crates/v/tissot" alt="crates.io"></a>
  <a href="https://pypi.org/project/tissot/"><img src="https://img.shields.io/pypi/v/tissot" alt="PyPI"></a>
  <a href="https://georust.org/"><img src="https://img.shields.io/badge/GeoRust-ecosystem-green?logo=rust" alt="GeoRust"></a>
  <a href="https://cloudnativegeo.org/"><img src="https://img.shields.io/badge/Cloud_Native_Geo-member-2ea44f" alt="Cloud Native Geo"></a>
</p>

<p align="center">
  <strong>A visual-first geospatial diagnostics engine.</strong><br>
  Named after <a href="https://en.wikipedia.org/wiki/Tissot%27s_indicatrix">Tissot's indicatrix</a> — the ellipses that reveal what map projections hide.
</p>

---

## What It Does

Tissot makes spatial data problems **visible**. One command, zero config, opens an interactive map in your browser.

```bash
# See how your projection distorts your data
tissot xray parcels.gpkg

# Check data quality (topology, schema, duplicates)
tissot check parcels.gpkg

# Get a quality score (like Lighthouse, but for maps)
tissot score project.qgz

# Visual before/after diff with slider
tissot diff Q3_parcels.gpkg Q4_parcels.gpkg

# Auto-fix: reproject to optimal CRS
tissot fix parcels.gpkg --reproject

# Watch a directory for changes
tissot watch ./pipeline/output/
```

## The Hero Feature: Projection X-Ray

Every GIS professional has been told "don't use Web Mercator for area calculations." But have you ever **seen** the actual error on your actual data?

`tissot xray` computes per-feature distortion, generates a heatmap overlaid on your data, draws Tissot ellipses at sample locations, and recommends a better CRS — with quantified proof.

```
$ tissot xray kentucky_permits.gpkg

  Current CRS: EPSG:3857 (Web Mercator)
  Area distortion — Max: 18.3%  Mean: 11.7%

  Recommended: EPSG:3089 (NAD83 / Kentucky Single Zone)
  Area distortion — Max: 0.02%  Mean: 0.01%

  → Interactive report opened in browser
```

## Install

```bash
# From crates.io (Rust)
cargo install tissot

# From PyPI (Python)
pip install tissot
```

## Python API

```python
import tissot

# X-Ray analysis
report = tissot.xray("data.gpkg")
print(f"Max area error: {report.area_distortion.max_pct:.1f}%")
for rec in report.recommendations:
    print(f"  {rec.epsg}: {rec.area_error_pct:.2f}% area error")

# Data quality check
report = tissot.check("data.gpkg")
print(f"Score: {report.score}/100 ({report.grade})")

# Diff
report = tissot.diff("v1.gpkg", "v2.gpkg")
print(f"Added: {report.added}, Removed: {report.removed}, Modified: {report.modified}")
```

## Key Principles

- **Visual-first** — Browser maps are the default output, not terminal text
- **Zero-config** — Works immediately, no setup required
- **Autofix** — Don't just report problems, fix them
- **Scored** — 0-100 quality rating like Lighthouse for websites
- **Cloud-native** — Validates FlatGeobuf, GeoParquet, and cloud-optimized format best practices
- **Offline** — All reports work without internet
- **Fast** — Rust core, Rayon parallelism, sub-second for typical datasets

## Built With

Rust core using the [GeoRust](https://georust.org/) ecosystem. Python bindings via [PyO3](https://pyo3.rs). Visual reports powered by [MapLibre GL JS](https://maplibre.org/). Cloud-native format guidance aligned with the [CNG Formats Guide](https://guide.cloudnativegeo.org/).

## Status

🚧 **In Development** — Phase 1 (X-Ray + Data Quality + Cloud Optimization + Score)

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Contributing

Tissot uses an AI-assisted development workflow. See `CLAUDE.md` for project context and `ai-dev/` for architecture docs, agent configurations, and coding standards.

The easiest way to contribute is to **add a new checker rule**: implement the `Rule` trait in a new file under `src/checkers/`. See existing rules for the pattern.
# tissot
