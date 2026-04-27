use clap::Parser;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

/// Prints its configuration to STDOUT.
#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
struct Cli {
    /// Enables debug mode
    #[arg(short, long)]
    debug: bool,

    /// Path to configuration file
    #[arg(short, long, env = "CONF_FILE", default_value = "config.toml")]
    conf: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct ModeConfig {
    debug: bool,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self { debug: false }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct ServerConfig {
    external_url: String,
    http_port: u16,
    grpc_port: u16,
    healthz_port: u16,
    metrics_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            external_url: "http://127.0.0.1".into(),
            http_port: 8081,
            grpc_port: 8082,
            healthz_port: 10025,
            metrics_port: 9199,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct DbMysqlConnections {
    max_idle: u32,
    max_open: u32,
}

impl Default for DbMysqlConnections {
    fn default() -> Self {
        Self { max_idle: 30, max_open: 30 }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct DbMysqlConfig {
    host: String,
    port: u16,
    dating: String,
    user: String,
    pass: String,
    connections: DbMysqlConnections,
}

impl Default for DbMysqlConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3306,
            dating: "default".into(),
            user: "root".into(),
            pass: String::new(),
            connections: DbMysqlConnections::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct DbConfig {
    mysql: DbMysqlConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct LogAppConfig {
    level: String,
}

impl Default for LogAppConfig {
    fn default() -> Self {
        Self { level: "info".into() }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct LogConfig {
    app: LogAppConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct WatchdogConfig {
    period: String,
    limit: u32,
    lock_timeout: String,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            period: "5s".into(),
            limit: 10,
            lock_timeout: "4s".into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct BackgroundConfig {
    watchdog: WatchdogConfig,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct AppConfig {
    mode: ModeConfig,
    server: ServerConfig,
    db: DbConfig,
    log: LogConfig,
    background: BackgroundConfig,
}

fn main() {
    let cli = Cli::parse();

    let defaults = Config::try_from(&AppConfig::default())
        .expect("Failed to serialize defaults");

    let mut app_config: AppConfig = Config::builder()
        .add_source(defaults)
        .add_source(File::with_name(&cli.conf).required(false))
        .add_source(
            Environment::with_prefix("CONF")
                .separator("__")
                .try_parsing(true),
        )
        .build()
        .expect("Failed to build configuration")
        .try_deserialize()
        .expect("Failed to deserialize configuration");

    if cli.debug {
        app_config.mode.debug = true;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&app_config)
            .unwrap_or_else(|_| format!("{app_config:#?}"))
    );
}
