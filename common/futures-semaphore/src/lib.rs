// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! `Semaphore` holds a set of permits. Permits are used to synchronize access
//! to a shared resource. Before accessing the shared resource, callers must
//! acquire a permit from the semaphore. Once the permit is acquired, the caller
//! then enters the critical section. When the caller is finished, they drop the
//! permit to release it back to the semaphore.
//!
//! `Semaphore` is futures-aware. Acquiring a [`Permit`] is an async function,
//! which will yield to the futures executor if no permits are available and be
//! woken up when one becomes available.
//!
//! In contrast, acquiring a permit from a [`std::sync::Mutex`] +
//! [`std::sync::Condvar`] style semaphore will block the entire OS thread, which
//! could potentially deadlock the futures runtime if enough tasks are waiting on
//! the semaphore.

use std::sync::Arc;
use tokio::sync::Semaphore as TokioSemaphore;

/// The wrapped tokio [`Semaphore`](TokioSemaphore) and total permit capacity.
#[derive(Debug)]
struct Inner {
    semaphore: TokioSemaphore,
    capacity: usize,
}

/// A futures-aware semaphore.
#[derive(Clone, Debug)]
pub struct Semaphore {
    inner: Arc<Inner>,
}

/// A permit acquired from a semaphore, allowing access to a shared resource.
/// Dropping a `Permit` will release it back to the semaphore.
#[derive(Debug)]
pub struct Permit {
    inner: Arc<Inner>,
}

impl Semaphore {
    /// Create a new semaphore with `capacity` number of available permits.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                semaphore: TokioSemaphore::new(capacity),
                capacity,
            }),
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    pub fn available_permits(&self) -> usize {
        self.inner.semaphore.available_permits()
    }

    pub fn is_idle(&self) -> bool {
        self.available_permits() == self.capacity()
    }

    pub fn is_full(&self) -> bool {
        self.available_permits() == 0
    }

    /// Acquire an available permit from the semaphore.
    ///
    /// If there are no permits currently available, the future will wait until
    /// a permit is released back to the semaphore.
    pub async fn acquire(&self) -> Permit {
        // Acquire a permit and then immediately "forget" it (drop it without
        // returning the permit to the pool of available permits). We will
        // manually add a new permit when our `Permit` RAII guard is dropped.
        //
        // We do this instead of holding tokio's `SemaphorePermit` in the RAII
        // guard due to lifetime issues--tokio's `SemaphorePermit` needs a
        // direct reference to the `Semaphore`; however, we want to send our
        // `Permit` to other tasks or threads, so we need an `Arc` to the
        // semaphore so the permit doesn't outlive the semaphore.
        let permit = self.inner.semaphore.acquire().await;
        permit.forget();

        Permit {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Try to acquire an available permit from the semaphore. If no permits are
    /// available, return `None`.
    pub fn try_acquire(&self) -> Option<Permit> {
        match self.inner.semaphore.try_acquire() {
            Ok(permit) => {
                // See above comment in `Semaphore::acquire`.
                permit.forget();
                Some(Permit {
                    inner: Arc::clone(&self.inner),
                })
            }
            Err(_) => None,
        }
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.inner.semaphore.add_permits(1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{
        executor::block_on,
        future::{Future, FutureExt},
    };
    use std::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };
    use tokio::{runtime::Runtime, time::delay_for};

    #[test]
    fn basic_functionality_semaphore() {
        let s = Semaphore::new(3);

        assert!(s.is_idle());

        let _p1 = block_on(s.acquire());
        let p2 = block_on(s.acquire());
        let p3 = block_on(s.acquire());

        assert!(s.is_full());
        assert!(s.try_acquire().is_none());

        drop(p2);

        assert!(!s.is_full());
        assert!(!s.is_idle());

        let _p4 = block_on(s.acquire());

        assert!(s.is_full());
        assert!(s.try_acquire().is_none());

        drop(p3);

        assert!(!s.is_full());
        assert!(!s.is_idle());

        assert!(s.try_acquire().is_some());
    }

    fn yield_task() -> impl Future<Output = ()> {
        delay_for(Duration::from_millis(1)).map(|_| ())
    }

    // spawn NUM_TASKS futures that acquire a common semaphore, ensuring that no
    // more than MAX_WORKERS ever enter the critical section.
    #[test]
    fn concurrent_semaphore() {
        const MAX_WORKERS: u32 = 20;
        const NUM_TASKS: u32 = 1000;
        static WORKERS: AtomicU32 = AtomicU32::new(0);
        static COMPLETED_TASKS: AtomicU32 = AtomicU32::new(0);
        let semaphore = Semaphore::new(MAX_WORKERS as usize);

        let rt = Runtime::new().unwrap();

        for _ in 0..NUM_TASKS {
            let semaphore = semaphore.clone();
            let f = async move {
                let _permit = semaphore.acquire().await;

                // acquired permit, there should only ever be MAX_WORKERS in this
                // critical section

                let prev_workers = WORKERS.fetch_add(1, Ordering::SeqCst);
                assert!(prev_workers < MAX_WORKERS);

                // yield back to the tokio scheduler
                yield_task().await;

                let prev_workers = WORKERS.fetch_sub(1, Ordering::SeqCst);
                assert!(prev_workers > 0 && prev_workers <= MAX_WORKERS);

                COMPLETED_TASKS.fetch_add(1, Ordering::Relaxed);

                // drop _permit and release access
            };
            rt.spawn(f);
        }

        // spin until completed
        loop {
            let completed = COMPLETED_TASKS.load(Ordering::Relaxed);
            if completed == NUM_TASKS {
                break;
            } else {
                ::std::sync::atomic::spin_loop_hint();
            }
        }
    }
}
