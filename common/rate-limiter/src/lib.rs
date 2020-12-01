// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_logger::prelude::*;
use std::{cmp::min, collections::HashMap, fmt::Debug, hash::Hash, time::Instant};
use tokio::time::Duration;

/// Interface for a keyed rate limiter.  This should allow us to
/// drop in a different keyed rate limiter if required.
pub trait KeyedRateLimiter<Key: Eq + Hash + Clone + Debug> {
    /// Determine if a throttle is needed on the key for the given count
    /// This does not block, or wait and returns `true` if throttled
    fn throttle(&mut self, key: Key, count: u64) -> bool;

    /// Fills the throttling mechanism for a number of ticks.  Can be manually called
    fn fill_throttles(&mut self, intervals: u64);
}

impl<Key: Eq + Hash + Clone + Debug> KeyedRateLimiter<Key> for TokenBucketRateLimiter<Key> {
    /// Processes a throttle check, for synchronous bookkeeping it also ensures
    /// all buckets are filled in a timely manner, and buckets exist.
    /// TODO: Do we want to add a waiting mechanism, this only returns immediately
    fn throttle(&mut self, key: Key, count: u64) -> bool {
        // We could be more strict on 0, but there's nothing to throttle then, exit quickly
        if count == 0 {
            return false;
        }

        // Refill buckets if we need to
        // TODO: Should we do this separately, or continue doing synchronously here
        self.check_refill(self.last_refresh_time.elapsed());

        // A request larger than the current number of tokens will be throttled
        // Note: a request that is larger than the bucket size will ALWAYS be throttled
        let bucket = self.bucket(
            key.clone(),
            self.default_bucket_size,
            self.default_fill_rate,
        );

        // Only count against limit if we're able to actually process the request
        if count <= bucket.tokens {
            bucket.tokens = bucket.tokens.saturating_sub(count);
            false
        } else {
            trace!("Throttling key: {:?}", key);
            true
        }
    }

    /// Fill the buckets with the given rate, saturating at the max size
    /// This should occur on an interval basis
    fn fill_throttles(&mut self, intervals: u64) {
        self.buckets.values_mut().for_each(|bucket| {
            // Skip if bucket is already filled
            if bucket.size != bucket.tokens {
                let new_tokens = intervals.saturating_mul(bucket.rate);

                // Saturate at bucket size
                bucket.tokens = min(bucket.size, bucket.tokens.saturating_add(new_tokens));
            }
        });
    }
}

/// A generic token bucket filter
///
/// # Terms
/// ## Key
/// A `key` is an identifier of the item being rate limited
///
/// ## Token
/// A `token` is the smallest discrete value that we want to rate limit by.  In a situation involving
/// network requests, this may represent a request or a byte.  `Tokens` are the counters for the
/// rate limiting, and when there are no `tokens` left in a `bucket`, the `key` is throttled.
///
/// ## Bucket
/// A `bucket` is the tracker of the number of `tokens`.  It has a `bucket size`, and any additional
/// tokens added to it will "spill" out of the `bucket`.  The `buckets` are filled at an `interval`
/// with a given `fill rate`.
///
/// ## Interval
/// The `interval` at which we refill *all* of the `buckets` in the token bucket filter. Configured
/// across the whole token bucket filter.
///
/// ## Fill Rate
/// The rate at which we fill a `bucket` with tokens. Configured per bucket.
///
/// ## Bucket Size
/// Maximum size of a bucket.  A bucket saturates at this size.  Configured per bucket.
///
/// # Features
/// ## Keys
/// The token bucket takes any key as long as it's hashable.  This should allow it to apply to
/// many applications that need rate limiters.
///
/// ## Bucket sizes and Rates
/// ### Defaults
/// There are defaults for bucket size and fill rate, which will apply to unknown keys.
///
/// ### Per key overrides
/// A framework was put in for allowing to make custom bucket sizes and custom fill rates
/// There is a default bucket size, and a default fill rate, but has the framework to have custom
/// made buckets and fill rates for specific keys that we want to be different.
///
/// ### Refill Interval
/// Buckets are refilled automatically at an interval.  To do this synchronously, it calculates the
/// number of intervals that have passed.  This is done synchronously and in the future may be done
/// asynchronously.
///
/// TODO: Make thread safe interface
/// TODO: Implement an eviction policy based on inactivity (LRU)?
pub struct TokenBucketRateLimiter<Key: Eq + Hash + Clone + Debug> {
    buckets: HashMap<Key, Bucket>,
    last_refresh_time: Instant,
    default_bucket_size: u64,
    default_fill_rate: u64,
    interval: Duration,
}

const ONE_SECOND: Duration = Duration::from_secs(1);

#[allow(dead_code)]
impl<Key: Eq + Hash + Clone + Debug> TokenBucketRateLimiter<Key> {
    pub(crate) fn new(
        default_bucket_size: u64,
        default_fill_rate: u64,
        interval: Duration,
    ) -> Self {
        // A safeguard to ensure we don't spend all of our time refilling throttle buckets
        assert!(
            interval >= ONE_SECOND,
            "Rate limiter cannot be used at less than 1 second granularity"
        );

        Self {
            buckets: HashMap::new(),
            last_refresh_time: Instant::now(),
            default_bucket_size,
            default_fill_rate,
            interval,
        }
    }

    /// Gets the bucket, and if it doesn't exist creates one with the values given
    fn bucket(&mut self, key: Key, bucket_size: u64, fill_rate: u64) -> &mut Bucket {
        self.buckets
            .entry(key)
            .or_insert_with(|| Bucket::new(bucket_size, fill_rate))
    }

    /// Checks if we need to refill the buckets, based on time, and will fill
    /// appropriately if we missed some fill intervals
    fn check_refill(&mut self, elapsed: Duration) {
        if elapsed >= self.interval {
            let num_intervals = elapsed.as_secs() / self.interval.as_secs();
            self.last_refresh_time = Instant::now();
            self.fill_throttles(num_intervals);
        }
    }
}

/// A builder for RateLimiter
/// In the future, this will allow us to predetermine keys, and change rates and sizes for them
pub struct RateLimiterBuilder {
    default_bucket_size: u64,
    default_fill_rate: u64,
    interval: Duration,
}

impl RateLimiterBuilder {
    pub fn new(default_bucket_size: u64, default_fill_rate: u64) -> Self {
        RateLimiterBuilder {
            default_bucket_size,
            default_fill_rate,
            interval: ONE_SECOND,
        }
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn build<Key: Eq + Hash + Clone + Debug>(self) -> TokenBucketRateLimiter<Key> {
        TokenBucketRateLimiter::new(
            self.default_bucket_size,
            self.default_fill_rate,
            self.interval,
        )
    }
}

/// A token bucket object that keeps track of everything related to a key
#[derive(Debug)]
struct Bucket {
    tokens: u64,
    size: u64,
    rate: u64,
}

impl Bucket {
    pub(crate) fn new(size: u64, rate: u64) -> Self {
        assert!(rate > 0, "Fill rate must be greater than 0");
        assert!(size > 0, "Bucket size must be greater than 0");
        assert!(
            size >= rate,
            "Bucket size must be greater than or equal to fill rate"
        );
        Self {
            tokens: size,
            size,
            rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    // Helper function, checks the throttle limit
    fn assert_num_allowed<Key: Eq + Hash + Clone + Debug>(
        rate_limiter: &mut TokenBucketRateLimiter<Key>,
        key: Key,
        num_allowed: u64,
    ) {
        assert!(!rate_limiter.throttle(key.clone(), num_allowed));
        assert!(rate_limiter.throttle(key, 1));
    }

    #[test]
    fn test_rate_limiting() {
        let key_a = "Hello";
        let key_b = "World";
        let default_bucket_size = 2;
        let default_fill_rate = 1;
        // Make interval long enough that it won't refill manually in this test
        let builder = RateLimiterBuilder::new(default_bucket_size, default_fill_rate)
            .interval(Duration::from_secs(3600));

        let mut rate_limiter = builder.build();
        assert_num_allowed(&mut rate_limiter, key_a, 2);

        // Buckets can be refilled
        rate_limiter.fill_throttles(1);
        assert_num_allowed(&mut rate_limiter, key_a, 1);

        // Buckets can be refilled more than one interval
        rate_limiter.fill_throttles(2);
        assert_num_allowed(&mut rate_limiter, key_a, 2);

        // Requests larger than the number of tokens don't count, but are throttled
        rate_limiter.fill_throttles(1);
        assert!(rate_limiter.throttle(key_a, 2));
        assert_num_allowed(&mut rate_limiter, key_a, 1);

        // Buckets that can't be larger than the bucket size
        rate_limiter.fill_throttles(5);
        assert_num_allowed(&mut rate_limiter, key_a, 2);

        // Other keys aren't affected by empty buckets
        assert_num_allowed(&mut rate_limiter, key_b, 2);

        // All buckets are filled at once
        rate_limiter.fill_throttles(2);
        assert_num_allowed(&mut rate_limiter, key_a, 2);
        assert_num_allowed(&mut rate_limiter, key_b, 2);
    }

    #[test]
    fn test_refilling_checks() {
        let key_a = "Hello";
        let default_bucket_size = 2;
        let default_fill_rate = 1;
        // Make interval long enough that it won't refill manually in this test
        let builder = RateLimiterBuilder::new(default_bucket_size, default_fill_rate)
            .interval(Duration::from_secs(3600));

        let mut rate_limiter = builder.build();
        assert_num_allowed(&mut rate_limiter, key_a, 2);

        // We should not fill the bucket if the time hasn't passed
        rate_limiter.check_refill(Duration::from_secs(0));
        assert_num_allowed(&mut rate_limiter, key_a, 0);

        // Filling the bucket should occur once if it's been the interval
        rate_limiter.check_refill(Duration::from_secs(3600));
        assert_num_allowed(&mut rate_limiter, key_a, 1);

        // Filling the bucket should occur once if it's between 1-2 intervals
        rate_limiter.check_refill(Duration::from_secs(3800));
        assert_num_allowed(&mut rate_limiter, key_a, 1);

        // Filling the bucket should occur once if it's been 2 intervals
        rate_limiter.check_refill(Duration::from_secs(7200));
        assert_num_allowed(&mut rate_limiter, key_a, 2);
    }

    #[test]
    fn test_default_fill() {
        let key_a = "Hello";
        let default_bucket_size = 1;
        let default_fill_rate = 1;
        // Make interval long enough that it won't refill manually in this test
        let builder = RateLimiterBuilder::new(default_bucket_size, default_fill_rate);

        let mut rate_limiter = builder.build();
        assert_num_allowed(&mut rate_limiter, key_a, 1);
        sleep(Duration::from_secs(1));

        // After the default duration, the bucket should be filled again!
        assert_num_allowed(&mut rate_limiter, key_a, 1);
    }
}
