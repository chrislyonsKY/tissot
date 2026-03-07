# Projection X-Ray — Hero Feature Specification

> `tissot xray` — See your distortion. The command that makes people install Tissot.

## Overview

The Projection X-Ray takes a geospatial dataset and generates an interactive browser-based visualization showing exactly how the current coordinate reference system distorts the data. It renders a distortion heatmap overlaid on the user's actual features, draws Tissot ellipses at sample locations, quantifies error metrics, and recommends better CRS options with side-by-side comparison.

## User Experience

```bash
$ tissot xray kentucky_permits.gpkg

  ╔═══════════════════════════════════════════════════╗
  ║  TISSOT X-RAY — Projection Distortion Analysis     ║
  ╚═══════════════════════════════════════════════════╝

  Source:   kentucky_permits.gpkg
  CRS:      EPSG:3857 (WGS 84 / Pseudo-Mercator)
  Features: 12,847
  Sampled:  500 points (stratified grid)

  ┌─────────────────────────────────────────────┐
  │ AREA DISTORTION                              │
  │   Max:    18.3%  (PERMIT-4521, Pike County)  │
  │   Mean:   11.7%                              │
  │   Median: 11.2%                              │
  │   Std:     3.4%                              │
  ├─────────────────────────────────────────────┤
  │ DISTANCE DISTORTION                          │
  │   Max:     9.1%                              │
  │   Mean:    5.8%                              │
  ├─────────────────────────────────────────────┤
  │ RECOMMENDATION                               │
  │   → EPSG:3089 (NAD83 / Kentucky Single Zone) │
  │     Area max: 0.02%  Distance max: 0.01%     │
  │   → EPSG:2205 (NAD83 / KY FIPS 1600)        │
  │     Area max: 0.03%  Distance max: 0.02%     │
  └─────────────────────────────────────────────┘

  ▸ Interactive report: http://localhost:48721/xray
    (opened in default browser)
```

## Browser Report Layout

```
┌──────────────────────────────────────────────────────────┐
│  TISSOT X-RAY                          [Current CRS ▾]   │
├──────────────────────────────────────────────────────────┤
│                                                          │
│                   ┌──────────────────┐                   │
│                   │                  │                   │
│                   │   MAP VIEWPORT   │                   │
│                   │                  │                   │
│                   │  Features +      │                   │
│                   │  Heatmap +       │                   │
│                   │  Ellipses        │                   │
│                   │                  │                   │
│                   └──────────────────┘                   │
│                                                          │
│  Layer Controls:                                         │
│  [✓] Features  [✓] Distortion Heatmap  [✓] Ellipses    │
│                                                          │
├─────────────────────┬────────────────────────────────────┤
│  METRICS            │  CRS RECOMMENDATIONS               │
│                     │                                    │
│  Area Error         │  1. EPSG:3089 — KY Single Zone    │
│  ██████████ 18.3%   │     Area: 0.02%  Dist: 0.01%     │
│  mean: 11.7%        │     [Preview]                     │
│                     │                                    │
│  Distance Error     │  2. EPSG:2205 — KY FIPS 1600     │
│  █████ 9.1%         │     Area: 0.03%  Dist: 0.02%     │
│  mean: 5.8%         │     [Preview]                     │
│                     │                                    │
│  Shape Error        │  3. EPSG:32617 — UTM Zone 17N    │
│  ███ 4.2%           │     Area: 0.08%  Dist: 0.04%     │
│  mean: 2.1%         │     [Preview]                     │
└─────────────────────┴────────────────────────────────────┘
```

Clicking [Preview] on a CRS recommendation morphs the map to show the data in that CRS, with the distortion heatmap updating in real-time.

## Algorithm Detail

### Distortion Computation

For each sample point (centroid of selected features):

1. **Forward/inverse transform test**: Transform point to geographic (lon/lat), then back. Measure roundtrip error.

2. **Jacobian computation**: At point (x, y) in projected CRS:
   - Perturb by small delta in x and y directions
   - Transform perturbed points to geographic coordinates
   - Compute partial derivatives: ∂lon/∂x, ∂lat/∂x, ∂lon/∂y, ∂lat/∂y
   - Assemble 2×2 Jacobian matrix

3. **Tissot parameters from Jacobian**:
   - Compute singular values (σ₁, σ₂) of Jacobian → semimajor (a) and semiminor (b)
   - Area scale factor: h = σ₁ × σ₂
   - Area distortion: |h - 1| × 100%
   - Angular distortion: ω = 2 × arcsin((σ₁ - σ₂)/(σ₁ + σ₂))
   - Maximum scale factor: max(σ₁, σ₂)

4. **Per-feature area error** (for features, not just sample points):
   - Compute feature area in current CRS
   - Compute feature area in equal-area reference CRS
   - Error = |A_current - A_reference| / A_reference × 100%

### Sampling Strategy

For datasets with N features:
- N ≤ 1,000: Use all feature centroids
- 1,000 < N ≤ 50,000: Stratified grid sampling — divide extent into grid cells, sample one feature per cell, target 500 samples
- N > 50,000: Same stratified grid, target 1,000 samples

Stratified grid ensures spatial coverage across the entire extent, avoiding bias toward feature-dense areas.

### Heatmap Generation

Distortion values at sample points are interpolated into a continuous surface using Inverse Distance Weighting (IDW) on a raster grid covering the data extent. The grid is then rendered as a semi-transparent color overlay using MapLibre's heatmap layer:

- Green (0-2% error): Minimal distortion
- Yellow (2-5% error): Moderate distortion
- Orange (5-10% error): Significant distortion
- Red (>10% error): Severe distortion

### Ellipse Rendering

Tissot ellipses at sample points are generated as GeoJSON Polygon features:
- 72-vertex ellipse approximation
- Semimajor axis aligned with direction of maximum scale
- Scaled so a perfect circle (no distortion) has a consistent visual radius
- Filled with semi-transparent blue, stroked with dark blue
- On hover: popup shows exact distortion values for that point

### CRS Recommendation

1. Determine dataset geographic extent (in lon/lat)
2. Classify extent scope: local (< 2° span), regional (< 10°), continental, global
3. Generate candidate list based on location and scope:
   - **USA**: State Plane (HARN/NAD83), UTM zones, Albers Equal Area Conic, Lambert Conformal Conic
   - **Europe**: ETRS89 / UTM zones, national grids, Lambert Azimuthal Equal Area
   - **Global**: UTM zone, World Cylindrical Equal Area, Winkel Tripel
   - Always include: current CRS (as baseline), WGS 84 geographic
4. For each candidate, run distortion computation on same sample points
5. Rank by optimization target (default: minimize max area error)
6. Return top 5 candidates with full metric comparison

### Comparison Mode

`tissot xray data.gpkg --compare 3089` generates a synchronized dual-map view:
- Left panel: data in current CRS with distortion overlay
- Right panel: data in EPSG:3089 with distortion overlay
- Maps are sync'd: pan/zoom on one pans/zooms the other
- Metrics panel shows side-by-side comparison table

## Configuration

```yaml
# .tissot.yml (optional — X-Ray works without config)
xray:
  sample_size: 500                    # Number of sample points
  heatmap_resolution: 100             # Grid cells per axis for IDW
  optimize_for: area                  # area | distance | shape | balanced
  max_candidates: 5                   # Number of CRS recommendations
  include_crs: [3089, 2205]           # Always evaluate these EPSG codes
  exclude_crs: []                     # Never recommend these
  ellipse_count: 50                   # Number of Tissot ellipses to render
```

## Acceptance Criteria

- [ ] X-Ray correctly identifies that EPSG:3857 introduces >10% area distortion for mid-latitude US data
- [ ] X-Ray correctly shows ~0% distortion for data already in an appropriate projection
- [ ] Heatmap visually concentrates red in high-distortion areas (e.g., high-latitude edges)
- [ ] Tissot ellipses are circles in conformal projections, stretched in non-conformal
- [ ] CRS recommendations include at least one equal-area option when area distortion > 5%
- [ ] Comparison mode sync'd maps stay in sync during pan/zoom
- [ ] Report loads in < 1 second for datasets up to 100K features
- [ ] Report works fully offline (no external resource loading)
- [ ] `--terminal` mode prints full metrics without opening browser
- [ ] `--json` mode outputs structured JSON suitable for programmatic consumption
