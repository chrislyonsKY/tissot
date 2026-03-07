//! Cloud-native format validation rules.
//!
//! Aligned with the CNG (Cloud-Native Geospatial) Formats Guide.
//! Validates format choice, metadata, spatial indexing, compression,
//! file size, and multi-file integrity.

pub mod compression;
pub mod crs_metadata;
pub mod file_size;
pub mod format_recommendation;
pub mod multi_file_integrity;
pub mod spatial_index;

pub use compression::Compression;
pub use crs_metadata::CrsMetadata;
pub use file_size::FileSize;
pub use format_recommendation::FormatRecommendation;
pub use multi_file_integrity::MultiFileIntegrity;
pub use spatial_index::SpatialIndex;
