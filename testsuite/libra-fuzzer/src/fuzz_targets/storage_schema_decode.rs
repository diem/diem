// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use libradb::schema::fuzzing::fuzz_decode;
use proptest::{collection::vec, prelude::*};

#[derive(Clone, Debug, Default)]
pub struct StorageSchemaDecode;

impl FuzzTargetImpl for StorageSchemaDecode {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "Storage schemas do not panic on corrupted bytes."
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen.generate(prop_oneof![
            100 => vec(any::<u8>(), 0..1024),
            1 => vec(any::<u8>(), 1024..1024 * 10),
        ]))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_decode(data)
    }
}
