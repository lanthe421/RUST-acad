use std::fs::File;
use std::io;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    prelude::*,
    Layer,
    filter::LevelFilter,
};

fn main() {
    // stdout: INFO/DEBUG only (excludes WARN and above)
    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_filter(LevelFilter::INFO)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.level() > &Level::WARN
        }));

    // stderr: WARN and above
    let stderr_layer = fmt::layer()
        .with_writer(io::stderr)
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_filter(LevelFilter::WARN);

    // file: only events with target "access_log"
    let file = File::create("access.log").expect("failed to create access.log");
    let access_log_layer = fmt::layer()
        .with_writer(file)
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target() == "access_log"
        }));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(stderr_layer)
        .with(access_log_layer)
        .init();

    info!(file = "app.log", "App started");
    warn!(file = "app.log", "Something looks fishy");
    error!(file = "app.log", "Error occurred");

    info!(
        target: "access_log",
        file = "access.log",
        method = "POST",
        path = "/some",
        "http"
    );
}
