/// GeoParquet reader — reads `.parquet` files with GeoParquet metadata.
///
/// Uses the `parquet` and `arrow` crates (behind the `geoparquet` feature flag)
/// to read Parquet files, extract GeoParquet metadata from the file's key-value
/// metadata, parse WKB geometries from the geometry column, and return features
/// matching Tissot's `Layer` / `Feature` types.
///
/// When the `geoparquet` feature is not enabled, calling `read()` returns a
/// helpful error directing the user to enable the feature or convert to another
/// format.

#[cfg(feature = "geoparquet")]
mod inner {
    use crate::core::error::{Result, TissotError};
    use crate::core::rule::{Feature, Layer};
    use arrow::array::{Array, AsArray, BinaryArray, LargeBinaryArray, StringArray};
    use arrow::datatypes::DataType;
    use geo::{BoundingRect, Geometry};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::path::Path;

    /// GeoParquet metadata stored in the Parquet file's key-value metadata
    /// under the key `"geo"`.
    #[derive(Debug, Deserialize)]
    struct GeoParquetMetadata {
        /// Primary geometry column name.
        #[serde(default = "default_geometry_column")]
        primary_column: String,
        /// Per-column metadata.
        #[serde(default)]
        columns: HashMap<String, ColumnMeta>,
    }

    /// Metadata for a single geometry column.
    #[derive(Debug, Deserialize)]
    struct ColumnMeta {
        /// Geometry encoding: `"WKB"`, `"point"`, `"multipolygon"`, etc.
        #[serde(default = "default_encoding")]
        encoding: String,
        /// CRS in PROJJSON format (optional).
        #[serde(default)]
        crs: Option<serde_json::Value>,
        /// Bounding box [xmin, ymin, xmax, ymax].
        #[serde(default)]
        bbox: Option<Vec<f64>>,
    }

    fn default_geometry_column() -> String {
        "geometry".to_string()
    }

    fn default_encoding() -> String {
        "WKB".to_string()
    }

    /// Read a GeoParquet file and return layers.
    pub fn read(path: &Path) -> Result<Vec<Layer>> {
        let file = std::fs::File::open(path)?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            TissotError::GeoParquet(format!("Failed to open Parquet file: {e}"))
        })?;

        // Extract GeoParquet metadata from Parquet key-value metadata.
        let geo_meta = extract_geo_metadata(&builder)?;
        let geom_col = &geo_meta.primary_column;
        let crs = extract_crs(&geo_meta);

        let reader = builder.build().map_err(|e| {
            TissotError::GeoParquet(format!("Failed to build Parquet reader: {e}"))
        })?;

        let schema = reader.schema();

        // Find the geometry column index.
        let geom_idx = schema
            .fields()
            .iter()
            .position(|f| f.name() == geom_col)
            .ok_or_else(|| {
                TissotError::GeoParquet(format!(
                    "Geometry column '{geom_col}' not found in schema"
                ))
            })?;

        let mut features = Vec::new();

        for batch_result in reader {
            let batch = batch_result.map_err(|e| {
                TissotError::GeoParquet(format!("Failed to read record batch: {e}"))
            })?;

            let geom_array = batch.column(geom_idx);
            let num_rows = batch.num_rows();

            // Build property columns (everything except the geometry column).
            let prop_fields: Vec<(usize, &str)> = schema
                .fields()
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != geom_idx)
                .map(|(i, f)| (i, f.name().as_str()))
                .collect();

            for row in 0..num_rows {
                let geometry = parse_geometry_from_array(geom_array.as_ref(), row)?;

                let mut properties = HashMap::new();
                for &(col_idx, col_name) in &prop_fields {
                    if let Some(value) = column_value_to_json(batch.column(col_idx), row) {
                        properties.insert(col_name.to_string(), value);
                    }
                }

                features.push(Feature {
                    id: None,
                    geometry,
                    properties,
                });
            }
        }

        let bounds = compute_bounds(&features);

        Ok(vec![Layer {
            name: path.to_string_lossy().to_string(),
            crs,
            features,
            bounds,
        }])
    }

    /// Extract the `"geo"` key-value metadata from the Parquet file metadata.
    fn extract_geo_metadata(
        builder: &ParquetRecordBatchReaderBuilder<std::fs::File>,
    ) -> Result<GeoParquetMetadata> {
        let file_meta = builder.metadata().file_metadata();
        let kv_meta = file_meta.key_value_metadata();

        let geo_json = kv_meta
            .and_then(|kvs| kvs.iter().find(|kv| kv.key == "geo"))
            .and_then(|kv| kv.value.as_ref())
            .ok_or_else(|| {
                TissotError::GeoParquet(
                    "No GeoParquet metadata found (missing 'geo' key in file metadata). \
                     This may be a plain Parquet file without geospatial metadata."
                        .to_string(),
                )
            })?;

        serde_json::from_str(geo_json).map_err(|e| {
            TissotError::GeoParquet(format!("Failed to parse GeoParquet metadata: {e}"))
        })
    }

    /// Extract CRS identifier from GeoParquet column metadata.
    ///
    /// Attempts to find an EPSG code from PROJJSON; falls back to WGS 84
    /// if no CRS is specified (GeoParquet default).
    fn extract_crs(meta: &GeoParquetMetadata) -> Option<String> {
        let col_meta = meta.columns.get(&meta.primary_column)?;

        match &col_meta.crs {
            Some(crs_json) => {
                // Try to extract EPSG code from PROJJSON id field.
                if let Some(id) = crs_json.get("id") {
                    if let (Some(authority), Some(code)) =
                        (id.get("authority"), id.get("code"))
                    {
                        let auth = authority.as_str().unwrap_or("EPSG");
                        if let Some(code_num) = code.as_u64() {
                            return Some(format!("{auth}:{code_num}"));
                        }
                        if let Some(code_str) = code.as_str() {
                            return Some(format!("{auth}:{code_str}"));
                        }
                    }
                }
                // Fallback: store the raw PROJJSON as a string representation.
                Some(crs_json.to_string())
            }
            // GeoParquet spec: if crs is null/missing, the data is in WGS 84.
            None => Some("EPSG:4326".to_string()),
        }
    }

    /// Parse a geometry from a WKB byte array at the given row index.
    fn parse_geometry_from_array(
        array: &dyn Array,
        row: usize,
    ) -> Result<Option<Geometry>> {
        if array.is_null(row) {
            return Ok(None);
        }

        let wkb_bytes: Option<&[u8]> = match array.data_type() {
            DataType::Binary => {
                let bin_array = array
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    .ok_or_else(|| {
                        TissotError::GeoParquet("Failed to cast to BinaryArray".into())
                    })?;
                Some(bin_array.value(row))
            }
            DataType::LargeBinary => {
                let bin_array = array
                    .as_any()
                    .downcast_ref::<LargeBinaryArray>()
                    .ok_or_else(|| {
                        TissotError::GeoParquet(
                            "Failed to cast to LargeBinaryArray".into(),
                        )
                    })?;
                Some(bin_array.value(row))
            }
            dt => {
                log::warn!(
                    "Geometry column has unsupported type {:?}, skipping WKB parse",
                    dt
                );
                None
            }
        };

        match wkb_bytes {
            Some(bytes) => parse_wkb(bytes).map(Some),
            None => Ok(None),
        }
    }

    /// Parse a WKB byte sequence into a `geo::Geometry`.
    fn parse_wkb(wkb: &[u8]) -> Result<Geometry> {
        if wkb.is_empty() {
            return Err(TissotError::GeoParquet("Empty WKB geometry".into()));
        }

        // Minimal WKB parser for the most common types.
        // WKB format: byte_order (1 byte) + type (4 bytes) + coordinates...
        if wkb.len() < 5 {
            return Err(TissotError::GeoParquet(format!(
                "WKB too short ({} bytes)",
                wkb.len()
            )));
        }

        let little_endian = wkb[0] == 1;
        let geom_type = if little_endian {
            u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        } else {
            u32::from_be_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        };

        // Mask off SRID and Z/M flags to get the base type.
        let base_type = geom_type & 0xFF;

        match base_type {
            1 => parse_wkb_point(wkb, little_endian),
            2 => parse_wkb_linestring(wkb, little_endian),
            3 => parse_wkb_polygon(wkb, little_endian),
            4 => parse_wkb_multipoint(wkb, little_endian),
            5 => parse_wkb_multilinestring(wkb, little_endian),
            6 => parse_wkb_multipolygon(wkb, little_endian),
            _ => Err(TissotError::GeoParquet(format!(
                "Unsupported WKB geometry type: {geom_type} (base type: {base_type})"
            ))),
        }
    }

    /// Determine the byte offset where coordinates begin, accounting for
    /// optional SRID prefix in EWKB.
    fn coord_offset(wkb: &[u8], little_endian: bool) -> usize {
        let geom_type = if little_endian {
            u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        } else {
            u32::from_be_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        };
        // EWKB SRID flag is 0x20000000.
        if geom_type & 0x20000000 != 0 {
            5 + 4 // skip byte_order(1) + type(4) + srid(4)
        } else {
            5 // skip byte_order(1) + type(4)
        }
    }

    /// Read a `f64` from `buf` at `offset` with the given endianness.
    fn read_f64(buf: &[u8], offset: usize, le: bool) -> Result<f64> {
        let bytes: [u8; 8] = buf.get(offset..offset + 8).ok_or_else(|| {
            TissotError::GeoParquet(format!(
                "WKB truncated at offset {offset} (need 8 bytes, have {})",
                buf.len()
            ))
        })?.try_into().map_err(|_| {
            TissotError::GeoParquet("WKB slice conversion failed".into())
        })?;
        Ok(if le {
            f64::from_le_bytes(bytes)
        } else {
            f64::from_be_bytes(bytes)
        })
    }

    /// Read a `u32` from `buf` at `offset` with the given endianness.
    fn read_u32(buf: &[u8], offset: usize, le: bool) -> Result<u32> {
        let bytes: [u8; 4] = buf.get(offset..offset + 4).ok_or_else(|| {
            TissotError::GeoParquet(format!(
                "WKB truncated at offset {offset} (need 4 bytes, have {})",
                buf.len()
            ))
        })?.try_into().map_err(|_| {
            TissotError::GeoParquet("WKB slice conversion failed".into())
        })?;
        Ok(if le {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    }

    /// Check if the WKB geometry type has a Z component.
    fn has_z(wkb: &[u8], le: bool) -> bool {
        let gt = if le {
            u32::from_le_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        } else {
            u32::from_be_bytes([wkb[1], wkb[2], wkb[3], wkb[4]])
        };
        // ISO WKB: types 1001-1007 have Z. EWKB: 0x80000000 flag.
        (gt & 0xFF00 == 0x3E8) || (gt & 0x80000000 != 0)
    }

    /// Number of bytes per coordinate (16 for 2D, 24 for 3D).
    fn coord_size(wkb: &[u8], le: bool) -> usize {
        if has_z(wkb, le) { 24 } else { 16 }
    }

    fn parse_wkb_point(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let x = read_f64(wkb, off, le)?;
        let y = read_f64(wkb, off + 8, le)?;
        Ok(Geometry::Point(geo::Point::new(x, y)))
    }

    fn parse_wkb_linestring(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let cs = coord_size(wkb, le);
        let num_points = read_u32(wkb, off, le)? as usize;
        let data_start = off + 4;
        let mut coords = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let base = data_start + i * cs;
            let x = read_f64(wkb, base, le)?;
            let y = read_f64(wkb, base + 8, le)?;
            coords.push(geo::Coord { x, y });
        }
        Ok(Geometry::LineString(geo::LineString::new(coords)))
    }

    fn parse_wkb_ring(wkb: &[u8], offset: usize, le: bool, cs: usize) -> Result<(geo::LineString, usize)> {
        let num_points = read_u32(wkb, offset, le)? as usize;
        let data_start = offset + 4;
        let mut coords = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let base = data_start + i * cs;
            let x = read_f64(wkb, base, le)?;
            let y = read_f64(wkb, base + 8, le)?;
            coords.push(geo::Coord { x, y });
        }
        let consumed = 4 + num_points * cs;
        Ok((geo::LineString::new(coords), consumed))
    }

    fn parse_wkb_polygon(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let cs = coord_size(wkb, le);
        let num_rings = read_u32(wkb, off, le)? as usize;
        let mut cursor = off + 4;
        let mut rings = Vec::with_capacity(num_rings);
        for _ in 0..num_rings {
            let (ring, consumed) = parse_wkb_ring(wkb, cursor, le, cs)?;
            rings.push(ring);
            cursor += consumed;
        }
        if rings.is_empty() {
            return Err(TissotError::GeoParquet(
                "WKB Polygon with zero rings".into(),
            ));
        }
        let exterior = rings.remove(0);
        Ok(Geometry::Polygon(geo::Polygon::new(exterior, rings)))
    }

    fn parse_wkb_multipoint(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let num_geoms = read_u32(wkb, off, le)? as usize;
        let mut cursor = off + 4;
        let mut points = Vec::with_capacity(num_geoms);
        for _ in 0..num_geoms {
            if let Geometry::Point(p) = parse_wkb_point(&wkb[cursor..], wkb[cursor] == 1)? {
                points.push(p);
            }
            let sub_cs = coord_size(&wkb[cursor..], wkb[cursor] == 1);
            cursor += coord_offset(&wkb[cursor..], wkb[cursor] == 1) + sub_cs;
        }
        Ok(Geometry::MultiPoint(geo::MultiPoint::new(points)))
    }

    fn parse_wkb_multilinestring(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let num_geoms = read_u32(wkb, off, le)? as usize;
        let mut cursor = off + 4;
        let mut lines = Vec::with_capacity(num_geoms);
        for _ in 0..num_geoms {
            let sub_wkb = &wkb[cursor..];
            let sub_le = sub_wkb[0] == 1;
            if let Geometry::LineString(ls) = parse_wkb_linestring(sub_wkb, sub_le)? {
                let sub_off = coord_offset(sub_wkb, sub_le);
                let sub_cs = coord_size(sub_wkb, sub_le);
                let np = read_u32(sub_wkb, sub_off, sub_le)? as usize;
                cursor += sub_off + 4 + np * sub_cs;
                lines.push(ls);
            }
        }
        Ok(Geometry::MultiLineString(geo::MultiLineString::new(lines)))
    }

    fn parse_wkb_multipolygon(wkb: &[u8], le: bool) -> Result<Geometry> {
        let off = coord_offset(wkb, le);
        let num_geoms = read_u32(wkb, off, le)? as usize;
        let mut cursor = off + 4;
        let mut polygons = Vec::with_capacity(num_geoms);
        for _ in 0..num_geoms {
            let sub_wkb = &wkb[cursor..];
            let sub_le = sub_wkb[0] == 1;
            let sub_off = coord_offset(sub_wkb, sub_le);
            let sub_cs = coord_size(sub_wkb, sub_le);
            let num_rings = read_u32(sub_wkb, sub_off, sub_le)? as usize;
            let mut ring_cursor = sub_off + 4;
            let mut rings = Vec::with_capacity(num_rings);
            for _ in 0..num_rings {
                let (ring, consumed) = parse_wkb_ring(sub_wkb, ring_cursor, sub_le, sub_cs)?;
                rings.push(ring);
                ring_cursor += consumed;
            }
            cursor += ring_cursor;
            if rings.is_empty() {
                return Err(TissotError::GeoParquet(
                    "WKB MultiPolygon contains polygon with zero rings".into(),
                ));
            }
            let exterior = rings.remove(0);
            polygons.push(geo::Polygon::new(exterior, rings));
        }
        Ok(Geometry::MultiPolygon(geo::MultiPolygon::new(polygons)))
    }

    /// Convert an Arrow column value at a given row to a JSON value for properties.
    fn column_value_to_json(
        array: &dyn Array,
        row: usize,
    ) -> Option<serde_json::Value> {
        if array.is_null(row) {
            return None;
        }

        match array.data_type() {
            DataType::Utf8 => {
                let arr = array.as_any().downcast_ref::<StringArray>()?;
                Some(serde_json::Value::String(arr.value(row).to_string()))
            }
            DataType::Int8 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Int8Type>().value(row))),
            DataType::Int16 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Int16Type>().value(row))),
            DataType::Int32 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Int32Type>().value(row))),
            DataType::Int64 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Int64Type>().value(row))),
            DataType::UInt8 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::UInt8Type>().value(row))),
            DataType::UInt16 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::UInt16Type>().value(row))),
            DataType::UInt32 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::UInt32Type>().value(row))),
            DataType::UInt64 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::UInt64Type>().value(row))),
            DataType::Float32 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Float32Type>().value(row))),
            DataType::Float64 => Some(serde_json::json!(array.as_primitive::<arrow::datatypes::Float64Type>().value(row))),
            DataType::Boolean => {
                let arr = array.as_boolean();
                Some(serde_json::Value::Bool(arr.value(row)))
            }
            _ => {
                // For unsupported column types, skip silently.
                None
            }
        }
    }

    /// Compute bounding box from features.
    pub fn compute_bounds(features: &[Feature]) -> Option<[f64; 4]> {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut found = false;

        for f in features {
            if let Some(ref geom) = f.geometry {
                if let Some(rect) = geom.bounding_rect() {
                    found = true;
                    min_x = min_x.min(rect.min().x);
                    min_y = min_y.min(rect.min().y);
                    max_x = max_x.max(rect.max().x);
                    max_y = max_y.max(rect.max().y);
                }
            }
        }

        if found {
            Some([min_x, min_y, max_x, max_y])
        } else {
            None
        }
    }
}

#[cfg(not(feature = "geoparquet"))]
mod inner {
    use crate::core::error::{Result, TissotError};
    use crate::core::rule::Layer;
    use std::path::Path;

    /// Stub reader when the `geoparquet` feature is not enabled.
    ///
    /// Returns an error directing the user to enable the feature flag
    /// or convert their data to a supported format.
    pub fn read(_path: &Path) -> Result<Vec<Layer>> {
        Err(TissotError::UnsupportedFormat(
            "GeoParquet support requires the 'geoparquet' feature flag. \
             Build with `cargo build --features geoparquet`, or convert your data \
             to GeoJSON or FlatGeobuf (e.g., `ogr2ogr output.fgb input.parquet`)."
                .to_string(),
        ))
    }
}

/// Read a GeoParquet (`.parquet`) file and return layers.
///
/// Requires the `geoparquet` feature flag to be enabled. Without it, returns
/// a descriptive error suggesting how to enable support or convert the data.
pub fn read(path: &std::path::Path) -> crate::core::error::Result<Vec<crate::core::rule::Layer>> {
    inner::read(path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn read_nonexistent_file_returns_error() {
        let path = PathBuf::from("/tmp/does_not_exist.parquet");
        let result = super::read(&path);
        assert!(result.is_err());
    }

    #[cfg(not(feature = "geoparquet"))]
    #[test]
    fn stub_returns_unsupported_format_error() {
        let path = PathBuf::from("test.parquet");
        let result = super::read(&path);
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("geoparquet"),
            "Error should mention geoparquet feature: {msg}"
        );
    }

    #[cfg(feature = "geoparquet")]
    mod geoparquet_tests {
        use super::super::inner::*;
        use crate::core::rule::Feature;
        use geo::Geometry;
        use std::collections::HashMap;

        #[test]
        fn compute_bounds_empty() {
            assert!(compute_bounds(&[]).is_none());
        }

        #[test]
        fn compute_bounds_with_point() {
            let features = vec![Feature {
                id: None,
                geometry: Some(Geometry::Point(geo::Point::new(-84.5, 38.0))),
                properties: HashMap::new(),
            }];
            let bounds = compute_bounds(&features);
            assert!(bounds.is_some());
            let b = bounds.unwrap();
            assert!((b[0] - (-84.5)).abs() < f64::EPSILON);
            assert!((b[1] - 38.0).abs() < f64::EPSILON);
        }

        #[test]
        fn compute_bounds_null_geometry() {
            let features = vec![Feature {
                id: None,
                geometry: None,
                properties: HashMap::new(),
            }];
            assert!(compute_bounds(&features).is_none());
        }

        #[test]
        fn parse_wkb_point_little_endian() {
            // WKB Point: byte_order=1 (LE), type=1 (Point), x=-84.5, y=38.0
            let mut wkb = vec![0x01]; // LE
            wkb.extend_from_slice(&1u32.to_le_bytes()); // Point type
            wkb.extend_from_slice(&(-84.5f64).to_le_bytes());
            wkb.extend_from_slice(&38.0f64.to_le_bytes());

            let result = super::super::inner::parse_wkb(&wkb);
            assert!(result.is_ok(), "Failed to parse WKB point: {:?}", result);
            if let Geometry::Point(p) = result.unwrap() {
                assert!((p.x() - (-84.5)).abs() < f64::EPSILON);
                assert!((p.y() - 38.0).abs() < f64::EPSILON);
            } else {
                panic!("Expected Point geometry");
            }
        }

        #[test]
        fn parse_wkb_point_big_endian() {
            let mut wkb = vec![0x00]; // BE
            wkb.extend_from_slice(&1u32.to_be_bytes());
            wkb.extend_from_slice(&(-84.5f64).to_be_bytes());
            wkb.extend_from_slice(&38.0f64.to_be_bytes());

            let result = super::super::inner::parse_wkb(&wkb);
            assert!(result.is_ok());
            if let Geometry::Point(p) = result.unwrap() {
                assert!((p.x() - (-84.5)).abs() < f64::EPSILON);
                assert!((p.y() - 38.0).abs() < f64::EPSILON);
            } else {
                panic!("Expected Point geometry");
            }
        }

        #[test]
        fn parse_wkb_too_short() {
            let wkb = vec![0x01, 0x00, 0x00];
            let result = super::super::inner::parse_wkb(&wkb);
            assert!(result.is_err());
        }

        #[test]
        fn parse_wkb_empty() {
            let result = super::super::inner::parse_wkb(&[]);
            assert!(result.is_err());
        }

        #[test]
        fn parse_wkb_linestring() {
            let mut wkb = vec![0x01]; // LE
            wkb.extend_from_slice(&2u32.to_le_bytes()); // LineString type
            wkb.extend_from_slice(&2u32.to_le_bytes()); // 2 points
            // Point 1: (0.0, 0.0)
            wkb.extend_from_slice(&0.0f64.to_le_bytes());
            wkb.extend_from_slice(&0.0f64.to_le_bytes());
            // Point 2: (1.0, 1.0)
            wkb.extend_from_slice(&1.0f64.to_le_bytes());
            wkb.extend_from_slice(&1.0f64.to_le_bytes());

            let result = super::super::inner::parse_wkb(&wkb);
            assert!(result.is_ok());
            if let Geometry::LineString(ls) = result.unwrap() {
                assert_eq!(ls.0.len(), 2);
            } else {
                panic!("Expected LineString geometry");
            }
        }
    }
}
