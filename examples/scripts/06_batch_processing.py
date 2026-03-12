"""
Example: Batch Processing

Process multiple geospatial files and generate a summary report.
"""

import json
import subprocess
import sys
from pathlib import Path


def tissot_json(args: list[str]) -> dict:
    """Run a tissot command with --json and return parsed output."""
    result = subprocess.run(
        ["tissot"] + args + ["--json"],
        capture_output=True, text=True, check=True,
    )
    return json.loads(result.stdout)


def process_file(file_path: str) -> dict:
    """Run all analyses on a single file."""
    report = {"file": file_path}

    # Check
    try:
        check = tissot_json(["check", file_path])
        report["check"] = check.get("summary", {})
    except subprocess.CalledProcessError:
        report["check"] = {"error": True}

    # Score
    try:
        score = tissot_json(["score", file_path])
        report["score"] = score.get("overall_score", 0)
        report["grade"] = score.get("grade", "?")
    except subprocess.CalledProcessError:
        report["score"] = 0
        report["grade"] = "?"

    return report


def main():
    directory = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets"
    extensions = {".geojson", ".gpkg", ".shp", ".fgb"}

    files = sorted(
        p for p in Path(directory).rglob("*")
        if p.suffix.lower() in extensions
    )

    if not files:
        print(f"No geospatial files found in {directory}")
        sys.exit(1)

    print(f"Processing {len(files)} files from {directory}\n")

    results = []
    for f in files:
        print(f"  Processing {f.name}...", end=" ", flush=True)
        report = process_file(str(f))
        results.append(report)
        print(f"Score: {report['score']}/100 ({report['grade']})")

    # Summary table
    print(f"\n{'='*60}")
    print(f"{'File':<35} {'Score':>6} {'Grade':>6} {'Findings':>9}")
    print(f"{'-'*35} {'-'*6} {'-'*6} {'-'*9}")
    for r in results:
        name = Path(r["file"]).name[:34]
        findings = r.get("check", {}).get("total", "?")
        print(f"{name:<35} {r['score']:>6} {r['grade']:>6} {findings:>9}")

    # Average score
    scores = [r["score"] for r in results if isinstance(r["score"], (int, float))]
    if scores:
        avg = sum(scores) / len(scores)
        print(f"\nAverage score: {avg:.1f}/100")

    # Write JSON report
    output_path = "batch_report.json"
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Full report: {output_path}")


if __name__ == "__main__":
    main()
