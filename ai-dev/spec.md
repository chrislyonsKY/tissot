# Tissot Specification

> Requirements, acceptance criteria, and constraints.

## Product Vision

Tissot is the first geospatial tool that **shows you what your projection is hiding**. It's a visual diagnostics engine that makes spatial data problems visible, quantifiable, and fixable — in one command with zero config.

Named after Nicolas Auguste Tissot, who in 1859 invented the indicatrix — ellipses drawn on a map to reveal how a projection distorts reality. Tissot the tool does the same thing, but on your actual data, in your browser, in seconds.

## Target Users

| Persona | First command they run | What hooks them |
|---|---|---|
| GIS Analyst | `tissot xray mydata.gpkg` | "I had no idea my parcels were 15% wrong" |
| GIS Developer | `tissot check data.gpkg --json` | CI/CD integration, SARIF output |
| Cartographer | `tissot score project.qgz` | Quality score for their portfolio |
| Data Engineer | `tissot watch ./pipeline/output/` | Live monitoring of ETL output quality |
| GIS Manager | `tissot diff Q3.gpkg Q4.gpkg` | Visual change report for stakeholders |
| Open source contributor | `tissot check --help` | Clear rule system, easy to add new rules |

## Release Phases

### Phase 1 — "See Your Distortion" (MVP)

**Goal**: Ship the X-Ray hero feature, basic data quality checks, and the visual report server. One install, one command, mind blown.

**Deliverables**:
- `tissot xray <file>` — full projection distortion analysis with interactive browser report
- `tissot check <file>` — data quality checks (topology, schema, null geometry, duplicates, extent)
- `tissot score <file>` — quality score (projection + data quality categories only)
- Visual report server (axum + MapLibre GL JS)
- Terminal output mode (`--terminal`)
- JSON output mode (`--json`)
- Zero-config operation with smart defaults
- Python bindings: `tissot.xray()`, `tissot.check()`, `tissot.score()`
- IO: GeoPackage, GeoJSON, Shapefile
- Sample test datasets with known issues

**Acceptance Criteria**:
- [ ] `tissot xray sample.gpkg` opens a browser with a distortion heatmap within 3 seconds
- [ ] Distortion heatmap correctly shows higher error at edges of data extent for Web Mercator
- [ ] Tissot ellipses render at sample points, visually deforming based on local distortion
- [ ] CRS recommendations are listed with quantified max/mean area error
- [ ] `tissot xray sample.gpkg --compare 3089` shows side-by-side map comparison
- [ ] `tissot check sample_with_errors.gpkg` identifies topology gaps, overlaps, null geometries
- [ ] `tissot score sample.gpkg` outputs a 0-100 score with letter grade
- [ ] `pip install tissot` works; Python API returns same results as CLI
- [ ] All visual reports work offline (no CDN dependencies)
- [ ] `--terminal` flag suppresses browser and prints findings to stdout

### Phase 2 — "Fix It and Diff It"

**Goal**: Autofix engine and interactive visual diffing.

**Deliverables**:
- `tissot fix <file> --reproject` — reproject to recommended CRS
- `tissot fix <file> --topology` — heal gaps and overlaps
- `tissot diff <a> <b>` — interactive before/after slider map
- SARIF output for CI/CD
- `.tissot.yml` config file support
- `tissot init` command
- Badge generation (`tissot score --badge`)
- Python: `tissot.fix()`, `tissot.diff()`

**Acceptance Criteria**:
- [ ] `tissot fix sample.gpkg --reproject` writes a new file with recommended CRS
- [ ] `tissot fix sample.gpkg --topology` heals at least simple gaps
- [ ] Fix never modifies input file unless `--in-place` is passed
- [ ] `tissot diff v1.gpkg v2.gpkg` opens browser with slider map
- [ ] Slider shows geometry changes highlighted, with before/after toggle
- [ ] SARIF output integrates with GitHub Advanced Security
- [ ] Score badge SVG renders correctly in GitHub README

### Phase 3 — "Cartographic Intelligence"

**Goal**: Full cartographic linting with project file support, watch mode.

**Deliverables**:
- QGIS .qgz/.qgs project parsing
- ArcGIS .aprx/.mapx project parsing
- Cartographic rules: color contrast, label overlap, symbology scale, classification, color-only
- `tissot fix <project> --accessibility` — WCAG autofix for symbology
- `tissot watch <dir>` — file watching with live browser dashboard
- QGIS Processing plugin wrapper
- GitHub Action

**Acceptance Criteria**:
- [ ] `tissot check project.qgz` runs cartographic rules using parsed symbology
- [ ] Color contrast findings include WCAG ratio and affected layer pairs
- [ ] `tissot watch ./data/` opens live dashboard that updates on file changes
- [ ] Watch mode re-checks in < 3 seconds after file change

## Non-Functional Requirements

| Requirement | Target |
|---|---|
| Time to first result | < 3 seconds for `tissot xray` on 10K features |
| Memory | < 2× dataset size peak memory |
| Report load time | < 1 second for visual report in browser |
| Portability | Linux, macOS, Windows (x86_64 + ARM64) |
| Python support | 3.9 – 3.13 |
| Offline operation | All visual reports work without internet |
| Test coverage | > 80% line coverage on core + checkers + xray |
| Install experience | `cargo install tissot` or `pip install tissot` — no extra steps |

## Prior Art and Differentiation

| Existing Tool | What It Does | How Tissot Is Different |
|---|---|---|
| QGIS Geometry Checker | Topology validation (point-and-click) | Tissot: CLI-first, visual browser reports, CRS analysis, scoring, CI/CD-ready |
| GeoLinter (academic, 2023) | Choropleth map linting for VegaLite | Tissot: general-purpose, multi-format, works on real GIS data not just VegaLite |
| geodiff (MerginMaps) | GeoPackage changeset diffing | Tissot: visual slider diff, diagnostic focus (not patch/apply), part of larger toolchain |
| Kart | Geospatial version control | Tissot: diagnostics not versioning, no repo required, zero-config |
| ArcGIS Data Reviewer | Enterprise data quality rules | Tissot: open source, visual-first, projection analysis, not tied to Esri ecosystem |
| Great Expectations | Data quality framework | Tissot: spatially-aware, understands geometry types, CRS, topology |

**Tissot's unique position**: The only tool that makes spatial data problems **visible** — not just reportable — in one command with zero config.
