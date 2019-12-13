// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use serde_json::{self, value as json};
use std::{collections::VecDeque, sync::Mutex, time::SystemTime};

pub struct JsonLogEntry {
    pub name: &'static str,
    pub timestamp: u128,
    pub json: json::Value,
}

const MAX_EVENTS_IN_QUEUE: usize = 1_000;

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

lazy_static! {
    // This queue maintains last MAX_EVENTS_IN_QUEUE events
    // This is very efficiently implemented with circular buffer with fixed capacity
    static ref JSON_LOG_ENTRY_QUEUE: Mutex<VecDeque<JsonLogEntry>> = Mutex::new(
        VecDeque::with_capacity(MAX_EVENTS_IN_QUEUE)
    );
}

impl JsonLogEntry {
    pub fn new(name: &'static str, json: json::Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("now > UNIX_EPOCH")
            .as_millis();
        JsonLogEntry {
            name,
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
    let mut queue = JSON_LOG_ENTRY_QUEUE.lock().unwrap();
    if queue.len() >= MAX_EVENTS_IN_QUEUE {
        queue.pop_front();
    }
    queue.push_back(entry);
}

/// Get up to MAX_EVENTS_IN_QUEUE last events and clears the queue
pub fn pop_last_entries() -> Vec<JsonLogEntry> {
    let mut queue = JSON_LOG_ENTRY_QUEUE.lock().unwrap();
    queue.drain(..).collect()
}
