"""
Example: Quality Score and Badge Generation

Computes a Lighthouse-style quality score and generates an SVG badge.
"""

import json
import subprocess
import sys


def run_score(file_path: str) -> dict:
    """Get quality score as JSON."""
    result = subprocess.run(
        ["tissot", "score", file_path, "--json"],
        capture_output=True, text=True, check=True,
    )
    return json.loads(result.stdout)


def generate_badge(file_path: str, badge_path: str):
    """Generate an SVG badge file."""
    subprocess.run(
        ["tissot", "score", file_path, "--badge", badge_path],
        check=True,
    )


def main():
    file_path = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets/parcels_with_issues.geojson"

    print(f"Scoring: {file_path}\n")
    score = run_score(file_path)

    overall = score.get("overall_score", 0)
    grade = score.get("grade", "?")
    print(f"Overall Score: {overall}/100 (Grade: {grade})")

    # Category breakdown
    categories = score.get("categories", {})
    print("\nCategory Breakdown:")
    for name, cat in categories.items():
        cat_score = cat.get("score", 0) if isinstance(cat, dict) else cat
        print(f"  {name}: {cat_score}/100")

    # Generate badge
    badge_path = "map-score.svg"
    generate_badge(file_path, badge_path)
    print(f"\nBadge saved to: {badge_path}")
    print("Add to README: ![Map Score](map-score.svg)")


if __name__ == "__main__":
    main()
