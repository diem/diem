use std::env;
use std::process::Command;

const RUSTFLAGS: &str = "RUSTFLAGS";
const IGNORED_LINTS: &[&str] = &["dead_code"];

pub fn make_vec() -> Vec<&'static str> {
    let mut rustflags = vec!["--cfg", "trybuild"];

    for &lint in IGNORED_LINTS {
        rustflags.push("-A");
        rustflags.push(lint);
    }

    rustflags
}

pub fn set_env(cmd: &mut Command) {
    let mut rustflags = match env::var_os(RUSTFLAGS) {
        Some(rustflags) => rustflags,
        None => return,
    };

    for flag in make_vec() {
        rustflags.push(" ");
        rustflags.push(flag);
    }

    cmd.env(RUSTFLAGS, rustflags);
}
