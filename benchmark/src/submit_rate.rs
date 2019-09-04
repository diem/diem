use logger::prelude::*;
use std::{thread, time, vec::IntoIter};

/// Items are returned in the world of microseconds.
const MICROSECONDS_IN_ONE_SECOND: u64 = 1_000_000;

/// Utilize IntoIter to implement an iterator that returns constant number of items per second.
/// That is, an item is returned to user after waiting some time duration.
/// Return immediately (e.g. flood all available items) when rate is set to u64::MAX.
#[derive(Debug, Clone)]
pub struct ConstantRate<T> {
    /// Number of items to return per second.
    rate: u64,
    /// Time duration to return 1 item.
    interval_us: u64,
    /// Between two calls to next(), user may perform operations on the item, consuming some time.
    /// We calculate it as the duration between two last_call values and then exclude.
    last_call: time::Instant,
    /// Pointer to the items to be consumed.
    into_iter: IntoIter<T>,
}

impl<T> ConstantRate<T> {
    /// Init with rate, and the IntoIter of the vector to be consumed.
    /// For simplicity we assume user calls next() right after new().
    pub fn new(rate: u64, into_iter: IntoIter<T>) -> Self {
        assert!(rate > 0);
        let interval_us = MICROSECONDS_IN_ONE_SECOND / rate;
        debug!("Return 1 item after {} us", interval_us);
        ConstantRate {
            rate,
            interval_us,
            last_call: time::Instant::now(),
            into_iter,
        }
    }
}

impl<T> Iterator for ConstantRate<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let exclude_duration_us = self.last_call.elapsed().as_micros() as u64;
        if self.interval_us > exclude_duration_us {
            let rest = self.interval_us - exclude_duration_us;
            debug!("Sleep {} us before return next item", rest);
            thread::sleep(time::Duration::from_micros(rest as u64));
        }
        self.last_call = time::Instant::now();
        self.into_iter.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::submit_rate::ConstantRate;
    use std::{thread, time};

    #[test]
    fn test_submit_rate_empty_vec() {
        let empty_vec: Vec<u32> = vec![];
        let mut rate = ConstantRate::new(1, empty_vec.into_iter());
        let item = rate.next();
        assert_eq!(item.is_none(), true);
    }

    #[test]
    fn test_submit_rate_flood_all() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7];
        let flood = ConstantRate::new(std::u64::MAX, vec.into_iter());

        let now = time::Instant::now();
        // consume all elements
        flood.into_iter().for_each(drop);
        let elapsed = now.elapsed().as_micros();
        // Loop should finish instantly
        assert!(elapsed < 1000);
    }

    #[test]
    fn test_submit_rate_contant_rate() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let const_rate = ConstantRate::new(2, vec.into_iter());

        let mut now = time::Instant::now();
        for _item in const_rate {
            let new_now = time::Instant::now();
            let delta = new_now.duration_since(now).as_micros();
            // Interval between each call to next() should be roughly 0.5 second.
            assert!(delta < 510_000);
            assert!(delta > 490_000);
            now = new_now;
        }
    }

    #[test]
    fn test_submit_rate_exclude_duration() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let const_rate = ConstantRate::new(2, vec.into_iter());

        let mut now = time::Instant::now();
        for _item in const_rate {
            let new_now = time::Instant::now();
            let delta = new_now.duration_since(now).as_micros();
            // Interval between each call to next() should be roughly 0.5 second.
            assert!(delta < 510_000);
            assert!(delta > 490_000);
            now = time::Instant::now();
            // Use this sleep to emulate that user performed something on item.
            thread::sleep(time::Duration::from_micros(10_000));
        }
    }
}
