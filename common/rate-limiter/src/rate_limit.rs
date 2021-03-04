// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_infallible::{Mutex, RwLock};
use diem_logger::debug;
use diem_metrics::HistogramVec;
use std::{cmp::min, collections::HashMap, fmt::Debug, hash::Hash, sync::Arc, time::Instant};
use tokio::time::Duration;

pub type SharedBucket = Arc<Mutex<Bucket>>;

const ONE_SEC: Duration = Duration::from_secs(1);

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
/// ### Refill Interval
/// Buckets are refilled automatically at an interval.  To do this synchronously, it calculates the
/// number of intervals that have passed.  This is done synchronously and in the future may be done
/// asynchronously.
///
pub struct TokenBucketRateLimiter<Key: Eq + Hash + Clone + Debug> {
    label: &'static str,
    log_info: String,
    buckets: RwLock<HashMap<Key, SharedBucket>>,
    new_bucket_start_percentage: u8,
    default_bucket_size: usize,
    default_fill_rate: usize,
    enabled: bool,
    metrics: Option<HistogramVec>,
}

impl<Key: Eq + Hash + Clone + Debug> TokenBucketRateLimiter<Key> {
    pub fn new(
        label: &'static str,
        log_info: String,
        new_bucket_start_percentage: u8,
        default_bucket_size: usize,
        default_fill_rate: usize,
        metrics: Option<HistogramVec>,
    ) -> Self {
        // Ensure that we can actually use the rate limiter
        assert!(new_bucket_start_percentage <= 100);
        assert!(default_bucket_size > 0);
        assert!(default_fill_rate > 0);

        Self {
            label,
            log_info,
            buckets: RwLock::new(HashMap::new()),
            new_bucket_start_percentage,
            default_bucket_size,
            default_fill_rate,
            enabled: true,
            metrics,
        }
    }

    pub fn test(default_bucket_size: usize, default_fill_rate: usize) -> Self {
        Self::new(
            "test",
            "test".to_string(),
            100,
            default_bucket_size,
            default_fill_rate,
            None,
        )
    }

    /// Used for testing and to not have a rate limiter
    pub fn open(label: &'static str) -> Self {
        Self {
            label,
            log_info: String::new(),
            buckets: RwLock::new(HashMap::new()),
            new_bucket_start_percentage: 100,
            default_bucket_size: std::usize::MAX,
            default_fill_rate: std::usize::MAX,
            enabled: false,
            metrics: None,
        }
    }

    /// Retrieve bucket, or create a new one
    pub fn bucket(&self, key: Key) -> SharedBucket {
        self.bucket_inner(key, |label, log_info, key, initial, size, rate, metrics| {
            Arc::new(Mutex::new(if self.enabled {
                Bucket::new(label, log_info, key, initial, size, rate, metrics)
            } else {
                Bucket::open(label)
            }))
        })
    }

    fn bucket_inner<
        F: FnOnce(String, String, String, usize, usize, usize, Option<HistogramVec>) -> SharedBucket,
    >(
        &self,
        key: Key,
        bucket_create: F,
    ) -> SharedBucket {
        // Attempt to do a weaker read lock first, followed by a write lock if it's missing
        // For the common (read) case, there should be higher throughput
        // Note: This read must happen in a separate block, to ensure the read unlock for the write
        let maybe_bucket = { self.buckets.read().get(&key).cloned() };
        if let Some(bucket) = maybe_bucket {
            bucket
        } else {
            let size = self.default_bucket_size;
            let rate = self.default_fill_rate;

            // Write in a bucket, but make sure again that it isn't there first
            self.buckets
                .write()
                .entry(key.clone())
                .or_insert_with(|| {
                    bucket_create(
                        self.label.to_string(),
                        self.log_info.clone(),
                        format!("{:?}", key),
                        size.saturating_mul(self.new_bucket_start_percentage as usize) / 100,
                        size,
                        rate,
                        self.metrics.clone(),
                    )
                })
                .clone()
        }
    }

    /// Garbage collects a single key, if we know what it is
    pub fn try_garbage_collect_key(&self, key: &Key) -> bool {
        let mut buckets = self.buckets.write();
        let remove = buckets
            .get(key)
            .map_or(false, |bucket| Arc::strong_count(bucket) <= 1);
        if remove {
            buckets.remove(key);
        }
        remove
    }
}

/// A token bucket object that keeps track of everything related to a key
/// This can be used as a standalone rate limiter; however, to make it more useful
/// it should be wrapped in an `Arc` and a `Mutex` to be shared across threads.
#[derive(Debug)]
pub struct Bucket {
    /// Label for what rate limiter it's attached to for logging & metrics purposes
    label: String,
    /// Information to be logged, but can't be put in the metrics for performance reasons
    log_info: String,
    /// The key for metrics purposes
    key: String,
    /// The current number of available tokens to be used
    tokens: usize,
    /// Maximum number of `tokens` in the bucket
    size: usize,
    /// The fill rate of the bucket (`tokens/s`).  Amount added to `tokens` on a `refill`
    rate: usize,
    /// The last time buckets were refilled, to keep track of for amount to refill
    last_refresh_time: Instant,
    /// Determines whether the rate limiting should be ignored, useful for testing
    enabled: bool,
    /// Number of requests allowed through prior to next fill
    allowed_in_period: usize,
    /// Number of requests throttled prior to next fill
    throttled_in_period: usize,
    metrics: Option<HistogramVec>,
}

impl Bucket {
    pub fn new(
        label: String,
        log_info: String,
        key: String,
        initial: usize,
        size: usize,
        rate: usize,
        metrics: Option<HistogramVec>,
    ) -> Self {
        assert!(
            size >= rate,
            "Bucket size must be greater than or equal to fill rate"
        );
        // Store the stringified version of the key for logging
        Self {
            label,
            log_info,
            key,
            tokens: initial,
            size,
            rate,
            last_refresh_time: Instant::now(),
            enabled: true,
            allowed_in_period: 0,
            throttled_in_period: 0,
            metrics,
        }
    }

    /// A fully open rate limiter, to allow for ignoring rate limiting for tests
    pub fn open(label: String) -> Self {
        Self {
            label,
            log_info: String::new(),
            key: String::new(),
            tokens: std::usize::MAX,
            size: std::usize::MAX,
            rate: std::usize::MAX,
            last_refresh_time: Instant::now(),
            enabled: false,
            allowed_in_period: 0,
            throttled_in_period: 0,
            metrics: None,
        }
    }

    /// Refill tokens based on how many seconds have passed since last refresh
    pub(crate) fn refill(&mut self) {
        let num_intervals = self.last_refresh_time.elapsed().as_secs();
        if num_intervals > 0 {
            // Log how many were throttled in the period before refill
            if self.allowed_in_period > 0 || self.throttled_in_period > 0 {
                debug!(
                    throttle_label = self.label,
                    throttle_log_info = self.log_info,
                    throttle_key = self.key,
                    num_allowed_in_period = self.allowed_in_period,
                    num_throttled_in_period = self.throttled_in_period,
                    "{}-{}-{}={}/{}",
                    self.label,
                    self.log_info,
                    self.key,
                    self.allowed_in_period,
                    self.allowed_in_period
                        .saturating_add(self.throttled_in_period),
                );
            }

            // Optional metrics
            if let Some(metrics) = self.metrics.as_ref() {
                metrics
                    .with_label_values(&[self.label.as_str(), "allowed"])
                    .observe(self.allowed_in_period as f64);
                metrics
                    .with_label_values(&[self.label.as_str(), "throttled"])
                    .observe(self.throttled_in_period as f64);
            }
            self.allowed_in_period = 0;
            self.throttled_in_period = 0;
            self.add_tokens((num_intervals as usize).saturating_mul(self.rate));

            // We have to base everything off the original time, or we'll have drift where we slowly slow the bucket refill rate
            self.last_refresh_time += Duration::from_secs(num_intervals);
        }
    }

    /// Determine if an entire batch can be passed through
    /// This is important for message based rate limiting, where the whole message has
    /// to make it through, or else it must be rejected.  A result of `None` means it cannot
    /// ever be allowed through, as it's bigger than the size of the bucket.
    pub fn acquire_all_tokens(&mut self, requested: usize) -> Result<(), Option<Instant>> {
        // Skip over if we purposely have an open throttle
        if !self.enabled || requested == 0 {
            return Ok(());
        }

        // Refill if needed
        self.refill();

        if self.tokens >= requested {
            self.deduct_tokens(requested);
            self.allowed_in_period = self.allowed_in_period.saturating_add(requested);
            Ok(())
        } else {
            // Keep track of the requests we've throttled
            self.throttled_in_period = self.throttled_in_period.saturating_add(requested);
            Err(self.time_of_tokens_needed(requested))
        }
    }

    /// Returns `usize` of tokens allowed.  May be less than requested.
    /// For best effort, caller should return unused tokens with `add_tokens`
    pub fn acquire_tokens(&mut self, requested: usize) -> Result<usize, Instant> {
        // Skip over if we purposely have an open throttle
        if !self.enabled || requested == 0 {
            return Ok(requested);
        }

        // Refill if needed
        self.refill();

        let allowed = self.deduct_tokens(requested);
        if allowed > 0 {
            self.allowed_in_period = self.allowed_in_period.saturating_add(allowed);
            Ok(allowed)
        } else {
            // Keep track of the requests we've throttled
            self.throttled_in_period = self.throttled_in_period.saturating_add(requested);
            Err(self.time_of_next_refill())
        }
    }

    /// Retrieve the maximum amount of tokens up to `count`
    /// Tells us how much of the requested size we can send
    fn deduct_tokens(&mut self, requested: usize) -> usize {
        let tokens_allowed = min(self.tokens, requested);
        self.tokens = self.tokens.saturating_sub(requested);

        tokens_allowed
    }

    /// Tells us when the next refill is
    pub fn time_of_next_refill(&self) -> Instant {
        self.last_refresh_time + ONE_SEC
    }

    /// Tells us when an entire batch will make it through.  Useful for Async work to wait until
    /// all tokens are ready.  Returns `None` if it is never possible.
    pub fn time_of_tokens_needed(&self, requested: usize) -> Option<Instant> {
        if !self.enabled {
            Some(Instant::now())
        } else if self.size < requested {
            // This means the batch can never succeed
            None
        } else {
            let tokens_needed = requested.saturating_sub(self.tokens);

            let intervals = (tokens_needed as f64 / self.rate as f64).ceil() as u32;
            Some(self.last_refresh_time + (ONE_SEC * intervals))
        }
    }

    /// Add new tokens
    /// Ensures bucket doesn't overfill
    fn add_tokens(&mut self, new_tokens: usize) {
        self.tokens = min(self.size, self.tokens.saturating_add(new_tokens));
    }

    /// Returns tokens that were unused
    pub fn return_tokens(&mut self, new_tokens: usize) {
        self.allowed_in_period = self.allowed_in_period.saturating_sub(new_tokens);
        self.add_tokens(new_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::MutexGuard, thread::sleep};
    use tokio::time::Duration;

    // Helper function, checks the throttle limit
    fn assert_acquire(rate_limiter: &mut MutexGuard<Bucket>, num_allowed: usize) {
        assert_eq!(
            num_allowed,
            rate_limiter
                .acquire_tokens(num_allowed)
                .expect("Expected tokens")
        );
        rate_limiter
            .acquire_tokens(1)
            .expect_err("Expected time to wait");
    }

    // Check number of keys in the rate limiter
    fn assert_num_keys(rate_limiters: &TokenBucketRateLimiter<&str>, num_keys: usize) {
        assert_eq!(num_keys, rate_limiters.buckets.read().len())
    }

    #[test]
    fn test_rate_limiting() {
        let bucket_size = 5;
        let bucket_rate = 1;
        let key = "Key";
        let rate_limiter = TokenBucketRateLimiter::test(bucket_size, bucket_rate);

        let bucket_arc = rate_limiter.bucket(key);
        let mut bucket = bucket_arc.lock();
        assert_acquire(&mut bucket, bucket_size);

        // After adding 2 token, only 2 should be allowed
        bucket.return_tokens(2);
        assert_acquire(&mut bucket, 2);

        // Adding more tokens than the bucket size, should only give bucket size
        bucket.return_tokens(bucket_size + 1);
        assert_acquire(&mut bucket, bucket_size);
    }

    #[test]
    fn test_message_rate_limiting() {
        let bucket_size = 5;
        let bucket_rate = 3;
        let key = "Key";
        let rate_limiter = TokenBucketRateLimiter::test(bucket_size, bucket_rate);

        let bucket_arc = rate_limiter.bucket(key);
        let mut bucket = bucket_arc.lock();

        // Larger than bucket, should never succeed
        let result = bucket.acquire_all_tokens(bucket_size + 1);
        assert!(result.expect_err("Should not give tokens").is_none());

        // Normal case
        let result = bucket.acquire_all_tokens(bucket_size);
        result.expect("Should be successful");

        // Test future wait
        let result = bucket.acquire_all_tokens(bucket_size);
        let wait_time = result
            .expect_err("Should not succeed, but will in future")
            .expect("Should have a time it succeeds");

        sleep(wait_time.duration_since(Instant::now()));
        let result = bucket.acquire_all_tokens(bucket_size);
        result.expect("Should be successful");
    }

    #[test]
    fn test_refill() {
        let bucket_size = 5;
        let bucket_rate = 1;
        let key = "Key";
        let rate_limiter = TokenBucketRateLimiter::test(bucket_size, bucket_rate);

        let bucket_arc = rate_limiter.bucket(key);
        let mut bucket = bucket_arc.lock();
        assert_acquire(&mut bucket, bucket_size);

        // After 1 refill period, we should be at least 1 rate change if not more
        // TODO: Put in a mock time service
        sleep(bucket.time_of_next_refill().duration_since(Instant::now()));
        bucket.refill();
        let num_tokens = bucket.tokens;
        assert!(num_tokens >= bucket_rate);

        // Test the autorefill
        assert_acquire(&mut bucket, num_tokens);
        sleep(bucket.time_of_next_refill().duration_since(Instant::now()));
        bucket.acquire_tokens(1).unwrap();
    }

    #[test]
    fn test_time_checks() {
        let bucket_size = 5;
        let bucket_rate = 1;
        let rate_limiter = TokenBucketRateLimiter::test(bucket_size, bucket_rate);

        let bucket_arc = rate_limiter.bucket("Key");
        let mut bucket = bucket_arc.lock();

        // Should always be less than 1 second
        assert!(bucket.time_of_next_refill() < Instant::now() + Duration::from_secs(1));

        // If we have all the tokens, it should take 0 time
        assert!(
            bucket
                .time_of_tokens_needed(bucket_size)
                .expect("Should have a duration")
                <= Instant::now()
        );
        assert_acquire(&mut bucket, bucket_size);

        // Should have all the tokens after 5 periods
        assert!(
            bucket
                .time_of_tokens_needed(bucket_size)
                .expect("Should have a duration")
                > Instant::now() + Duration::from_secs(bucket_size as u64 - 1)
        );

        // Greater than bucket size will never succeed
        assert!(bucket.time_of_tokens_needed(bucket_size + 1).is_none());
    }

    #[test]
    fn test_bucket_creation() {
        let key = "key";
        let rate_limiter = TokenBucketRateLimiter::test(1, 1);
        assert_num_keys(&rate_limiter, 0);

        // Ensure the buckets aren't being recreated
        let bucket1 = rate_limiter.bucket(key);
        let bucket2 = rate_limiter.bucket(key);

        assert_eq!(Arc::as_ptr(&bucket1), Arc::as_ptr(&bucket2));
        assert_eq!(3, Arc::strong_count(&bucket1));
    }

    #[test]
    fn test_garbage_collection() {
        let key_to_keep = "don't gc";
        let key_to_gc = "do gc";
        let rate_limiter = TokenBucketRateLimiter::test(1, 1);
        assert_num_keys(&rate_limiter, 0);

        // Create a bucket to hold onto
        let _bucket_arc = rate_limiter.bucket(key_to_keep);
        assert_num_keys(&rate_limiter, 1);

        // Create this bucket, and let go of it!
        {
            let _bucket_arc = rate_limiter.bucket(key_to_gc);
        }
        assert_num_keys(&rate_limiter, 2);

        // After garbage collect, the reference should disappear only to the second one
        assert!(rate_limiter.try_garbage_collect_key(&key_to_gc));
        assert_num_keys(&rate_limiter, 1);

        // Can't GC something that's in use
        assert!(!rate_limiter.try_garbage_collect_key(&key_to_keep));
        assert_num_keys(&rate_limiter, 1);
    }
}
