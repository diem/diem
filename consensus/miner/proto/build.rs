fn main() {
    let proto_files = [
        "src/proto/miner.proto",
    ];

    let includes = [
        "src/proto/",
    ];
    let out_dir = &std::env::var("OUT_DIR").unwrap();
    println!("out put dir {:?}", out_dir);
    grpcio_compiler::prost_codegen::compile_protos(
        &proto_files,
        &includes,
        out_dir,
    ).unwrap();
}
