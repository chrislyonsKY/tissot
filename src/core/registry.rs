/// Rule registry — discovers and manages diagnostic rules via inventory.
use super::rule::{Rule, RuleEntry};

/// Registry of all available diagnostic rules.
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Build the registry from all rules registered via `inventory::submit!`.
    pub fn discover() -> Self {
        let rules: Vec<Box<dyn Rule>> = inventory::iter::<RuleEntry>
            .into_iter()
            .map(|entry| (entry.factory)())
            .collect();
        log::info!("Discovered {} rules", rules.len());
        Self { rules }
    }

    /// Get all registered rules.
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Get rules filtered by domain.
    pub fn rules_for_domain(&self, domain: super::rule::Domain) -> Vec<&dyn Rule> {
        self.rules
            .iter()
            .filter(|r| r.domain() == domain)
            .map(|r| r.as_ref())
            .collect()
    }

    /// Get a rule by ID.
    pub fn get(&self, id: &str) -> Option<&dyn Rule> {
        self.rules.iter().find(|r| r.id() == id).map(|r| r.as_ref())
    }

    /// Number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_discover() {
        let registry = RuleRegistry::discover();
        // Discovery should succeed and return a valid registry object.
        let _ = registry.len();
    }
}
