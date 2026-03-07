# Example Datasets

- `simple_points.geojson`: minimal non-empty dataset for smoke testing.
- `empty.geojson`: intentionally empty feature collection for data-quality rule checks.

Quick checks:

```bash
cargo run -- check examples/datasets/simple_points.geojson
cargo run -- check examples/datasets/empty.geojson
```
