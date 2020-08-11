// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_proptest_helpers::ValueGenerator;
use rand::RngCore;
use std::{fmt, ops::Deref, str::FromStr};

pub mod commands;
#[cfg(test)]
mod coverage;
pub mod fuzz_targets;

/// Implementation for a particular target of a fuzz operation.
pub trait FuzzTargetImpl: Sync + Send + fmt::Debug {
    /// The name of the fuzz target.
    /// By default, we use the struct name, however, implementations may prefer to override this.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .rsplit("::")
            .next()
            .expect("Implementation struct name must have at least one component")
    }

    /// A description for this target.
    fn description(&self) -> &'static str;

    /// Generates a new example for this target to store in the corpus. `idx` is the current index
    /// of the item being generated, starting from 0.
    ///
    /// Returns `Some(bytes)` if a value was generated, or `None` if no value can be generated.
    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        let mut output = vec![0u8; 4096];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut output);
        Some(output)
    }

    /// Fuzz the target with this data. The fuzzer tests for panics or OOMs with this method.
    fn fuzz(&self, data: &[u8]);
}

/// A fuzz target.
#[derive(Copy, Clone, Debug)]
pub struct FuzzTarget(&'static (dyn FuzzTargetImpl + 'static));

impl Deref for FuzzTarget {
    type Target = dyn FuzzTargetImpl + 'static;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl FromStr for FuzzTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FuzzTarget::by_name(s).ok_or_else(|| format!("Fuzz target '{}' not found (run `list`)", s))
    }
}
