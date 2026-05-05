pub mod config;
pub mod downloader;
pub mod input;
pub mod processor;
pub mod runner;

use std::process;

use config::Config;
use input::collect_sources;
use runner::run;

#[tokio::main]
async fn main() {
    // Initialise tracing with RUST_LOG support
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load and validate configuration
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("configuration error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = config.validate() {
        tracing::error!("invalid configuration: {}", e);
        process::exit(1);
    }

    // Collect input sources from CLI args, --input-file, or STDIN
    let sources = match collect_sources(&config.sources, config.input_file.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to collect input sources: {}", e);
            process::exit(1);
        }
    };

    if sources.is_empty() {
        tracing::warn!("no input sources provided; nothing to do");
        process::exit(0);
    }

    // Run the optimizer
    let stats = run(config, sources).await;

    // Requirement 6.4: log total execution time
    tracing::info!(
        "done — {} succeeded, {} failed, total elapsed: {:.2}s",
        stats.succeeded,
        stats.failed,
        stats.total_elapsed.as_secs_f64()
    );

    if stats.failed > 0 {
        process::exit(1);
    }
}
