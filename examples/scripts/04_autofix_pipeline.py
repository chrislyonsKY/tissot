"""
Example: Automated Fix Pipeline

Demonstrates a complete fix workflow: assess, reproject, heal, verify.
"""

import json
import subprocess
import sys


def tissot_cmd(args: list[str]) -> dict:
    """Run a tissot command and return JSON output."""
    result = subprocess.run(
        ["tissot"] + args + ["--json"],
        capture_output=True, text=True, check=True,
    )
    return json.loads(result.stdout)


def main():
    file_path = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets/us_states_mercator.geojson"

    print(f"=== Tissot Autofix Pipeline ===\n")
    print(f"Input: {file_path}")

    # Step 1: Assess current state
    print("\n--- Step 1: Assess ---")
    xray = tissot_cmd(["xray", file_path])
    distortion = xray.get("distortion", {})
    print(f"Current CRS: {xray.get('crs', 'Unknown')}")
    print(f"Mean area distortion: {distortion.get('mean_area_pct', 0):.2f}%")

    # Step 2: Determine best CRS
    recommendations = xray.get("recommendations", [])
    if recommendations:
        best_crs = recommendations[0].get("epsg", "EPSG:4326")
        print(f"\nBest CRS recommendation: {best_crs}")
    else:
        best_crs = "EPSG:5070"
        print(f"\nNo recommendations available, defaulting to: {best_crs}")

    # Step 3: Reproject
    print("\n--- Step 2: Reproject ---")
    fix_result = tissot_cmd(["fix", file_path, "--reproject", best_crs])
    output_path = fix_result.get("output", file_path.replace(".geojson", "_fixed.geojson"))
    print(f"Reprojected to: {best_crs}")
    print(f"Output: {output_path}")

    # Step 4: Verify
    print("\n--- Step 3: Verify ---")
    score = tissot_cmd(["score", output_path])
    print(f"Final score: {score.get('overall_score', 0)}/100 ({score.get('grade', '?')})")

    # Quality gate
    overall = score.get("overall_score", 0)
    if overall >= 80:
        print("\nPASS: Data meets quality threshold")
    elif overall >= 60:
        print("\nWARN: Data needs improvement")
    else:
        print("\nFAIL: Data below minimum quality")
        sys.exit(1)


if __name__ == "__main__":
    main()
