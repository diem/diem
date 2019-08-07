// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    build_helpers::build_helpers::compile_proto("src/proto", vec!["../types/src/proto"], true);
}
