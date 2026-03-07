/// Rule: Detect duplicate features by geometry.
use crate::core::rule::{CheckContext, Domain, Finding, Rule, Severity, SpatialLocation};
use std::collections::HashMap;

/// Flags features with identical geometry (potential duplicates).
pub struct DuplicateFeatures;

impl Default for DuplicateFeatures {
    fn default() -> Self {
        Self
    }
}

impl Rule for DuplicateFeatures {
    fn id(&self) -> &str {
        "data_quality/duplicate-features"
    }

    fn name(&self) -> &str {
        "Duplicate Features"
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
            // Hash geometries by their debug representation for simple dedup.
            let mut seen: HashMap<String, Vec<usize>> = HashMap::new();

            for (idx, feature) in layer.features.iter().enumerate() {
                if let Some(ref geom) = feature.geometry {
                    let key = format!("{geom:?}");
                    seen.entry(key).or_default().push(idx);
                }
            }

            for indices in seen.values() {
                if indices.len() > 1 {
                    let labels: Vec<String> = indices
                        .iter()
                        .map(|&i| {
                            layer.features[i]
                                .id
                                .clone()
                                .unwrap_or_else(|| format!("#{i}"))
                        })
                        .collect();

                    let first_idx = indices[0];
                    let geom = layer.features[first_idx].geometry.clone();

                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        severity: self.default_severity(),
                        message: format!(
                            "{} features with identical geometry in layer '{}': [{}]",
                            indices.len(),
                            layer.name,
                            labels.join(", ")
                        ),
                        location: Some(SpatialLocation::Layer {
                            name: layer.name.clone(),
                        }),
                        geometry: geom,
                        metric: Some(indices.len() as f64),
                        suggestion: Some(
                            "Review and remove duplicate features, or run `tissot fix --dedup`"
                                .into(),
                        ),
                        fixable: true,
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
    crate::core::rule::RuleEntry {
        factory: || Box::new(DuplicateFeatures),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::rule::{Feature, Layer};
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn detects_duplicate_points() {
        let geom = geo::Geometry::Point(geo::Point::new(1.0, 2.0));
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(geom.clone()),
                    properties: StdHashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(geom),
                    properties: StdHashMap::new(),
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

        let rule = DuplicateFeatures;
        let findings = rule.check(&ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("identical geometry"));
    }

    #[test]
    fn no_duplicates_when_unique() {
        let layer = Layer {
            name: "test".into(),
            crs: Some("EPSG:4326".into()),
            features: vec![
                Feature {
                    id: Some("a".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(1.0, 2.0))),
                    properties: StdHashMap::new(),
                },
                Feature {
                    id: Some("b".into()),
                    geometry: Some(geo::Geometry::Point(geo::Point::new(3.0, 4.0))),
                    properties: StdHashMap::new(),
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

        let rule = DuplicateFeatures;
        assert!(rule.check(&ctx).is_empty());
    }
}
