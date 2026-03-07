/// Configuration for Tissot — zero-config defaults with optional .tissot.yml override.
use serde::{Deserialize, Serialize};

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// X-Ray engine settings.
    pub xray: XrayConfig,
    /// Checker engine settings.
    pub check: CheckConfig,
    /// Score engine settings.
    pub score: ScoreConfig,
    /// Output settings.
    pub output: OutputConfig,
}

/// X-Ray engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct XrayConfig {
    /// Maximum number of sample points for distortion computation.
    pub max_samples: usize,
    /// Number of optimization targets to recommend.
    pub top_recommendations: usize,
}

/// Checker engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckConfig {
    /// Maximum distortion percentage before flagging as error.
    pub max_distortion_pct: f64,
    /// Topology gap tolerance in CRS units.
    pub topology_gap_tolerance: f64,
    /// Rules to disable (by rule ID).
    pub disabled_rules: Vec<String>,
}

/// Score engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoreConfig {
    /// Category weights (must sum to 1.0).
    pub projection_weight: f64,
    /// Data integrity weight.
    pub data_integrity_weight: f64,
    /// Accessibility weight.
    pub accessibility_weight: f64,
    /// Cloud readiness weight.
    pub cloud_readiness_weight: f64,
    /// Classification weight.
    pub classification_weight: f64,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Whether to open the browser automatically.
    pub open_browser: bool,
    /// Terminal-only output (no browser).
    pub terminal_only: bool,
}

impl Default for XrayConfig {
    fn default() -> Self {
        Self {
            max_samples: 1000,
            top_recommendations: 5,
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            max_distortion_pct: 10.0,
            topology_gap_tolerance: 0.001,
            disabled_rules: Vec::new(),
        }
    }
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            projection_weight: 0.25,
            data_integrity_weight: 0.30,
            accessibility_weight: 0.20,
            cloud_readiness_weight: 0.20,
            classification_weight: 0.05,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            open_browser: true,
            terminal_only: false,
        }
    }
}

impl Config {
    /// Load config from a YAML file path, falling back to defaults.
    pub fn load(path: Option<&str>) -> super::error::Result<Self> {
        match path {
            Some(p) => {
                let content = std::fs::read_to_string(p)?;
                serde_yaml::from_str(&content).map_err(|e| {
                    super::error::TissotError::Config(format!("Failed to parse config: {e}"))
                })
            }
            None => Ok(Self::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.xray.max_samples, 1000);
        assert_eq!(cfg.score.projection_weight, 0.25);
        assert!(cfg.output.open_browser);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.xray.max_samples, cfg.xray.max_samples);
    }
}
