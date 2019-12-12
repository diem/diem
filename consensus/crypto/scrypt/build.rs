use autotools::Config;

fn main() {
    let openssl = pkg_config::probe_library("openssl")
        .map_err(|_e| panic!("openssl package not found in PKG_CONFIG_PATH environment"))
        .unwrap();
    let mut dst = Config::new("scrypt-sys");
    for p in openssl.link_paths {
        dst.cflag(format!("-L{}", p.to_str().unwrap()));
    }
    for p in openssl.include_paths {
        dst.cflag(format!("-I{}", p.to_str().unwrap()));
    }
    dst.enable("libscrypt-kdf", None).build();
}
