<p align="center">
  <img src="assets/banner.svg" alt="Tissot — See Your Distortion" width="800"/>
</p>

<p align="center">
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust" alt="Rust"></a>
  <a href="https://www.python.org/"><img src="https://img.shields.io/badge/python-3.9--3.13-blue?logo=python&logoColor=white" alt="Python"></a>
  <a href="https://crates.io/crates/tissot"><img src="https://img.shields.io/crates/v/tissot" alt="crates.io"></a>
  <a href="https://pypi.org/project/tissot/"><img src="https://img.shields.io/pypi/v/tissot" alt="PyPI"></a>
  <a href="https://plugins.qgis.org/plugins/tissot_processing_provider/"><img src="https://plugins.qgis.org/plugins/tissot_processing_provider/badges/latest_version.svg" alt="QGIS Plugin"></a>
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

## QGIS Plugin

Available from the [QGIS Plugin Repository](https://plugins.qgis.org/plugins/tissot_processing_provider/).

**Step 1** — Install the `tissot` Python package into the same Python runtime used by QGIS:

```bash
# macOS
"/Applications/QGIS.app/Contents/MacOS/python" -m pip install tissot

# Windows (OSGeo4W Shell)
python -m pip install tissot

# Linux
python3 -m pip install tissot
```

**Step 2** — In QGIS: `Plugins` → `Manage and Install Plugins...` → search for **Tissot Processing Provider** → `Install`.

The plugin adds four Processing algorithms (available in the toolbox and model builder):
- **Projection X-Ray** — per-feature distortion analysis
- **Data Quality Check** — linting with spatial findings
- **Map Quality Score** — 0-100 Lighthouse-style rating
- **Spatial Diff** — categorized change layer between two datasets

## Python Integration

PyO3 bindings are in progress. Today, Python environments install the `tissot`
CLI, which can be called from Python via `subprocess`.

```python
import json
import subprocess

result = subprocess.run(
  ["tissot", "check", "data.gpkg", "--json"],
  check=True,
  capture_output=True,
  text=True,
)
report = json.loads(result.stdout)
print(report["summary"]["total"])
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

🚧 **In Development** — Building toward first release.

### What's Implemented

**Projection X-Ray** (`tissot xray`) — Jacobian-based per-feature distortion analysis, distortion heatmap generation (IDW interpolation), Tissot ellipse rendering as GeoJSON polygons, CRS recommendation engine with UTM/state-plane/continental candidate ranking, stratified sampling for large datasets.

**Checker Engine** — 20 diagnostic rules across three domains:

| Domain | Rules | Examples |
|--------|-------|---------|
| Data Quality (9) | null geometry, duplicate features/geometry, self-intersection, topology gaps & overlaps, schema validation, extent bounds, empty dataset | `data/null-geometry`, `data/topology-gaps` |
| Projection (5) | area distortion, distance distortion, datum mismatch, high distortion, missing CRS | `proj/area-distortion`, `proj/datum-mismatch` |
| Cloud (6) | format recommendation, CRS metadata, multi-file integrity, spatial index, compression, file size | `cloud/format-recommendation`, `cloud/crs-metadata` |

**Score Engine** (`tissot score`) — Weighted 0-100 quality score with category breakdown (Projection 0.25, Data Integrity 0.30, Accessibility 0.20, Cloud Readiness 0.20, Classification 0.05). Letter grades A-F. SVG badge generation.

**Profile & Explain** — Dataset summary (format, layers, CRS, extents, field counts) and curated EPSG reference database with plain-English CRS explanations.

**IO Layer** — Pure Rust readers for GeoJSON, Shapefile, FlatGeobuf via geozero. Optional GDAL fallback behind feature flag.

**Report Outputs** — Terminal, JSON, SARIF (for CI/CD), and visual HTML report scaffolding.

**CLI** — All commands wired: `xray`, `check`, `score`, `profile`, `explain`, `fix`, `diff`, `watch`, `init`.

### What's Next

- Visual report server (interactive MapLibre browser maps)
- Fix engine implementation (reproject, topology healing)
- Diff engine (spatial change detection with slider)
- Watch mode (live directory monitoring)
- Python bindings via PyO3

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Contributing

Tissot uses an AI-assisted development workflow. See `CLAUDE.md` for project context and `ai-dev/` for architecture docs, agent configurations, and coding standards.

The easiest way to contribute is to **add a new checker rule**: implement the `Rule` trait in a new file under `src/checkers/`. See existing rules for the pattern.
# tissot
