// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let protos = ["src/proto/storage.proto"];

    let includes = ["../../types/src/proto", "src/proto"];

    tonic_build::configure()
        .compile(&protos, &includes)
        .unwrap();
}
