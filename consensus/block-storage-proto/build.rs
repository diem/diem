fn main() {
    let protos = ["src/proto/block_storage.proto"];

    let includes = [
        "src/proto/",
        "../../network/src/proto",
        "../../types/src/proto",
    ];

    grpcio_compiler::prost_codegen::compile_protos(
        &protos,
        &includes,
        &std::env::var("OUT_DIR").unwrap(),
    )
    .unwrap();
}
