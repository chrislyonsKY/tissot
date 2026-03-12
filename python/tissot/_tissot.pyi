"""Type stubs for the Tissot native extension module (_tissot).

All functions return JSON strings. Use ``json.loads()`` to parse results.
"""

def xray(file_path: str) -> str:
    """Run Projection X-Ray analysis on a geospatial file.

    Computes per-feature distortion metrics, generates a heatmap grid,
    renders Tissot ellipses, and recommends optimal CRS candidates.

    Args:
        file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).

    Returns:
        JSON string of the XrayReport containing:
        - file_path: Source file path.
        - source_crs: CRS of the input data.
        - samples: Distortion sample points with metrics.
        - summary: Summary statistics (max/mean/median distortion).
        - heatmap: Distortion heatmap grid for visualization.
        - ellipses: Tissot ellipse polygons (GeoJSON-ready).
        - recommendations: CRS recommendations ranked by fitness.

    Raises:
        RuntimeError: If the file cannot be read or analysis fails.
    """
    ...

def check(file_path: str, domain: str | None = None) -> str:
    """Run diagnostic checks on a geospatial file.

    Executes all registered checker rules against the data and returns
    an array of findings with severity levels and spatial locations.

    Args:
        file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
        domain: Optional domain filter. One of:
            - "projection" / "proj" / "crs"
            - "quality" / "data_quality" / "data-quality"
            - "cartography" / "carto"
            - "diff"
            - "cloud" / "cloud-native"
            If None, all domains are checked.

    Returns:
        JSON string of a findings array. Each finding contains:
        - rule_id: Identifier of the triggered rule.
        - severity: "error", "warning", or "info".
        - message: Human-readable description.
        - location: Optional spatial location reference.
        - geometry: Optional GeoJSON geometry of the affected area.
        - suggestion: Optional fix suggestion.
        - fixable: Whether autofix is available.

    Raises:
        RuntimeError: If the file cannot be read or checks fail.
    """
    ...

def score(file_path: str) -> str:
    """Compute a quality score (0-100) for a geospatial file.

    Runs all diagnostic checks and aggregates results into a
    Lighthouse-style score with category breakdown and letter grade.

    Args:
        file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).

    Returns:
        JSON string of the ScoreReport containing:
        - overall: Numeric score (0-100).
        - grade: Letter grade ("A" through "F").
        - categories: Per-category scores with weights.
        - finding_count: Total number of findings.

    Raises:
        RuntimeError: If the file cannot be read or scoring fails.
    """
    ...

def fix(
    file_path: str,
    reproject: str | None = None,
    topology: bool = False,
) -> str:
    """Apply automatic fixes to a geospatial file.

    Supports reprojection to a target CRS and topology healing.
    Writes a new file with a "_fixed" suffix.

    Args:
        file_path: Path to a geospatial file (.geojson, .shp, .fgb, .gpkg).
        reproject: Optional target CRS (e.g. "EPSG:3857"). If provided,
            reprojects all geometries from the source CRS.
        topology: If True, removes null geometries and deduplicates
            exact geometry representations.

    Returns:
        JSON string of the FixReport containing:
        - input: Input file path.
        - output: Output file path.
        - updated_features: Number of features processed.
        - actions: List of human-readable actions applied.

    Raises:
        RuntimeError: If the file cannot be read or fix operations fail.
        ValueError: If neither reproject nor topology is specified.
    """
    ...

def diff(left: str, right: str) -> str:
    """Compare two geospatial files and compute a structural diff.

    Computes feature count differences and extent changes between
    two datasets.

    Args:
        left: Path to the first (baseline) geospatial file.
        right: Path to the second (comparison) geospatial file.

    Returns:
        JSON string of the DiffReport containing:
        - left_file: Left file path.
        - right_file: Right file path.
        - left_features: Feature count in left file.
        - right_features: Feature count in right file.
        - added: Approximate number of added features.
        - removed: Approximate number of removed features.
        - extent_changed: Whether the bounding box differs.

    Raises:
        RuntimeError: If either file cannot be read.
    """
    ...
