// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{health::ValidatorEvent, util::unix_timestamp_now};
use libra_logger::*;
use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub struct LogTail {
    pub event_receiver: mpsc::Receiver<ValidatorEvent>,
    pub pending_messages: Arc<AtomicI64>,
}

impl LogTail {
    pub fn recv_all_until_deadline(&self, deadline: Instant) -> Vec<ValidatorEvent> {
        let mut events = vec![];
        while Instant::now() < deadline {
            match self.event_receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(..) => thread::sleep(Duration::from_millis(1)),
            }
        }
        let events_count = events.len() as i64;
        let prev = self
            .pending_messages
            .fetch_sub(events_count, Ordering::Relaxed);
        let pending = prev - events_count;
        let now = unix_timestamp_now();
        if let Some(last) = events.last() {
            let delay = now - last.received_timestamp;
            if delay > Duration::from_secs(1) {
                warn!(
                    "{} Last event delay: {}, pending {}",
                    now.as_millis(),
                    delay.as_millis(),
                    pending
                );
            }
        } else {
            debug!("{} No events", now.as_millis());
        }
        events
    }

    pub fn recv_all(&self) -> Vec<ValidatorEvent> {
        let mut events = vec![];
        while let Ok(event) = self.event_receiver.try_recv() {
            self.pending_messages.fetch_sub(1, Ordering::Relaxed);
            events.push(event);
        }
        events
    }
}
