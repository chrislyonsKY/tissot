# Tutorial: Map Score for CI/CD

Use Tissot's scoring system as a quality gate in your data pipelines.

## Concept

Tissot Score works like [Lighthouse](https://developer.chrome.com/docs/lighthouse/) for websites — a 0-100 quality rating with category breakdown. Use it to enforce minimum quality standards in CI/CD.

## Score Categories

| Category | Weight | What It Measures |
|----------|--------|------------------|
| Projection Quality | 0.25 | CRS appropriateness, distortion levels |
| Data Integrity | 0.30 | Geometry validity, topology, schema |
| Accessibility | 0.20 | WCAG compliance, readability |
| Cloud Readiness | 0.20 | Format optimization, spatial indexing |
| Classification | 0.05 | Data categorization quality |

## Letter Grades

| Grade | Score Range | Meaning |
|-------|------------|---------|
| A | 90-100 | Excellent — production ready |
| B | 80-89 | Good — minor issues |
| C | 70-79 | Acceptable — improvements needed |
| D | 60-69 | Poor — significant issues |
| F | 0-59 | Failing — major problems |

## Basic Usage

```bash
# Interactive dashboard
tissot score data.geojson

# Terminal summary
tissot score data.geojson --terminal

# JSON for scripting
tissot score data.geojson --json
```

## Generate README Badge

```bash
tissot score data.geojson --badge map-score.svg
```

Add to your README:

```markdown
![Map Score](map-score.svg)
```

## GitHub Actions Quality Gate

```yaml
name: Map Quality Gate

on:
  pull_request:
    paths: ['data/**', '*.geojson', '*.gpkg']

jobs:
  score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Tissot
        run: pip install tissot

      - name: Score all datasets
        run: |
          PASS=true
          for f in $(find data -name "*.geojson" -o -name "*.gpkg"); do
            RESULT=$(tissot score "$f" --json)
            SCORE=$(echo "$RESULT" | jq '.overall_score')
            GRADE=$(echo "$RESULT" | jq -r '.grade')
            echo "| $f | $SCORE | $GRADE |"

            if (( $(echo "$SCORE < 70" | bc -l) )); then
              echo "::error::$f scored $SCORE/100 (grade: $GRADE)"
              PASS=false
            fi
          done

          if [ "$PASS" = false ]; then
            exit 1
          fi

      - name: Update badge
        if: github.ref == 'refs/heads/main'
        run: |
          tissot score data/primary.geojson --badge docs/assets/map-score.svg
          git add docs/assets/map-score.svg
          git diff --staged --quiet || git commit -m "Update map score badge"
```

## Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

GEOJSON_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(geojson|gpkg)$')

if [ -z "$GEOJSON_FILES" ]; then
  exit 0
fi

echo "Running Tissot score check..."
for f in $GEOJSON_FILES; do
  SCORE=$(tissot score "$f" --json | jq '.overall_score')
  if (( $(echo "$SCORE < 60" | bc -l) )); then
    echo "BLOCKED: $f scored $SCORE/100 (minimum: 60)"
    exit 1
  fi
done
```
