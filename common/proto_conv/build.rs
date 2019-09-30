// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let proto_root = "tests/proto";

    libra_build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![],
        false, /* generate_client_stub */
    );
}
