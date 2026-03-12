# Tutorial: Projection X-Ray

Learn how to use Tissot's hero feature to visualize and fix projection distortion.

## The Problem

You have a dataset in Web Mercator (EPSG:3857). You've heard it distorts areas, but by how much? And what should you use instead?

## Step 1: Run X-Ray Analysis

```bash
tissot xray us_counties.geojson --recommend
```

This opens an interactive map showing:

- **Distortion heatmap** overlaid on your features (red = high distortion, green = low)
- **Tissot ellipses** at sample points showing how circles become ovals
- **CRS recommendations** ranked by distortion reduction

## Step 2: Read the Terminal Summary

```
Current CRS: EPSG:3857 (Web Mercator)
  Area distortion — Max: 47.2%  Mean: 23.1%
  Distance distortion — Max: 31.8%  Mean: 15.6%

Recommendations:
  1. EPSG:5070 (NAD83 / Conus Albers)
     Area distortion — Max: 0.1%  Mean: 0.04%
  2. EPSG:2163 (US National Atlas Equal Area)
     Area distortion — Max: 0.3%  Mean: 0.1%
```

## Step 3: Compare CRS Options

Use the `--crs` flag to analyze a specific projection:

```bash
tissot xray us_counties.geojson --crs EPSG:5070
```

## Step 4: Fix It

Once you've chosen a better CRS, apply the fix:

```bash
tissot fix us_counties.geojson --reproject EPSG:5070
```

This creates `us_counties_fixed.geojson` reprojected to NAD83 / Conus Albers.

## Step 5: Verify

Run X-Ray again on the fixed file:

```bash
tissot xray us_counties_fixed.geojson
```

Area distortion should now be negligible.

## Understanding the Output

### Distortion Heatmap

The heatmap uses IDW (Inverse Distance Weighting) interpolation from sample points. Colors represent area distortion percentage:

| Color | Distortion |
|-------|-----------|
| Green | < 1% |
| Yellow | 1-5% |
| Orange | 5-15% |
| Red | > 15% |

### Tissot Ellipses

Each ellipse shows how a small circle at that location gets distorted by the projection:

- **Circular** = no distortion (conformal at that point)
- **Stretched** = area/shape distortion
- **Rotated** = angular distortion

### CRS Recommendations

Tissot evaluates candidates from these categories:

1. **UTM zones** — Best for small areas (< 6 degrees longitude)
2. **State Plane** — Optimized for US state-level work
3. **Continental** — Equal-area projections for large regions
4. **Custom** — Transverse Mercator centered on your data

## JSON Output for Scripting

```bash
tissot xray us_counties.geojson --json > report.json
```

```python
import json

with open("report.json") as f:
    report = json.load(f)

print(f"Mean area distortion: {report['distortion']['mean_area_pct']:.2f}%")
print(f"Recommended CRS: {report['recommendations'][0]['epsg']}")
```

## CI/CD Integration

Add projection quality gates to your pipeline:

```yaml
# GitHub Actions example
- name: Check projection quality
  run: |
    DISTORTION=$(tissot xray data.geojson --json | jq '.distortion.mean_area_pct')
    if (( $(echo "$DISTORTION > 5.0" | bc -l) )); then
      echo "Area distortion too high: ${DISTORTION}%"
      exit 1
    fi
```
