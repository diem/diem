// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters::STRUCT_LOG_COUNT, Event, Metadata};

use once_cell::sync::OnceCell;
use std::sync::Arc;

static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();

/// A trait encapsulating the operations required of a logger.
pub trait Logger: Sync + Send + 'static {
    /// Determines if an event with the specified metadata would be logged
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Record an event
    fn record(&self, event: &Event);
}

pub(crate) fn dispatch(event: &Event) {
    if let Some(logger) = LOGGER.get() {
        STRUCT_LOG_COUNT.inc();
        logger.record(event)
    }
}

pub(crate) fn enabled(metadata: &Metadata) -> bool {
    LOGGER
        .get()
        .map(|logger| logger.enabled(metadata))
        .unwrap_or(false)
}

pub fn set_global_logger(logger: Arc<dyn Logger>) {
    if LOGGER.set(logger).is_err() {
        eprintln!("Global logger has already been set");
    }
}
