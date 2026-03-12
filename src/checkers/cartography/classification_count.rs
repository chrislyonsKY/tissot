//! Rule: Check if categorical fields have appropriate unique value counts for thematic mapping.
//!
//! Too few categories (< 3) make a map uninformative, while too many (> 8)
//! make it hard to read. This is distinct from color-contrast (which checks
//! the hard perceptual limit); this rule targets the cartographic best-practice
//! sweet spot for thematic maps.

use std::collections::{HashMap, HashSet};

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};

/// Minimum recommended categories for a meaningful thematic map.
const MIN_CATEGORIES: usize = 3;

/// Maximum recommended categories for a readable thematic map.
const MAX_CATEGORIES: usize = 8;

/// Checks if categorical fields have too few or too many unique values
/// for effective thematic mapping.
pub struct ClassificationCount;

impl Default for ClassificationCount {
    fn default() -> Self {
        Self
    }
}

/// Determine if a JSON value is categorical (string, integer, or boolean).
fn categorical_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                Some(n.to_string())
            } else {
                None
            }
        }
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

impl Rule for ClassificationCount {
    fn id(&self) -> &str {
        "cartography/classification-count"
    }

    fn name(&self) -> &str {
        "Classification Count"
    }

    fn domain(&self) -> Domain {
        Domain::Cartography
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn tags(&self) -> &[&str] {
        &["cartography", "classification", "thematic"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            if layer.features.is_empty() {
                continue;
            }

            // Gather unique categorical values per field.
            let mut field_values: HashMap<String, HashSet<String>> = HashMap::new();

            for feature in &layer.features {
                for (key, value) in &feature.properties {
                    if let Some(v) = categorical_value(value) {
                        field_values.entry(key.clone()).or_default().insert(v);
                    }
                }
            }

            for (field_name, unique_values) in &field_values {
                let count = unique_values.len();

                if count < MIN_CATEGORIES {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Field '{}' in layer '{}' has only {} unique value{} — too few for an effective thematic map",
                            field_name,
                            layer.name,
                            count,
                            if count == 1 { "" } else { "s" },
                        ),
                        location: Some(SpatialLocation::Layer {
                            name: layer.name.clone(),
                        }),
                        geometry: None,
                        metric: Some(count as f64),
                        suggestion: Some(
                            "Consider combining with other attributes or using a different visualization method (e.g., proportional symbols)".to_string()
                        ),
                        fixable: false,
                    });
                } else if count > MAX_CATEGORIES {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Field '{}' in layer '{}' has {} unique values — consider grouping into {} or fewer classes for readability",
                            field_name, layer.name, count, MAX_CATEGORIES
                        ),
                        location: Some(SpatialLocation::Layer {
                            name: layer.name.clone(),
                        }),
                        geometry: None,
                        metric: Some(count as f64),
                        suggestion: Some(format!(
                            "Use natural breaks (Jenks), quantile, or manual classification to reduce to {MAX_CATEGORIES} or fewer classes"
                        )),
                        fixable: false,
                    });
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.4
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(ClassificationCount),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};

    fn make_feature_with_class(class: &str) -> Feature {
        let mut props = HashMap::new();
        props.insert(
            "category".to_string(),
            serde_json::Value::String(class.to_string()),
        );
        Feature {
            id: None,
            geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
            properties: props,
        }
    }

    #[test]
    fn flags_too_few_categories() {
        let features = vec![
            make_feature_with_class("urban"),
            make_feature_with_class("urban"),
            make_feature_with_class("rural"),
        ];

        let layer = Layer {
            name: "zones".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ClassificationCount;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("too few"));
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn flags_too_many_categories() {
        let classes = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
        assert!(classes.len() > MAX_CATEGORIES);

        let features: Vec<Feature> = classes.into_iter().map(make_feature_with_class).collect();

        let layer = Layer {
            name: "land_use".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ClassificationCount;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("grouping"));
    }

    #[test]
    fn no_finding_in_sweet_spot() {
        let classes = vec!["low", "medium", "high", "very_high"];
        let features: Vec<Feature> = classes.into_iter().map(make_feature_with_class).collect();

        let layer = Layer {
            name: "risk".into(),
            crs: Some("EPSG:4326".into()),
            features,
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ClassificationCount;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn rule_metadata() {
        let rule = ClassificationCount;
        assert_eq!(rule.id(), "cartography/classification-count");
        assert_eq!(rule.domain(), Domain::Cartography);
        assert_eq!(rule.default_severity(), Severity::Info);
    }

    #[test]
    fn handles_empty_layer() {
        let layer = Layer {
            name: "empty".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![],
            bounds: None,
        };

        let config = Config::default();
        let ctx = CheckContext {
            layers: &[layer],
            config: &config,
            file_path: "test.geojson",
        };

        let rule = ClassificationCount;
        assert!(rule.check(&ctx).is_empty());
    }
}
