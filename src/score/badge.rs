/// SVG badge generation for quality scores.
use super::calculator::ScoreReport;

/// Generate a shields.io-style SVG badge.
pub fn generate_badge(report: &ScoreReport) -> String {
    let color = badge_color(report.overall);
    let label = "Tissot Score";
    let value = format!("{}/100 — {}", report.overall, report.grade);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="20" role="img" aria-label="{label}: {value}">
  <title>{label}: {value}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="200" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="90" height="20" fill="#555"/>
    <rect x="90" width="110" height="20" fill="{color}"/>
    <rect width="200" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="11">
    <text aria-hidden="true" x="45" y="15" fill="#010101" fill-opacity=".3">{label}</text>
    <text x="45" y="14">{label}</text>
    <text aria-hidden="true" x="145" y="15" fill="#010101" fill-opacity=".3">{value}</text>
    <text x="145" y="14">{value}</text>
  </g>
</svg>"##
    )
}

/// Determine badge color based on score.
fn badge_color(score: u32) -> &'static str {
    match score {
        90..=100 => "#4c1",   // bright green
        75..=89 => "#97ca00", // green
        60..=74 => "#dfb317", // yellow
        40..=59 => "#fe7d37", // orange
        _ => "#e05d44",       // red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_colors() {
        assert_eq!(badge_color(95), "#4c1");
        assert_eq!(badge_color(80), "#97ca00");
        assert_eq!(badge_color(65), "#dfb317");
        assert_eq!(badge_color(50), "#fe7d37");
        assert_eq!(badge_color(20), "#e05d44");
    }

    #[test]
    fn badge_generation() {
        use crate::score::categories::{Category, CategoryScore};

        let report = ScoreReport {
            overall: 87,
            grade: "B".into(),
            categories: vec![CategoryScore {
                category: Category::Projection,
                score: 85,
                weight: 0.25,
                finding_count: 1,
                grade: "B".into(),
            }],
            finding_count: 1,
        };

        let svg = generate_badge(&report);
        assert!(svg.contains("Tissot Score"));
        assert!(svg.contains("87/100"));
        assert!(svg.contains("#97ca00")); // green for B grade
    }
}
