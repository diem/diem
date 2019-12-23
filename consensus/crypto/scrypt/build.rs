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
    let target = "/usr/local/lib/";
    dst.target(target);
    dst.build();
    println!("cargo:rustc-link-search=native={}", target);
}
