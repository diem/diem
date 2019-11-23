// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{thread::sleep, time::Duration};
use std::iter::Take;

/// Given an operation retries it successfully sleeping everytime it fails
/// If the operation succeeds before the iterator runs out, it returns success
pub fn retry<I, O, T, E>(iterable: I, mut operation: O) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    O: FnMut() -> Result<T, E>,
{
    let mut iterator = iterable.into_iter();
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) => {
                if let Some(delay) = iterator.next() {
                    sleep(delay);
                } else {
                    return Err(err);
                }
            }
        }
    }
}

pub fn fixed_retry_strategy(delay_ms: u64, tries: usize) -> Take<FixedDelay> {
    FixedDelay::new(delay_ms).take(tries)
}

/// An iterator which uses a fixed delay
pub struct FixedDelay {
    duration: Duration,
}

impl FixedDelay {
    /// Create a new `FixedDelay` using the given duration in milliseconds.
    fn new(millis: u64) -> Self {
        FixedDelay {
            duration: Duration::from_millis(millis),
        }
    }
}

impl Iterator for FixedDelay {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        Some(self.duration)
    }
}
