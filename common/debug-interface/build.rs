// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    tonic_build::compile_protos("src/proto/node_debug_interface.proto").unwrap();
}
