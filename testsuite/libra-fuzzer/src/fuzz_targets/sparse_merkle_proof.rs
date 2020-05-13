// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use grpc_types::proto::types::SparseMerkleProof as ProtoSparseMerkleProof;
use libra_types::proof::SparseMerkleProof;

proto_fuzz_target!(SparseMerkleProofTarget => SparseMerkleProof, ProtoSparseMerkleProof);
