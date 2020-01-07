// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides common slog with glog formatting.

use crate::{
    collector_serializer::CollectorSerializer,
    kv_categorizer::{KVCategorizer, KVCategory},
};
use chrono;
use itertools::{Either, Itertools};
use slog::{Drain, Key, Level, OwnedKVList, Record, KV};
use slog_term::{Decorator, RecordDecorator};
use std::{io, str};
use thread_id;

/// A slog `Drain` for glog-formatted logs.
pub struct GlogFormat<D: Decorator, C: KVCategorizer> {
    decorator: D,
    categorizer: C,
}

impl<D: Decorator, C: KVCategorizer> GlogFormat<D, C> {
    /// Create a glog-formatted `Drain` using the provided `Decorator`, and `Categorizer`
    pub fn new(decorator: D, categorizer: C) -> GlogFormat<D, C> {
        GlogFormat {
            decorator,
            categorizer,
        }
    }
}

fn write_logline(
    decorator: &mut dyn RecordDecorator,
    level: Level,
    metadata: &OnelineMetadata,
) -> io::Result<()> {
    // Convert log level to a single character representation.
    let level = match level {
        Level::Critical => 'C',
        Level::Error => 'E',
        Level::Warning => 'W',
        Level::Info => 'I',
        Level::Debug => 'D',
        Level::Trace => 'T',
    };

    decorator.start_level()?;
    write!(decorator, "{}", level)?;

    decorator.start_timestamp()?;
    write!(decorator, "{}", metadata.now.format("%m%d %H:%M:%S%.6f"))?;

    decorator.start_whitespace()?;
    write!(decorator, " ")?;

    // Write the message.
    decorator.start_msg()?;
    write!(
        decorator,
        "{tid:>5} {file}:{line}] ",
        tid = metadata.tid,
        file = metadata.file,
        line = metadata.line,
    )
}

fn print_inline_kv<C: KVCategorizer>(
    decorator: &mut dyn RecordDecorator,
    categorizer: &C,
    kv: Vec<(Key, String)>,
) -> io::Result<()> {
    for (k, v) in kv {
        decorator.start_comma()?;
        write!(decorator, ", ")?;
        decorator.start_key()?;
        write!(decorator, "{}", categorizer.name(k))?;
        decorator.start_separator()?;
        write!(decorator, ": ")?;
        decorator.start_value()?;
        write!(decorator, "{}", v)?;
    }
    Ok(())
}

fn finish_logline(decorator: &mut dyn RecordDecorator) -> io::Result<()> {
    decorator.start_whitespace()?;
    writeln!(decorator)?;
    decorator.flush()
}

impl<D: Decorator, C: KVCategorizer> Drain for GlogFormat<D, C> {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> io::Result<Self::Ok> {
        self.decorator.with_record(record, values, |decorator| {
            let (inline_kv, level_kv): (Vec<_>, Vec<_>) = {
                let mut serializer = CollectorSerializer::new(&self.categorizer);
                values.serialize(record, &mut serializer)?;
                record.kv().serialize(record, &mut serializer)?;

                serializer
                    .into_inner()
                    .into_iter()
                    .filter_map(|(k, v)| match self.categorizer.categorize(k) {
                        KVCategory::Ignore => None,
                        KVCategory::Inline => Some((None, k, v)),
                        KVCategory::LevelLog(level) => Some((Some(level), k, v)),
                    })
                    .partition_map(|(l, k, v)| match l {
                        None => Either::Left((k, v)),
                        Some(level) => Either::Right((level, k, v)),
                    })
            };

            let metadata = OnelineMetadata::new(record);

            write_logline(decorator, record.level(), &metadata)?;
            write!(decorator, "{}", record.msg())?;
            print_inline_kv(decorator, &self.categorizer, inline_kv)?;
            finish_logline(decorator)?;

            for (level, k, v) in level_kv {
                write_logline(decorator, level, &metadata)?;
                write!(decorator, "{}: {}", self.categorizer.name(k), v)?;
                finish_logline(decorator)?;
            }
            Ok(())
        })
    }
}

struct OnelineMetadata {
    now: chrono::DateTime<chrono::Local>,
    tid: usize,
    file: &'static str,
    line: u32,
}

impl OnelineMetadata {
    fn new(record: &Record<'_>) -> Self {
        OnelineMetadata {
            now: chrono::Local::now(),
            tid: thread_id::get(),
            file: record.file(),
            line: record.line(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GlogFormat;

    use std::{
        io,
        sync::{Arc, Mutex},
    };

    use crate::kv_categorizer::InlineCategorizer;
    use once_cell::sync::Lazy;
    use regex::{Captures, Regex};
    use slog::{info, o, Drain, Logger};
    use slog_term::PlainSyncDecorator;
    use thread_id;

    // Create a regex that matches log lines.
    static LOG_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(.)(\d{4} \d\d:\d\d:\d\d\.\d{6}) +(\d+) ([^:]+):(\d+)\] (.*)$").unwrap()
    });

    /// Wrap a buffer so that it can be used by slog as a log output.
    #[derive(Clone)]
    pub struct TestBuffer {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestBuffer {
        pub fn new() -> TestBuffer {
            TestBuffer {
                buffer: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn get_string(&self) -> String {
            let buffer = self.buffer.lock().unwrap();
            String::from_utf8(buffer.clone()).unwrap()
        }
    }

    impl io::Write for TestBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.buffer.lock().unwrap().flush()
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct TestLine {
        level: String,
        tid: String,
        file: String,
        line: String,
        msg: String,
    }

    impl<'a> TestLine {
        fn new(level: &'static str, line: u32, msg: &'static str) -> Self {
            TestLine {
                level: level.to_owned(),
                tid: thread_id::get().to_string(),
                file: file!().to_owned(),
                line: line.to_string(),
                msg: msg.to_owned(),
            }
        }

        fn with_captures(captures: Captures<'a>) -> Self {
            TestLine {
                level: captures.get(1).unwrap().as_str().to_owned(),
                tid: captures.get(3).unwrap().as_str().to_owned(),
                file: captures.get(4).unwrap().as_str().to_owned(),
                line: captures.get(5).unwrap().as_str().to_owned(),
                msg: captures.get(6).unwrap().as_str().to_owned(),
            }
        }
    }

    #[test]
    fn test_inline() {
        // Create a logger that logs to a buffer instead of stderr.
        let test_buffer = TestBuffer::new();
        let decorator = PlainSyncDecorator::new(test_buffer.clone());
        let drain = GlogFormat::new(decorator, InlineCategorizer).fuse();
        let log = Logger::root(drain, o!("mode" => "test"));

        // Send a log to the buffer. Remember the line the log was on.
        let line = line!() + 1;
        info!(log, "Test log {}", 1; "tau" => 6.28);

        // Get the log string back out of the buffer.
        let log_string = test_buffer.get_string();

        // Check the log line's fields to make sure they match expected values.
        // For the timestamp, it's sufficient to just check it has the right form.
        let captures = LOG_REGEX.captures(log_string.as_str().trim_end()).unwrap();
        assert_eq!(
            TestLine::with_captures(captures),
            TestLine::new("I", line, "Test log 1, mode: test, tau: 6.28",)
        );
    }
}
