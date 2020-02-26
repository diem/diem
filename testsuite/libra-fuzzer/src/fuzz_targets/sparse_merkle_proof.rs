// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use libra_types::{
    proof::SparseMerkleProof, proto::types::SparseMerkleProof as ProtoSparseMerkleProof,
};

proto_fuzz_target!(SparseMerkleProofTarget => SparseMerkleProof, ProtoSparseMerkleProof);
