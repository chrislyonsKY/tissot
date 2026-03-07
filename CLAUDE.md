# CLAUDE.md — Tissot
> Geospatial diagnostics engine: projection x-ray, cartographic linting, spatial diffing, and autofix.
> Named after Tissot's indicatrix — the distortion ellipses that reveal what projections hide.
> Rust core with Python bindings via PyO3. CLI-first, visual-first, zero-config to start.

Read this file completely before doing anything.
Then read `ai-dev/architecture.md` for full system design.
Then read `ai-dev/guardrails/` for hard constraints.

---

## What Makes Tissot Different

Tissot is NOT another linter that prints text warnings. It is a **visual diagnostics engine**.

Seven core principles — these are identity, not backlog:

1. **Visual-first output** — The default output is an interactive map in the browser, not terminal text. Every finding has a spatial location rendered on the data. Terminal summaries exist but are secondary.
2. **Projection X-Ray (hero feature)** — `tissot xray data.gpkg` generates a distortion heatmap on your actual features with Tissot ellipses, quantified error metrics, and CRS recommendations. This is the demo. This is the screenshot. This is why people install it.
3. **Autofix** — `tissot fix` doesn't just report problems, it fixes them. Reproject to optimal CRS, heal topology gaps, rewrite symbology for WCAG compliance. The `prettier --write` of GIS.
4. **Map Score** — `tissot score project.qgz` produces a 0-100 quality score with category breakdown (Projection, Data Quality, Accessibility, Classification). Lighthouse for maps.
5. **Interactive diff slider** — `tissot diff v1.gpkg v2.gpkg` generates a before/after slider map, not a text changelog.
6. **Watch mode** — `tissot watch ./data/` monitors files and streams results to a live browser dashboard.
7. **Zero-config first run** — No config file needed. `tissot xray whatever.gpkg` works immediately with smart defaults.

If a feature or design decision conflicts with these principles, the principles win.

---

## Workflow Protocol

When starting a new task:
1. Read CLAUDE.md (this file)
2. Read ai-dev/architecture.md
3. Read ai-dev/guardrails/ for constraints that override all other guidance
4. Read the relevant ai-dev/agents/ file for your role
5. Check ai-dev/decisions/ for prior decisions that may affect your work
6. Check ai-dev/skills/ for domain patterns specific to this project

Before writing code:
- Confirm you understand the module's responsibility
- List the files you will create or modify
- Check ai-dev/decisions/ for prior decisions affecting this module
- Show me the plan

Do not proceed until I type **Engage**.

---

## Compatibility Matrix

| Component | Version | Notes |
|---|---|---|
| Rust | 1.83+ (2024 edition) | Required for PyO3 0.27+ |
| Python | 3.9 – 3.13 | ArcGIS Pro 3.x ships 3.9; target broad range |
| PyO3 | 0.27.x | Python bindings via maturin |
| maturin | 1.x | Build system for Python wheels |
| geo (crate) | latest | GeoRust primitives and algorithms |
| proj (crate) | latest | PROJ bindings for CRS transforms |
| geozero | latest | Primary IO — zero-copy format reading (DL-004) |
| shapefile (crate) | latest | Pure Rust shapefile reader |
| flatgeobuf (crate) | latest | Pure Rust FlatGeobuf reader |
| gdal (crate) | latest | **Optional feature** — fallback IO + GPKG write |
| clap | 4.x | CLI argument parsing |
| axum | latest | Local web server for visual reports + watch mode |
| askama | latest | HTML template engine for reports |
| wasm-bindgen | latest | Wasm↔JS bridge for browser target (DL-005) |
| wgpu | latest | **Phase 2** — WebGPU compute for heatmaps (DL-006) |

---

## Project Structure

```
tissot/
├── CLAUDE.md                         # This file — AI reads first
├── README.md                         # Human-facing project description
├── Cargo.toml                        # Rust workspace root
├── pyproject.toml                    # Python package config (maturin)
├── .github/
│   └── copilot-instructions.md       # GitHub Copilot project instructions
├── ai-dev/                           # AI development infrastructure
│   ├── architecture.md               # System design, module interfaces, data flow
│   ├── spec.md                       # Requirements, acceptance criteria, constraints
│   ├── patterns.md                   # Code patterns, anti-patterns, lessons learned
│   ├── prompt-templates.md           # Reusable AI tool prompts
│   ├── agents/                       # Specialized agent configurations
│   ├── decisions/                    # Architectural Decision Records (DL-xxx)
│   ├── skills/                       # Domain-specific SKILL.md files
│   └── guardrails/                   # Hard constraints (override everything)
├── specs/                            # Detailed feature specifications
│   ├── projection-xray.md            # Hero feature spec
│   ├── map-score.md                  # Lighthouse-for-maps scoring spec
│   ├── autofix.md                    # Autofix engine spec
│   ├── visual-diff.md               # Interactive diff slider spec
│   ├── cartographic-linter.md        # Cartographic checker rules spec
│   └── watch-mode.md                 # File watching + live dashboard spec
├── src/                              # Rust source (library + CLI binary)
│   ├── lib.rs                        # Library root — re-exports public API
│   ├── main.rs                       # CLI binary entry point (clap)
│   ├── core/                         # Core types, config, rule engine
│   │   ├── mod.rs
│   │   ├── config.rs                 # .tissot.yml config parsing (optional)
│   │   ├── rule.rs                   # Rule trait, severity, result types
│   │   ├── report.rs                 # Finding aggregation
│   │   └── registry.rs              # Rule registry
│   ├── checkers/                     # Diagnostic rule modules
│   │   ├── mod.rs
│   │   ├── cartography/             # Map quality rules
│   │   ├── projection/              # CRS distortion analysis
│   │   ├── diff/                    # Spatial change detection
│   │   └── data_quality/            # Schema, topology, extent checks
│   ├── xray/                        # Projection X-Ray engine (hero feature)
│   │   ├── mod.rs
│   │   ├── distortion.rs            # Per-feature distortion computation
│   │   ├── heatmap.rs               # Distortion heatmap generation
│   │   ├── ellipse.rs               # Tissot ellipse rendering on features
│   │   ├── recommend.rs             # CRS recommendation with tradeoff analysis
│   │   └── compare.rs              # Side-by-side CRS comparison
│   ├── fix/                         # Autofix engine
│   │   ├── mod.rs
│   │   ├── reproject.rs             # CRS autofix
│   │   ├── topology.rs              # Gap/overlap healing
│   │   ├── symbology.rs            # WCAG accessibility autofix
│   │   └── schema.rs               # Schema normalization
│   ├── score/                       # Map quality scoring
│   │   ├── mod.rs
│   │   ├── calculator.rs            # Score computation (0-100)
│   │   ├── categories.rs            # Category breakdown
│   │   └── badge.rs                # SVG badge generation
│   ├── io/                          # Format readers/writers (DL-004: geozero-first)
│   │   ├── mod.rs
│   │   ├── geojson.rs               # geozero + serde_json (pure Rust)
│   │   ├── geopackage.rs            # geozero reader, gdal writer (feature-gated)
│   │   ├── shapefile.rs             # shapefile crate (pure Rust)
│   │   ├── flatgeobuf.rs            # flatgeobuf crate (pure Rust)
│   │   ├── qgis.rs                  # .qgz/.qgs project parsing (XML + zip)
│   │   ├── arcgis.rs               # .aprx/.mapx parsing
│   │   └── wasm.rs                  # Byte-array IO for Wasm target (DL-005)
│   ├── wasm/                        # Browser entry points (DL-005, cfg(wasm32))
│   │   ├── mod.rs                   # wasm-bindgen exports
│   │   └── bridge.rs               # JS↔Rust data conversion
│   └── report/                      # Output renderers
│       ├── mod.rs
│       ├── visual/                  # Browser-based visual outputs
│       │   ├── mod.rs
│       │   ├── server.rs            # Local axum server for reports
│       │   ├── xray_map.rs          # X-Ray interactive map
│       │   ├── diff_slider.rs       # Before/after slider map
│       │   ├── findings_map.rs      # Diagnostic findings map
│       │   ├── score_dashboard.rs   # Score breakdown dashboard
│       │   └── watch_dashboard.rs   # Live watch mode dashboard
│       ├── terminal.rs              # Rich terminal output (secondary)
│       ├── json.rs                  # Machine-readable JSON
│       └── sarif.rs                # SARIF for CI/CD
├── templates/                       # Askama HTML templates
│   ├── xray.html                    # X-Ray report template
│   ├── diff.html                    # Diff slider template
│   ├── findings.html                # Findings map template
│   ├── score.html                   # Score dashboard template
│   └── watch.html                   # Watch mode live dashboard
├── assets/                          # Static assets for visual reports
│   ├── tissot.css                   # Report styling
│   └── tissot.js                    # Map interaction (Leaflet/MapLibre)
└── python/                          # Python bindings (PyO3 + maturin)
    └── tissot/
        ├── __init__.py
        ├── _tissot.pyi              # Type stubs
        └── qgis_plugin/            # QGIS plugin wrapper (future)
```

---

## CLI Commands

```bash
# Hero feature — Projection X-Ray
tissot xray data.gpkg                    # Distortion analysis → opens browser map
tissot xray data.gpkg --recommend        # Include CRS recommendations
tissot xray data.gpkg --compare 3089     # Side-by-side with EPSG:3089

# Check — diagnostic linting
tissot check data.gpkg                   # All checks → opens findings map
tissot check project.qgz                # Cartographic + data checks on project
tissot check data.gpkg --domain quality  # Only data quality checks
tissot check data.gpkg --terminal        # Terminal-only output (no browser)

# Score — map quality rating
tissot score project.qgz                 # 0-100 score → opens dashboard
tissot score data.gpkg --badge score.svg # Generate SVG badge for README

# Diff — visual change detection
tissot diff v1.gpkg v2.gpkg              # Interactive slider → opens browser
tissot diff v1.gpkg v2.gpkg --json       # Machine-readable diff

# Fix — autofix problems
tissot fix data.gpkg --reproject         # Reproject to recommended CRS
tissot fix data.gpkg --topology          # Heal gaps and overlaps
tissot fix project.qgz --accessibility   # Fix WCAG issues in symbology

# Watch — live monitoring
tissot watch ./data/                     # Monitor directory → live dashboard

# Utility
tissot init                              # Create .tissot.yml with smart defaults
tissot --version
tissot --help
```

---

## Critical Conventions

### Rust
- **Edition 2024** — Rust 1.83+
- **Error handling** — `thiserror` for library, `anyhow` for CLI binary only
- **Geometry types** — always use `geo` crate primitives
- **CRS operations** — always through `proj` crate
- **Serialization** — `serde` + `serde_json` for config and data interchange
- **Traits over enums** — checker rules implement the `Rule` trait
- **Tests** — every module has `#[cfg(test)] mod tests`; integration tests in `tests/`
- **No `unwrap()` in library code** — propagate with `?`, use `thiserror`
- **No `println!` in library code** — use `log` + `env_logger`

### Visual Output
- **Browser is the primary output** — visual commands spawn a local axum server and open the default browser. The server shuts down on Ctrl+C.
- **MapLibre GL JS** — use MapLibre (not Leaflet) for the interactive maps. It's open source, WebGL-powered, and handles large datasets.
- **Self-contained HTML** — reports must work offline. Inline all JS/CSS. No CDN dependencies.
- **Dark mode default** — visual reports use a dark theme that makes data pop.

### Python Bindings
- **Thin wrapper** — all computation in Rust, Python is API surface only
- **Type stubs** — maintain `_tissot.pyi` for every public function
- **ArcPy interop** — accept/return WKT, WKB, GeoJSON strings

---

## Architecture Summary

Tissot has six major subsystems. See `ai-dev/architecture.md` for full design.

1. **X-Ray Engine** (`src/xray/`) — Computes per-feature distortion metrics, generates heatmaps, renders Tissot ellipses, recommends CRS candidates.
2. **Checker Engine** (`src/checkers/`) — Rule-based diagnostics across four domains: cartography, projection, diff, data quality.
3. **Fix Engine** (`src/fix/`) — Autofix transformations: reproject, heal topology, fix symbology.
4. **Score Engine** (`src/score/`) — Aggregates checker results into a 0-100 quality score with category breakdown.
5. **Visual Report Server** (`src/report/visual/`) — Local axum web server that serves interactive HTML/MapLibre reports.
6. **IO Layer** (`src/io/`) — Format readers for GeoPackage, GeoJSON, Shapefile, QGIS projects, ArcGIS projects.

---

## What NOT To Do

- Do NOT make terminal text the primary output — visual map is always the default
- Do NOT require config for first run — zero-config must work
- Do NOT bypass `geo` crate types with raw coordinate tuples
- Do NOT use `unwrap()` or `expect()` in library code
- Do NOT put computation logic in the Python binding layer
- Do NOT hardcode CRS identifiers — always use config or auto-detection
- Do NOT add dependencies without a DL- decision record
- Do NOT write rules as standalone functions — they must implement the `Rule` trait
- Do NOT use CDN-hosted assets in visual reports — everything must be self-contained/offline
- Do NOT assume WGS 84 — always read CRS from the data source
- Do NOT generate boring reports — every visual output should make someone want to screenshot it
