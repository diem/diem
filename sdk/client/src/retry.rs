// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Result;
use std::{future::Future, time::Duration};

#[derive(Clone, Debug)]
pub struct Retry {
    max_retries: u32,
    delay: Duration,
}

impl Default for Retry {
    fn default() -> Self {
        Self::new(5, Duration::from_millis(50))
    }
}

impl Retry {
    pub fn new(max_retries: u32, delay: Duration) -> Self {
        Self { max_retries, delay }
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }

    fn next_delay(&self, remaining_attempts: u32) -> Duration {
        self.delay * self.max_retries.saturating_sub(remaining_attempts)
    }

    pub(crate) fn retry<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let mut remaining_attempts = self.max_retries();
        loop {
            match f() {
                Ok(r) => return Ok(r),
                Err(error) if error.is_retriable() && remaining_attempts > 0 => {
                    remaining_attempts = remaining_attempts.saturating_sub(1);
                    std::thread::sleep(self.next_delay(remaining_attempts));
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(crate) async fn retry_async<T, F, O>(&self, f: F) -> Result<T>
    where
        F: Fn() -> O,
        O: Future<Output = Result<T>>,
    {
        let mut remaining_attempts = self.max_retries();
        loop {
            match f().await {
                Ok(r) => return Ok(r),
                Err(error) if error.is_retriable() && remaining_attempts > 0 => {
                    remaining_attempts = remaining_attempts.saturating_sub(1);
                    tokio::time::sleep(self.next_delay(remaining_attempts)).await;
                }
                Err(error) => return Err(error),
            }
        }
    }
}
