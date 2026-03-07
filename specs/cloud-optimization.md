# Cloud Optimization Checker — Feature Specification

> Rules that validate whether geospatial data is packaged for cloud-native access patterns.
> Aligned with the Cloud Native Geospatial Forum's Cloud-Optimized Geospatial Formats Guide.

## Overview

The Cloud Native Geospatial movement is driving adoption of formats designed for partial, parallel reads over HTTP range requests: GeoParquet, FlatGeobuf, Cloud Optimized GeoTIFF, PMTiles, and Zarr. Legacy formats like Shapefile and GeoPackage require full download to access any data, creating bottlenecks in cloud pipelines.

Tissot's cloud optimization checker validates whether a dataset's format, structure, and packaging follow cloud-native best practices. These rules help data engineers and GIS analysts identify when their data could perform better in modern infrastructure.

Reference: https://guide.cloudnativegeo.org/

## Rules

### `cloud/format-recommendation`

**Purpose**: Flags when a dataset uses a format that is not cloud-optimized and suggests alternatives appropriate for the data type and use case.

**Logic**:

| Input Format | Data Type | Recommendation | Rationale |
|---|---|---|---|
| Shapefile (.shp) | Vector | FlatGeobuf or GeoParquet | Shapefile has no internal tiling, 2GB limit, multi-file mess, no CRS embedding in modern standard |
| GeoPackage (.gpkg) | Vector | FlatGeobuf or GeoParquet | SQLite-based; entire file must be downloaded for any read. Not HTTP range-request friendly |
| GeoJSON | Vector (small) | No flag if < 10MB | GeoJSON is acceptable for small datasets; flag if > 10MB with suggestion to use FlatGeobuf |
| GeoJSON | Vector (large) | FlatGeobuf or GeoParquet | Large GeoJSON is slow to parse and not streamable |
| FlatGeobuf | Vector | No flag | Already cloud-optimized (spatial index, streamable) |
| GeoParquet | Vector | No flag | Already cloud-optimized (columnar, compressed, spatial metadata) |

**Configuration**:
```yaml
cloud/format-recommendation:
  enabled: true
  geojson_size_threshold_mb: 10   # Only flag GeoJSON above this size
  suppress_for_local: false        # Set true to skip this rule for local-only workflows
```

**Finding Example**:
- Severity: Info
- Message: "Dataset is in Shapefile format, which is not cloud-optimized. For cloud-native workflows, consider converting to FlatGeobuf (streamable, spatially indexed) or GeoParquet (columnar, compressed). See: https://guide.cloudnativegeo.org/"
- Fixable: true (Phase 2 — `tissot fix --convert-format fgb`)

---

### `cloud/spatial-index`

**Purpose**: Checks whether the file format includes a spatial index that supports partial reads.

**Logic**:
- FlatGeobuf: Check if the spatial index header is present (FlatGeobuf optionally includes a packed Hilbert R-tree)
- GeoParquet: Check for `bbox` column or spatial metadata in the Parquet footer per the GeoParquet spec
- GeoPackage: Check for `rtree_<table>_<geom>` tables (SQLite R-tree index)

**Configuration**:
```yaml
cloud/spatial-index:
  enabled: true
  require_for: [fgb, parquet]  # Which formats must have a spatial index
```

**Finding Example**:
- Severity: Warning
- Message: "FlatGeobuf file lacks a spatial index. Spatial queries will require full file scan. Regenerate with spatial indexing enabled for efficient partial reads."
- Fixable: true (future — regenerate FlatGeobuf with index)

---

### `cloud/file-size`

**Purpose**: Flags files that are too large for efficient single-file cloud access, or too small to benefit from cloud optimization overhead.

**Logic**:
- Flag files > 2GB: "Consider partitioning this dataset. Files > 2GB create long initial load times even with range requests. Consider spatial partitioning or use a multi-file GeoParquet dataset."
- Flag GeoParquet files < 1MB: "This dataset is very small. The overhead of Parquet's columnar format may not provide benefits at this size. GeoJSON may be simpler."
- Flag Shapefile > 2GB: "This file exceeds Shapefile's 2GB limit. Data may be truncated. Convert to GeoParquet or FlatGeobuf."

**Configuration**:
```yaml
cloud/file-size:
  enabled: true
  max_single_file_gb: 2.0
  min_parquet_benefit_mb: 1.0
```

---

### `cloud/compression`

**Purpose**: Checks whether the dataset uses appropriate internal compression for cloud access.

**Logic**:
- GeoParquet: Check metadata for compression codec (snappy, zstd, gzip). Flag if uncompressed.
- FlatGeobuf: FlatGeobuf does not support internal compression by design (optimized for streaming). No flag.
- GeoJSON: Flag if > 10MB and not served with HTTP compression. Suggest FlatGeobuf instead.

**Finding Example**:
- Severity: Info
- Message: "GeoParquet file uses no compression. Adding Zstandard (zstd) compression typically reduces file size by 50-70% with minimal decompression overhead."

---

### `cloud/crs-metadata`

**Purpose**: Validates that the dataset's CRS metadata is properly embedded and readable without downloading the full file.

**Logic**:
- GeoParquet: Check that `geo` metadata key exists in the Parquet schema with `crs` field per GeoParquet spec
- FlatGeobuf: Check header for CRS definition (organization + code, or WKT)
- Shapefile: Check for .prj file existence. Flag if missing.
- GeoPackage: Check `gpkg_spatial_ref_sys` table

**Finding Example**:
- Severity: Error
- Message: "Shapefile is missing .prj file. CRS is unknown. All downstream spatial operations will assume an arbitrary coordinate system."

---

### `cloud/multi-file-integrity`

**Purpose**: For multi-file formats, validates that all required sidecar files are present and consistent.

**Logic**:
- Shapefile: Requires .shp + .shx + .dbf at minimum. Flag missing .prj, .cpg. Flag orphaned .shp without .dbf.
- GeoParquet (partitioned): Validate that `_metadata` or `_common_metadata` files are present for Hive-partitioned datasets.

**Finding Example**:
- Severity: Error
- Message: "Shapefile is missing .shx (spatial index) file. The .shp file cannot be read without it."

## Score Integration

Cloud optimization rules contribute to a new **Cloud Readiness** score category:

```
Categories (updated):
  - Projection Quality (weight: 0.20)
  - Data Integrity (weight: 0.25)
  - Accessibility (weight: 0.20)
  - Classification Quality (weight: 0.15)
  - Cloud Readiness (weight: 0.20)    ← NEW
```

This category is only active when Tissot detects the data is in (or destined for) a cloud pipeline. Detection heuristic: if the input is GeoParquet, FlatGeobuf, or served from an HTTP URL, cloud readiness rules activate automatically.

## Relationship to CNG Formats Guide

Each rule's finding message includes a direct link to the relevant section of the Cloud-Optimized Geospatial Formats Guide (https://guide.cloudnativegeo.org/):

| Rule | Guide Section |
|---|---|
| `cloud/format-recommendation` | Format overview + specific format primer |
| `cloud/spatial-index` | Relevant format's "how to create" cookbook |
| `cloud/file-size` | Overview: "Subsetted access is facilitated via addressable chunks" |
| `cloud/compression` | Format-specific compression guidance |
| `cloud/crs-metadata` | Format-specific metadata requirements |
| `cloud/multi-file-integrity` | Shapefile limitations, migration guidance |

## Phase

- **Phase 1**: `cloud/format-recommendation`, `cloud/crs-metadata`, `cloud/multi-file-integrity` (these require only file metadata inspection, no deep parsing)
- **Phase 2**: `cloud/spatial-index`, `cloud/compression`, `cloud/file-size` (these require format-specific header parsing)
- **Future**: `cloud/partitioning` (validate spatial partitioning strategy for large datasets), `cloud/stac-compliance` (validate STAC metadata if present)
