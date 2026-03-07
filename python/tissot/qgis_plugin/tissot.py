"""Plugin-local Tissot API bridge.

This module mirrors the minimal Python API used by the QGIS algorithms by
calling the installed ``tissot`` CLI with ``--json`` and normalizing output.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any


class TissotCliError(RuntimeError):
    """Raised when the Tissot CLI is unavailable or returns an error."""


def _resolve_cli() -> str:
    env_path = os.environ.get("TISSOT_CLI")
    candidates = [
        env_path,
        shutil.which("tissot"),
        str(Path.home() / ".local" / "bin" / "tissot"),
        "/opt/homebrew/bin/tissot",
        "/usr/local/bin/tissot",
    ]

    for candidate in candidates:
        if not candidate:
            continue
        if Path(candidate).exists():
            return candidate

    raise TissotCliError(
        "Could not find 'tissot' CLI. Install it first or set TISSOT_CLI to the executable path."
    )


def _run_json(args: list[str]) -> dict[str, Any]:
    cli = _resolve_cli()
    process = subprocess.run(
        [cli, *args, "--json"],
        check=False,
        capture_output=True,
        text=True,
    )

    if process.returncode != 0:
        message = process.stderr.strip() or process.stdout.strip() or "unknown error"
        raise TissotCliError(f"tissot CLI failed ({process.returncode}): {message}")

    payload = process.stdout.strip()
    if not payload:
        return {}

    try:
        data = json.loads(payload)
    except json.JSONDecodeError as exc:
        raise TissotCliError(f"Failed to parse JSON output from tissot CLI: {exc}") from exc

    if not isinstance(data, dict):
        raise TissotCliError("Unexpected non-object JSON output from tissot CLI.")

    return data


def _severity_for_distortion(distortion_pct: float) -> str:
    if distortion_pct >= 25.0:
        return "error"
    if distortion_pct >= 10.0:
        return "warning"
    return "info"


def xray(input_path: str) -> dict[str, Any]:
    report = _run_json(["xray", input_path])
    samples = report.get("samples") or []
    findings = []

    for sample in samples:
        if not isinstance(sample, dict):
            continue
        value = float(sample.get("area_distortion_pct") or 0.0)
        lon = sample.get("lon")
        lat = sample.get("lat")
        finding = {
            "metric": "area_distortion_pct",
            "value": value,
            "severity": _severity_for_distortion(value),
            "message": f"Area distortion {value:.2f}%",
        }
        if lon is not None and lat is not None:
            finding["geometry"] = {"x": lon, "y": lat}
        findings.append(finding)

    report["findings"] = findings
    return report


def check(input_path: str) -> dict[str, Any]:
    report = _run_json(["check", input_path])

    # Enrich with score for QGIS output compatibility.
    try:
        score_report = _run_json(["score", input_path])
        report["score"] = float(score_report.get("overall") or 0.0)
    except TissotCliError:
        report.setdefault("score", 0.0)

    return report


def score(input_path: str, badge: str | None = None) -> dict[str, Any]:
    args = ["score", input_path]
    if badge:
        args.extend(["--badge", badge])

    report = _run_json(args)

    # Normalize key names for existing plugin expectations.
    if "overall" in report and "score" not in report:
        report["score"] = report["overall"]

    categories = report.get("categories")
    if isinstance(categories, list):
        category_breakdown: dict[str, Any] = {}
        for category in categories:
            if not isinstance(category, dict):
                continue
            name = str(category.get("category") or "category")
            category_breakdown[name] = category.get("score")
        report["category_breakdown"] = category_breakdown

    return report


def diff(baseline_path: str, comparison_path: str) -> dict[str, Any]:
    report = _run_json(["diff", baseline_path, comparison_path])

    # The CLI currently returns summary-level diff info; synthesize changes for
    # compatibility with the QGIS algorithm output schema.
    changes = []
    added = int(report.get("added") or 0)
    removed = int(report.get("removed") or 0)

    for idx in range(added):
        changes.append(
            {
                "change_type": "added",
                "feature_id": f"added-{idx + 1}",
                "message": "Feature added",
            }
        )

    for idx in range(removed):
        changes.append(
            {
                "change_type": "removed",
                "feature_id": f"removed-{idx + 1}",
                "message": "Feature removed",
            }
        )

    if not changes and bool(report.get("extent_changed")):
        changes.append(
            {
                "change_type": "modified",
                "feature_id": "extent",
                "message": "Dataset extent changed",
            }
        )

    report["changes"] = changes
    return report
