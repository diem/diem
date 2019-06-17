// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    io,
    sync::{Arc, Mutex},
};

/// Rust test runner normally captures output of tests
/// However, this does not work if test spawns new threads, only main thread output is captured:
/// https://github.com/rust-lang/rust/issues/42474
/// This struct solves this problem by grabbing output capture in main thread and allowing to
/// apply it to threads created by test code with it.
///
/// It is not magical though, you need to grab the capturing writer in main thread and manually
/// call apply() on every thread that you create in test in order for this to work
///
/// For tokio runtime, runtime::Builder::after_start can be used to setup capture in tokio threads
///
/// See more details in description of 15fb112518f01d729ca49abe0c900c5c550783ab
#[derive(Clone)]
pub struct OutputCapture {
    writer: Option<AggregateWriter>,
}

impl OutputCapture {
    /// Grabs override on current thread.
    /// Call this method in main thread of test to grab current stdout override
    /// If no override is set, this function will return no-op OutputCapture
    pub fn grab() -> OutputCapture {
        OutputCapture {
            writer: AggregateWriter::grab(),
        }
    }

    /// Apply output capture to current thread
    /// If no capture was grabbed in grab(), this method will be no-op
    pub fn apply(&self) {
        if let Some(ref writer) = self.writer {
            io::set_print(Some(Box::new(writer.clone())));
            io::set_panic(Some(Box::new(writer.clone())));
        }
    }
}

// This is cloneable writer
// It aggregates output from all cloned instances into single inner writer
#[derive(Clone)]
struct AggregateWriter {
    inner: Arc<Mutex<dyn io::Write + Send>>,
}

impl AggregateWriter {
    fn grab() -> Option<AggregateWriter> {
        // Because dyn Writer is not cloneable, the only way to take current writer
        // is to push it out by setting print to some new value (None in this case)
        let previous = io::set_print(None);
        if let Some(previous) = previous {
            let writer = AggregateWriter::new(previous);
            io::set_print(Some(Box::new(writer.clone())));
            Some(writer)
        } else {
            None
        }
    }

    fn new(inner: Box<dyn io::Write + Send>) -> AggregateWriter {
        AggregateWriter {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl io::Write for AggregateWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}
