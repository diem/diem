use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let target = env::var("TARGET").ok();
    let path = Path::new(&out_dir).join("target.rs");
    let value = match target {
        Some(target) => format!("Some({:?})", target),
        None => "None".to_owned(),
    };
    let content = format!("const TARGET: Option<&'static str> = {};", value);
    fs::write(path, content)
}
