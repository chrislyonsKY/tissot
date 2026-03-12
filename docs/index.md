# Tissot

**Visual-first geospatial diagnostics engine.**

Named after [Tissot's indicatrix](https://en.wikipedia.org/wiki/Tissot%27s_indicatrix) — the distortion ellipses that reveal what map projections hide.

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

=== "pip"

    ```bash
    pip install tissot
    ```

=== "cargo"

    ```bash
    cargo install tissot
    ```

=== "QGIS Plugin"

    Available from the [QGIS Plugin Repository](https://plugins.qgis.org/plugins/tissot_processing_provider/).

    ```bash
    # Install the CLI into QGIS Python first
    # macOS
    "/Applications/QGIS.app/Contents/MacOS/python" -m pip install tissot

    # Windows (OSGeo4W Shell)
    python -m pip install tissot

    # Linux
    python3 -m pip install tissot
    ```

    Then in QGIS: **Plugins > Manage and Install Plugins** > search **Tissot Processing Provider** > **Install**.

## Key Capabilities

| Command | What It Does | Output |
|---------|-------------|--------|
| `tissot xray` | Per-feature projection distortion analysis | Interactive heatmap + ellipses |
| `tissot check` | 20+ diagnostic rules across 3 domains | Findings map with severity |
| `tissot score` | 0-100 quality rating (Lighthouse for maps) | Score dashboard + SVG badge |
| `tissot fix` | Autofix: reproject, heal topology | Fixed output file |
| `tissot diff` | Spatial before/after comparison | Interactive slider map |
| `tissot watch` | Live directory monitoring | Streaming dashboard |

## Checker Domains

| Domain | Rules | Examples |
|--------|-------|---------|
| **Data Quality** (9) | null geometry, duplicates, self-intersection, topology gaps/overlaps, schema, extent, empty dataset | `data/null-geometry`, `data/topology-gaps` |
| **Projection** (5) | area distortion, distance distortion, datum mismatch, high distortion, missing CRS | `proj/area-distortion`, `proj/datum-mismatch` |
| **Cloud Native** (6) | format recommendation, CRS metadata, multi-file integrity, spatial index, compression, file size | `cloud/format-recommendation`, `cloud/spatial-index` |

## Built With

Rust core using the [GeoRust](https://georust.org/) ecosystem. Python bindings via [PyO3](https://pyo3.rs). Visual reports powered by [MapLibre GL JS](https://maplibre.org/). Cloud-native format guidance aligned with the [CNG Formats Guide](https://guide.cloudnativegeo.org/).

## License

Dual-licensed under [MIT](https://github.com/chrislyonsKY/tissot/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/chrislyonsKY/tissot/blob/main/LICENSE-APACHE), at your option.
