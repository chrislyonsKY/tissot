//! Projection diagnostic rules — CRS suitability checks.

pub mod area_distortion;
pub mod datum_mismatch;
pub mod distance_distortion;
pub mod high_distortion;
pub mod missing_crs;

pub use area_distortion::AreaDistortion;
pub use datum_mismatch::DatumMismatch;
pub use distance_distortion::DistanceDistortion;
