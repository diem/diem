// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Sleep, SleepTrait, TimeServiceTrait, ZERO_DURATION};
use diem_infallible::Mutex;
use futures::future::Future;
use std::{
    cmp::max,
    collections::btree_map::BTreeMap,
    fmt::Debug,
    pin::Pin,
    sync::{Arc, MutexGuard},
    task::{Context, Poll, Waker},
    time::Duration,
};

/// Each waiter in the pending queue has a unique index to distinguish waiters
/// with the same deadline.
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
    /// The next unique index that we can allocate to a new waiter.
    next_sleep_index: SleepIndex,
    /// A queue of pending `MockSleep`s. The entries are stored in a `BTreeMap`
    /// ordered by the wait deadline (and entry index, meaning ties are broken by
    /// insertion order). Rather than store the `MockSleep`s themselves, we store
    /// a `Waker` to alert the `MockSleep` future when it's complete.
    ///
    /// Note that we store an `Option<Waker>` since there is a gap between when
    /// a `MockSleep` is created + registered and when it's first polled (and
    /// has a `Waker` that it can register).
    pending: BTreeMap<(Duration, SleepIndex), Option<Waker>>,
}

/// A [`Future`] that resolves when the simulated time in the [`MockTimeService`]
/// advances past its deadline. When these are pending, they have an entry in the
/// `MockTimeService`'s `pending` queue.
#[derive(Debug)]
pub struct MockSleep {
    /// A handle to the `MockTimeService` so we can get the underlying `SleepEntry`.
    time_service: MockTimeService,
    /// Wait until the simulated time has at least passed this `deadline`.
    deadline: Duration,
    /// This waiter's unique index.
    index: SleepIndex,
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
                pending: BTreeMap::new(),
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
        let deadline = self.trigger_min_sleep()?;

        // If the deadline was in the past, don't actually advance the time.
        self.now = max(self.now, deadline);

        Some(self.now)
    }

    fn advance(&mut self, duration: Duration) -> usize {
        // Advance the simulated time.
        self.now += duration;

        // Get the number of now-expired waiters.
        let num_expired = self
            .pending
            .keys()
            .filter(|&(deadline, _index)| deadline <= &self.now)
            .count();

        // Wake up and unregister all the expired waiters.
        for _ in 0..num_expired {
            self.trigger_min_sleep()
                .expect("must be at least num_expired waiters");
        }

        num_expired
    }

    fn next_sleep_index(&mut self) -> SleepIndex {
        let index = self.next_sleep_index;
        self.next_sleep_index = self
            .next_sleep_index
            .checked_add(1)
            .expect("too many sleep entries");
        index
    }

    fn get_mut_sleep(
        &mut self,
        deadline: Duration,
        index: SleepIndex,
    ) -> Option<&mut Option<Waker>> {
        self.pending.get_mut(&(deadline, index))
    }

    fn is_sleep_registered(&self, deadline: Duration, index: SleepIndex) -> bool {
        self.pending.contains_key(&(deadline, index))
    }

    // Register a new waiter that will sleep for `duration`. Return an index that
    // can be used to query, trigger, or unregister this waiter later on.
    fn register_sleep(
        &mut self,
        duration: Duration,
        maybe_waker: Option<Waker>,
    ) -> (Duration, SleepIndex) {
        let deadline = self.now + duration;
        let index = self.next_sleep_index();
        let prev_entry = self.pending.insert((deadline, index), maybe_waker);
        assert!(
            prev_entry.is_none(),
            "there can never be an entry at an unused SleepIndex"
        );
        (deadline, index)
    }

    // Unregister a specific waiter with the given `deadline` and `index` and
    // return its waker (if there is one).
    fn unregister_sleep(&mut self, deadline: Duration, index: SleepIndex) -> Option<Option<Waker>> {
        self.pending.remove(&(deadline, index))
    }

    // Unregister and return the next waiter with the earliest deadline.
    fn unregister_min_sleep(&mut self) -> Option<((Duration, SleepIndex), Option<Waker>)> {
        // TODO(philiphayes): use `BTreeMap::pop_first()` when that stabilizes.
        let (deadline, index) = self.pending.keys().next()?;
        let deadline = *deadline;
        let index = *index;
        self.pending.remove_entry(&(deadline, index))
    }

    // Wake up the next waiter with the earliest deadline and return the wake
    // time. Return `None` if there are no waiters.
    fn trigger_min_sleep(&mut self) -> Option<Duration> {
        self.unregister_min_sleep()
            .map(|((deadline, _index), maybe_waker)| {
                if let Some(waker) = maybe_waker {
                    waker.wake();
                }
                deadline
            })
    }
}

///////////////
// MockSleep //
///////////////

impl MockSleep {
    fn new(time_service: MockTimeService, duration: Duration) -> Self {
        let (deadline, index) = time_service.lock().register_sleep(duration, None);

        Self {
            time_service,
            deadline,
            index,
        }
    }
}

impl SleepTrait for MockSleep {
    fn is_elapsed(&self) -> bool {
        let inner = self.time_service.lock();
        !inner.is_sleep_registered(self.deadline, self.index)
    }

    fn reset(&mut self, duration: Duration) {
        let mut inner = self.time_service.lock();

        // Unregister us from the time service (if we're not triggered yet)
        // and pull out our waker (if it's there).
        let maybe_waker = inner.unregister_sleep(self.deadline, self.index).flatten();

        // Register us with the time service with our new deadline.
        let (deadline, index) = inner.register_sleep(duration, maybe_waker);
        self.deadline = deadline;
        self.index = index;
    }
}

impl Future for MockSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.time_service.lock();

        let maybe_entry = inner.get_mut_sleep(self.deadline, self.index);
        match maybe_entry {
            Some(maybe_waker) => {
                // We're still waiting. Update our `Waker` so we can get notified.
                maybe_waker.replace(cx.waker().clone());
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
        self.time_service
            .lock()
            .unregister_sleep(self.deadline, self.index);
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
