// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Periodic sampling for logs, metrics, and other use cases through a simple macro

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

/// The rate at which a `sample!` macro will run it's given function
#[derive(Debug)]
pub enum SampleRate {
    /// Only sample a single time during a window of time. This rate only has a resolution in
    /// seconds.
    Duration(Duration),
    /// Sample based on the frequency of the event. The provided u64 is the inverse of the
    /// frequency (1/x), for example Frequency(2) means that 1 out of every 2 events will be
    /// sampled (1/2).
    Frequency(u64),
    /// Always Sample
    Always,
}

/// An internal struct that can be checked if a sample is ready for the `sample!` macro
pub struct Sampling {
    rate: SampleRate,
    state: AtomicU64,
}

impl Sampling {
    pub const fn new(rate: SampleRate) -> Self {
        Self {
            rate,
            state: AtomicU64::new(0),
        }
    }

    pub fn sample(&self) -> bool {
        match &self.rate {
            SampleRate::Duration(rate) => Self::sample_duration(rate, &self.state),
            SampleRate::Frequency(rate) => Self::sample_frequency(*rate, &self.state),
            SampleRate::Always => true,
        }
    }

    fn sample_frequency(rate: u64, count: &AtomicU64) -> bool {
        let previous_count = count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                let new_count = if count == 0 {
                    rate.saturating_sub(1)
                } else {
                    count.saturating_sub(1)
                };
                Some(new_count)
            })
            .expect("Closure should always returns 'Some'. This is a Bug.");

        previous_count == 0
    }

    fn sample_duration(rate: &Duration, last_sample: &AtomicU64) -> bool {
        let rate = rate.as_secs();
        // Seconds since Unix Epoch
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        last_sample
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |last_sample| {
                if now.saturating_sub(last_sample) >= rate {
                    Some(now)
                } else {
                    None
                }
            })
            .is_ok()
    }
}

/// Samples a given function at a `SampleRate`, useful for periodically emitting logs or metrics on
/// high throughput pieces of code.
#[macro_export]
macro_rules! sample {
    ($sample_rate:expr, $($args:expr)+ ,) => {
        $crate::sample!($sample_rate, $($args)+);
    };

    ($sample_rate:expr, $($args:tt)+) => {{
        static SAMPLING: Sampling = $crate::sample::Sampling::new($sample_rate);
        if SAMPLING.sample() {
            $($args)+
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency() {
        // Frequency
        let sampling = Sampling::new(SampleRate::Frequency(10));
        let mut v = Vec::new();
        for i in 0..=25 {
            if sampling.sample() {
                v.push(i);
            }
        }

        assert_eq!(v, vec![0, 10, 20]);
    }

    #[test]
    fn always() {
        // Always
        let sampling = Sampling::new(SampleRate::Always);
        let mut v = Vec::new();
        for i in 0..5 {
            if sampling.sample() {
                v.push(i);
            }
        }

        assert_eq!(v, vec![0, 1, 2, 3, 4]);
    }

    #[ignore]
    #[test]
    fn duration() {
        // Duration
        let sampling = Sampling::new(SampleRate::Duration(Duration::from_secs(1)));
        let mut v = Vec::new();
        for i in 0..5 {
            if sampling.sample() {
                v.push(i);
            }

            std::thread::sleep(Duration::from_millis(500));
        }

        assert_eq!(v.len(), 2);
    }

    #[test]
    fn macro_expansion() {
        for i in 0..10 {
            sample!(
                SampleRate::Frequency(2),
                println!("loooooooooooooooooooooooooong hello {}", i),
            );

            sample!(SampleRate::Frequency(2), {
                println!("hello {}", i);
            });

            sample!(SampleRate::Frequency(2), println!("hello {}", i));

            sample! {
                SampleRate::Frequency(2),

                for j in 10..20 {
                    println!("hello {}", j);
                }
            }
        }
    }

    #[test]
    fn threaded() {
        fn work() -> usize {
            let mut count = 0;

            for _ in 0..1000 {
                sample!(SampleRate::Frequency(5), count += 1);
            }

            count
        }

        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(std::thread::spawn(work));
        }

        let mut count = 0;
        for handle in handles {
            count += handle.join().unwrap();
        }

        assert_eq!(count, 2000);
    }
}
