# Example Scripts

Python scripts demonstrating Tissot's capabilities.

## Prerequisites

```bash
pip install tissot
```

## Scripts

| Script | Description |
|--------|-------------|
| `01_xray_analysis.py` | Projection distortion analysis with CRS recommendations |
| `02_data_quality_check.py` | Run diagnostic checks and group findings |
| `03_score_and_badge.py` | Generate quality scores and SVG badges |
| `04_autofix_pipeline.py` | Automated assess-fix-verify pipeline |
| `05_cloud_native_audit.py` | Cloud-native format compliance audit |
| `06_batch_processing.py` | Batch process multiple files with summary report |

## Usage

```bash
# Run with default example data
python examples/scripts/01_xray_analysis.py

# Run with your own data
python examples/scripts/01_xray_analysis.py path/to/your/data.geojson
```
