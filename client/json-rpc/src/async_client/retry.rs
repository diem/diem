// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{defaults, Error};
use std::time::Duration;

pub trait RetryStrategy: std::fmt::Debug + Send + Sync {
    fn max_retries(&self, err: &Error) -> u32;
    fn delay(&self, err: &Error, retries: u32) -> Duration;
    fn is_retriable(&self, err: &Error) -> bool;
}

#[derive(Debug)]
pub struct Retry {
    pub max_retries: u32,
    pub delay: Duration,
}

impl Retry {
    pub fn default() -> Self {
        Self {
            max_retries: defaults::MAX_RETRIES,
            delay: defaults::WAIT_DELAY,
        }
    }
}

impl RetryStrategy for Retry {
    fn max_retries(&self, _: &Error) -> u32 {
        self.max_retries
    }

    fn delay(&self, _: &Error, retries: u32) -> Duration {
        self.delay * retries
    }

    fn is_retriable(&self, err: &Error) -> bool {
        match err {
            Error::StaleResponseError(_) => true,
            Error::NetworkError(err) => err.is_timeout() || err.is_request(),
            _ => false,
        }
    }
}
