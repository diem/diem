// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::{Config, TestRunner},
};

/// Context for generating single values out of strategies.
///
/// Proptest is designed to be built around "value trees", which represent a spectrum from complex
/// values to simpler ones. But in some contexts, like benchmarking or generating corpuses, one just
/// wants a single value. This is a convenience struct for that.
pub struct ValueGenerator {
    runner: TestRunner,
}

impl ValueGenerator {
    /// Creates a new value generator with the default RNG.
    pub fn new() -> Self {
        Self {
            runner: TestRunner::new(Config::default()),
        }
    }

    /// Creates a new value generator with a deterministic RNG.
    ///
    /// This generator has a hardcoded seed, so its results are predictable across test runs.
    /// However, a new proptest version may change the seed.
    pub fn deterministic() -> Self {
        Self {
            runner: TestRunner::deterministic(),
        }
    }

    /// Generates a single value for this strategy.
    ///
    /// Panics if generating the new value fails. The only situation in which this can happen is if
    /// generating the value causes too many internal rejects.
    pub fn generate<S: Strategy>(&mut self, strategy: S) -> S::Value {
        strategy
            .new_tree(&mut self.runner)
            .expect("creating a new value should succeed")
            .current()
    }
}
