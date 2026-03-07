# DL-005: WebAssembly Compilation Target

**Date:** 2026-03-07
**Status:** Accepted
**Author:** Chris Lyons

## Context

Tissot's visual-first philosophy already serves interactive reports via a local axum web server. But requiring users to install a binary (via `cargo install` or `pip install`) creates friction. The deep-research paper on next-gen GIS architecture identifies WebAssembly as the key enabler for running native-speed spatial computation directly in the browser with zero installation.

If Tissot's core Rust library compiles to Wasm, we can offer a **browser-only mode**: a static webpage where users drag-and-drop a file and get a full X-Ray report without installing anything. This is the ultimate zero-config experience and the strongest possible demo vehicle.

## Decision

Architect the Tissot core library (`src/lib.rs` and all non-IO, non-CLI modules) to be **Wasm-compatible from day one**. This means:

1. **No GDAL in the Wasm build** — IO in the browser uses geozero + JavaScript `FileReader` API to parse files client-side.
2. **No filesystem access in Wasm** — all data enters via function parameters (byte arrays), not file paths.
3. **No axum/tokio in Wasm** — the browser IS the visual layer; no server needed.
4. **Conditional compilation** — use `#[cfg(not(target_arch = "wasm32"))]` for native-only code (file IO, server, CLI).

### Build Targets

```
tissot (native binary)
├── CLI (clap) ─────────────── #[cfg(not(wasm))]
├── Local server (axum) ────── #[cfg(not(wasm))]
├── File IO (geozero + gdal) ─ #[cfg(not(wasm))]
├── Core engine ────────────── ✅ Wasm-compatible
│   ├── X-Ray distortion ───── ✅
│   ├── Checker rules ──────── ✅
│   ├── Score calculator ───── ✅
│   └── Report data builder ── ✅
└── PyO3 bindings ──────────── #[cfg(not(wasm))]

tissot-wasm (browser package)
├── wasm-bindgen API ───────── Thin JS↔Rust bridge
├── Core engine ────────────── Same code as native
└── Browser IO (geozero from ArrayBuffer)
```

### Wasm API Surface

```rust
// src/wasm.rs — only compiled for wasm32 target
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn xray_from_bytes(data: &[u8], format: &str) -> Result<JsValue, JsError> {
    let layers = crate::io::wasm::parse_bytes(data, format)?;
    let config = XrayConfig::default();
    let report = crate::xray::analyze_layers(&layers, &config)?;
    Ok(serde_wasm_bindgen::to_value(&report)?)
}

#[wasm_bindgen]
pub fn check_from_bytes(data: &[u8], format: &str) -> Result<JsValue, JsError> {
    let layers = crate::io::wasm::parse_bytes(data, format)?;
    let config = Config::default();
    let report = crate::checkers::run_all(&layers, &config)?;
    Ok(serde_wasm_bindgen::to_value(&report)?)
}
```

### Browser Demo Page

A static HTML page (hosted on GitHub Pages) that:
1. Presents a drag-and-drop zone
2. Loads the Tissot Wasm module
3. Parses the dropped file client-side via geozero
4. Runs X-Ray analysis in the browser
5. Renders the distortion heatmap + ellipses via MapLibre GL JS
6. Zero server involvement — everything runs in the browser tab

This becomes the ultimate conference demo and the "try before you install" experience.

## Alternatives Considered

- **Server-only, no Wasm** — Rejected: misses the zero-install demo opportunity and limits Tissot to users willing to install a binary. The browser demo is the single best adoption driver.
- **Wasm-only, no native binary** — Rejected: browser has no filesystem access, can't do watch mode, can't integrate with CI/CD pipelines. Native binary is the production tool; Wasm is the onramp.

## Consequences

- **Positive**: "Try Tissot" becomes a URL, not an install command. Conference demo is a webpage.
- **Positive**: Forces clean architecture — Wasm-incompatible code (filesystem, networking, CLI) is cleanly separated from core logic via conditional compilation.
- **Positive**: Future path to embedding Tissot in QGIS Web, ArcGIS Experience Builder, or any web app.
- **Negative**: Wasm build requires `wasm-pack` or `wasm-bindgen` tooling. Adds CI complexity.
- **Negative**: Large files (>100MB) may be slow to parse in the browser. Document limits clearly.
- **Constraint**: All core engine code must avoid `std::fs`, `std::net`, `tokio`, and any OS-specific APIs. Use the `io::wasm` module to abstract data ingestion for the Wasm target.

## References

- wasm-bindgen: https://github.com/rustwasm/wasm-bindgen
- wasm-pack: https://github.com/rustwasm/wasm-pack
- serde-wasm-bindgen: https://github.com/RReverser/serde-wasm-bindgen
- geozero Wasm compatibility: geozero's core is pure Rust, no system dependencies
- Deep-research paper pages 8-9: WebAssembly + WebGPU for browser-based spatial computation
