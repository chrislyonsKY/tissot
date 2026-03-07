/// Watch mode — monitor a directory and run checks on file changes.
use crate::core::config::Config;
use crate::core::error::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;

/// Run watch mode loop for a directory.
pub fn run(dir: &Path, config: &Config) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default()).map_err(|e| {
        crate::core::error::TissotError::Internal(format!("Watch initialization failed: {e}"))
    })?;
    watcher.watch(dir, RecursiveMode::Recursive).map_err(|e| {
        crate::core::error::TissotError::Internal(format!("Watch registration failed: {e}"))
    })?;

    log::info!("Watching {} for changes", dir.display());

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                for p in event.paths {
                    if p.is_file() {
                        if let Ok(layers) = crate::io::read_file(&p) {
                            let findings = crate::checkers::run_checks(
                                &layers,
                                config,
                                &p.display().to_string(),
                                None,
                            );
                            let score = crate::score::compute_score(&findings, config);
                            log::info!(
                                "Changed: {} -> {} findings, score {}/100",
                                p.display(),
                                findings.len(),
                                score.overall
                            );
                        }
                    }
                }
            }
            Ok(Err(e)) => log::warn!("Watch error: {e}"),
            Err(e) => log::warn!("Watch channel error: {e}"),
        }
    }
}
