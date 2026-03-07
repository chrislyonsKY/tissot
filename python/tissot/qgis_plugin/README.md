# Tissot QGIS Processing Provider

QGIS Processing provider that exposes Tissot algorithms as native Processing tools:

- Projection X-Ray
- Data Quality Check
- Map Quality Score
- Spatial Diff

## Requirements

- QGIS 3.28+
- Python 3.9+
- `tissot` CLI available (installed from this repo via pip/maturin)

## Install dependency in QGIS Python environment

The plugin is UI glue and calls the Tissot Rust core through the local
`tissot` CLI (`--json` mode). Install `tissot` in the active QGIS Python
environment before running algorithms.

Example (from QGIS Python console or matching shell):

```python
import sys
print(sys.executable)
```

Use that interpreter to install:

```bash
"<path-to-qgis-python>" -m pip install tissot
```

Optional: override CLI location if needed.

```bash
export TISSOT_CLI="$HOME/.local/bin/tissot"
```

For local development from a checkout, editable installs are still valid:

```bash
"<path-to-qgis-python>" -m pip install -e /path/to/tissot
```

## Plugin package notes

- Contains no bundled binaries.
- Uses metadata links for homepage/repository/tracker.
- Designed as a Processing provider for model/batch compatibility.
