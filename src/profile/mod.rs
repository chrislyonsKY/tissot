//! Profile module — dataset summary and format detection.
//!
//! Generates a structural overview of geospatial data files including
//! layer stats, format info, and metadata for quick inspection.

pub mod format_info;
pub mod summary;

pub use format_info::{detect_format, FormatInfo};
pub use summary::{generate_profile, LayerProfile, ProfileSummary};
