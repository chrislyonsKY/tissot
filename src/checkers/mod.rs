/// Checker engine — runs diagnostic rules against geospatial data.
pub mod cartography;
pub mod cloud;
pub mod data_quality;
pub mod projection;

use crate::core::config::Config;
use crate::core::registry::RuleRegistry;
use crate::core::rule::{CheckContext, Domain, Finding, Layer};

/// Run all registered rules against the given layers.
pub fn run_checks(
    layers: &[Layer],
    config: &Config,
    file_path: &str,
    domain_filter: Option<Domain>,
) -> Vec<Finding> {
    let registry = RuleRegistry::discover();
    let ctx = CheckContext {
        layers,
        config,
        file_path,
    };

    let mut findings = Vec::new();
    for rule in registry.rules() {
        // Skip disabled rules.
        if config.check.disabled_rules.contains(&rule.id().to_string()) {
            continue;
        }
        // Apply domain filter if specified.
        if let Some(domain) = domain_filter {
            if rule.domain() != domain {
                continue;
            }
        }
        log::debug!("Running rule: {} ({})", rule.id(), rule.name());
        let mut rule_findings = rule.check(&ctx);
        findings.append(&mut rule_findings);
    }

    // Sort by severity (errors first).
    findings.sort_by(|a, b| b.severity.cmp(&a.severity));

    log::info!(
        "Check complete: {} findings from {} rules",
        findings.len(),
        registry.len()
    );

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_checks_empty_layers() {
        let config = Config::default();
        let findings = run_checks(&[], &config, "empty.geojson", None);
        // With no layers, rules may still produce findings (e.g., empty dataset).
        // Just ensure it doesn't panic.
        let _ = findings;
    }
}
