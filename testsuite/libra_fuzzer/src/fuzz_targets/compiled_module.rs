// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{fuzz_targets::new_value, FuzzTargetImpl};
use proptest::{prelude::*, test_runner::TestRunner};
use vm::file_format::{CompiledModule, CompiledModuleMut};

#[derive(Clone, Debug, Default)]
pub struct CompiledModuleTarget;

impl FuzzTargetImpl for CompiledModuleTarget {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "VM CompiledModule (custom deserializer)"
    }

    fn generate(&self, runner: &mut TestRunner) -> Vec<u8> {
        let value = new_value(runner, any_with::<CompiledModuleMut>(16));
        let mut out = vec![];
        value
            .serialize(&mut out)
            .expect("serialization should work");
        out
    }

    fn fuzz(&self, data: &[u8]) {
        // Errors are OK -- the fuzzer cares about panics and OOMs. Note that
        // `CompiledModule::deserialize` also runs the bounds checker, which is desirable here.
        let _ = CompiledModule::deserialize(data);
    }
}
