/// The Rule trait, diagnostic types, and check context for Tissot's checker engine.
use geo::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error;

/// Diagnostic domain — groups related rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// CRS distortion, datum mismatch, EPSG validity.
    Projection,
    /// Topology, schema, null geometry, duplicates, extent.
    DataQuality,
    /// Color contrast, label overlap, classification, symbology.
    Cartography,
    /// Geometry change detection, feature add/remove, attribute diff.
    Diff,
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Domain::Projection => write!(f, "projection"),
            Domain::DataQuality => write!(f, "data_quality"),
            Domain::Cartography => write!(f, "cartography"),
            Domain::Diff => write!(f, "diff"),
        }
    }
}

/// Severity level for a diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational note — does not affect score significantly.
    Info,
    /// Warning — may indicate a problem.
    Warning,
    /// Error — definite problem that should be fixed.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A spatial location reference for a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpatialLocation {
    /// A bounding box [min_x, min_y, max_x, max_y].
    BoundingBox {
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    },
    /// A specific feature by ID.
    Feature { id: String },
    /// A layer by name.
    Layer { name: String },
}

/// A single diagnostic finding produced by a Rule.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Rule that produced this finding.
    pub rule_id: String,
    /// Severity of the finding.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Spatial location reference.
    pub location: Option<SpatialLocation>,
    /// Geometry for rendering on the map.
    #[serde(skip)]
    pub geometry: Option<Geometry>,
    /// Quantitative metric (e.g., distortion percentage).
    pub metric: Option<f64>,
    /// Suggested fix or next step.
    pub suggestion: Option<String>,
    /// Whether this finding can be auto-fixed.
    pub fixable: bool,
}

/// A layer of geospatial features with metadata.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Layer name.
    pub name: String,
    /// CRS identifier (e.g., "EPSG:4326").
    pub crs: Option<String>,
    /// Feature geometries.
    pub features: Vec<Feature>,
    /// Bounding box [min_x, min_y, max_x, max_y].
    pub bounds: Option<[f64; 4]>,
}

/// A single geospatial feature.
#[derive(Debug, Clone)]
pub struct Feature {
    /// Feature identifier.
    pub id: Option<String>,
    /// Feature geometry.
    pub geometry: Option<Geometry>,
    /// Feature properties / attributes.
    pub properties: HashMap<String, serde_json::Value>,
}

/// Context passed to rules during checking.
pub struct CheckContext<'a> {
    /// Layers loaded from the input file.
    pub layers: &'a [Layer],
    /// Configuration for thresholds and rule options.
    pub config: &'a super::config::Config,
    /// Path to the input file.
    pub file_path: &'a str,
}

/// The core Rule trait — all diagnostic rules must implement this.
///
/// Rules are registered via the `inventory` crate for automatic discovery.
pub trait Rule: Send + Sync {
    /// Unique identifier for the rule (e.g., "data_quality/null-geometry").
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Domain this rule belongs to.
    fn domain(&self) -> Domain;

    /// Default severity for findings from this rule.
    fn default_severity(&self) -> Severity;

    /// Run the diagnostic check and return findings.
    fn check(&self, ctx: &CheckContext) -> Vec<Finding>;

    /// Tags for filtering (optional).
    fn tags(&self) -> &[&str] {
        &[]
    }

    /// Whether this rule supports autofix.
    fn can_fix(&self) -> bool {
        false
    }

    /// Apply autofix for this rule's findings.
    fn fix(&self, _ctx: &CheckContext) -> error::Result<Vec<Finding>> {
        Err(error::TissotError::NoAutofix(self.id().to_string()))
    }

    /// Weight for score calculation (0.0 to 1.0).
    fn score_weight(&self) -> f64 {
        1.0
    }
}

/// Factory for creating rule instances — used for inventory-based registration.
///
/// `inventory` collects static items, and `Box::new()` isn't const,
/// so we store a function pointer that produces the boxed rule at runtime.
pub struct RuleEntry {
    /// Factory function that creates the boxed rule.
    pub factory: fn() -> Box<dyn Rule>,
}

inventory::collect!(RuleEntry);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_display() {
        assert_eq!(Domain::Projection.to_string(), "projection");
        assert_eq!(Domain::DataQuality.to_string(), "data_quality");
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn finding_creation() {
        let f = Finding {
            rule_id: "test/rule".into(),
            severity: Severity::Warning,
            message: "test finding".into(),
            location: None,
            geometry: None,
            metric: Some(0.5),
            suggestion: Some("fix it".into()),
            fixable: false,
        };
        assert_eq!(f.rule_id, "test/rule");
        assert_eq!(f.metric, Some(0.5));
    }

    #[test]
    fn severity_serialization() {
        let json = serde_json::to_string(&Severity::Error).unwrap();
        assert_eq!(json, "\"error\"");
    }
}
