# DL-006: WebGPU Compute Shaders for Distortion Heatmap

**Date:** 2026-03-07
**Status:** Proposed (Phase 2)
**Author:** Chris Lyons

## Context

The X-Ray engine computes distortion at sample points, then interpolates them into a continuous heatmap surface using Inverse Distance Weighting (IDW). In Phase 1, this interpolation runs on the CPU in Rust and is pre-baked into the report HTML as a GeoJSON grid. This works but has two limitations:

1. For high-resolution grids (200×200 = 40K cells), IDW against 500+ sample points is O(cells × samples) — about 20M distance calculations. On CPU this takes 1-2 seconds.
2. The pre-baked grid is static. Users can't dynamically adjust resolution or re-interpolate as they zoom.

The deep-research paper cites 20-100x speedups for raster operations via WebGPU compute shaders, with specific benchmarks showing Gaussian blur dropping from 500ms to 5ms. IDW interpolation is an embarrassingly parallel operation — each output cell is independent — making it ideal for GPU dispatch.

## Decision

In Phase 2, implement the heatmap interpolation as a **WebGPU compute shader** that runs in the browser's visual report. The Rust/Wasm module passes sample point data to the GPU, and the shader performs IDW interpolation at the current viewport resolution in real-time.

### Architecture

```
Browser Report (Phase 2)
├── MapLibre GL JS (map rendering)
├── Tissot Wasm module (distortion computation at sample points)
└── WebGPU Compute Pipeline
    ├── Input: Storage buffer of sample points [{x, y, distortion_value}]
    ├── Shader: IDW interpolation kernel (WGSL)
    │   - Each GPU thread computes one output cell
    │   - Reads all sample points, computes weighted average
    │   - Writes RGBA color to output texture
    ├── Output: Texture mapped to MapLibre custom layer
    └── Re-dispatches on viewport change (pan/zoom)
```

### WGSL Shader (Sketch)

```wgsl
struct SamplePoint {
    x: f32,
    y: f32,
    value: f32,
    _pad: f32,
};

@group(0) @binding(0) var<storage, read> samples: array<SamplePoint>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let cell_x = f32(id.x) / f32(params.grid_width) * params.extent_width + params.extent_min_x;
    let cell_y = f32(id.y) / f32(params.grid_height) * params.extent_height + params.extent_min_y;

    var numerator: f32 = 0.0;
    var denominator: f32 = 0.0;
    let p: f32 = 2.0;  // IDW power parameter

    for (var i: u32 = 0u; i < params.sample_count; i = i + 1u) {
        let dx = cell_x - samples[i].x;
        let dy = cell_y - samples[i].y;
        let dist = sqrt(dx * dx + dy * dy);
        if (dist < 0.0001) {
            numerator = samples[i].value;
            denominator = 1.0;
            break;
        }
        let w = 1.0 / pow(dist, p);
        numerator = numerator + w * samples[i].value;
        denominator = denominator + w;
    }

    let value = numerator / denominator;
    let color = distortion_to_color(value);
    textureStore(output_texture, vec2<i32>(id.xy), color);
}
```

### Fallback Strategy

WebGPU is not yet universally available (Chrome 113+, Firefox behind flag as of early 2026). The report must detect GPU availability and fall back gracefully:

1. **WebGPU available**: Real-time GPU-interpolated heatmap, updates on pan/zoom
2. **WebGPU unavailable**: Pre-baked GeoJSON grid from Rust/Wasm CPU computation (Phase 1 behavior)

Detection:
```javascript
if (navigator.gpu) {
    const adapter = await navigator.gpu.requestAdapter();
    if (adapter) { useWebGPU(adapter); }
    else { useCPUFallback(); }
} else {
    useCPUFallback();
}
```

## Alternatives Considered

- **CPU-only forever** — Viable for Phase 1 but limits interactivity. Users see a static heatmap at a fixed resolution. Acceptable short-term, not long-term.
- **WebGL fragment shader hack** — Rejected: the paper correctly identifies this as a convoluted approach with texture size limitations and CPU↔GPU transfer overhead. WebGPU compute shaders are purpose-built for this.
- **Server-side pre-rendering at multiple zoom levels** — Rejected: defeats the offline/Wasm architecture. All computation must happen client-side.

## Consequences

- **Positive**: Real-time, interactive distortion heatmap that updates as users explore the map. This is the "show-stopping" visual experience.
- **Positive**: Demonstrates Tissot's technical depth — a WebGPU-powered geospatial analysis tool running in the browser is genuinely novel.
- **Positive**: The same WGSL shader pipeline can later support viewshed analysis, terrain hillshading, and other raster operations.
- **Negative**: WebGPU API is still stabilizing. Shader code may need updates as the spec finalizes.
- **Negative**: Adds wgpu (Rust-side) or raw WebGPU JS to the dependency surface.
- **Constraint**: Phase 2 only. Phase 1 ships with CPU-precomputed heatmaps. The WebGPU path is an enhancement, not a blocker.

## References

- WebGPU spec: https://www.w3.org/TR/webgpu/
- WGSL spec: https://www.w3.org/TR/WGSL/
- wgpu (Rust WebGPU): https://github.com/gfx-rs/wgpu
- oxigdal-gpu benchmarks: https://lib.rs/crates/oxigdal-gpu
- Deep-research paper pages 8-9: WebGPU compute shader benchmarks (20-100x speedup)
- IDW interpolation: embarrassingly parallel, O(cells × samples), ideal for GPU dispatch
