// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_proptest_helpers::ValueGenerator;
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::{self, RngAlgorithm, TestRunner},
};
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
    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>>;

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

/// Helper to generate random bytes that can be used with proptest
/// to generate a value following the passed strategy.
fn corpus_from_strategy(strategy: impl Strategy) -> Vec<u8> {
    // randomly-seeded recording RNG
    let mut seed = [0u8; 32];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut seed);
    let recorder_rng = test_runner::TestRng::from_seed(RngAlgorithm::Recorder, &seed);
    let mut runner = TestRunner::new_with_rng(test_runner::Config::default(), recorder_rng);

    // generate the value
    strategy
        .new_tree(&mut runner)
        .expect("creating a new value should succeed")
        .current();

    // dump the bytes
    runner.bytes_used()
}

/// Helper to convert a bytearray to a value implementing the Arbitrary trait.
pub fn fuzz_data_to_value<T: std::fmt::Debug>(
    data: &[u8],
    strategy: impl Strategy<Value = T>,
) -> T {
    // setup proptest with passthrough RNG
    let passthrough_rng =
        test_runner::TestRng::from_seed(test_runner::RngAlgorithm::PassThrough, &data);
    let config = test_runner::Config::default();
    let mut runner = TestRunner::new_with_rng(config, passthrough_rng);

    // create a value based on the arbitrary implementation of T
    let strategy_tree = strategy.new_tree(&mut runner).expect("should not happen");
    strategy_tree.current()
}
