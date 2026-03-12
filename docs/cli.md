# CLI Reference

## Global Behavior

- All visual commands open an interactive map in the default browser
- The local web server shuts down on `Ctrl+C`
- All commands support `--json` for machine-readable output
- Zero configuration required — smart defaults applied automatically

---

## `tissot xray`

Projection distortion analysis — the hero feature.

```bash
tissot xray <FILE> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input geospatial file (GeoJSON, Shapefile, FlatGeobuf, GeoPackage) |

**Options:**

| Option | Description |
|--------|-------------|
| `--recommend` | Include CRS recommendations in the report |
| `--crs <EPSG>` | Target CRS to analyze (defaults to file's CRS) |
| `--terminal` | Output to terminal instead of browser |
| `--json` | Output machine-readable JSON |

**Examples:**

```bash
# Basic distortion analysis
tissot xray parcels.gpkg

# With CRS recommendations
tissot xray parcels.gpkg --recommend

# Analyze specific CRS
tissot xray parcels.gpkg --crs EPSG:3857

# JSON output for scripting
tissot xray parcels.gpkg --json | jq '.distortion.mean_area_pct'
```

---

## `tissot check`

Run diagnostic checks across multiple domains.

```bash
tissot check <FILE> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--domain <DOMAIN>` | Filter: `projection`, `quality`, `cloud`, `cartography`, `diff` |
| `--terminal` | Output to terminal instead of browser |
| `--json` | Output machine-readable JSON |
| `--sarif` | Output SARIF for CI/CD integration |

**Examples:**

```bash
# All checks
tissot check data.geojson

# Data quality only
tissot check data.geojson --domain quality

# CI/CD integration
tissot check data.geojson --sarif > results.sarif
```

---

## `tissot score`

Generate a 0-100 quality score with category breakdown.

```bash
tissot score <FILE> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--badge <PATH>` | Generate SVG badge at the given path |
| `--terminal` | Output to terminal instead of browser |
| `--json` | Output machine-readable JSON |

**Examples:**

```bash
# Interactive score dashboard
tissot score project.qgz

# Generate badge for README
tissot score data.geojson --badge map-score.svg

# CI gate: fail if score below 80
SCORE=$(tissot score data.geojson --json | jq '.overall_score')
if [ $(echo "$SCORE < 80" | bc) -eq 1 ]; then exit 1; fi
```

**Score Categories:**

| Category | Weight | What It Measures |
|----------|--------|------------------|
| Projection Quality | 0.25 | CRS appropriateness, distortion levels |
| Data Integrity | 0.30 | Geometry validity, topology, schema |
| Accessibility | 0.20 | WCAG compliance, readability |
| Cloud Readiness | 0.20 | Format optimization, spatial indexing |
| Classification | 0.05 | Data categorization quality |

---

## `tissot fix`

Apply automatic fixes to geospatial data.

```bash
tissot fix <FILE> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--reproject <CRS>` | Reproject to target CRS (e.g., `EPSG:5070`) |
| `--topology` | Heal topology gaps and overlaps |
| `--in-place` | Modify input file directly (default: create `_fixed` copy) |
| `--json` | Output machine-readable JSON report |

**Examples:**

```bash
# Reproject to NAD83 / Conus Albers
tissot fix parcels.geojson --reproject EPSG:5070

# Heal topology in place
tissot fix parcels.geojson --topology --in-place
```

---

## `tissot diff`

Compare two versions of a dataset.

```bash
tissot diff <LEFT> <RIGHT> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--terminal` | Output to terminal instead of browser |
| `--json` | Output machine-readable JSON |

**Examples:**

```bash
# Interactive slider comparison
tissot diff Q3_parcels.gpkg Q4_parcels.gpkg

# JSON change summary
tissot diff v1.geojson v2.geojson --json
```

---

## `tissot watch`

Monitor a directory and stream diagnostic updates to a live dashboard.

```bash
tissot watch <DIR>
```

**Examples:**

```bash
# Watch a pipeline output directory
tissot watch ./data/output/

# Watch current directory
tissot watch .
```

---

## `tissot init`

Create a starter configuration file.

```bash
tissot init [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force` | Overwrite existing `.tissot.yml` |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (file not found, parse failure, etc.) |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `TISSOT_NO_BROWSER` | Set to `1` to suppress browser auto-open |
