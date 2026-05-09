use clap::Parser;
use config::{Config, Environment, File};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::net::IpAddr;
use url::Url;

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

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct ModeConfig {
    #[default = false]
    debug: bool,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct ServerConfig {
    #[default(Url::parse("http://127.0.0.1").unwrap())]
    external_url: Url,
    #[default = 8081]
    http_port: u16,
    #[default = 8082]
    grpc_port: u16,
    #[default = 10025]
    healthz_port: u16,
    #[default = 9199]
    metrics_port: u16,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct DbMysqlConnections {
    #[default = 30]
    max_idle: u32,
    #[default = 30]
    max_open: u32,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct DbMysqlConfig {
    #[default("127.0.0.1".parse().unwrap())]
    host: IpAddr,
    #[default = 3306]
    port: u16,
    #[default = "default"]
    dating: String,
    #[default = "root"]
    user: String,
    pass: String,
    connections: DbMysqlConnections,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct DbConfig {
    mysql: DbMysqlConfig,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct LogAppConfig {
    #[default(LevelFilter::Info)]
    level: LevelFilter,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct LogConfig {
    app: LogAppConfig,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
struct WatchdogConfig {
    #[serde(with = "humantime_serde")]
    #[default(std::time::Duration::from_secs(5))]
    period: std::time::Duration,

    #[default = 10]
    limit: u32,

    #[serde(with = "humantime_serde")]
    #[default(std::time::Duration::from_secs(4))]
    lock_timeout: std::time::Duration,
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

    let defaults = Config::try_from(&AppConfig::default()).expect("Failed to serialize defaults");

    let mut builder = Config::builder()
        .add_source(defaults)
        .add_source(File::with_name(&cli.conf).required(false))
        .add_source(
            Environment::with_prefix("CONF")
                .separator("__")
                .try_parsing(true),
        );

    builder = builder
        .set_override("mode.debug", cli.debug)
        .expect("Failed to set debug override");

    let app_config: AppConfig = builder
        .build()
        .expect("Failed to build configuration")
        .try_deserialize()
        .expect("Failed to deserialize configuration");

    println!(
        "{}",
        serde_json::to_string_pretty(&app_config).unwrap_or_else(|_| format!("{app_config:#?}"))
    );
}
