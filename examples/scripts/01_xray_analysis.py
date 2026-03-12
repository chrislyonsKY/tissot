"""
Example: Projection X-Ray Analysis

Demonstrates running Tissot's projection distortion analysis
from Python and processing the JSON results.
"""

import json
import subprocess
import sys


def run_xray(file_path: str, recommend: bool = True) -> dict:
    """Run Tissot X-Ray analysis and return the JSON report."""
    cmd = ["tissot", "xray", file_path, "--json"]
    if recommend:
        cmd.append("--recommend")

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)


def main():
    file_path = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets/us_states_mercator.geojson"

    print(f"Running X-Ray on: {file_path}")
    report = run_xray(file_path)

    # Distortion summary
    distortion = report.get("distortion", {})
    print(f"\nCurrent CRS: {report.get('crs', 'Unknown')}")
    print(f"  Area distortion  — Mean: {distortion.get('mean_area_pct', 0):.2f}%")
    print(f"  Area distortion  — Max:  {distortion.get('max_area_pct', 0):.2f}%")

    # Recommendations
    recommendations = report.get("recommendations", [])
    if recommendations:
        print("\nRecommended CRS candidates:")
        for i, rec in enumerate(recommendations, 1):
            print(f"  {i}. {rec.get('epsg', '?')} — {rec.get('name', 'Unknown')}")
            print(f"     Area distortion: {rec.get('mean_area_pct', 0):.2f}%")

    # Sample count
    samples = report.get("sample_count", 0)
    print(f"\nSample points analyzed: {samples}")


if __name__ == "__main__":
    main()
