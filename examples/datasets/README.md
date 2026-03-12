# Example Datasets

Sample geospatial data for demonstrating Tissot features.

## Files

| File | Description | Use With |
|------|-------------|----------|
| `us_states_mercator.geojson` | 5 US states in Web Mercator (EPSG:3857) | `tissot xray` — shows projection distortion |
| `world_cities.geojson` | 15 major world cities (WGS 84) | `tissot check`, `tissot xray` — global point data |
| `parcels_with_issues.geojson` | 10 parcels with intentional data quality issues | `tissot check` — null geometry, duplicates, overlaps |
| `kentucky_roads.geojson` | Kentucky highway network (WGS 84) | `tissot xray`, `tissot check` — line geometry |
| `simple_points.geojson` | Simple 3-point dataset | `tissot check` — minimal test case |
| `empty.geojson` | Empty feature collection | `tissot check` — triggers empty dataset rule |

## Quick Start

```bash
# X-Ray: see distortion on Web Mercator data
tissot xray examples/datasets/us_states_mercator.geojson --recommend

# Check: find data quality issues
tissot check examples/datasets/parcels_with_issues.geojson

# Score: rate the data
tissot score examples/datasets/parcels_with_issues.geojson

# Diff: compare two files
tissot diff examples/datasets/simple_points.geojson examples/datasets/world_cities.geojson

# Fix: reproject from Web Mercator to Albers
tissot fix examples/datasets/us_states_mercator.geojson --reproject EPSG:5070
```
