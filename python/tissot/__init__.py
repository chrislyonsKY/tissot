"""Tissot — Visual-first geospatial diagnostics engine.

Projection x-ray, cartographic linting, spatial diffing, and autofix.
All computation happens in Rust; this module provides a thin Python API.

Functions return JSON strings. Use ``json.loads()`` to parse them into dicts::

    import json
    import tissot

    report = json.loads(tissot.xray("data.geojson"))
    print(report["summary"]["max_area_distortion_pct"])
"""

from tissot._tissot import check, diff, fix, score, xray

__all__ = [
    "xray",
    "check",
    "score",
    "fix",
    "diff",
]
