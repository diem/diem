// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::prelude::*;
use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, NegativeMultiDecision, Quota,
};
use std::{hash::Hash, num::NonZeroU32, sync::Arc};

/// Rate Limiter type to get rid of underlying store and clock impl
pub type RateLimiter<Key> = governor::RateLimiter<Key, DefaultKeyedStateStore<Key>, DefaultClock>;

/// Directives for the synchronous usage of Rate limiting, the service using it could choose to
/// drop the messages anyways in all situations.
/// TODO: Add a wait directive for backpressure
pub enum RateLimitError {
    DropMessage,
    MessageTooLarge,
}

/// Provides a per second throttle with arbitrary keys
pub fn new_per_second_keyed<Key: Hash + Clone + Eq>(
    throttle_rate: NonZeroU32,
    throttle_burst: NonZeroU32,
) -> Arc<RateLimiter<Key>> {
    let quota = Quota::per_second(throttle_rate).allow_burst(throttle_burst);
    Arc::new(governor::RateLimiter::keyed(quota))
}

/// Rate limits a message based on it's length
pub fn rate_limit_msg<Key: Hash + Clone + Eq>(
    rate_limiter: &RateLimiter<Key>,
    key: &Key,
    msg_length: usize,
) -> Result<(), RateLimitError> {
    // Governor doesn't support larger than u32
    if msg_length > u32::MAX as usize {
        error!(
            "Cannot process message, message size ({}) is greater than max possible u32",
            msg_length
        );
        return Err(RateLimitError::MessageTooLarge);
    } else if msg_length < 1 {
        // We can't rate limit something of 0 size, allow it through
        return Ok(());
    }

    let length = NonZeroU32::new(msg_length as u32).unwrap();

    rate_limiter.check_key_n(key, length).map_err(|error| {
        match error {
            NegativeMultiDecision::BatchNonConforming(_max_allowed, _not_until) => {
                // Here we can wait based on the `not_until` field for an expected time
                // For now, drop the message
                RateLimitError::DropMessage
            }
            NegativeMultiDecision::InsufficientCapacity(max_size) => {
                error!("Cannot process message, message size ({}) is greater than rate limiter allows ({})", msg_length, max_size);
                RateLimitError::MessageTooLarge
            }
        }
    })
}
