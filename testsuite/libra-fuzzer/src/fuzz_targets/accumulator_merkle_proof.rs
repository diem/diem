// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::proof::TransactionAccumulatorProof;
use libra_types::proto::types::AccumulatorProof as ProtoAccumulatorProof;

proto_fuzz_target!(AccumulatorProofTarget => TransactionAccumulatorProof, ProtoAccumulatorProof);
