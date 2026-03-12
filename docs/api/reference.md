# API Reference

## Python API

Tissot provides Python bindings via PyO3. The compiled extension module is `tissot._tissot`.

### Installation

```bash
pip install tissot
```

### Current API (CLI Wrapper)

While direct PyO3 bindings are being developed, the Python package provides CLI access:

```python
import json
import subprocess

def tissot_xray(file_path: str) -> dict:
    """Run X-Ray analysis and return JSON report."""
    result = subprocess.run(
        ["tissot", "xray", file_path, "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)

def tissot_check(file_path: str, domain: str | None = None) -> dict:
    """Run diagnostic checks and return JSON report."""
    cmd = ["tissot", "check", file_path, "--json"]
    if domain:
        cmd.extend(["--domain", domain])
    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    return json.loads(result.stdout)

def tissot_score(file_path: str) -> dict:
    """Get quality score as JSON."""
    result = subprocess.run(
        ["tissot", "score", file_path, "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)
```

### Planned PyO3 API

The following direct bindings are in development:

```python
import tissot

# Direct function calls (no subprocess)
report = tissot.xray("data.geojson")
findings = tissot.check("data.geojson", domain="quality")
score = tissot.score("data.geojson")
fix_result = tissot.fix("data.geojson", reproject="EPSG:5070")
```

## Rust API

The Rust library (`tissot`) exposes the following public modules:

### `tissot::io`

```rust
/// Read a geospatial file and return layers.
pub fn read_file(path: &Path) -> Result<Vec<Layer>, TissotError>;
```

### `tissot::xray`

```rust
/// Run projection distortion analysis on a layer.
pub fn analyze(layer: &Layer, config: &Config, source: &str) -> Result<XrayReport, TissotError>;
```

### `tissot::checkers`

```rust
/// Run diagnostic checks across all registered rules.
pub fn run_checks(
    layers: &[Layer],
    config: &Config,
    source: &str,
    domain: Option<Domain>,
) -> Vec<Finding>;
```

### `tissot::score`

```rust
/// Compute a quality score from findings.
pub fn compute_score(findings: &[Finding], config: &Config) -> ScoreReport;
```

### `tissot::fix`

```rust
/// Reproject a dataset to a target CRS.
pub fn reproject_file(
    path: &Path,
    layers: &[Layer],
    source_crs: &str,
    target_crs: &str,
    in_place: bool,
    config: &Config,
) -> Result<FixReport, TissotError>;

/// Heal topology issues in a dataset.
pub fn heal_topology_file(
    path: &Path,
    layers: &[Layer],
    in_place: bool,
) -> Result<FixReport, TissotError>;
```

### `tissot::diff`

```rust
/// Compare two datasets and return a diff report.
pub fn compare(
    left_source: &str,
    right_source: &str,
    left_layers: &[Layer],
    right_layers: &[Layer],
) -> DiffReport;
```

### `tissot::core::rule`

```rust
/// Trait that all checker rules must implement.
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn domain(&self) -> Domain;
    fn severity(&self) -> Severity;
    fn description(&self) -> &str;
    fn check(&self, layers: &[Layer], config: &Config, source: &str) -> Vec<Finding>;
    fn can_fix(&self) -> bool { false }
}

pub enum Domain {
    Projection,
    DataQuality,
    Cartography,
    Diff,
    Cloud,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}
```

## QGIS Processing Provider

The QGIS plugin registers five Processing algorithms:

| Algorithm | ID | Description |
|-----------|----|-------------|
| Projection X-Ray | `tissot:xray` | Per-feature distortion analysis |
| Data Quality Check | `tissot:check` | Diagnostic linting |
| Map Quality Score | `tissot:score` | 0-100 quality rating |
| Spatial Diff | `tissot:diff` | Change detection between datasets |
| Autofix | `tissot:fix` | Reproject, heal topology |

All algorithms accept standard QGIS vector layers as input and produce vector layers and/or HTML reports as output.
