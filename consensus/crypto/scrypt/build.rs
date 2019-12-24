use autotools::Config;
use std::env;

fn main() {
    let openssl = pkg_config::probe_library("openssl")
        .map_err(|_e| panic!("openssl package not found in PKG_CONFIG_PATH environment"))
        .unwrap();
    let mut dst = Config::new("scrypt-sys");
    for p in openssl.link_paths {
        dst.ldflag(format!("-L{}", p.to_str().unwrap()));
    }
    for p in openssl.include_paths {
        dst.cflag(format!("-I{}", p.to_str().unwrap()));
    }
    dst.enable("libscrypt-kdf", None);
    dst.enable_shared();
    dst.build();
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-lib=static=scrypt-kdf");
    println!("cargo:rustc-link-search=native={}/lib", out_dir);
}
