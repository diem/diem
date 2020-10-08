// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_infallible::{duration_since_epoch, Mutex};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{self, value as json};
use std::{collections::VecDeque, convert::TryInto};

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonLogEntry {
    pub name: String,
    pub timestamp: u64,
    pub json: json::Value,
}

const MAX_EVENTS_IN_QUEUE: usize = 10_000;

/// Writes event to event stream
/// Example:
///   event!("committed", block="b");
// TODO: ideally we want to unify it with existing logger
#[macro_export]
macro_rules! event {
    ($name:expr, $($json:tt)*) => {
        $crate::json_log::send_json_log($crate::json_log::JsonLogEntry::new(
            $name,
            serde_json::json!({$($json)+}),
        ));
    };
}

// This queue maintains last MAX_EVENTS_IN_QUEUE events
// This is very efficiently implemented with circular buffer with fixed capacity
static JSON_LOG_ENTRY_QUEUE: Lazy<Mutex<VecDeque<JsonLogEntry>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_EVENTS_IN_QUEUE)));

impl JsonLogEntry {
    pub fn new(name: &'static str, json: json::Value) -> Self {
        let timestamp = duration_since_epoch()
            .as_millis()
            .try_into()
            .expect("Unable to convert u128 into u64");
        JsonLogEntry {
            name: name.into(),
            timestamp,
            json,
        }
    }
}

/// Sends event to event stream.
///
/// Note that this method acquires global lock for brief moment.
/// This means that very hot threads can not use this method concurrently, otherwise they
/// will contend for same lock.
// TODO: if we use events more often we should rewrite it to be non-blocking
pub fn send_json_log(entry: JsonLogEntry) {
    let mut queue = JSON_LOG_ENTRY_QUEUE.lock();
    if queue.len() >= MAX_EVENTS_IN_QUEUE {
        queue.pop_front();
    }
    queue.push_back(entry);
}

/// Get up to MAX_EVENTS_IN_QUEUE last events and clears the queue
pub fn pop_last_entries() -> Vec<JsonLogEntry> {
    let mut queue = JSON_LOG_ENTRY_QUEUE.lock();
    queue.drain(..).collect()
}
