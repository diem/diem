// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Sleep, SleepTrait, TimeServiceTrait, ZERO_DURATION};
use diem_infallible::Mutex;
use futures::future::Future;
use std::{
    cmp::max,
    collections::hash_map::{DefaultHasher, HashMap},
    fmt::Debug,
    hash::BuildHasherDefault,
    pin::Pin,
    sync::{Arc, MutexGuard},
    task::{Context, Poll, Waker},
    time::Duration,
};

// TODO(philiphayes): use more robust solution that'll let us control the seed.
type DeterministicHasher = BuildHasherDefault<DefaultHasher>;

type SleepIndex = usize;

/// A [`TimeService`] that simulates time and allows for fine-grained control over
/// advancing time and waking up sleeping tasks.
#[derive(Clone, Debug)]
pub struct MockTimeService {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    /// The current simulated time.
    now: Duration,
    /// The next free index in the `pending` queue that we can allocate to a new waiter.
    next_sleep_index: SleepIndex,
    /// A queue of pending `SleepEntry`s, each with a deadline for when we should
    /// wake them.
    ///
    /// This `HashMap` also uses a hasher with constant seed (in the future the
    /// seed should be configurable) to help improve test determinism.
    pending: HashMap<SleepIndex, SleepEntry, DeterministicHasher>,
}

/// A timer entry owned by the `MockTimeService` in the `pending` queue.
///
/// These are indexed by `SleepIndex`, which you get when you `register_sleep` a
/// new waiter.
#[derive(Debug)]
struct SleepEntry {
    /// The time when this sleep task expires and should be woken up.
    deadline: Duration,
    /// When the corresponding `MockSleep` future polls and isn't expired
    /// yet, it will register its `Waker` here for the `MockTimeService`
    maybe_waker: Option<Waker>,
}

/// A [`Future`] that resolves when the simulated time in the [`MockTimeService`]
/// advances past the deadline. When these are pending, they have a `SleepEntry`
/// in the `MockTimeService`'s `pending` queue.
#[derive(Debug)]
pub struct MockSleep {
    /// A handle to the `MockTimeService` so we can get the underlying `SleepEntry`.
    time_service: MockTimeService,
    /// The index of this `MockSleep`'s `SleepEntry` in the `MockTimeService`'s
    /// `pending` queue. If the `pending` queue doesn't contain our entry, then
    /// this sleep must be completed.
    entry_index: SleepIndex,
}

/////////////////////
// MockTimeService //
/////////////////////

impl MockTimeService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                now: ZERO_DURATION,
                next_sleep_index: 0,
                pending: HashMap::default(),
            })),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock()
    }

    /// Return the number of pending `Sleep` waiters.
    pub fn num_waiters(&self) -> usize {
        self.lock().pending.len()
    }

    /// Advance time to the next pending waiter, wake it up, and return the wake
    /// time, or `None` if there are no waiters.
    pub async fn advance_to_next(&self) -> Option<Duration> {
        let wake_time = self.lock().advance_to_next();

        // Yield to the executor to run any freshly awoken tasks (which might
        // also create more `Sleep` futures).
        //
        // Imporant: don't hold the lock while yielding. We don't want to block
        // creating new `Sleep`s or letting existing `Sleep`s run.
        tokio::task::yield_now().await;

        wake_time
    }

    /// Advance time by `duration` and wake any pending waiters whose sleep has
    /// expired. Return the number of waiters that we woke up.
    pub async fn advance(&self, duration: Duration) -> usize {
        let num_woken = self.lock().advance(duration);

        // Yield to the executor to run any freshly awoken tasks (which might
        // also create more `Sleep` futures).
        //
        // Imporant: don't hold the lock while yielding. We don't want to block
        // creating new `Sleep`s or letting existing `Sleep`s run.
        tokio::task::yield_now().await;

        num_woken
    }
}

impl TimeServiceTrait for MockTimeService {
    fn now(&self) -> Duration {
        self.lock().now
    }

    fn sleep(&self, duration: Duration) -> Sleep {
        MockSleep::new(self.clone(), duration).into()
    }
}

///////////
// Inner //
///////////

impl Inner {
    fn advance_to_next(&mut self) -> Option<Duration> {
        // Find the next entry with the smallest deadline. Exit early without
        // advancing if there are no pending waiters.
        let earliest_index = self
            .pending
            .iter()
            .min_by_key(|(_index, entry)| entry.deadline)
            .map(|(index, _entry)| *index)?;

        // Wake up that entry.
        let wake_time = self.trigger_sleep(earliest_index).unwrap();

        // If the deadline was in the past, don't actually advance the time.
        self.now = max(self.now, wake_time);

        Some(self.now)
    }

    fn advance(&mut self, duration: Duration) -> usize {
        // Advance the simulated time.
        self.now += duration;

        // Get the indices and deadlines of all expired waiters.
        let mut expired_entries = self
            .pending
            .iter()
            .map(|(index, entry)| (*index, entry.deadline))
            .filter(|(_index, deadline)| deadline <= &self.now)
            .collect::<Vec<_>>();

        // Wake up the waiters in order.
        // TODO(philiphayes): break ties (same deadline) randomly but with a controlled seed.
        expired_entries.sort_by_key(|(_index, deadline)| *deadline);

        // Wake up all the expired waiters.
        let mut num_woken = 0;
        for (index, _deadline) in expired_entries.into_iter() {
            self.trigger_sleep(index).unwrap();
            num_woken += 1;
        }
        num_woken
    }

    fn next_sleep_index(&mut self) -> SleepIndex {
        let index = self.next_sleep_index;
        self.next_sleep_index = self
            .next_sleep_index
            .checked_add(1)
            .expect("too many sleep entries");
        index
    }

    fn get_mut_sleep(&mut self, index: SleepIndex) -> Option<&mut SleepEntry> {
        self.pending.get_mut(&index)
    }

    fn is_registered(&self, index: SleepIndex) -> bool {
        self.pending.contains_key(&index)
    }

    // Register a new waiter that will sleep for `duration`. Return an index that
    // can be used to query, trigger, or unregister this waiter later on.
    fn register_sleep(&mut self, duration: Duration, maybe_waker: Option<Waker>) -> SleepIndex {
        let entry = SleepEntry {
            deadline: self.now + duration,
            maybe_waker,
        };
        let index = self.next_sleep_index();
        let prev_entry = self.pending.insert(index, entry);
        assert!(
            prev_entry.is_none(),
            "there can never be a SleepEntry at an unused SleepIndex"
        );
        index
    }

    fn unregister_sleep(&mut self, index: SleepIndex) -> Option<SleepEntry> {
        self.pending.remove(&index)
    }

    // Wake up a waiter at the given `index` and return the wake time. Return
    // `None` if there is no waiter (presumably it was canceled).
    fn trigger_sleep(&mut self, index: SleepIndex) -> Option<Duration> {
        self.unregister_sleep(index).map(|entry| {
            if let Some(waker) = entry.maybe_waker {
                waker.wake();
            }
            entry.deadline
        })
    }
}

///////////////
// MockSleep //
///////////////

impl MockSleep {
    fn new(time_service: MockTimeService, duration: Duration) -> Self {
        let entry_index = time_service.lock().register_sleep(duration, None);

        Self {
            time_service,
            entry_index,
        }
    }
}

impl SleepTrait for MockSleep {
    fn is_elapsed(&self) -> bool {
        let inner = self.time_service.lock();
        !inner.is_registered(self.entry_index)
    }

    fn reset(&mut self, duration: Duration) {
        let mut inner = self.time_service.lock();

        // Unregister us from the time service (if we're not triggered yet)
        // and pull out our waker (if it's there).
        let maybe_waker = inner
            .unregister_sleep(self.entry_index)
            .and_then(|entry| entry.maybe_waker);

        // Register us with the time service with our new deadline.
        self.entry_index = inner.register_sleep(duration, maybe_waker);
    }
}

impl Future for MockSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.time_service.lock();

        let maybe_entry = inner.get_mut_sleep(self.entry_index);
        match maybe_entry {
            Some(entry) => {
                // We're still waiting. Update our `Waker` so we can get notified.
                entry.maybe_waker = Some(cx.waker().clone());
                Poll::Pending
            }
            // If we're not in the queue then we are done!
            None => Poll::Ready(()),
        }
    }
}

impl Drop for MockSleep {
    fn drop(&mut self) {
        // Be sure to unregister us from the pending queue if we get dropped/canceled.
        self.time_service.lock().unregister_sleep(self.entry_index);
    }
}

///////////
// Tests //
///////////

#[cfg(test)]
mod test {
    use super::*;
    use futures::channel::oneshot;
    use tokio_test::{
        assert_pending, assert_ready, assert_ready_eq, assert_ready_err, assert_ready_ok, task,
    };

    #[tokio::test]
    async fn test_sleep() {
        let time = MockTimeService::new();

        // Initial pending queue should be empty. Advancing time wakes nothing.
        assert_eq!(time.num_waiters(), 0);
        assert_eq!(time.advance_to_next().await, None);
        let start = time.now();
        assert_eq!(time.advance(ms(10)).await, 0);
        assert_eq!(time.now() - start, ms(10));

        // Create a new sleep future. Check that it's registered in the queue.
        let mut sleep = task::spawn(time.sleep(ms(10)));
        assert!(!sleep.is_woken());
        assert_eq!(time.num_waiters(), 1);
        assert_pending!(sleep.poll());

        // It won't wake up until we pass its deadline.
        assert_eq!(time.advance(ms(5)).await, 0);
        assert!(!sleep.is_woken());
        assert_pending!(sleep.poll());

        // When we pass the deadline, the sleep future should be unregistered,
        // woken, and resolved.
        assert_eq!(time.advance(ms(5)).await, 1);
        assert_eq!(time.num_waiters(), 0);
        assert!(sleep.is_woken());
        assert_ready!(sleep.poll());

        // Resetting the sleep future should register it again.
        sleep.reset(ms(5));
        assert!(!sleep.is_woken());
        assert_eq!(time.num_waiters(), 1);
        assert_pending!(sleep.poll());

        // Passing the deadline of a reset sleep future should also work.
        assert_eq!(time.advance(ms(5)).await, 1);
        assert_eq!(time.num_waiters(), 0);
        assert!(sleep.is_woken());
        assert_ready!(sleep.poll());

        // Should still work if we don't poll the sleep future before advance.
        sleep.reset(ms(5));
        assert!(!sleep.is_woken());

        assert_eq!(time.advance(ms(5)).await, 1);
        assert_eq!(time.num_waiters(), 0);
        assert!(!sleep.is_woken());
        assert_ready!(sleep.poll());
    }

    #[tokio::test]
    async fn test_many_sleep() {
        let time = MockTimeService::new();

        assert_eq!(time.num_waiters(), 0);

        // Make a bunch of sleep futures.
        let mut sleep_5ms = task::spawn(time.sleep(ms(5)));
        let mut sleep_10ms_1 = task::spawn(time.sleep(ms(10)));
        let mut sleep_10ms_2 = task::spawn(time.sleep(ms(10)));
        let mut sleep_10ms_3 = task::spawn(time.sleep(ms(10)));
        let mut sleep_15ms = task::spawn(time.sleep(ms(15)));
        let mut sleep_20ms = task::spawn(time.sleep(ms(20)));

        assert_eq!(time.num_waiters(), 6);

        assert_pending!(sleep_5ms.poll());
        assert_pending!(sleep_10ms_1.poll());
        assert_pending!(sleep_10ms_2.poll());
        assert_pending!(sleep_10ms_3.poll());
        assert_pending!(sleep_15ms.poll());
        assert_pending!(sleep_20ms.poll());

        // Advance 10ms leaving just the 15ms and 20ms waiters.
        assert_eq!(time.advance(ms(10)).await, 4);
        assert_eq!(time.num_waiters(), 2);

        assert_ready!(sleep_5ms.poll());
        assert_ready!(sleep_10ms_1.poll());
        assert_ready!(sleep_10ms_2.poll());
        assert_ready!(sleep_10ms_3.poll());
        assert_pending!(sleep_15ms.poll());
        assert_pending!(sleep_20ms.poll());

        // Advance to next, triggering the 15ms waiter.
        assert_eq!(time.advance_to_next().await, Some(ms(15)));
        assert_eq!(time.num_waiters(), 1);

        assert_ready!(sleep_15ms.poll());
        assert_pending!(sleep_20ms.poll());

        // Advance to next, triggering the final 20ms waiter.
        assert_eq!(time.advance_to_next().await, Some(ms(20)));
        assert_eq!(time.num_waiters(), 0);

        assert_ready!(sleep_20ms.poll());
    }

    #[tokio::test]
    async fn test_interval() {
        let time = MockTimeService::new();

        let mut interval = task::spawn(time.interval(ms(10)));

        assert_pending!(interval.poll_next());
        assert!(!interval.is_woken());

        // Interval should trigger immediately.
        assert_eq!(time.advance_to_next().await, Some(ms(0)));
        assert!(interval.is_woken());
        assert_ready_eq!(interval.poll_next(), Some(()));
        assert_pending!(interval.poll_next());

        // Interval won't trigger until we pass the period.
        assert_eq!(time.advance(ms(5)).await, 0);
        assert!(!interval.is_woken());
        assert_pending!(interval.poll_next());

        // Interval should trigger again.
        assert_eq!(time.advance(ms(5)).await, 1);
        assert!(interval.is_woken());
        assert_ready_eq!(interval.poll_next(), Some(()));
        assert_pending!(interval.poll_next());
    }

    #[tokio::test]
    async fn test_timeout() {
        // Timeout with a future that's immediately ready.
        let time = MockTimeService::new();
        let mut timeout = task::spawn(time.timeout(ms(10), async {}));
        assert_ready_ok!(timeout.poll());

        // Timeout with future and deadline: future wins.
        let time = MockTimeService::new();
        let (tx, rx) = oneshot::channel();
        let mut timeout = task::spawn(time.timeout(ms(10), rx));

        assert_pending!(timeout.poll());
        assert_eq!(time.advance(ms(5)).await, 0);
        assert_pending!(timeout.poll());

        tx.send(()).unwrap();

        assert!(timeout.is_woken());
        assert_ready_ok!(timeout.poll()).unwrap();

        // Timeout with future and deadline: timeout wins.
        let time = MockTimeService::new();
        let (_tx, rx) = oneshot::channel::<()>();
        let mut timeout = task::spawn(time.timeout(ms(10), rx));

        assert_pending!(timeout.poll());
        assert_eq!(time.advance(ms(15)).await, 1);
        assert!(timeout.is_woken());

        assert_ready_err!(timeout.poll());
    }

    fn ms(duration: u64) -> Duration {
        Duration::from_millis(duration)
    }
}
