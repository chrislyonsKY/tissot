//! Explain module — CRS reference database and human-readable explanations.
//!
//! Provides a curated database of common EPSG codes with preservation
//! properties, and functions to explain any CRS in plain English.

pub mod crs_database;
pub mod properties;

pub use crs_database::{CrsEntry, lookup};
pub use properties::{CrsExplanation, explain_crs};
