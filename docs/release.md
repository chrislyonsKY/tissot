# Release Guide

This project publishes Python wheels and the QGIS plugin on Git tag pushes.

## Triggering a release

1. Ensure `main` is green in CI.
2. Bump versions:
- `Cargo.toml` (`version`)
- `pyproject.toml` (`version`)
- `python/tissot/qgis_plugin/metadata.txt` (`version`)
3. Create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The `release.yml` workflow will:
- Build wheels on Linux, macOS, and Windows
- Publish wheels to PyPI
- Package `tissot_processing_provider-<version>.zip`
- Attach wheels and plugin zip to the GitHub Release

## Required repository configuration

1. Configure PyPI trusted publishing for this repository and `pypi` environment.
2. Allow workflow permission to write release contents.

## Public install instructions

CLI install:

```bash
pip install tissot
```

QGIS install dependency:

```bash
"/Applications/QGIS.app/Contents/MacOS/python" -m pip install tissot
```

QGIS plugin install:
- Download the plugin zip from the GitHub Release assets.
- In QGIS: `Plugins -> Manage and Install Plugins... -> Install from ZIP`.
