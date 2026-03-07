//! Rule: Validate that feature attributes have consistent types across the layer.
//!
//! Infers the expected schema from the first feature's properties and checks
//! all subsequent features for type mismatches and unexpected null values.

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};
use std::collections::HashMap;

/// Flags features whose attribute types differ from the inferred layer schema.
pub struct SchemaValidation;

impl Default for SchemaValidation {
    fn default() -> Self {
        Self
    }
}

/// Inferred field type from JSON values.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FieldType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Null => write!(f, "null"),
            FieldType::Bool => write!(f, "boolean"),
            FieldType::Number => write!(f, "number"),
            FieldType::String => write!(f, "string"),
            FieldType::Array => write!(f, "array"),
            FieldType::Object => write!(f, "object"),
        }
    }
}

/// Classify a serde_json::Value into a FieldType.
fn classify_value(val: &serde_json::Value) -> FieldType {
    match val {
        serde_json::Value::Null => FieldType::Null,
        serde_json::Value::Bool(_) => FieldType::Bool,
        serde_json::Value::Number(_) => FieldType::Number,
        serde_json::Value::String(_) => FieldType::String,
        serde_json::Value::Array(_) => FieldType::Array,
        serde_json::Value::Object(_) => FieldType::Object,
    }
}

impl Rule for SchemaValidation {
    fn id(&self) -> &str {
        "data/schema-validate"
    }

    fn name(&self) -> &str {
        "Schema Validation"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            if layer.features.is_empty() {
                continue;
            }

            // Infer schema from the first feature's properties.
            let first = &layer.features[0];
            let mut schema: HashMap<String, FieldType> = HashMap::new();
            let mut non_nullable: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            for (key, val) in &first.properties {
                let ft = classify_value(val);
                if ft != FieldType::Null {
                    non_nullable.insert(key.clone());
                }
                schema.insert(key.clone(), ft);
            }

            // Check subsequent features against inferred schema.
            for (idx, feature) in layer.features.iter().enumerate().skip(1) {
                let feature_label = feature
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("#{idx}"));

                for (key, expected_type) in &schema {
                    match feature.properties.get(key) {
                        None => {
                            // Missing field that existed in schema.
                            findings.push(Finding {
                                rule_id: self.id().to_string(),
                                severity: self.default_severity(),
                                message: format!(
                                    "Feature {feature_label} in layer '{}' is missing field '{key}'",
                                    layer.name
                                ),
                                location: Some(SpatialLocation::Feature {
                                    id: feature_label.clone(),
                                }),
                                geometry: feature.geometry.clone(),
                                metric: None,
                                suggestion: Some(format!(
                                    "Add the missing field '{key}' or update the schema"
                                )),
                                fixable: false,
                            });
                        }
                        Some(val) => {
                            let actual_type = classify_value(val);
                            // Allow null for nullable fields; flag null in non-nullable.
                            if actual_type == FieldType::Null && non_nullable.contains(key) {
                                findings.push(Finding {
                                    rule_id: self.id().to_string(),
                                    severity: Severity::Warning,
                                    message: format!(
                                        "Feature {feature_label} in layer '{}' has unexpected null for non-nullable field '{key}'",
                                        layer.name
                                    ),
                                    location: Some(SpatialLocation::Feature {
                                        id: feature_label.clone(),
                                    }),
                                    geometry: feature.geometry.clone(),
                                    metric: None,
                                    suggestion: Some(format!(
                                        "Provide a value for field '{key}' or mark the field as nullable"
                                    )),
                                    fixable: false,
                                });
                            } else if actual_type != FieldType::Null
                                && *expected_type != FieldType::Null
                                && actual_type != *expected_type
                            {
                                findings.push(Finding {
                                    rule_id: self.id().to_string(),
                                    severity: self.default_severity(),
                                    message: format!(
                                        "Feature {feature_label} in layer '{}': field '{key}' has type {actual_type}, expected {expected_type}",
                                        layer.name
                                    ),
                                    location: Some(SpatialLocation::Feature {
                                        id: feature_label.clone(),
                                    }),
                                    geometry: feature.geometry.clone(),
                                    metric: None,
                                    suggestion: Some(format!(
                                        "Convert field '{key}' to type {expected_type}"
                                    )),
                                    fixable: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.8
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(SchemaValidation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap;

    #[test]
    fn detects_type_mismatch() {
        let mut props1 = HashMap::new();
        props1.insert("name".to_string(), serde_json::Value::String("hello".into()));
        props1.insert("value".to_string(), serde_json::json!(42));

        let mut props2 = HashMap::new();
        props2.insert("name".to_string(), serde_json::json!(123)); // Wrong type
        props2.insert("value".to_string(), serde_json::json!(99));

        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: props1,
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 1.0))),
                    properties: props2,
                },
            ],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = SchemaValidation;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("type"));
        assert!(findings[0].geometry.is_some());
    }

    #[test]
    fn detects_unexpected_null() {
        let mut props1 = HashMap::new();
        props1.insert("name".to_string(), serde_json::Value::String("hello".into()));

        let mut props2 = HashMap::new();
        props2.insert("name".to_string(), serde_json::Value::Null);

        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: props1,
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 1.0))),
                    properties: props2,
                },
            ],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = SchemaValidation;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("unexpected null"));
    }

    #[test]
    fn no_findings_consistent_schema() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), serde_json::Value::String("a".into()));

        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("1".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: props.clone(),
                },
                Feature {
                    id: Some("2".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 1.0))),
                    properties: props,
                },
            ],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = SchemaValidation;
        assert!(rule.check(&ctx).is_empty());
    }
}
