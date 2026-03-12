"""
Example: Cloud-Native Format Audit

Checks datasets for cloud-native geo compliance and reports findings.
"""

import json
import subprocess
import sys
from pathlib import Path


def check_cloud(file_path: str) -> dict:
    """Run cloud-native domain checks."""
    result = subprocess.run(
        ["tissot", "check", file_path, "--domain", "cloud", "--json"],
        capture_output=True, text=True, check=True,
    )
    return json.loads(result.stdout)


def audit_directory(directory: str):
    """Audit all geospatial files in a directory for cloud-native compliance."""
    extensions = {".geojson", ".gpkg", ".shp", ".fgb"}
    data_dir = Path(directory)

    results = []
    for path in sorted(data_dir.rglob("*")):
        if path.suffix.lower() in extensions:
            print(f"Checking: {path.name}...", end=" ")
            try:
                report = check_cloud(str(path))
                summary = report.get("summary", {})
                total = summary.get("total", 0)
                warnings = summary.get("warnings", 0)

                status = "PASS" if warnings == 0 else "WARN"
                print(f"{status} ({total} findings, {warnings} warnings)")

                results.append({
                    "file": str(path),
                    "findings": total,
                    "warnings": warnings,
                    "status": status,
                })
            except subprocess.CalledProcessError as e:
                print(f"ERROR: {e}")
                results.append({
                    "file": str(path),
                    "findings": -1,
                    "warnings": -1,
                    "status": "ERROR",
                })

    # Summary
    total_files = len(results)
    passing = sum(1 for r in results if r["status"] == "PASS")
    print(f"\n=== Cloud-Native Audit Summary ===")
    print(f"Files checked: {total_files}")
    print(f"Passing: {passing}/{total_files}")

    if passing < total_files:
        print("\nRecommendations:")
        print("  - Convert Shapefiles to FlatGeobuf or GeoParquet")
        print("  - Add spatial indexes for range-request access")
        print("  - Include complete CRS metadata (EPSG authority)")
        print("  - Apply compression (Snappy/Zstd for Parquet, gzip for FlatGeobuf)")


def main():
    directory = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets"
    print(f"=== Cloud-Native Geo Audit: {directory} ===\n")
    audit_directory(directory)


if __name__ == "__main__":
    main()
