# DL-003: Visual-First Output via Local Web Server

**Date:** 2026-03-07
**Status:** Accepted
**Author:** Chris Lyons

## Context

Linters traditionally output text to the terminal. Geospatial diagnostics are inherently spatial — findings have locations, distortion varies across geography, and diffs are best understood visually. Text output fails to communicate the most important information.

## Decision

The default output for all Tissot commands is an interactive browser-based map report. Tissot spins up a lightweight local web server (axum), serves a self-contained HTML page with MapLibre GL JS, and opens the default browser. Terminal output is available via `--terminal` flag. JSON output via `--json`.

## Alternatives Considered

- **Terminal-first with optional HTML export** — Rejected: this is what every other linter does. It buries the most valuable output (the visual map) behind a flag. We want the "wow" moment to be the default experience.
- **Desktop GUI (egui/Tauri)** — Rejected: adds massive dependency surface, platform-specific builds, and installation complexity. A browser tab is universally available.
- **Static HTML file (no server)** — Considered but insufficient: watch mode requires SSE for live updates, and comparison mode benefits from server-side data preparation. A local server is lightweight and enables future features.
- **Electron app** — Rejected: enormous binary size, installation friction. The opposite of "zero-config."

## Consequences

- **Positive**: The first thing a user sees is their data on an interactive map with distortion visualized. This is the "where has this been" moment.
- **Positive**: Browser-based reports are shareable (save HTML file, send to colleague).
- **Positive**: MapLibre GL JS handles large datasets via WebGL, avoiding the rendering bottleneck of SVG-based solutions.
- **Negative**: Adds axum + askama + static asset bundling to the dependency tree. Justified by the core differentiator status.
- **Negative**: Users in headless/SSH environments need `--terminal` or `--json`. These must be fully functional alternatives, not afterthoughts.
- **Trade-off**: The axum server binds to localhost on a random port and auto-shuts-down. Security surface is minimal (no external network access), but document this clearly.

## References

- axum: https://github.com/tokio-rs/axum
- MapLibre GL JS: https://maplibre.org/
- Lighthouse (inspiration for browser-based reports): https://developer.chrome.com/docs/lighthouse
