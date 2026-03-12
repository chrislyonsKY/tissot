//! Rule: Check if a dataset has too many visually similar categories.
//!
//! When a classification field has more than ~12 unique values, it becomes
//! very difficult for map readers to distinguish the colors in a choropleth
//! or categorical map. This rule flags fields that exceed the threshold.

use std::collections::HashSet;

use crate::core::rule::{CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation};

/// Maximum number of unique categorical values before color distinction
/// becomes difficult for human perception.
const DEFAULT_MAX_CATEGORIES: usize = 12;

/// Checks if any classification/categorical field has too many unique values,
/// making it hard to assign visually distinct colors.
pub struct ColorContrast;

impl Default for ColorContrast {
    fn default() -> Self {
        Self
    }
}

impl Rule for ColorContrast {
    fn id(&self) -> &str {
        "cartography/color-contrast"
    }

    fn name(&self) -> &str {
        "Color Contrast"
    }

    fn domain(&self) -> Domain {
        Domain::Cartography
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn tags(&self) -> &[&str] {
        &["cartography", "color", "accessibility"]
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            if layer.features.is_empty() {
                continue;
            }

            // Collect all string-valued property keys across features.
            let mut field_values: std::collections::HashMap<String, HashSet<String>> =
                std::collections::HashMap::new();

            for feature in &layer.features {
                for (key, value) in &feature.properties {
                    // Only consider string and integer values as categorical candidates.
                    let cat_value = match value {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Number(n) => {
                            // Only treat integers as categorical (not floats).
                            if n.is_i64() || n.is_u64() {
                                Some(n.to_string())
                            } else {
                                None
                            }
                        }
                        serde_json::Value::Bool(b) => Some(b.to_string()),
                        _ => None,
                    };

                    if let Some(v) = cat_value {
                        field_values.entry(key.clone()).or_default().insert(v);
                    }
                }
            }

            // Check each field's unique count.
            for (field_name, unique_values) in &field_values {
                let count = unique_values.len();
                if count > DEFAULT_MAX_CATEGORIES {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "Field '{}' in layer '{}' has {} unique values, exceeding the {} category limit for distinguishable colors",
                            field_name, layer.name, count, DEFAULT_MAX_CATEGORIES
                        ),
                        location: Some(SpatialLocation::Layer {
                            name: layer.name.clone(),
                        }),
                        geometry: None,
                        metric: Some(count as f64),
                        suggestion: Some(format!(
                            "Group values into {} or fewer categories, or use a graduated/continuous color ramp instead of categorical colors",
                            DEFAULT_MAX_CATEGORIES
                        )),
                        fixable: false,
                    });
                }
            }
        }

        findings
    }

    fn score_weight(&self) -> f64 {
        0.6
    }
}

inventory::submit! {
    RuleEntry {
        factory: || Box::new(ColorContrast),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap;

    fn make_feature(class: &str) -> Feature {
        let mut props = HashMap::new();
        props.insert(
            "land_use".to_string(),
            serde_json::Value::String(class.to_string()),
        );
        Feature {
            id: None,
            geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
            properties: props,
        }
    }

    #[test]
    fn flags_too_many_categories() {
        let categories: Vec<&str> = vec![
            "residential", "commercial", "industrial", "agricultural",
            "forest", "water", "wetland", "barren", "grassland",
            "shrubland", "snow_ice", "developed_low", "developed_high",
        ];
        assert!(categories.len() > DEFAULT_MAX_CATEGORIES);

        let features: Vec<Feature> = categories.into_iter().map(make_feature).collect();

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

        let rule = ColorContrast;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(findings[0].message.contains("land_use"));
        assert!(findings[0].metric.is_some());
    }

    #[test]
    fn no_finding_under_threshold() {
        let categories: Vec<&str> = vec!["urban", "rural", "water"];
        let features: Vec<Feature> = categories.into_iter().map(make_feature).collect();

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

        let rule = ColorContrast;
        assert!(rule.check(&ctx).is_empty());
    }

    #[test]
    fn rule_metadata() {
        let rule = ColorContrast;
        assert_eq!(rule.id(), "cartography/color-contrast");
        assert_eq!(rule.domain(), Domain::Cartography);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }

    #[test]
    fn ignores_float_fields() {
        let mut props = HashMap::new();
        props.insert(
            "temperature".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(23.5).unwrap()),
        );
        let features: Vec<Feature> = (0..20)
            .map(|i| {
                let mut p = HashMap::new();
                p.insert(
                    "temperature".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(20.0 + i as f64 * 0.5).unwrap(),
                    ),
                );
                Feature {
                    id: None,
                    geometry: Some(geo::Geometry::Point(geo::Point::new(0.0, 0.0))),
                    properties: p,
                }
            })
            .collect();

        let layer = Layer {
            name: "temps".into(),
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

        let rule = ColorContrast;
        // Float fields should not be treated as categorical.
        assert!(rule.check(&ctx).is_empty());
    }
}
