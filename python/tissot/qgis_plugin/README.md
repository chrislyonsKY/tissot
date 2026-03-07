# Tissot QGIS Processing Provider

QGIS Processing provider that exposes Tissot algorithms as native Processing tools:

- Projection X-Ray
- Data Quality Check
- Map Quality Score
- Spatial Diff

## Requirements

- QGIS 3.28+
- Python 3.9+
- `tissot` Python package installed in the same Python environment used by QGIS

## Install dependency in QGIS Python environment

The plugin is UI glue and calls the Tissot Rust core through Python bindings.
Install `tissot` in the active QGIS Python environment before running algorithms.

Example (from QGIS Python console or matching shell):

```python
import sys
print(sys.executable)
```

Use that interpreter to install:

```bash
"<path-to-qgis-python>" -m pip install tissot
```

## Plugin package notes

- Contains no bundled binaries.
- Uses metadata links for homepage/repository/tracker.
- Designed as a Processing provider for model/batch compatibility.
