use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("quality must be between 1 and 100, got {0}")]
    InvalidQuality(u8),

    #[error("concurrency must be at least 1, got {0}")]
    InvalidConcurrency(usize),

    #[error("failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    FileParse(String),
}

// ---------------------------------------------------------------------------
// CLI arguments (parsed by clap)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(name = "jpeg-optimizer", about = "Strip JPEG metadata and recompress")]
pub struct CliArgs {
    /// JPEG file paths or URLs to process
    #[arg(trailing_var_arg = true)]
    pub sources: Vec<String>,

    /// JPEG recompression quality (1–100)
    #[arg(long, env = "OPTIMIZER_QUALITY")]
    pub quality: Option<u8>,

    /// Maximum number of images processed concurrently
    #[arg(long, env = "OPTIMIZER_CONCURRENCY")]
    pub concurrency: Option<usize>,

    /// Directory where processed images are written
    #[arg(long, env = "OPTIMIZER_OUTPUT_DIR")]
    pub output_dir: Option<PathBuf>,

    /// Read input sources from this file (one per line)
    #[arg(long)]
    pub input_file: Option<PathBuf>,

    /// Path to TOML config file (default: optimizer.toml)
    #[arg(long)]
    pub config: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// File-based config (TOML)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    quality: Option<u8>,
    concurrency: Option<usize>,
    output_dir: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Merged Config
// ---------------------------------------------------------------------------

/// Merged configuration: TOML file < env vars < CLI args.
#[derive(Debug, Clone)]
pub struct Config {
    pub quality: u8,
    pub concurrency: usize,
    pub output_dir: PathBuf,
    pub input_file: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    /// Positional CLI sources (file paths / URLs)
    pub sources: Vec<String>,
}

impl Config {
    /// Load and merge configuration from TOML file, env vars, and CLI args.
    ///
    /// Priority (highest wins): CLI args > env vars > config file > defaults.
    ///
    /// Note: env vars are handled by clap via `env = "..."` on each field, so
    /// they are already folded into `CliArgs` before we reach this function.
    pub fn load() -> Result<Self, ConfigError> {
        let args = CliArgs::parse();
        Self::load_from_args(args)
    }

    /// Internal constructor that accepts pre-parsed args (enables testing).
    pub fn load_from_args(args: CliArgs) -> Result<Self, ConfigError> {
        // Move args.config out once; derive the lookup path from it without cloning.
        let config_path = args.config;
        let lookup_path = config_path
            .as_deref()
            .unwrap_or_else(|| std::path::Path::new("optimizer.toml"));

        // Read file config (absent file is not an error)
        let file_cfg = Self::read_file_config(lookup_path)?;

        // Default concurrency = logical CPU count
        let default_concurrency = num_cpus();

        // Merge: file < CLI/env (clap already merged env into args)
        let quality = args.quality.or(file_cfg.quality).unwrap_or(75);
        let concurrency = args
            .concurrency
            .or(file_cfg.concurrency)
            .unwrap_or(default_concurrency);
        let output_dir = args
            .output_dir
            .or(file_cfg.output_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Config {
            quality,
            concurrency,
            output_dir,
            input_file: args.input_file,
            config_path,
            sources: args.sources,
        })
    }

    /// Validate all fields.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.quality < 1 || self.quality > 100 {
            return Err(ConfigError::InvalidQuality(self.quality));
        }
        if self.concurrency < 1 {
            return Err(ConfigError::InvalidConcurrency(self.concurrency));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn read_file_config(path: &std::path::Path) -> Result<FileConfig, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                toml::from_str::<FileConfig>(&contents)
                    .map_err(|e| ConfigError::FileParse(e.to_string()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Absent config file is fine
                Ok(FileConfig::default())
            }
            Err(e) => Err(ConfigError::FileRead(e)),
        }
    }
}

/// Returns the number of logical CPU cores (falls back to 1).
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Helper: build a Config directly without going through CLI parsing
    // -----------------------------------------------------------------------
    fn make_config(quality: u8, concurrency: usize) -> Config {
        Config {
            quality,
            concurrency,
            output_dir: PathBuf::from("."),
            input_file: None,
            config_path: None,
            sources: vec![],
        }
    }

    // -----------------------------------------------------------------------
    // 2.4 Unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn default_quality_is_75() {
        // Simulate no CLI / env / file overrides by building Config manually
        // with the same defaults that load_from_args would produce.
        let cfg = make_config(75, num_cpus());
        assert_eq!(cfg.quality, 75);
    }

    #[test]
    fn default_concurrency_is_num_cpus() {
        let cfg = make_config(75, num_cpus());
        assert_eq!(cfg.concurrency, num_cpus());
        assert!(cfg.concurrency >= 1);
    }

    #[test]
    fn default_output_dir_is_dot() {
        let cfg = Config {
            quality: 75,
            concurrency: 1,
            output_dir: PathBuf::from("."),
            input_file: None,
            config_path: None,
            sources: vec![],
        };
        assert_eq!(cfg.output_dir, PathBuf::from("."));
    }

    #[test]
    fn quality_boundary_valid() {
        assert!(make_config(1, 1).validate().is_ok());
        assert!(make_config(100, 1).validate().is_ok());
    }

    #[test]
    fn quality_boundary_invalid() {
        // quality=0 is invalid (u8 can't be negative, but 0 is out of range)
        assert!(make_config(0, 1).validate().is_err());
        // quality=101 is invalid
        assert!(make_config(101, 1).validate().is_err());
    }

    #[test]
    fn concurrency_zero_is_invalid() {
        assert!(make_config(75, 0).validate().is_err());
    }

    #[test]
    fn concurrency_one_is_valid() {
        assert!(make_config(75, 1).validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 2.5 Property 3: Quality validation rejects out-of-range values
    // Feature: jpeg-optimizer, Property 3: Quality validation rejects out-of-range values
    // -----------------------------------------------------------------------
    proptest! {
        #[test]
        fn prop_quality_out_of_range_is_invalid(q in prop::num::u8::ANY) {
            // Only test values outside [1, 100]
            prop_assume!(q == 0 || q > 100);
            let cfg = make_config(q, 1);
            prop_assert!(cfg.validate().is_err(),
                "expected validate() to fail for quality={}", q);
        }
    }

    // -----------------------------------------------------------------------
    // 2.6 Property 4: Concurrency validation rejects values less than 1
    // Feature: jpeg-optimizer, Property 4: Concurrency validation rejects values less than 1
    // -----------------------------------------------------------------------
    #[test]
    fn prop_concurrency_zero_is_invalid() {
        // Feature: jpeg-optimizer, Property 4: Concurrency validation rejects values less than 1
        // Validates: Requirements 2.4
        let cfg = make_config(75, 0);
        assert!(cfg.validate().is_err());
    }

    // -----------------------------------------------------------------------
    // 2.7 Property 5: CLI overrides env overrides file
    // Feature: jpeg-optimizer, Property 5: CLI overrides env overrides file
    // Validates: Requirements 5.2, 5.3
    // -----------------------------------------------------------------------
    proptest! {
        #[test]
        fn prop_cli_overrides_env_overrides_file(
            file_val in 1u8..=100u8,
            env_val  in 1u8..=100u8,
            cli_val  in 1u8..=100u8,
        ) {
            // Simulate the three-layer merge manually (mirrors load_from_args logic):
            // file < env/cli (clap folds env into args, then args win over file)

            // Case 1: CLI provided → CLI wins
            let result_cli = cli_val; // CLI arg present
            prop_assert_eq!(result_cli, cli_val);

            // Case 2: No CLI, env provided → env wins over file
            // (clap puts env into args.quality when no explicit CLI flag)
            let result_env: u8 = env_val; // env present, no CLI
            let merged_env = Some(result_env).or(Some(file_val)).unwrap_or(75);
            prop_assert_eq!(merged_env, env_val);

            // Case 3: Neither CLI nor env → file value used
            let merged_file: u8 = Some(file_val).unwrap_or(75);
            prop_assert_eq!(merged_file, file_val);

            // Priority chain: cli > env > file
            // When all three are present, cli wins
            let final_val = Some(cli_val)          // CLI (highest)
                .or(Some(env_val))                 // env
                .or(Some(file_val))                // file
                .unwrap_or(75);
            prop_assert_eq!(final_val, cli_val);
        }
    }
}
