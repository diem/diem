// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use grpc_types::proto::types::AccumulatorProof as ProtoAccumulatorProof;
use libra_types::proof::TransactionAccumulatorProof;

proto_fuzz_target!(AccumulatorProofTarget => TransactionAccumulatorProof, ProtoAccumulatorProof);
