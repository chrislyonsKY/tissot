"""
Example: Data Quality Check

Runs all diagnostic checks on a file and groups findings by severity.
"""

import json
import subprocess
import sys
from collections import Counter


def run_check(file_path: str, domain: str | None = None) -> dict:
    """Run Tissot checks and return the JSON report."""
    cmd = ["tissot", "check", file_path, "--json"]
    if domain:
        cmd.extend(["--domain", domain])
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)


def main():
    file_path = sys.argv[1] if len(sys.argv) > 1 else "examples/datasets/parcels_with_issues.geojson"

    print(f"Checking: {file_path}\n")
    report = run_check(file_path)

    # Summary
    summary = report.get("summary", {})
    print(f"Total findings: {summary.get('total', 0)}")
    print(f"  Errors:   {summary.get('errors', 0)}")
    print(f"  Warnings: {summary.get('warnings', 0)}")
    print(f"  Info:     {summary.get('info', 0)}")

    # Group by rule
    findings = report.get("findings", [])
    rule_counts = Counter(f.get("rule_id", "unknown") for f in findings)

    print("\nFindings by rule:")
    for rule_id, count in rule_counts.most_common():
        severity = next(
            (f["severity"] for f in findings if f.get("rule_id") == rule_id),
            "unknown",
        )
        print(f"  [{severity}] {rule_id}: {count}")

    # Fixable findings
    fixable = [f for f in findings if f.get("fixable", False)]
    if fixable:
        print(f"\n{len(fixable)} findings are auto-fixable with `tissot fix`")


if __name__ == "__main__":
    main()
