use std::borrow::Cow;
use std::env;

use clap::Parser;

const DEFAULT_CONF: &str = "/etc/app/app.conf";

#[derive(Parser)]
struct Cli {
    /// Path to configuration file
    #[arg(long)]
    conf: Option<String>,
}

fn get_conf_path(cli_conf: Option<String>) -> Cow<'static, str> {
    // --conf argument has highest priority
    if let Some(val) = cli_conf {
        assert!(!val.is_empty(), "--conf value must not be empty");
        return Cow::Owned(val);
    }

    // APP_CONF env var has second priority
    if let Ok(val) = env::var("APP_CONF") {
        if !val.is_empty() {
            return Cow::Owned(val);
        }
    }

    // Default — no allocation
    Cow::Borrowed(DEFAULT_CONF)
}

fn main() {
    let cli = Cli::parse();
    println!("{}", get_conf_path(cli.conf));
}
