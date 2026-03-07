//! Data quality diagnostic rules — structural integrity checks.

pub mod duplicate_features;
pub mod duplicate_geometry;
pub mod empty_dataset;
pub mod extent_bounds;
pub mod null_geometry;
pub mod schema_validation;
pub mod self_intersection;
pub mod topology_gaps;
pub mod topology_overlaps;

pub use duplicate_geometry::DuplicateGeometry;
pub use extent_bounds::ExtentBounds;
pub use null_geometry::NullGeometry;
pub use schema_validation::SchemaValidation;
pub use self_intersection::SelfIntersection;
pub use topology_gaps::TopologyGaps;
pub use topology_overlaps::TopologyOverlaps;
