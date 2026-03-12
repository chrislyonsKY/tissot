# Tutorial: Cloud-Native Validation

Validate your geospatial data against cloud-native best practices using Tissot's cloud checker domain.

## Why Cloud-Native Matters

Cloud-native geospatial formats (FlatGeobuf, GeoParquet, Cloud-Optimized GeoTIFF) are designed for efficient HTTP range requests, enabling data access without downloading entire files. Tissot checks whether your data follows these best practices.

## Run Cloud Checks

```bash
tissot check parcels.shp --domain cloud
```

```
Tissot Check — parcels.shp (cloud domain)
  Findings: 4 (0 errors, 2 warnings, 2 info)

  WARNINGS:
    [cloud/spatial-index] No spatial index detected
    [cloud/crs-metadata] CRS metadata incomplete — missing EPSG authority

  INFO:
    [cloud/format-recommendation] Shapefile is not cloud-optimized;
      consider FlatGeobuf or GeoParquet
    [cloud/compression] Data is uncompressed (42 MB);
      compression could reduce to ~12 MB
```

## Cloud-Native Rules

| Rule | Severity | What It Checks |
|------|----------|----------------|
| `cloud/format-recommendation` | Info | Is the format cloud-optimized? |
| `cloud/crs-metadata` | Warning | Complete CRS/EPSG metadata present? |
| `cloud/multi-file-integrity` | Warning | Shapefile companions (.dbf, .shx, .prj) present? |
| `cloud/spatial-index` | Warning | Spatial index available for range queries? |
| `cloud/compression` | Info | Could the data benefit from compression? |
| `cloud/file-size` | Info | Is the file too large without partitioning? |

## Format Comparison

| Format | Cloud-Optimized | Spatial Index | Compression | Streaming |
|--------|----------------|---------------|-------------|-----------|
| GeoJSON | No | No | No | No |
| Shapefile | No | .shx only | No | No |
| FlatGeobuf | Yes | Built-in | Optional | Yes |
| GeoParquet | Yes | Built-in | Snappy/Zstd | Yes |
| GeoPackage | Partial | SQLite R-Tree | No | No |

## Cloud Migration Workflow

### Step 1: Audit current format

```bash
tissot check legacy_data.shp --domain cloud --json
```

### Step 2: Fix projection and topology first

```bash
tissot fix legacy_data.shp --reproject EPSG:4326
tissot fix legacy_data_fixed.geojson --topology
```

### Step 3: Convert to cloud-native format

Use GDAL/ogr2ogr to convert to FlatGeobuf:

```bash
ogr2ogr -f FlatGeobuf output.fgb legacy_data_fixed.geojson
```

### Step 4: Re-validate

```bash
tissot check output.fgb --domain cloud --terminal
```

## CI/CD Cloud Readiness Gate

```yaml
- name: Validate cloud-native compliance
  run: |
    FINDINGS=$(tissot check data.fgb --domain cloud --json | jq '.summary.warnings')
    if [ "$FINDINGS" -gt 0 ]; then
      echo "Cloud-native warnings found"
      tissot check data.fgb --domain cloud --terminal
      exit 1
    fi
```
