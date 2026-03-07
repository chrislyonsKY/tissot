/// Tissot CLI — geospatial diagnostics engine.
///
/// Visual-first projection analysis, cartographic linting, scoring, and autofix.
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tissot",
    version,
    about = "⊕ Tissot — Geospatial diagnostics engine",
    long_about = "Projection x-ray, cartographic linting, spatial diffing, and autofix.\nVisual-first output — opens interactive maps in your browser."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Projection distortion analysis — the hero feature.
    Xray {
        /// Input geospatial file (GeoJSON, GeoPackage, Shapefile).
        file: PathBuf,

        /// Include CRS recommendations in the report.
        #[arg(long)]
        recommend: bool,

        /// Output to terminal instead of opening browser.
        #[arg(long)]
        terminal: bool,

        /// Output machine-readable JSON.
        #[arg(long)]
        json: bool,

        /// Target CRS (EPSG code) to analyze. Defaults to the file's CRS.
        #[arg(long)]
        crs: Option<String>,
    },

    /// Run diagnostic checks on geospatial data.
    Check {
        /// Input geospatial file.
        file: PathBuf,

        /// Only run checks for a specific domain (projection, quality, cartography).
        #[arg(long)]
        domain: Option<String>,

        /// Output to terminal instead of opening browser.
        #[arg(long)]
        terminal: bool,

        /// Output machine-readable JSON.
        #[arg(long)]
        json: bool,

        /// Output SARIF for CI/CD code scanning.
        #[arg(long)]
        sarif: bool,
    },

    /// Generate a map quality score (0-100).
    Score {
        /// Input geospatial file or project.
        file: PathBuf,

        /// Generate an SVG badge at the given path.
        #[arg(long)]
        badge: Option<PathBuf>,

        /// Output to terminal instead of opening browser.
        #[arg(long)]
        terminal: bool,

        /// Output machine-readable JSON.
        #[arg(long)]
        json: bool,
    },

    /// Apply automatic fixes to a dataset.
    Fix {
        /// Input geospatial file.
        file: PathBuf,

        /// Reproject to the given CRS (for example EPSG:5070).
        #[arg(long)]
        reproject: Option<String>,

        /// Heal simple topology problems (null/duplicate geometry cleanup).
        #[arg(long)]
        topology: bool,

        /// Modify the input file directly.
        #[arg(long)]
        in_place: bool,

        /// Output machine-readable JSON.
        #[arg(long)]
        json: bool,
    },

    /// Compare two versions of a dataset.
    Diff {
        /// Left/before dataset path.
        left: PathBuf,

        /// Right/after dataset path.
        right: PathBuf,

        /// Output machine-readable JSON.
        #[arg(long)]
        json: bool,

        /// Output to terminal instead of opening browser.
        #[arg(long)]
        terminal: bool,
    },

    /// Watch a directory and stream diagnostic updates.
    Watch {
        /// Directory to monitor.
        dir: PathBuf,
    },

    /// Create a starter .tissot.yml config.
    Init {
        /// Overwrite existing config file.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();
    let runtime_config = load_runtime_config().context("Failed to load configuration")?;

    match cli.command {
        Commands::Xray {
            file,
            recommend,
            terminal,
            json,
            crs,
        } => cmd_xray(file, recommend, terminal, json, crs, &runtime_config),
        Commands::Check {
            file,
            domain,
            terminal,
            json,
            sarif,
        } => cmd_check(file, domain, terminal, json, sarif, &runtime_config),
        Commands::Score {
            file,
            badge,
            terminal,
            json,
        } => cmd_score(file, badge, terminal, json, &runtime_config),
        Commands::Fix {
            file,
            reproject,
            topology,
            in_place,
            json,
        } => cmd_fix(file, reproject, topology, in_place, json, &runtime_config),
        Commands::Diff {
            left,
            right,
            json,
            terminal,
        } => cmd_diff(left, right, json, terminal),
        Commands::Watch { dir } => cmd_watch(dir, &runtime_config),
        Commands::Init { force } => cmd_init(force),
    }
}

fn cmd_xray(
    file: PathBuf,
    recommend: bool,
    terminal: bool,
    json: bool,
    crs: Option<String>,
    runtime_config: &tissot::core::config::Config,
) -> Result<()> {
    let layers = tissot::io::read_file(&file).context("Failed to read input file")?;

    let layer = layers.first().context("No layers found in input file")?;

    let _target_crs = crs
        .or_else(|| layer.crs.clone())
        .unwrap_or_else(|| "EPSG:4326".to_string());

    let config = tissot::core::config::Config {
        xray: tissot::core::config::XrayConfig {
            max_samples: runtime_config.xray.max_samples,
            top_recommendations: if recommend { 5 } else { 0 },
        },
        check: runtime_config.check.clone(),
        score: runtime_config.score.clone(),
        output: runtime_config.output.clone(),
    };

    let report = tissot::xray::analyze(layer, &config, &file.display().to_string())
        .context("X-Ray analysis failed")?;

    if json {
        let output = tissot::report::json::xray_json(&report)?;
        println!("{output}");
    } else if terminal {
        tissot::report::terminal::print_xray(&report);
    } else {
        let report_json = serde_json::to_string(&report).context("Failed to serialize report")?;
        let heatmap_geojson = report.heatmap.to_geojson().to_string();
        let ellipses_geojson =
            tissot::xray::ellipse::to_geojson_collection(&report.ellipses).to_string();

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(tissot::report::visual::serve_report(
            tissot::report::visual::server::ReportKind::Xray {
                report_json,
                heatmap_geojson,
                ellipses_geojson,
            },
        ))?;
    }

    Ok(())
}

fn cmd_check(
    file: PathBuf,
    domain: Option<String>,
    terminal: bool,
    json: bool,
    sarif: bool,
    runtime_config: &tissot::core::config::Config,
) -> Result<()> {
    let layers = tissot::io::read_file(&file).context("Failed to read input file")?;

    let config = runtime_config.clone();
    let domain_filter = domain.as_deref().and_then(parse_domain);

    let findings =
        tissot::checkers::run_checks(&layers, &config, &file.display().to_string(), domain_filter);

    let report =
        tissot::core::report::ReportData::from_findings(file.display().to_string(), findings);

    if sarif {
        let output = tissot::report::sarif::check_sarif(&report)?;
        println!("{output}");
    } else if json {
        let output = tissot::report::json::check_json(&report)?;
        println!("{output}");
    } else if terminal {
        tissot::report::terminal::print_check(&report);
    } else {
        let report_json = serde_json::to_string(&report).context("Failed to serialize report")?;
        let findings_geojson =
            tissot::report::geojson::findings_to_geojson(&report.findings).to_string();

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(tissot::report::visual::serve_report(
            tissot::report::visual::server::ReportKind::Findings {
                report_json,
                findings_geojson,
            },
        ))?;
    }

    Ok(())
}

fn cmd_score(
    file: PathBuf,
    badge: Option<PathBuf>,
    terminal: bool,
    json: bool,
    runtime_config: &tissot::core::config::Config,
) -> Result<()> {
    let layers = tissot::io::read_file(&file).context("Failed to read input file")?;

    let config = runtime_config.clone();
    let findings =
        tissot::checkers::run_checks(&layers, &config, &file.display().to_string(), None);
    let score_report = tissot::score::compute_score(&findings, &config);

    if let Some(badge_path) = badge {
        let svg = tissot::score::badge::generate_badge(&score_report);
        std::fs::write(&badge_path, svg)
            .with_context(|| format!("Failed to write badge to {}", badge_path.display()))?;
        eprintln!("  Badge saved to {}", badge_path.display());
    }

    if json {
        let output = tissot::report::json::score_json(&score_report)?;
        println!("{output}");
    } else if terminal {
        tissot::report::terminal::print_score(&score_report);
    } else {
        let report_json =
            serde_json::to_string(&score_report).context("Failed to serialize score")?;

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(tissot::report::visual::serve_report(
            tissot::report::visual::server::ReportKind::Score { report_json },
        ))?;
    }

    Ok(())
}

fn parse_domain(s: &str) -> Option<tissot::core::rule::Domain> {
    match s.to_lowercase().as_str() {
        "projection" | "proj" | "crs" => Some(tissot::core::rule::Domain::Projection),
        "quality" | "data_quality" | "data-quality" => {
            Some(tissot::core::rule::Domain::DataQuality)
        }
        "cartography" | "carto" => Some(tissot::core::rule::Domain::Cartography),
        "diff" => Some(tissot::core::rule::Domain::Diff),
        "cloud" | "cloud-native" => Some(tissot::core::rule::Domain::Cloud),
        _ => None,
    }
}

fn cmd_fix(
    file: PathBuf,
    reproject: Option<String>,
    topology: bool,
    in_place: bool,
    json: bool,
    runtime_config: &tissot::core::config::Config,
) -> Result<()> {
    let layers = tissot::io::read_file(&file).context("Failed to read input file")?;
    let cfg = runtime_config.clone();

    let report = if let Some(target_crs) = reproject {
        let source_crs = layers
            .first()
            .and_then(|l| l.crs.clone())
            .unwrap_or_else(|| "EPSG:4326".to_string());
        tissot::fix::reproject_file(&file, &layers, &source_crs, &target_crs, in_place, &cfg)?
    } else if topology {
        tissot::fix::heal_topology_file(&file, &layers, in_place)?
    } else {
        anyhow::bail!("No fix selected. Use --reproject <CRS> or --topology");
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Fix complete");
        println!("  Input: {}", report.input);
        println!("  Output: {}", report.output);
        println!("  Updated features: {}", report.updated_features);
        for action in report.actions {
            println!("  - {action}");
        }
    }

    Ok(())
}

fn cmd_diff(left: PathBuf, right: PathBuf, json: bool, terminal: bool) -> Result<()> {
    let left_layers = tissot::io::read_file(&left).context("Failed reading left input")?;
    let right_layers = tissot::io::read_file(&right).context("Failed reading right input")?;

    let report = tissot::diff::compare(
        &left.display().to_string(),
        &right.display().to_string(),
        &left_layers,
        &right_layers,
    );

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if terminal {
        println!("Diff summary");
        println!("  Left features:  {}", report.left_features);
        println!("  Right features: {}", report.right_features);
        println!("  Added: {}", report.added);
        println!("  Removed: {}", report.removed);
        println!("  Extent changed: {}", report.extent_changed);
    } else {
        let report_json = serde_json::to_string(&report).context("Failed to serialize diff")?;
        let left_geojson = tissot::diff::layers_to_geojson(&left_layers).to_string();
        let right_geojson = tissot::diff::layers_to_geojson(&right_layers).to_string();

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(tissot::report::visual::serve_report(
            tissot::report::visual::server::ReportKind::Diff {
                report_json,
                left_geojson,
                right_geojson,
            },
        ))?;
    }

    Ok(())
}

fn cmd_watch(dir: PathBuf, config: &tissot::core::config::Config) -> Result<()> {
    tissot::watch::run(&dir, config).context("Watch mode failed")
}

fn load_runtime_config() -> Result<tissot::core::config::Config> {
    let config_path = PathBuf::from(".tissot.yml");
    if config_path.exists() {
        tissot::core::config::Config::load(Some(config_path.to_string_lossy().as_ref()))
            .map_err(anyhow::Error::from)
    } else {
        Ok(tissot::core::config::Config::default())
    }
}

fn cmd_init(force: bool) -> Result<()> {
    let path = PathBuf::from(".tissot.yml");
    if path.exists() && !force {
        anyhow::bail!(".tissot.yml already exists. Use --force to overwrite.");
    }

    let template = r#"# Tissot configuration
xray:
  max_samples: 1000
  top_recommendations: 5

check:
  max_distortion_pct: 10.0
  topology_gap_tolerance: 0.001
  disabled_rules: []

score:
  projection_weight: 0.25
  data_integrity_weight: 0.30
  accessibility_weight: 0.25
  classification_weight: 0.20

output:
  open_browser: true
  terminal_only: false
"#;

    std::fs::write(&path, template)?;
    println!("Created {}", path.display());
    Ok(())
}
