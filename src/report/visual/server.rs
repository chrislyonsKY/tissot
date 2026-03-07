use crate::core::error::Result;
/// Local axum web server for visual reports.
///
/// Serves self-contained HTML with embedded MapLibre GL JS.
/// Auto-opens the default browser, shuts down on Ctrl+C.
use axum::{Router, response::Html, routing::get};
use std::net::TcpListener;

/// The type of report to serve.
pub enum ReportKind {
    /// X-Ray distortion analysis.
    Xray {
        /// Serialized report data as JSON.
        report_json: String,
        /// Heatmap GeoJSON.
        heatmap_geojson: String,
        /// Ellipses GeoJSON.
        ellipses_geojson: String,
    },
    /// Diagnostic check findings.
    Findings {
        /// Serialized findings as JSON.
        report_json: String,
        /// Findings as GeoJSON.
        findings_geojson: String,
    },
    /// Quality score dashboard.
    Score {
        /// Serialized score report as JSON.
        report_json: String,
    },
    /// Visual before/after diff report.
    Diff {
        /// Serialized diff summary report as JSON.
        report_json: String,
        /// Left dataset as GeoJSON.
        left_geojson: String,
        /// Right dataset as GeoJSON.
        right_geojson: String,
    },
}

/// Serve a visual report on a random port and open the browser.
pub async fn serve_report(kind: ReportKind) -> Result<()> {
    let html = match kind {
        ReportKind::Xray {
            report_json,
            heatmap_geojson,
            ellipses_geojson,
        } => build_xray_html(&report_json, &heatmap_geojson, &ellipses_geojson),
        ReportKind::Findings {
            report_json,
            findings_geojson,
        } => build_findings_html(&report_json, &findings_geojson),
        ReportKind::Score { report_json } => build_score_html(&report_json),
        ReportKind::Diff {
            report_json,
            left_geojson,
            right_geojson,
        } => build_diff_html(&report_json, &left_geojson, &right_geojson),
    };

    let app = Router::new().route("/", get(move || async move { Html(html) }));

    // Bind to a random available port.
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let url = format!("http://127.0.0.1:{port}");

    log::info!("Report server running at {url}");
    log::info!("Opening report at {url}");

    // Open browser (ignore errors — may not have a browser in CI).
    let _ = open::that(&url);

    let listener = tokio::net::TcpListener::from_std(listener)?;
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::core::error::TissotError::Internal(format!("Server error: {e}")))?;

    Ok(())
}

/// Build self-contained diff HTML report.
fn build_diff_html(report_json: &str, left_geojson: &str, right_geojson: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tissot Diff — Before/After Comparison</title>
<script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js"></script>
<link href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css" rel="stylesheet" />
<style>{CSS}</style>
</head>
<body>
<div id="header">
  <h1>⊕ Tissot Diff</h1>
  <span id="subtitle">Before/After Comparison</span>
</div>
<div id="container">
  <div id="map"></div>
  <div id="panel">
    <div id="diff-summary"></div>
    <h2 style="margin-top:20px">Comparison Slider</h2>
    <input id="right-opacity" type="range" min="0" max="100" value="70" style="width:100%;margin:8px 0" />
    <div style="display:flex;justify-content:space-between;font-size:12px;color:#94a3b8">
      <span>Before (blue)</span>
      <span>After (orange)</span>
    </div>
  </div>
</div>
<script>
const REPORT = {report_json};
const LEFT = {left_geojson};
const RIGHT = {right_geojson};
{DIFF_JS}
</script>
</body>
</html>"##,
        CSS = REPORT_CSS,
        DIFF_JS = DIFF_JS,
    )
}

/// Build self-contained X-Ray HTML report.
fn build_xray_html(report_json: &str, heatmap_geojson: &str, ellipses_geojson: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tissot X-Ray — Projection Distortion Analysis</title>
<script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js"></script>
<link href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css" rel="stylesheet" />
<style>{CSS}</style>
</head>
<body>
<div id="header">
  <h1>⊕ Tissot X-Ray</h1>
  <span id="subtitle">Projection Distortion Analysis</span>
</div>
<div id="container">
  <div id="map"></div>
  <div id="panel">
    <div id="metrics"></div>
    <div id="recommendations"></div>
    <div id="legend"></div>
  </div>
</div>
<script>
const REPORT = {report_json};
const HEATMAP = {heatmap_geojson};
const ELLIPSES = {ellipses_geojson};
{XRAY_JS}
</script>
</body>
</html>"##,
        CSS = REPORT_CSS,
        XRAY_JS = XRAY_JS,
    )
}

/// Build self-contained findings HTML report.
fn build_findings_html(report_json: &str, findings_geojson: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tissot Check — Diagnostic Findings</title>
<script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js"></script>
<link href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css" rel="stylesheet" />
<style>{CSS}</style>
</head>
<body>
<div id="header">
  <h1>⊕ Tissot Check</h1>
  <span id="subtitle">Diagnostic Findings</span>
</div>
<div id="container">
  <div id="map"></div>
  <div id="panel">
    <div id="findings-list"></div>
  </div>
</div>
<script>
const REPORT = {report_json};
const FINDINGS_GEOJSON = {findings_geojson};
{FINDINGS_JS}
</script>
</body>
</html>"##,
        CSS = REPORT_CSS,
        FINDINGS_JS = FINDINGS_JS,
    )
}

/// Build self-contained score dashboard HTML.
fn build_score_html(report_json: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tissot Score — Map Quality Rating</title>
<style>{CSS}</style>
</head>
<body>
<div id="header">
  <h1>⊕ Tissot Score</h1>
  <span id="subtitle">Map Quality Rating</span>
</div>
<div id="score-container">
  <div id="overall-score"></div>
  <div id="categories"></div>
</div>
<script>
const REPORT = {report_json};
{SCORE_JS}
</script>
</body>
</html>"##,
        CSS = REPORT_CSS,
        SCORE_JS = SCORE_JS,
    )
}

/// Shared CSS for all reports — dark theme.
const REPORT_CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0f172a; color: #e2e8f0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
#header { padding: 16px 24px; background: #1e293b; border-bottom: 1px solid #334155; display: flex; align-items: baseline; gap: 12px; }
#header h1 { font-size: 20px; color: #38bdf8; }
#subtitle { font-size: 14px; color: #94a3b8; }
#container { display: flex; height: calc(100vh - 57px); }
#map { flex: 1; }
#panel { width: 360px; padding: 20px; overflow-y: auto; background: #1e293b; border-left: 1px solid #334155; }
#panel h2 { font-size: 14px; text-transform: uppercase; color: #94a3b8; margin-bottom: 12px; letter-spacing: 0.5px; }
.metric { display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #334155; }
.metric-label { color: #94a3b8; font-size: 13px; }
.metric-value { font-weight: 600; font-size: 13px; }
.metric-value.green { color: #22c55e; }
.metric-value.yellow { color: #eab308; }
.metric-value.orange { color: #f97316; }
.metric-value.red { color: #ef4444; }
.rec { padding: 12px; margin-bottom: 8px; background: #0f172a; border-radius: 8px; border: 1px solid #334155; }
.rec-name { font-weight: 600; font-size: 14px; color: #f8fafc; }
.rec-crs { font-size: 12px; color: #38bdf8; margin-top: 2px; }
.rec-rationale { font-size: 12px; color: #94a3b8; margin-top: 6px; }
#score-container { max-width: 700px; margin: 40px auto; padding: 0 24px; }
#overall-score { text-align: center; margin-bottom: 40px; }
.score-number { font-size: 80px; font-weight: 700; }
.score-grade { font-size: 32px; font-weight: 600; margin-top: 4px; }
.category { padding: 16px; margin-bottom: 12px; background: #1e293b; border-radius: 8px; border: 1px solid #334155; }
.cat-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
.cat-name { font-weight: 600; }
.cat-score { font-weight: 700; }
.bar-bg { height: 8px; background: #334155; border-radius: 4px; overflow: hidden; }
.bar-fill { height: 100%; border-radius: 4px; transition: width 0.5s; }
.finding-item { padding: 10px; margin-bottom: 6px; background: #0f172a; border-radius: 6px; border-left: 3px solid; }
.finding-item.error { border-color: #ef4444; }
.finding-item.warning { border-color: #eab308; }
.finding-item.info { border-color: #38bdf8; }
.finding-rule { font-size: 11px; color: #64748b; }
.finding-msg { font-size: 13px; margin-top: 4px; }
.finding-suggestion { font-size: 12px; color: #94a3b8; margin-top: 4px; font-style: italic; }
"#;

/// X-Ray report JavaScript — initializes map with heatmap + ellipses.
const XRAY_JS: &str = r#"
(function() {
  // Populate metrics panel
  const mp = document.getElementById('metrics');
  mp.innerHTML = '<h2>Distortion Metrics</h2>';
  const s = REPORT.summary;
  const metrics = [
    ['Max Area Distortion', s.max_area_distortion_pct.toFixed(1) + '%'],
    ['Mean Area Distortion', s.mean_area_distortion_pct.toFixed(1) + '%'],
    ['Median Area Distortion', s.median_area_distortion_pct.toFixed(1) + '%'],
    ['Max Angular Distortion', s.max_angular_distortion_deg.toFixed(1) + '°'],
    ['Mean Angular Distortion', s.mean_angular_distortion_deg.toFixed(1) + '°'],
    ['Sample Points', s.sample_count],
    ['Source CRS', REPORT.source_crs],
  ];
  metrics.forEach(([label, value]) => {
    const cls = typeof value === 'string' && value.endsWith('%')
      ? (parseFloat(value) < 2 ? 'green' : parseFloat(value) < 5 ? 'yellow' : parseFloat(value) < 10 ? 'orange' : 'red')
      : '';
    mp.innerHTML += `<div class="metric"><span class="metric-label">${label}</span><span class="metric-value ${cls}">${value}</span></div>`;
  });

  // Populate recommendations
  const rp = document.getElementById('recommendations');
  rp.innerHTML = '<h2 style="margin-top:20px">CRS Recommendations</h2>';
  (REPORT.recommendations || []).forEach((r, i) => {
    rp.innerHTML += `<div class="rec"><div class="rec-name">${i+1}. ${r.name}</div><div class="rec-crs">${r.crs}</div><div class="rec-rationale">${r.rationale}</div></div>`;
  });

  // Legend
  const lp = document.getElementById('legend');
  lp.innerHTML = '<h2 style="margin-top:20px">Distortion Scale</h2>';
  const colors = [['< 2%', '#22c55e'], ['2-5%', '#eab308'], ['5-10%', '#f97316'], ['> 10%', '#ef4444']];
  colors.forEach(([label, color]) => {
    lp.innerHTML += `<div style="display:flex;align-items:center;gap:8px;margin:4px 0"><div style="width:14px;height:14px;border-radius:3px;background:${color}"></div><span style="font-size:13px">${label}</span></div>`;
  });

  // Initialize map
  const map = new maplibregl.Map({
    container: 'map',
    style: { version: 8, sources: {}, layers: [{ id: 'bg', type: 'background', paint: { 'background-color': '#0f172a' } }] },
    center: [(REPORT.heatmap?.bounds?.[0] ?? 0 + REPORT.heatmap?.bounds?.[2] ?? 0) / 2 || 0, (REPORT.heatmap?.bounds?.[1] ?? 0 + REPORT.heatmap?.bounds?.[3] ?? 0) / 2 || 0],
    zoom: 4
  });

  map.on('load', () => {
    // Add heatmap layer
    if (HEATMAP && HEATMAP.features) {
      map.addSource('heatmap', { type: 'geojson', data: HEATMAP });
      map.addLayer({
        id: 'heatmap-fill',
        type: 'fill',
        source: 'heatmap',
        paint: { 'fill-color': ['get', 'color'], 'fill-opacity': 0.6 }
      });
    }

    // Add ellipses layer
    if (ELLIPSES && ELLIPSES.features) {
      map.addSource('ellipses', { type: 'geojson', data: ELLIPSES });
      map.addLayer({
        id: 'ellipses-fill',
        type: 'fill',
        source: 'ellipses',
        paint: { 'fill-color': '#38bdf8', 'fill-opacity': 0.3 }
      });
      map.addLayer({
        id: 'ellipses-line',
        type: 'line',
        source: 'ellipses',
        paint: { 'line-color': '#38bdf8', 'line-width': 1.5 }
      });
    }

    // Fit to bounds
    if (HEATMAP.features && HEATMAP.features.length > 0) {
      const b = REPORT.heatmap?.bounds;
      if (b) map.fitBounds([[b[0], b[1]], [b[2], b[3]]], { padding: 40 });
    }
  });
})();
"#;

/// Findings report JavaScript.
const FINDINGS_JS: &str = r#"
(function() {
  const fl = document.getElementById('findings-list');
  fl.innerHTML = `<h2>Findings (${REPORT.findings.length})</h2>
    <div style="margin-bottom:12px;font-size:13px;color:#94a3b8">
      ${REPORT.summary.errors} errors · ${REPORT.summary.warnings} warnings · ${REPORT.summary.info} info
    </div>`;
  REPORT.findings.forEach(f => {
    fl.innerHTML += `<div class="finding-item ${f.severity}">
      <div class="finding-rule">${f.rule_id}</div>
      <div class="finding-msg">${f.message}</div>
      ${f.suggestion ? `<div class="finding-suggestion">→ ${f.suggestion}</div>` : ''}
    </div>`;
  });

  // Map with findings geometries.
  const map = new maplibregl.Map({
    container: 'map',
    style: { version: 8, sources: {}, layers: [{ id: 'bg', type: 'background', paint: { 'background-color': '#0f172a' } }] },
    center: [0, 20],
    zoom: 2
  });

  map.on('load', () => {
    map.addSource('findings', { type: 'geojson', data: FINDINGS_GEOJSON });

    map.addLayer({
      id: 'findings-fill',
      type: 'fill',
      source: 'findings',
      paint: {
        'fill-color': ['match', ['get', 'severity'], 'error', '#ef4444', 'warning', '#eab308', '#38bdf8'],
        'fill-opacity': 0.35,
      },
      filter: ['==', '$type', 'Polygon'],
    });

    map.addLayer({
      id: 'findings-line',
      type: 'line',
      source: 'findings',
      paint: {
        'line-color': ['match', ['get', 'severity'], 'error', '#ef4444', 'warning', '#eab308', '#38bdf8'],
        'line-width': 2,
      },
      filter: ['==', '$type', 'LineString'],
    });

    map.addLayer({
      id: 'findings-points',
      type: 'circle',
      source: 'findings',
      paint: {
        'circle-radius': 5,
        'circle-color': ['match', ['get', 'severity'], 'error', '#ef4444', 'warning', '#eab308', '#38bdf8'],
        'circle-stroke-width': 1,
        'circle-stroke-color': '#0f172a',
      },
      filter: ['==', '$type', 'Point'],
    });

    const extent = geojsonExtent(FINDINGS_GEOJSON);
    if (extent) {
      map.fitBounds([[extent[0], extent[1]], [extent[2], extent[3]]], { padding: 40 });
    }

    const popup = new maplibregl.Popup({ closeButton: false, closeOnClick: false });
    map.on('mousemove', 'findings-points', (e) => {
      const f = e.features && e.features[0];
      if (!f) return;
      const p = f.properties || {};
      popup
        .setLngLat(e.lngLat)
        .setHTML(`<strong>${p.rule_id || ''}</strong><br/>${p.message || ''}`)
        .addTo(map);
    });
    map.on('mouseleave', 'findings-points', () => popup.remove());
  });

  function geojsonExtent(fc) {
    if (!fc || !fc.features || fc.features.length === 0) return null;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    function scan(coords) {
      if (!Array.isArray(coords)) return;
      if (typeof coords[0] === 'number' && typeof coords[1] === 'number') {
        minX = Math.min(minX, coords[0]);
        minY = Math.min(minY, coords[1]);
        maxX = Math.max(maxX, coords[0]);
        maxY = Math.max(maxY, coords[1]);
      } else {
        coords.forEach(scan);
      }
    }

    fc.features.forEach(f => {
      if (f && f.geometry) scan(f.geometry.coordinates);
    });

    if (!isFinite(minX)) return null;
    return [minX, minY, maxX, maxY];
  }
})();
"#;

/// Score dashboard JavaScript.
const SCORE_JS: &str = r#"
(function() {
  const os = document.getElementById('overall-score');
  const color = REPORT.overall >= 90 ? '#22c55e' : REPORT.overall >= 75 ? '#22c55e' : REPORT.overall >= 60 ? '#eab308' : REPORT.overall >= 40 ? '#f97316' : '#ef4444';
  os.innerHTML = `<div class="score-number" style="color:${color}">${REPORT.overall}</div>
    <div class="score-grade" style="color:${color}">${REPORT.grade}</div>
    <div style="font-size:14px;color:#94a3b8;margin-top:8px">${REPORT.finding_count} findings analyzed</div>`;

  const cats = document.getElementById('categories');
  (REPORT.categories || []).forEach(c => {
    const cc = c.score >= 90 ? '#22c55e' : c.score >= 75 ? '#22c55e' : c.score >= 60 ? '#eab308' : c.score >= 40 ? '#f97316' : '#ef4444';
    cats.innerHTML += `<div class="category">
      <div class="cat-header"><span class="cat-name">${c.category.replace('_', ' ')}</span><span class="cat-score" style="color:${cc}">${c.score}/100 ${c.grade}</span></div>
      <div class="bar-bg"><div class="bar-fill" style="width:${c.score}%;background:${cc}"></div></div>
      <div style="font-size:12px;color:#64748b;margin-top:6px">${c.finding_count} findings · weight: ${(c.weight*100).toFixed(0)}%</div>
    </div>`;
  });
})();
"#;

/// Diff report JavaScript.
const DIFF_JS: &str = r#"
(function() {
  const summary = document.getElementById('diff-summary');
  summary.innerHTML = `
    <h2>Diff Summary</h2>
    <div class="metric"><span class="metric-label">Before Features</span><span class="metric-value">${REPORT.left_features}</span></div>
    <div class="metric"><span class="metric-label">After Features</span><span class="metric-value">${REPORT.right_features}</span></div>
    <div class="metric"><span class="metric-label">Added</span><span class="metric-value">${REPORT.added}</span></div>
    <div class="metric"><span class="metric-label">Removed</span><span class="metric-value">${REPORT.removed}</span></div>
    <div class="metric"><span class="metric-label">Extent Changed</span><span class="metric-value">${REPORT.extent_changed ? 'yes' : 'no'}</span></div>
  `;

  const map = new maplibregl.Map({
    container: 'map',
    style: { version: 8, sources: {}, layers: [{ id: 'bg', type: 'background', paint: { 'background-color': '#0f172a' } }] },
    center: [0, 20],
    zoom: 2,
  });

  map.on('load', () => {
    map.addSource('before', { type: 'geojson', data: LEFT });
    map.addSource('after', { type: 'geojson', data: RIGHT });

    map.addLayer({
      id: 'before-fill',
      type: 'fill',
      source: 'before',
      paint: { 'fill-color': '#38bdf8', 'fill-opacity': 0.35 },
      filter: ['==', '$type', 'Polygon'],
    });
    map.addLayer({
      id: 'before-line',
      type: 'line',
      source: 'before',
      paint: { 'line-color': '#38bdf8', 'line-width': 1.2, 'line-opacity': 0.9 },
    });

    map.addLayer({
      id: 'after-fill',
      type: 'fill',
      source: 'after',
      paint: { 'fill-color': '#f97316', 'fill-opacity': 0.7 },
      filter: ['==', '$type', 'Polygon'],
    });
    map.addLayer({
      id: 'after-line',
      type: 'line',
      source: 'after',
      paint: { 'line-color': '#f97316', 'line-width': 1.2, 'line-opacity': 1.0 },
    });

    const slider = document.getElementById('right-opacity');
    slider.addEventListener('input', () => {
      const opacity = Number(slider.value) / 100;
      map.setPaintProperty('after-fill', 'fill-opacity', opacity);
      map.setPaintProperty('after-line', 'line-opacity', opacity);
    });

    const beforeExtent = geojsonExtent(LEFT);
    const afterExtent = geojsonExtent(RIGHT);
    const extent = combineExtents(beforeExtent, afterExtent);
    if (extent) {
      map.fitBounds([[extent[0], extent[1]], [extent[2], extent[3]]], { padding: 40 });
    }
  });

  function geojsonExtent(fc) {
    if (!fc || !fc.features || fc.features.length === 0) return null;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    function scan(coords) {
      if (!Array.isArray(coords)) return;
      if (typeof coords[0] === 'number' && typeof coords[1] === 'number') {
        minX = Math.min(minX, coords[0]);
        minY = Math.min(minY, coords[1]);
        maxX = Math.max(maxX, coords[0]);
        maxY = Math.max(maxY, coords[1]);
      } else {
        coords.forEach(scan);
      }
    }

    fc.features.forEach(f => {
      if (f && f.geometry) scan(f.geometry.coordinates);
    });

    if (!isFinite(minX)) return null;
    return [minX, minY, maxX, maxY];
  }

  function combineExtents(a, b) {
    if (!a && !b) return null;
    if (!a) return b;
    if (!b) return a;
    return [
      Math.min(a[0], b[0]),
      Math.min(a[1], b[1]),
      Math.max(a[2], b[2]),
      Math.max(a[3], b[3]),
    ];
  }
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xray_html_contains_essentials() {
        let html = build_xray_html("{}", "{}", "{}");
        assert!(html.contains("Tissot X-Ray"));
        assert!(html.contains("maplibre-gl"));
        assert!(html.contains("Distortion"));
    }

    #[test]
    fn findings_html_contains_essentials() {
        let html = build_findings_html("{}", "{}");
        assert!(html.contains("Tissot Check"));
    }

    #[test]
    fn score_html_contains_essentials() {
        let html = build_score_html("{}");
        assert!(html.contains("Tissot Score"));
    }

    #[test]
    fn diff_html_contains_essentials() {
        let html = build_diff_html("{}", "{}", "{}");
        assert!(html.contains("Tissot Diff"));
        assert!(html.contains("Comparison Slider"));
    }
}
