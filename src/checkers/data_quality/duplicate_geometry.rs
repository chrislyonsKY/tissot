//! Rule: Detect features with identical geometry (duplicates by geometry hash).

use crate::core::rule::{
    CheckContext, Domain, Finding, Rule, RuleEntry, Severity, SpatialLocation,
};
use std::collections::HashMap;

/// Flags features that share identical geometry, hashed via debug string representation.
///
/// Unlike `DuplicateFeatures` which uses a broader check, this rule specifically
/// targets geometry-level duplicates using a canonical string representation.
pub struct DuplicateGeometry;

impl Default for DuplicateGeometry {
    fn default() -> Self {
        Self
    }
}

impl Rule for DuplicateGeometry {
    fn id(&self) -> &str {
        "data/duplicate-geometry"
    }

    fn name(&self) -> &str {
        "Duplicate Geometry"
    }

    fn domain(&self) -> Domain {
        Domain::DataQuality
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, ctx: &CheckContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for layer in ctx.layers {
            // Hash geometries by their canonical string representation (approximating WKT).
            let mut seen: HashMap<String, Vec<usize>> = HashMap::new();

            for (idx, feature) in layer.features.iter().enumerate() {
                if let Some(ref geom) = feature.geometry {
                    let key = format!("{geom:?}");
                    seen.entry(key).or_default().push(idx);
                }
            }

            for indices in seen.values() {
                if indices.len() > 1 {
                    let dup_count = indices.len();
                    let labels: Vec<String> = indices
                        .iter()
                        .map(|&i| {
                            layer.features[i]
                                .id
                                .clone()
                                .unwrap_or_else(|| format!("#{i}"))
                        })
                        .collect();

                    // Attach the first duplicate's geometry for map rendering.
                    let first_geom = layer.features[indices[0]].geometry.clone();

                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "{dup_count} features with identical geometry in layer '{}': [{}]",
                            layer.name,
                            labels.join(", ")
                        ),
                        location: Some(SpatialLocation::Layer {
                            name: layer.name.clone(),
                        }),
                        geometry: first_geom,
                        metric: Some(dup_count as f64),
                        suggestion: Some(
                            "Review and remove duplicate geometries, or run `tissot fix --dedup`"
                                .into(),
                        ),
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
        factory: || Box::new(DuplicateGeometry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap;

    #[test]
    fn detects_duplicate_geometries() {
        let point = geo::Geometry::Point(geo::Point::new(1.0, 2.0));
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(point.clone()),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(point.clone()),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("c".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(9.0, 9.0))),
                    properties: HashMap::new(),
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

        let rule = DuplicateGeometry;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(findings[0].geometry.is_some());
        assert!(findings[0].message.contains("2 features"));
    }

    #[test]
    fn no_findings_when_unique() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 2.0))),
                    properties: HashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(3.0, 4.0))),
                    properties: HashMap::new(),
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

        let rule = DuplicateGeometry;
        assert!(rule.check(&ctx).is_empty());
    }
}
