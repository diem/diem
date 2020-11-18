// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters::STRUCT_LOG_COUNT, Event, Metadata};

use once_cell::sync::OnceCell;
use std::sync::Arc;

thread_local! {
  static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();
}

/// A trait encapsulating the operations required of a logger.
pub trait Logger: Sync + Send + 'static {
    /// Determines if an event with the specified metadata would be logged
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Record an event
    fn record(&self, event: &Event);
}

pub(crate) fn dispatch(event: &Event) {
    LOGGER.with(|logger| {
        if let Some(logger) = logger.get() {
            STRUCT_LOG_COUNT.inc();
            logger.record(event)
        }
    })
}

pub(crate) fn enabled(metadata: &Metadata) -> bool {
    LOGGER.with(|logger| {
        logger
            .get()
            .map(|logger| logger.enabled(metadata))
            .unwrap_or(false)
    })
}

pub fn set_global_logger(global_logger: Arc<dyn Logger>) {
    LOGGER.with(|logger| {
        if logger.set(global_logger).is_err() {
            eprintln!("Global logger has already been set");
        }
    })
}
