#![no_main]
use libfuzzer_sys::fuzz_target;
use step_3_1::parse_secret;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Must never panic — only return Some or None
        let _ = parse_secret(s);
    }
});