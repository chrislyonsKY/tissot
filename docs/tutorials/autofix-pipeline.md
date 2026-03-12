# Tutorial: Autofix Pipeline

Build an automated data cleaning pipeline with Tissot's fix engine.

## The Problem

You receive raw geospatial data that needs standardization before publishing:

- Wrong projection (Web Mercator instead of a local CRS)
- Topology gaps between adjacent parcels
- No spatial index for cloud serving

## Step 1: Assess the Data

```bash
tissot check raw_parcels.geojson --json | jq '.summary'
```

```json
{
  "total": 8,
  "errors": 2,
  "warnings": 5,
  "info": 1
}
```

## Step 2: Reproject

```bash
tissot fix raw_parcels.geojson --reproject EPSG:5070
```

Output: `raw_parcels_fixed.geojson`

## Step 3: Heal Topology

```bash
tissot fix raw_parcels_fixed.geojson --topology
```

## Step 4: Verify

```bash
tissot score raw_parcels_fixed.geojson --terminal
```

```
Map Score: 87/100 (B+)

  Projection Quality:   95/100
  Data Integrity:       82/100
  Accessibility:        85/100
  Cloud Readiness:      78/100
```

## Scripted Pipeline

Combine steps into a shell script:

```bash
#!/bin/bash
set -e

INPUT="$1"
OUTPUT="${INPUT%.geojson}_clean.geojson"

echo "=== Tissot Autofix Pipeline ==="

# Step 1: Determine best CRS
BEST_CRS=$(tissot xray "$INPUT" --json | jq -r '.recommendations[0].epsg // "EPSG:4326"')
echo "Best CRS: $BEST_CRS"

# Step 2: Reproject
tissot fix "$INPUT" --reproject "$BEST_CRS"
REPROJECTED="${INPUT%.geojson}_fixed.geojson"

# Step 3: Heal topology
tissot fix "$REPROJECTED" --topology
mv "${REPROJECTED%.geojson}_fixed.geojson" "$OUTPUT"

# Step 4: Quality gate
SCORE=$(tissot score "$OUTPUT" --json | jq '.overall_score')
echo "Final score: $SCORE/100"

if (( $(echo "$SCORE < 70" | bc -l) )); then
  echo "FAIL: Score below 70"
  exit 1
fi

echo "Output: $OUTPUT"
```

## GitHub Actions Pipeline

```yaml
name: Geo Data Quality

on:
  push:
    paths: ['data/**']

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Tissot
        run: pip install tissot

      - name: Check data quality
        run: |
          for f in data/*.geojson; do
            echo "Checking $f..."
            tissot check "$f" --sarif > "${f%.geojson}.sarif"
          done

      - name: Score gate
        run: |
          for f in data/*.geojson; do
            SCORE=$(tissot score "$f" --json | jq '.overall_score')
            echo "$f: $SCORE/100"
            if (( $(echo "$SCORE < 70" | bc -l) )); then
              echo "FAIL: $f scored below 70"
              exit 1
            fi
          done

      - name: Upload SARIF results
        if: always()
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: data/
```
