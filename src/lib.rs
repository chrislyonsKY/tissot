pub mod cartography;
pub mod checkers;
/// Tissot — Visual-first geospatial diagnostics engine.
///
/// Provides projection x-ray, cartographic linting, spatial diffing,
/// and autofix capabilities for geospatial data.
pub mod core;
pub mod diff;
pub mod explain;
pub mod fix;
pub mod io;
pub mod profile;
pub mod report;
pub mod score;
pub mod watch;
pub mod xray;

#[cfg(feature = "python")]
mod python;
