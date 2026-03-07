/// Score engine — aggregates findings into a Lighthouse-style 0-100 quality score.
pub mod badge;
pub mod calculator;
pub mod categories;

pub use calculator::{ScoreReport, compute_score};
