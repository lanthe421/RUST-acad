use std::borrow::Cow;
use std::env;

const DEFAULT_CONF: &str = "/etc/app/app.conf";

fn get_conf_path() -> Cow<'static, str> {
    // --conf argument has highest priority
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--conf") {
        let val = args.get(pos + 1).expect("--conf requires a value");
        assert!(!val.is_empty(), "--conf value must not be empty");
        return Cow::Owned(val.clone());
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
    println!("{}", get_conf_path());
}
