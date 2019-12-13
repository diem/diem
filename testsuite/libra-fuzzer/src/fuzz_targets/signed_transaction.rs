// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::proto::types::SignedTransaction as ProtoSignedTransaction;
use libra_types::transaction::SignedTransaction;
proto_fuzz_target!(SignedTransactionTarget => SignedTransaction, ProtoSignedTransaction);
