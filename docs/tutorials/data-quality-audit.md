# Tutorial: Data Quality Audit

Run a comprehensive data quality check and fix issues automatically.

## Step 1: Run All Checks

```bash
tissot check parcels.geojson
```

This opens a browser map with all findings plotted spatially, color-coded by severity.

## Step 2: Filter by Domain

Focus on specific issue types:

```bash
# Data quality only (geometry, topology, schema)
tissot check parcels.geojson --domain quality

# Projection issues only
tissot check parcels.geojson --domain projection

# Cloud-native format compliance
tissot check parcels.geojson --domain cloud
```

## Step 3: Review Findings

### Terminal Output

```bash
tissot check parcels.geojson --terminal
```

```
Tissot Check — parcels.geojson
  Findings: 12 (3 errors, 7 warnings, 2 info)

  ERRORS:
    [data/null-geometry] 3 features have null geometry
    [data/self-intersection] 1 polygon has self-intersection
    [proj/missing-crs] No CRS defined

  WARNINGS:
    [data/topology-gaps] 4 gaps detected between adjacent parcels
    [data/duplicate-geometry] 2 features share identical geometry
    [cloud/spatial-index] No spatial index detected
```

### JSON Output

```bash
tissot check parcels.geojson --json | jq '.findings[] | {rule: .rule_id, severity: .severity}'
```

## Step 4: Fix What You Can

Heal topology issues:

```bash
tissot fix parcels.geojson --topology
```

Add a proper projection:

```bash
tissot fix parcels.geojson --reproject EPSG:5070
```

## Step 5: Re-check

```bash
tissot check parcels_fixed.geojson --terminal
```

## Available Rules

### Data Quality Domain

| Rule ID | Severity | What It Checks |
|---------|----------|----------------|
| `data/null-geometry` | Error | Features with null/missing geometry |
| `data/duplicate-features` | Warning | Identical feature pairs |
| `data/duplicate-geometry` | Warning | Features sharing identical geometry |
| `data/self-intersection` | Error | Self-intersecting polygons |
| `data/topology-gaps` | Warning | Gaps between adjacent polygons |
| `data/topology-overlaps` | Warning | Overlapping polygon areas |
| `data/schema-validation` | Info | Schema consistency issues |
| `data/extent-bounds` | Warning | Features outside expected bounds |
| `data/empty-dataset` | Error | Dataset with no features |

### Projection Domain

| Rule ID | Severity | What It Checks |
|---------|----------|----------------|
| `proj/missing-crs` | Error | No CRS defined |
| `proj/area-distortion` | Warning | Area distortion above threshold |
| `proj/distance-distortion` | Warning | Distance distortion above threshold |
| `proj/high-distortion` | Error | Extreme distortion levels |
| `proj/datum-mismatch` | Warning | Inconsistent datums across layers |

### Cloud Native Domain

| Rule ID | Severity | What It Checks |
|---------|----------|----------------|
| `cloud/format-recommendation` | Info | Non-cloud-optimized format |
| `cloud/crs-metadata` | Warning | Missing/incomplete CRS metadata |
| `cloud/multi-file-integrity` | Warning | Shapefile companion file issues |
| `cloud/spatial-index` | Warning | Missing spatial index |
| `cloud/compression` | Info | Uncompressed data |
| `cloud/file-size` | Info | Large file without partitioning |

## SARIF Output for CI/CD

Upload findings to GitHub Code Scanning:

```bash
tissot check data.geojson --sarif > results.sarif
```

```yaml
# .github/workflows/geo-quality.yml
- name: Run Tissot checks
  run: tissot check data.geojson --sarif > results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```
