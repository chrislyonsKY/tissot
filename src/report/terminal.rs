/// Terminal output renderer — rich text summaries for CLI use.
use crate::core::report::ReportData;
use crate::score::ScoreReport;
use crate::xray::XrayReport;

/// Print X-Ray summary to terminal.
pub fn print_xray(report: &XrayReport) {
    println!();
    println!("  \x1b[1;36m⊕ Tissot X-Ray\x1b[0m — Projection Distortion Analysis");
    println!("  ─────────────────────────────────────────────────");
    println!("  File:   {}", report.file_path);
    println!("  CRS:    {}", report.source_crs);
    println!("  Points: {}", report.summary.sample_count);
    println!();
    println!("  \x1b[1mDistortion Metrics\x1b[0m");
    println!(
        "    Max area distortion:     \x1b[{}m{:.1}%\x1b[0m",
        distortion_color_code(report.summary.max_area_distortion_pct),
        report.summary.max_area_distortion_pct
    );
    println!(
        "    Mean area distortion:    {:.1}%",
        report.summary.mean_area_distortion_pct
    );
    println!(
        "    Median area distortion:  {:.1}%",
        report.summary.median_area_distortion_pct
    );
    println!(
        "    Max angular distortion:  {:.1}°",
        report.summary.max_angular_distortion_deg
    );
    println!(
        "    Mean angular distortion: {:.1}°",
        report.summary.mean_angular_distortion_deg
    );

    if !report.recommendations.is_empty() {
        println!();
        println!("  \x1b[1mCRS Recommendations\x1b[0m");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!(
                "    {}. {} — {} (fitness: {:.0}%)",
                i + 1,
                rec.crs,
                rec.name,
                rec.fitness * 100.0
            );
            println!("       {}", rec.rationale);
        }
    }
    println!();
}

/// Print check findings summary to terminal.
pub fn print_check(report: &ReportData) {
    println!();
    println!("  \x1b[1;36m⊕ Tissot Check\x1b[0m — Diagnostic Results");
    println!("  ───────────────────────────────────────");
    println!("  File: {}", report.file_path);
    println!(
        "  Found: {} issues ({} errors, {} warnings, {} info)",
        report.summary.total, report.summary.errors, report.summary.warnings, report.summary.info
    );
    println!();

    for finding in &report.findings {
        let icon = match finding.severity {
            crate::core::rule::Severity::Error => "\x1b[31m✗\x1b[0m",
            crate::core::rule::Severity::Warning => "\x1b[33m⚠\x1b[0m",
            crate::core::rule::Severity::Info => "\x1b[34mℹ\x1b[0m",
        };
        println!("  {} [{}] {}", icon, finding.rule_id, finding.message);
        if let Some(ref suggestion) = finding.suggestion {
            println!("    → {suggestion}");
        }
    }
    println!();
}

/// Print score summary to terminal.
pub fn print_score(report: &ScoreReport) {
    println!();
    println!("  \x1b[1;36m⊕ Tissot Score\x1b[0m — Map Quality Rating");
    println!("  ──────────────────────────────────────");

    let grade_color = match report.grade.as_str() {
        "A" => "32", // green
        "B" => "32",
        "C" => "33", // yellow
        "D" => "33",
        _ => "31", // red
    };

    println!(
        "  Overall: \x1b[1;{grade_color}m{}/100 — {}\x1b[0m",
        report.overall, report.grade
    );
    println!();

    for cat in &report.categories {
        let bar = score_bar(cat.score);
        println!(
            "    {:<18} {} {}/100 ({})",
            cat.category.to_string(),
            bar,
            cat.score,
            cat.grade
        );
    }

    println!();
    println!("  {} findings analyzed", report.finding_count);
    println!();
}

/// Generate a colored score bar.
fn score_bar(score: u32) -> String {
    let filled = (score as usize) / 5;
    let empty = 20 - filled;
    let color = if score >= 75 {
        "32"
    } else if score >= 40 {
        "33"
    } else {
        "31"
    };

    format!(
        "\x1b[{color}m{}\x1b[0m{}",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}

/// ANSI color code for distortion percentage.
fn distortion_color_code(pct: f64) -> &'static str {
    if pct < 2.0 {
        "32" // green
    } else if pct < 10.0 {
        "33" // yellow
    } else {
        "31" // red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_bar_full() {
        let bar = score_bar(100);
        assert!(bar.contains("████████████████████"));
    }

    #[test]
    fn score_bar_empty() {
        let bar = score_bar(0);
        assert!(bar.contains("░░░░░░░░░░░░░░░░░░░░"));
    }
}
