fn main() {
    cc::Build::new()
        .file("src/cuckoo.c")
        .include("src/")
        .flag("-O3")
        // .flag("-march=native")
        .static_flag(true)
        .compile("libmean.a");
}
