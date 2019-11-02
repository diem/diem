// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use libra_types::transaction::SignedTransaction;
use proptest::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SignedTransactionTarget;

impl FuzzTargetImpl for SignedTransactionTarget {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "SignedTransaction (LCS deserializer)"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let value = gen.generate(any_with::<SignedTransaction>(()));
        Some(lcs::to_bytes(&value).expect("serialization should work"))
    }

    fn fuzz(&self, data: &[u8]) {
        let _: Result<SignedTransaction, _> = lcs::from_bytes(&data);
    }
}
