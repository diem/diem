// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_tools::tempdir::TempPath;
use protobuf::{compiler_plugin, descriptor::FileDescriptorSet, error::ProtobufError};
use protoc::{DescriptorSetOutArgs, Protoc};
use protoc_grpcio::CompileResult;
use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// copied from protoc-grpcio
/// it's not public there
pub(crate) fn write_out_generated_files<P>(
    generation_results: Vec<compiler_plugin::GenResult>,
    output_dir: P,
) -> CompileResult<()>
where
    P: AsRef<Path>,
{
    for result in generation_results {
        let file = output_dir.as_ref().join(result.name);
        File::create(&file)
            .expect("failed to create file")
            .write_all(&result.content)
            .expect("failed to write file");
    }

    Ok(())
}

/// Generate snake case names. This is useful
/// helloWorldFoo => hello_world_foo
/// ID => :( not working for this case
///
/// This needs to be the same as in grpcio-compiler, but I
/// didn't copy it.
pub fn to_snake_case(name: &str) -> String {
    let re = Regex::new("((:?^|(:?[A-Z]))[a-z0-9_]+)").unwrap();
    let mut words = vec![];
    for cap in re.captures_iter(name) {
        words.push(cap.get(1).unwrap().as_str().to_lowercase());
    }
    words.join("_") // my best line of code
}

// TODO: frumious: make camel case
pub fn to_camel_case(name: &str) -> String {
    // do nothing for now
    name.to_string()
}

pub fn fq_grpc(item: &str) -> String {
    format!("::grpcio::{}", item)
}

pub enum MethodType {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Duplex,
}

pub fn protoc_descriptor_set(
    from: &[&str],
    includes: &[&str],
) -> Result<FileDescriptorSet, ProtobufError> {
    let protoc = Protoc::from_env_path();
    protoc
        .check()
        .expect("failed to find `protoc`, `protoc` must be available in `PATH`");

    let descriptor_set = TempPath::new();
    descriptor_set.create_as_file()?;

    protoc
        .write_descriptor_set(DescriptorSetOutArgs {
            out: descriptor_set
                .as_ref()
                .to_str()
                .unwrap_or_else(|| unreachable!("failed to convert path to string")),
            input: from,
            includes,
            include_imports: true,
        })
        .expect("failed to write descriptor set");

    let mut serialized_descriptor_set = Vec::new();
    File::open(&descriptor_set)
        .expect("failed to open descriptor set")
        .read_to_end(&mut serialized_descriptor_set)
        .expect("failed to read descriptor set");

    protobuf::parse_from_bytes::<FileDescriptorSet>(&serialized_descriptor_set)
}
