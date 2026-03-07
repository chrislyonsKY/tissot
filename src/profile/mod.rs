//! Profile module — dataset summary and format detection.
//!
//! Generates a structural overview of geospatial data files including
//! layer stats, format info, and metadata for quick inspection.

pub mod format_info;
pub mod summary;

pub use format_info::{FormatInfo, detect_format};
pub use summary::{LayerProfile, ProfileSummary, generate_profile};
