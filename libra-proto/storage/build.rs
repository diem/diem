// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let protos = ["src/storage.proto"];

    let includes = ["../types/src", "src"];

    tonic_build::configure()
        .compile(&protos, &includes)
        .unwrap();
}
