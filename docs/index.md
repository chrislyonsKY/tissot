# Tissot

**Visual-first geospatial diagnostics engine.**

Working with geospatial data means trusting that projections are appropriate, geometry is valid, topology is clean, and formats are cloud-ready — but verifying any of this means cobbling together `gdalinfo`, `ogrinfo`, custom Python scripts, and manual QGIS inspection, each with different outputs, none of them visual.

Tissot is one diagnostic toolkit that makes all of these problems **visible**. One CLI. Zero config. Every command opens an interactive map in your browser showing exactly what's wrong and where. Every command also produces machine-readable JSON for CI/CD pipelines.

Named after [Tissot's indicatrix](https://en.wikipedia.org/wiki/Tissot%27s_indicatrix) — the distortion ellipses that reveal what map projections hide.

---

## Install

=== "pip"

    ```bash
    pip install tissot
    ```

=== "cargo"

    ```bash
    cargo install tissot
    ```

=== "QGIS Plugin"

    Install the CLI into QGIS Python, then install the Processing Provider plugin:

    ```bash
    # macOS
    "/Applications/QGIS.app/Contents/MacOS/python" -m pip install tissot

    # Windows (OSGeo4W Shell)
    python -m pip install tissot

    # Linux
    python3 -m pip install tissot
    ```

    Then in QGIS: **Plugins > Manage and Install Plugins** > search **Tissot Processing Provider** > **Install**.

---

## Quick Start

```bash
# X-Ray: see exactly how your projection distorts your data
tissot xray kentucky_permits.gpkg --recommend

# Check: run 23 diagnostic rules across 4 domains
tissot check parcels.geojson --domain quality

# Score: get a Lighthouse-style 0-100 quality rating
tissot score parcels.geojson --badge map-score.svg

# Fix: reproject to the recommended CRS automatically
tissot fix parcels.geojson --reproject EPSG:5070

# Diff: visual before/after slider of two dataset versions
tissot diff Q3_parcels.gpkg Q4_parcels.gpkg

# Watch: monitor a directory and stream updates to a live dashboard
tissot watch ./pipeline/output/
```

Every command defaults to opening an interactive browser map. Add `--json` for machine-readable output or `--terminal` for rich terminal text.

---

## The Hero Feature: Projection X-Ray

Every GIS professional has been told "don't use Web Mercator for area calculations." But have you ever **seen** the actual error on your actual data?

`tissot xray` computes per-feature distortion using Jacobian matrix analysis, generates a heatmap overlaid on your features, draws Tissot ellipses at sample locations, and recommends a better CRS — with quantified proof.

```
$ tissot xray kentucky_permits.gpkg --recommend

  Current CRS: EPSG:3857 (Web Mercator)
  Area distortion — Max: 18.3%  Mean: 11.7%
  Distance distortion — Max: 12.1%  Mean: 7.4%

  Recommendations:
    1. EPSG:3089 (NAD83 / Kentucky Single Zone)
       Area distortion — Max: 0.02%  Mean: 0.01%
    2. EPSG:5070 (NAD83 / Conus Albers)
       Area distortion — Max: 0.08%  Mean: 0.03%

  Samples: 847 points analyzed
  → Interactive report opened in browser
```

---

## Supported Formats

| Format | Support | Commands |
|--------|---------|----------|
| GeoJSON | Full | xray, check, score, fix, diff |
| Shapefile | Read | xray, check, score, diff |
| FlatGeobuf | Read | xray, check, score, diff |
| GeoParquet | Read (feature-gated) | xray, check, score, diff |
| GeoPackage | Read (optional GDAL) | xray, check, score, diff |

---

## Checker Domains

| Domain | Rules | What It Checks |
|--------|-------|----------------|
| **Data Quality** (9) | Null geometry, duplicates, self-intersection, topology gaps/overlaps, schema, extent, empty dataset | Geometry validity and data integrity |
| **Projection** (5) | Area/distance distortion, datum mismatch, high distortion, missing CRS | CRS appropriateness and accuracy |
| **Cloud Native** (6) | Format recommendation, CRS metadata, multi-file integrity, spatial index, compression, file size | Cloud-optimized format best practices |
| **Cartography** (3) | Color contrast, label density, classification count | Visual quality and readability |

---

## What Tissot Is NOT

Tissot is a **diagnostic and autofix CLI** for geospatial data quality. It is not:

- **Not a GIS desktop application** — use [QGIS](https://qgis.org/) for that (Tissot has a QGIS plugin)
- **Not a spatial database** — use [PostGIS](https://postgis.net/) for storage and queries
- **Not a tile server** — use [Martin](https://maplibre.org/martin/) or [TiTiler](https://developmentseed.org/titiler/) for serving tiles
- **Not a format converter** — use [GDAL/OGR](https://gdal.org/) for format transformations
- **Not a geocoding service** — Tissot analyzes existing spatial data, it doesn't create it

Tissot is the CLI toolkit you reach for **alongside** those tools — to verify projections, lint data quality, score readiness, and autofix problems before publishing.

---

## Python Library

Every CLI command is backed by a Rust function exposed via PyO3 bindings:

```python
import json
import tissot

# Projection X-Ray analysis
report = json.loads(tissot.xray("kentucky_permits.gpkg"))
print(f"Mean area distortion: {report['distortion']['mean_area_pct']:.2f}%")
print(f"Recommended CRS: {report['recommendations'][0]['epsg']}")

# Data quality check
findings = json.loads(tissot.check("parcels.geojson", domain="quality"))
print(f"Total findings: {findings['summary']['total']}")

# Quality score
score = json.loads(tissot.score("parcels.geojson"))
print(f"Score: {score['overall_score']}/100 ({score['grade']})")
```

---

## Built With

Rust core using the [GeoRust](https://georust.org/) ecosystem. Python bindings via [PyO3](https://pyo3.rs). Visual reports powered by [MapLibre GL JS](https://maplibre.org/). Cloud-native format guidance aligned with the [Cloud Native Geo Formats Guide](https://guide.cloudnativegeo.org/).

[Get started :material-arrow-right:](getting-started.md){ .md-button .md-button--primary }
[CLI Reference :material-arrow-right:](cli.md){ .md-button }
