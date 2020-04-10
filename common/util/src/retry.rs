// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{future::Future, pin::Pin, thread, time::Duration};

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
                    thread::sleep(delay);
                } else {
                    return Err(err);
                }
            }
        }
    }
}

pub async fn retry_async<'a, I, O, T, E>(iterable: I, mut operation: O) -> Result<T, E>
where
    I: IntoIterator<Item = Duration>,
    O: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>,
{
    let mut iterator = iterable.into_iter();
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if let Some(delay) = iterator.next() {
                    tokio::time::delay_for(delay).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
}

pub fn fixed_retry_strategy(delay_ms: u64, tries: usize) -> impl Iterator<Item = Duration> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_retry_strategy_success() {
        let mut collection = vec![1, 2, 3, 4, 5].into_iter();
        let result = retry(fixed_retry_strategy(0, 10), || match collection.next() {
            Some(n) if n == 5 => Ok(n),
            Some(_) => Err("not 5"),
            None => Err("not 5"),
        })
        .unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_fixed_retry_strategy_error() {
        let mut collection = vec![1, 2, 3, 4, 5].into_iter();
        let result = retry(fixed_retry_strategy(0, 3), || match collection.next() {
            Some(n) if n == 5 => Ok(n),
            Some(_) => Err("not 5"),
            None => Err("not 5"),
        });
        assert_eq!(result, Err("not 5"));
    }
}
