/// Core types, traits, and configuration for Tissot.
pub mod config;
pub mod error;
pub mod registry;
pub mod report;
pub mod rule;

pub use config::Config;
pub use error::TissotError;
pub use registry::RuleRegistry;
pub use report::ReportData;
pub use rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};
