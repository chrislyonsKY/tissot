# Getting Started

## Requirements

- **Rust 1.85+** (if building from source)
- **Python 3.9 - 3.13** (for pip install or QGIS plugin)

## Installation

=== "pip"

    ```bash
    pip install tissot
    ```

=== "cargo"

    ```bash
    cargo install tissot
    ```

=== "From source"

    ```bash
    git clone https://github.com/chrislyonsKY/tissot.git
    cd tissot
    cargo build --release
    # Binary at target/release/tissot
    ```

## Quick Start

### 1. X-Ray Your Data

Run projection distortion analysis on any geospatial file:

```bash
tissot xray my_data.geojson
```

This opens an interactive map in your browser showing:

- **Distortion heatmap** — color-coded area/distance error across your features
- **Tissot ellipses** — classic indicatrix ellipses rendered at sample points
- **CRS recommendation** — a better projection for your data with quantified improvement

### 2. Check Data Quality

Run all 20+ diagnostic rules:

```bash
tissot check my_data.geojson
```

Filter by domain:

```bash
tissot check my_data.geojson --domain quality    # Data quality rules only
tissot check my_data.geojson --domain projection  # Projection rules only
tissot check my_data.geojson --domain cloud       # Cloud-native rules only
```

### 3. Get a Score

Generate a Lighthouse-style quality score:

```bash
tissot score my_data.geojson
```

Generate an SVG badge for your README:

```bash
tissot score my_data.geojson --badge score.svg
```

### 4. Fix Problems

Reproject to an optimal CRS:

```bash
tissot fix my_data.geojson --reproject EPSG:5070
```

Heal topology issues:

```bash
tissot fix my_data.geojson --topology
```

## Output Modes

Every command supports multiple output formats:

| Flag | Output | Use Case |
|------|--------|----------|
| *(default)* | Interactive browser map | Exploration, presentations |
| `--terminal` | Rich terminal text | SSH sessions, quick checks |
| `--json` | Machine-readable JSON | Scripting, pipelines |
| `--sarif` | SARIF format | CI/CD code scanning |

## Configuration

Tissot works with zero configuration. To customize behavior:

```bash
tissot init  # Creates .tissot.yml with smart defaults
```

Example `.tissot.yml`:

```yaml
xray:
  max_samples: 1000
  top_recommendations: 5

check:
  max_distortion_pct: 10.0
  topology_gap_tolerance: 0.001
  disabled_rules: []

score:
  projection_weight: 0.25
  data_integrity_weight: 0.30
  accessibility_weight: 0.25
  classification_weight: 0.20

output:
  open_browser: true
  terminal_only: false
```

## Supported Formats

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| GeoJSON | Yes | Yes | Pure Rust (geozero) |
| Shapefile | Yes | - | Pure Rust (shapefile crate) |
| FlatGeobuf | Yes | - | Pure Rust (flatgeobuf crate) |
| GeoPackage | Yes | Optional | Requires `gdal` feature flag |

## Next Steps

- [CLI Reference](cli.md) — full command documentation
- [Projection X-Ray Tutorial](tutorials/projection-xray.md) — step-by-step walkthrough
- [Architecture](architecture.md) — how Tissot works under the hood
