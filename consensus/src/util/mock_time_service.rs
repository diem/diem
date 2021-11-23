// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::util::time_service::{ScheduledTask, TimeService};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use futures::future::AbortHandle;
use std::{sync::Arc, time::Duration};

/// SimulatedTimeService implements TimeService, however it does not depend on actual time
/// There are multiple ways to use it:
/// SimulatedTimeService::new will create time service that simply 'stuck' on time 0
/// SimulatedTimeService::update_auto_advance_limit can then be used to allow time to advance up to
/// certain limit. SimulatedTimeService::auto_advance_until will create time service that will 'run'
/// until certain time limit Note that SimulatedTimeService does not actually wait for any timeouts,
/// notion of time in it is abstract. Tasks run asap as long as they are scheduled before configured
/// time limit
pub struct SimulatedTimeService {
    inner: Arc<Mutex<SimulatedTimeServiceInner>>,
}

struct SimulatedTimeServiceInner {
    now: Duration,
    pending: Vec<(Duration, Box<dyn ScheduledTask>)>,
    time_limit: Duration,
    /// Maximum duration self.now is allowed to advance to
    max: Duration,
}

impl TimeService for SimulatedTimeService {
    fn run_after(&self, timeout: Duration, mut t: Box<dyn ScheduledTask>) -> AbortHandle {
        let mut inner = self.inner.lock();
        let now = inner.now;
        let deadline = now + timeout;
        if deadline > inner.time_limit {
            debug!(
                "sched for deadline: {}, now: {}, limit: {}",
                deadline.as_millis(),
                now.as_millis(),
                inner.time_limit.as_millis()
            );
            inner.pending.push((deadline, t));
        } else {
            debug!(
                "exec deadline: {}, now: {}",
                deadline.as_millis(),
                now.as_millis()
            );
            inner.now = deadline;
            if inner.now > inner.max {
                inner.now = inner.max;
            }
            // Perhaps this could be done better, but I think its good enough for tests...
            futures::executor::block_on(t.run());
        }
        let (handle, _) = AbortHandle::new_pair();
        handle
    }

    fn get_current_timestamp(&self) -> Duration {
        self.inner.lock().now
    }

    fn sleep(&self, t: Duration) {
        let inner = self.inner.clone();
        let mut inner = inner.lock();
        inner.now += t;
        if inner.now > inner.max {
            inner.now = inner.max;
        }
    }
}

impl SimulatedTimeService {
    /// Creates new SimulatedTimeService in disabled state (time not running)
    pub fn new() -> SimulatedTimeService {
        SimulatedTimeService {
            inner: Arc::new(Mutex::new(SimulatedTimeServiceInner {
                now: Duration::from_secs(0),
                pending: vec![],
                time_limit: Duration::from_secs(0),
                max: Duration::from_secs(std::u64::MAX),
            })),
        }
    }

    /// Creates new SimulatedTimeService that automatically advance time up to time_limit
    pub fn auto_advance_until(time_limit: Duration) -> SimulatedTimeService {
        SimulatedTimeService {
            inner: Arc::new(Mutex::new(SimulatedTimeServiceInner {
                now: Duration::from_secs(0),
                pending: vec![],
                time_limit,
                max: Duration::from_secs(std::u64::MAX),
            })),
        }
    }

    /// Update time_limit of this SimulatedTimeService instance and run pending tasks that has
    /// deadline lower then new time_limit
    #[allow(dead_code)]
    pub fn update_auto_advance_limit(&mut self, time: Duration) {
        let mut inner = self.inner.lock();
        inner.time_limit += time;
        let time_limit = inner.time_limit;
        let mut i = 0;
        let mut drain = vec![];
        while i != inner.pending.len() {
            let deadline = inner.pending[i].0;
            if deadline <= time_limit {
                drain.push(inner.pending.remove(i));
            } else {
                i += 1;
            }
        }
        for (_, mut t) in drain {
            // probably could be done better then that, but for now I feel its good enough for tests
            futures::executor::block_on(t.run());
        }
    }
}

impl Clone for SimulatedTimeService {
    fn clone(&self) -> SimulatedTimeService {
        SimulatedTimeService {
            inner: self.inner.clone(),
        }
    }
}
