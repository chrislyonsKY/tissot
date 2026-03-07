/// Score engine — aggregates findings into a Lighthouse-style 0-100 quality score.
pub mod badge;
pub mod calculator;
pub mod categories;

pub use badge::generate_badge;
pub use calculator::{ScoreReport, compute_score};
pub use categories::{Category as ScoreCategory, CategoryScore};
