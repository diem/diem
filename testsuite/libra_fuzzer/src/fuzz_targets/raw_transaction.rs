// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use types::transaction::RawTransaction;
proto_fuzz_target!(RawTransactionTarget => RawTransaction);
