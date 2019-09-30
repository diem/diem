// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Contains helpers for build.rs files.  Includes helpers for proto compilation
use std::path::{Path, PathBuf};

use std::env;
use walkdir::WalkDir;

// Compiles all proto files under proto root and dependent roots.
// For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
// `src/a/b/c_grpc.rs`.
pub fn compile_proto(proto_root: &str, dependent_roots: Vec<&str>, generate_client_code: bool) {
    let mut additional_includes = vec![];
    env::remove_var("GO111MODULE");
    for dependent_root in dependent_roots {
        // First compile dependent directories
        compile_dir(
            &dependent_root,
            vec![], /* additional_includes */
            false,  /* generate_client_code */
        );
        additional_includes.push(Path::new(dependent_root).to_path_buf());
    }
    // Now compile this directory
    compile_dir(&proto_root, additional_includes, generate_client_code);
}

// Compile all of the proto files in proto_root directory and use the additional
// includes when compiling.
pub fn compile_dir(
    proto_root: &str,
    additional_includes: Vec<PathBuf>,
    generate_client_code: bool,
) {
    for entry in WalkDir::new(proto_root) {
        let p = entry.unwrap();
        if p.file_type().is_dir() {
            continue;
        }

        let path = p.path();
        if let Some(ext) = path.extension() {
            if ext != "proto" {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            compile(&path, &additional_includes, generate_client_code);
        }
    }
}

fn compile(path: &Path, additional_includes: &[PathBuf], generate_client_code: bool) {
    let parent = path.parent().unwrap();
    let mut src_path = parent.to_owned().to_path_buf();
    src_path.push("src");

    let mut includes = additional_includes.to_owned();
    includes.push(parent.to_path_buf());

    ::protoc_grpcio::compile_grpc_protos(&[path], includes.as_slice(), parent, None)
        .unwrap_or_else(|_| panic!("Failed to compile protobuf input: {:?}", path));

    if generate_client_code {
        let file_string = path
            .file_name()
            .expect("unable to get filename")
            .to_str()
            .unwrap();
        let includes_strings = includes
            .iter()
            .map(|x| x.to_str().unwrap())
            .collect::<Vec<&str>>();

        // generate client code
        libra_grpcio_client::client_stub_gen(
            &[file_string],
            includes_strings.as_slice(),
            &parent.to_str().unwrap(),
        )
        .expect("Unable to generate client stub");
    }
}
