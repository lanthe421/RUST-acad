use std::io::BufRead;
use std::path::{Path, PathBuf};

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum InputError {
    #[error("failed to read input file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read from stdin: {0}")]
    StdinRead(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// 3.1 InputSource enum
// ---------------------------------------------------------------------------

/// Represents a single input: either a local file path or a remote URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    LocalFile(PathBuf),
    RemoteUrl(String),
}

impl InputSource {
    /// Classify a trimmed, non-empty line as a local file or remote URL.
    fn from_line(line: &str) -> Self {
        if line.starts_with("http://") || line.starts_with("https://") {
            InputSource::RemoteUrl(line.to_owned())
        } else {
            InputSource::LocalFile(PathBuf::from(line))
        }
    }
}

// ---------------------------------------------------------------------------
// 3.2 parse_sources
// ---------------------------------------------------------------------------

/// Parse EOL-separated lines from a reader, ignoring blank/whitespace-only lines.
/// Each non-blank line is classified as a local file path or a remote URL.
///
/// Requirements: 1.3, 1.5
pub fn parse_sources(reader: impl BufRead) -> Vec<InputSource> {
    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(InputSource::from_line(trimmed))
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 3.3 collect_sources
// ---------------------------------------------------------------------------

/// Collect input sources from (in priority order):
/// 1. CLI positional arguments (if any)
/// 2. `--input-file <path>` (if provided)
/// 3. STDIN (if not a TTY and no other source was given)
///
/// Requirements: 1.1, 1.2, 1.3
pub fn collect_sources(
    cli_args: &[String],
    input_file: Option<&Path>,
) -> Result<Vec<InputSource>, InputError> {
    // 1. CLI positional args take priority
    if !cli_args.is_empty() {
        return Ok(cli_args
            .iter()
            .map(|s| InputSource::from_line(s.trim()))
            .filter(|_| true) // keep all; blank CLI args are unusual but harmless
            .collect());
    }

    // 2. --input-file
    if let Some(path) = input_file {
        let file = std::fs::File::open(path).map_err(|e| InputError::FileRead {
            path: path.to_owned(),
            source: e,
        })?;
        return Ok(parse_sources(std::io::BufReader::new(file)));
    }

    // 3. STDIN (only when not a TTY)
    if !is_stdin_tty() {
        let stdin = std::io::stdin();
        return Ok(parse_sources(stdin.lock()));
    }

    Ok(vec![])
}

/// Returns `true` when stdin is connected to a terminal (interactive).
fn is_stdin_tty() -> bool {
    // Use the `atty` approach via std: check if stdin fd is a tty.
    // We use a simple cross-platform heuristic via `std::io::IsTerminal` (stable since 1.70).
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // 3.4 Unit tests for parse_sources
    // -----------------------------------------------------------------------

    #[test]
    fn empty_input_returns_empty_vec() {
        let input = b"";
        let sources = parse_sources(input.as_ref());
        assert!(sources.is_empty());
    }

    #[test]
    fn only_blank_lines_returns_empty_vec() {
        let input = b"\n   \n\t\n\n";
        let sources = parse_sources(input.as_ref());
        assert!(sources.is_empty());
    }

    #[test]
    fn local_file_lines_are_classified_correctly() {
        let input = b"/tmp/photo.jpg\n./image.jpeg\n";
        let sources = parse_sources(input.as_ref());
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0], InputSource::LocalFile(PathBuf::from("/tmp/photo.jpg")));
        assert_eq!(sources[1], InputSource::LocalFile(PathBuf::from("./image.jpeg")));
    }

    #[test]
    fn url_lines_are_classified_correctly() {
        let input = b"https://example.com/photo.jpg\nhttp://cdn.example.com/img.jpeg\n";
        let sources = parse_sources(input.as_ref());
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0], InputSource::RemoteUrl("https://example.com/photo.jpg".to_owned()));
        assert_eq!(sources[1], InputSource::RemoteUrl("http://cdn.example.com/img.jpeg".to_owned()));
    }

    #[test]
    fn mix_of_files_urls_and_blanks() {
        let input = b"/tmp/a.jpg\n\nhttps://example.com/b.jpg\n   \n./c.jpg\n";
        let sources = parse_sources(input.as_ref());
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0], InputSource::LocalFile(PathBuf::from("/tmp/a.jpg")));
        assert_eq!(sources[1], InputSource::RemoteUrl("https://example.com/b.jpg".to_owned()));
        assert_eq!(sources[2], InputSource::LocalFile(PathBuf::from("./c.jpg")));
    }

    // -----------------------------------------------------------------------
    // 3.5 Property 1: Input file parsing ignores blank lines
    // Feature: jpeg-optimizer, Property 1: Input file parsing ignores blank lines
    // Validates: Requirements 1.3, 1.5
    // -----------------------------------------------------------------------
    proptest! {
        #[test]
        fn prop_blank_lines_are_ignored(lines in prop::collection::vec(
            prop_oneof![
                // blank / whitespace-only lines
                Just("".to_owned()),
                Just("   ".to_owned()),
                Just("\t".to_owned()),
                // non-blank lines (file paths and URLs)
                "[a-zA-Z0-9_/.-]{1,20}\\.jpg".prop_map(|s| s),
                "https://[a-z]{3,10}\\.com/[a-z]{1,10}\\.jpg".prop_map(|s| s),
            ],
            0..50,
        )) {
            // Feature: jpeg-optimizer, Property 1: Input file parsing ignores blank lines
            // Validates: Requirements 1.3, 1.5
            let non_blank_count = lines.iter().filter(|l| !l.trim().is_empty()).count();
            let input = lines.join("\n");
            let sources = parse_sources(input.as_bytes());
            prop_assert_eq!(
                sources.len(),
                non_blank_count,
                "expected {} sources but got {} for input {:?}",
                non_blank_count,
                sources.len(),
                lines
            );
        }
    }
}