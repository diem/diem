// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FuzzTarget, FuzzTargetImpl};
use failure::prelude::*;
use lazy_static::lazy_static;
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};
use std::{collections::BTreeMap, env, fmt};

/// Convenience macro to return the module name.
macro_rules! module_name {
    () => {
        module_path!()
            .rsplit("::")
            .next()
            .expect("module path must have at least one component")
    };
}

/// A fuzz target implementation for protobuf-compiled targets.
macro_rules! proto_fuzz_target {
    ($target:ident => $ty:ty) => {
        #[derive(Clone, Debug, Default)]
        pub struct $target;

        impl $crate::FuzzTargetImpl for $target {
            fn name(&self) -> &'static str {
                module_name!()
            }

            fn description(&self) -> &'static str {
                concat!(stringify!($ty), " (protobuf)")
            }

            fn generate(&self, runner: &mut ::proptest::test_runner::TestRunner) -> Vec<u8> {
                use proto_conv::IntoProtoBytes;

                let value =
                    $crate::fuzz_targets::new_value(runner, ::proptest::arbitrary::any::<$ty>());
                value
                    .into_proto_bytes()
                    .expect("failed to convert to bytes")
            }

            fn fuzz(&self, data: &[u8]) {
                use proto_conv::FromProtoBytes;

                // Errors are OK -- the fuzzer cares about panics and OOMs.
                let _ = <$ty>::from_proto_bytes(data);
            }
        }
    };
}

// List fuzz target modules here.
mod compiled_module;
mod raw_transaction;
mod signed_transaction;
mod vm_value;

lazy_static! {
    static ref ALL_TARGETS: BTreeMap<&'static str, Box<dyn FuzzTargetImpl>> = {
        let targets: Vec<Box<dyn FuzzTargetImpl>> = vec![
            // List fuzz targets here in this format.
            Box::new(compiled_module::CompiledModuleTarget::default()),
            Box::new(raw_transaction::RawTransactionTarget::default()),
            Box::new(signed_transaction::SignedTransactionTarget::default()),
            Box::new(vm_value::ValueTarget::default()),
        ];
        targets.into_iter().map(|target| (target.name(), target)).collect()
    };
}

impl FuzzTarget {
    /// The environment variable used for passing fuzz targets to child processes.
    pub(crate) const ENV_VAR: &'static str = "FUZZ_TARGET";

    /// Get the current fuzz target from the environment.
    pub fn from_env() -> Result<Self> {
        let name = env::var(Self::ENV_VAR)?;
        match Self::by_name(&name) {
            Some(target) => Ok(target),
            None => bail!("Unknown fuzz target '{}'", name),
        }
    }

    /// Get a fuzz target by name.
    pub fn by_name(name: &str) -> Option<Self> {
        ALL_TARGETS.get(name).map(|target| FuzzTarget(&**target))
    }

    /// A list of all fuzz targets.
    pub fn all_targets() -> impl Iterator<Item = Self> {
        ALL_TARGETS.values().map(|target| FuzzTarget(&**target))
    }
}

/// Produce a value from this strategy.
fn new_value<T: fmt::Debug>(runner: &mut TestRunner, strategy: impl Strategy<Value = T>) -> T {
    let value_tree = strategy
        .new_tree(runner)
        .expect("failed to create value tree");
    value_tree.current()
}
