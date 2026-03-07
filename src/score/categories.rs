/// Score categories and their mappings.
use serde::Serialize;

/// Score category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Projection quality.
    Projection,
    /// Data integrity (topology, schema, nulls, duplicates).
    DataIntegrity,
    /// Accessibility (color contrast, symbology).
    Accessibility,
    /// Classification quality.
    Classification,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Projection => write!(f, "Projection"),
            Category::DataIntegrity => write!(f, "Data Integrity"),
            Category::Accessibility => write!(f, "Accessibility"),
            Category::Classification => write!(f, "Classification"),
        }
    }
}

impl Category {
    /// Get the rule ID prefix that maps to this category.
    pub fn rule_prefix(&self) -> &str {
        match self {
            Category::Projection => "projection",
            Category::DataIntegrity => "data_quality",
            Category::Accessibility => "cartography",
            Category::Classification => "classification",
        }
    }
}

/// Score for a single category.
#[derive(Debug, Serialize)]
pub struct CategoryScore {
    /// Which category.
    pub category: Category,
    /// Score (0-100).
    pub score: u32,
    /// Weight in overall score.
    pub weight: f64,
    /// Number of findings in this category.
    pub finding_count: usize,
    /// Letter grade for this category.
    pub grade: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_display() {
        assert_eq!(Category::Projection.to_string(), "Projection");
        assert_eq!(Category::DataIntegrity.to_string(), "Data Integrity");
    }

    #[test]
    fn rule_prefix_mapping() {
        assert_eq!(Category::Projection.rule_prefix(), "projection");
        assert_eq!(Category::DataIntegrity.rule_prefix(), "data_quality");
    }
}
