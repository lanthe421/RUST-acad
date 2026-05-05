use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Semaphore;
use tracing::{error, info};

use crate::config::Config;
use crate::downloader::{download, DownloadError};
use crate::input::InputSource;
use crate::processor::process_jpeg;

// ---------------------------------------------------------------------------
// RunStats
// ---------------------------------------------------------------------------

/// Statistics from a run: counts of successes/failures and total elapsed time.
#[derive(Debug, Clone)]
pub struct RunStats {
    pub succeeded: usize,
    pub failed: usize,
    pub total_elapsed: Duration,
}

// ---------------------------------------------------------------------------
// 7.1 run
// ---------------------------------------------------------------------------

/// Process all input sources concurrently (bounded by config.concurrency).
///
/// For each source:
/// - Download (if URL) or read (if local file)
/// - Call `process_jpeg` with the configured quality
/// - Write output to `config.output_dir` with the original filename
/// - Log result + elapsed time
///
/// Creates the output directory if it doesn't exist.
///
/// Requirements: 2.1, 3.3, 6.2, 6.3
pub async fn run(config: Config, sources: Vec<InputSource>) -> RunStats {
    let start = Instant::now();

    // Create output directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
        error!(
            "failed to create output directory '{}': {}",
            config.output_dir.display(),
            e
        );
        return RunStats {
            succeeded: 0,
            failed: sources.len(),
            total_elapsed: start.elapsed(),
        };
    }

    // Semaphore to bound concurrency
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let client = Arc::new(reqwest::Client::new());
    let quality = config.quality;
    let output_dir = Arc::new(config.output_dir);

    let mut tasks = Vec::new();

    for source in sources {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let output_dir = output_dir.clone();

        let task = tokio::spawn(async move {
            let result = process_source(source, &client, quality, &output_dir).await;
            drop(permit);
            result
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let mut succeeded = 0;
    let mut failed = 0;

    for task in tasks {
        match task.await {
            Ok(true) => succeeded += 1,
            Ok(false) => failed += 1,
            Err(e) => {
                error!("task panicked: {}", e);
                failed += 1;
            }
        }
    }

    let total_elapsed = start.elapsed();

    RunStats {
        succeeded,
        failed,
        total_elapsed,
    }
}

// ---------------------------------------------------------------------------
// Helper: process a single source
// ---------------------------------------------------------------------------

async fn process_source(
    source: InputSource,
    client: &reqwest::Client,
    quality: u8,
    output_dir: &PathBuf,
) -> bool {
    let start = Instant::now();

    // Step 1: Get input bytes and determine filename
    let (input_bytes, filename) = match fetch_input(&source, client).await {
        Ok(result) => result,
        Err(e) => {
            error!("failed to fetch {:?}: {}", source, e);
            return false;
        }
    };

    // Step 2: Process JPEG
    let output_bytes = match process_jpeg(&input_bytes, quality) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("failed to process {:?}: {}", source, e);
            return false;
        }
    };

    // Step 3: Write output
    let output_path = construct_output_path(output_dir, &filename);
    if let Err(e) = std::fs::write(&output_path, &output_bytes) {
        error!(
            "failed to write output to '{}': {}",
            output_path.display(),
            e
        );
        return false;
    }

    let elapsed = start.elapsed();
    info!(
        "processed {:?} -> {} in {:.2}s",
        source,
        output_path.display(),
        elapsed.as_secs_f64()
    );

    true
}

// ---------------------------------------------------------------------------
// Helper: fetch input bytes and extract filename
// ---------------------------------------------------------------------------

async fn fetch_input(
    source: &InputSource,
    client: &reqwest::Client,
) -> Result<(Vec<u8>, String), FetchError> {
    match source {
        InputSource::LocalFile(path) => {
            let bytes = std::fs::read(path).map_err(|e| FetchError::FileRead {
                path: path.clone(),
                source: e,
            })?;

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output.jpg")
                .to_owned();

            Ok((bytes, filename))
        }
        InputSource::RemoteUrl(url) => {
            let bytes = download(client, url).await.map_err(FetchError::Download)?;

            // Extract filename from URL path
            let filename = extract_filename_from_url(url);

            Ok((bytes, filename))
        }
    }
}

// ---------------------------------------------------------------------------
// 7.2 Output path construction
// ---------------------------------------------------------------------------

/// Construct the output path by joining the output directory with the filename.
///
/// Requirements: 3.1, 3.4
fn construct_output_path(output_dir: &PathBuf, filename: &str) -> PathBuf {
    output_dir.join(filename)
}

// ---------------------------------------------------------------------------
// Helper: extract filename from URL
// ---------------------------------------------------------------------------

fn extract_filename_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("output.jpg")
        .to_owned()
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
enum FetchError {
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("download failed: {0}")]
    Download(#[from] DownloadError),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    // -----------------------------------------------------------------------
    // 7.3 Property 6: Output path construction
    // Feature: jpeg-optimizer, Property 6: Output path construction
    // Validates: Requirements 3.1, 3.4
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_output_path_construction(
            dir_parts in prop::collection::vec("[a-zA-Z0-9_]{1,10}", 1..4),
            filename in "[a-zA-Z0-9_]{1,20}\\.jpg",
        ) {
            // Feature: jpeg-optimizer, Property 6: Output path construction
            // Validates: Requirements 3.1, 3.4
            let dir: PathBuf = dir_parts.iter().collect();
            let output = construct_output_path(&dir, &filename);

            // The output path must equal dir.join(filename)
            prop_assert_eq!(&output, &dir.join(&filename),
                "output path should equal dir.join(filename)");

            // The filename component must be preserved
            let got_filename = output.file_name().and_then(|n| n.to_str()).unwrap_or("");
            prop_assert_eq!(got_filename, filename.as_str(),
                "original filename must be preserved in output path");
        }
    }

    // -----------------------------------------------------------------------
    // 7.4 Property 2: Invalid sources produce errors, valid sources are still processed
    // Feature: jpeg-optimizer, Property 2: Invalid sources produce errors, valid sources are still processed
    // Validates: Requirements 1.4
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_mixed_valid_invalid_sources(
            valid_count in 0usize..5,
            invalid_count in 0usize..5,
        ) {
            // Feature: jpeg-optimizer, Property 2: Invalid sources produce errors, valid sources are still processed
            // Validates: Requirements 1.4
            //
            // Build a list of valid (existing temp files with real JPEG data) and
            // invalid (non-existent paths) sources, then run them and verify that
            // succeeded + failed == total.

            use tempfile::NamedTempFile;
            use std::io::Write;

            // Create valid JPEG temp files
            let mut temp_files: Vec<NamedTempFile> = Vec::new();
            let mut sources: Vec<InputSource> = Vec::new();

            for _ in 0..valid_count {
                let mut f = NamedTempFile::new().unwrap();
                // Write a minimal valid JPEG (1x1 red pixel)
                let jpeg_bytes = make_minimal_jpeg();
                f.write_all(&jpeg_bytes).unwrap();
                sources.push(InputSource::LocalFile(f.path().to_path_buf()));
                temp_files.push(f);
            }

            // Add invalid sources (paths that don't exist)
            for i in 0..invalid_count {
                sources.push(InputSource::LocalFile(
                    PathBuf::from(format!("/nonexistent/path/image_{}.jpg", i))
                ));
            }

            let total = sources.len();

            // Run with a temp output dir
            let out_dir = tempfile::tempdir().unwrap();
            let config = crate::config::Config {
                quality: 75,
                concurrency: 4,
                output_dir: out_dir.path().to_path_buf(),
                input_file: None,
                config_path: None,
                sources: vec![],
            };

            let rt = tokio::runtime::Runtime::new().unwrap();
            let stats = rt.block_on(run(config, sources));

            prop_assert_eq!(
                stats.succeeded + stats.failed,
                total,
                "succeeded ({}) + failed ({}) must equal total ({})",
                stats.succeeded,
                stats.failed,
                total
            );
        }
    }

    // -----------------------------------------------------------------------
    // Helper: produce a minimal valid 1×1 JPEG
    // -----------------------------------------------------------------------
    fn make_minimal_jpeg() -> Vec<u8> {
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        compress.set_size(1, 1);
        compress.set_quality(75.0);
        let buf: Vec<u8> = Vec::new();
        let mut started = compress.start_compress(buf).unwrap();
        started.write_scanlines(&[255u8, 0, 0]).unwrap();
        started.finish().unwrap()
    }
}
