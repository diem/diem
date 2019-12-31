// Copyright 2017 The Grin Developers

extern crate cmake;
extern crate fs_extra;

mod sanity;
use cmake::Config;
use fs_extra::dir::*;
use std::path::PathBuf;
use std::{env, fs};

#[cfg(feature = "build-cuda-plugins")]
const BUILD_CUDA_PLUGINS: &str = "TRUE";
#[cfg(not(feature = "build-cuda-plugins"))]
const BUILD_CUDA_PLUGINS: &str = "FALSE";

/// Tests whether source cuckoo directory exists

pub fn fail_on_empty_directory(name: &str) {
    if fs::read_dir(name).unwrap().count() == 0 {
        println!(
            "The `{}` directory is empty. Did you forget to pull the submodules?",
            name
        );
        println!("Try `git submodule update --init --recursive`");
        panic!();
    }
}

fn main() {
    #[cfg(feature = "no-plugin-build")]
    return;
    fail_on_empty_directory("src/cuckoo_sys/plugins/cuckoo");
    let path_str = env::var("OUT_DIR").unwrap();
    let mut out_path = PathBuf::from(&path_str);
    out_path.pop();
    out_path.pop();
    out_path.pop();
    let mut plugin_path = PathBuf::from(&path_str);
    plugin_path.push("build");
    plugin_path.push("plugins");
    // Collect the files and directories we care about
    let p = PathBuf::from("src/cuckoo_sys/plugins");
    let dir_content = match get_dir_content(p) {
        Ok(c) => c,
        Err(e) => panic!("Error getting directory content: {}", e),
    };
    for d in dir_content.directories {
        let file_content = get_dir_content(d).unwrap();
        for f in file_content.files {
            println!("cargo:rerun-if-changed={}", f);
        }
    }
    for f in dir_content.files {
        println!("cargo:rerun-if-changed={}", f);
    }

    let dst = Config::new("src/cuckoo_sys/plugins")
        .define("BUILD_CUDA_PLUGINS", BUILD_CUDA_PLUGINS) //whatever flags go here
        //.cflag("-foo") //and here
        .build_target("")
        .build();

    println!("Plugin path: {:?}", plugin_path);
    println!("OUT PATH: {:?}", out_path);
    let mut options = CopyOptions::new();
    options.overwrite = true;
    if let Err(e) = copy(plugin_path, out_path, &options) {
        println!("{:?}", e);
    }

    println!("cargo:rustc-link-search=native={}", dst.display());
}
