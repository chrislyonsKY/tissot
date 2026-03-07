/// GeoJSON helpers for report visualization.
use crate::core::rule::Finding;

/// Convert findings with geometry into a GeoJSON FeatureCollection.
pub fn findings_to_geojson(findings: &[Finding]) -> serde_json::Value {
    let features: Vec<geojson::Feature> = findings
        .iter()
        .filter_map(|finding| {
            let geometry = finding.geometry.as_ref()?;
            let mut properties = serde_json::Map::new();
            properties.insert(
                "rule_id".to_string(),
                serde_json::Value::String(finding.rule_id.clone()),
            );
            properties.insert(
                "severity".to_string(),
                serde_json::Value::String(finding.severity.to_string()),
            );
            properties.insert(
                "message".to_string(),
                serde_json::Value::String(finding.message.clone()),
            );
            if let Some(metric) = finding.metric {
                properties.insert("metric".to_string(), serde_json::json!(metric));
            }

            Some(geojson::Feature {
                bbox: None,
                geometry: Some(geojson::Geometry::from(geometry)),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            })
        })
        .collect();

    serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_findings_without_geometry() {
        let findings = vec![Finding {
            rule_id: "test/rule".to_string(),
            severity: crate::core::rule::Severity::Info,
            message: "hello".to_string(),
            location: None,
            geometry: None,
            metric: None,
            suggestion: None,
            fixable: false,
        }];

        let geojson = findings_to_geojson(&findings);
        assert_eq!(geojson["features"].as_array().map(|f| f.len()), Some(0));
    }
}
