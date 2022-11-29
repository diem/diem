// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of writing logs to both local printers (e.g. stdout) and remote loggers
//! (e.g. Logstash)

use crate::{
    counters::{
        PROCESSED_STRUCT_LOG_COUNT, SENT_STRUCT_LOG_BYTES, SENT_STRUCT_LOG_COUNT,
        STRUCT_LOG_PARSE_ERROR_COUNT, STRUCT_LOG_QUEUE_ERROR_COUNT, STRUCT_LOG_SEND_ERROR_COUNT,
    },
    logger::Logger,
    struct_log::TcpWriter,
    Event, Filter, Level, LevelFilter, Metadata,
};
use backtrace::Backtrace;
use chrono::{SecondsFormat, Utc};
use diem_infallible::RwLock;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    env, fmt,
    io::Write,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
};

const RUST_LOG: &str = "RUST_LOG";
const RUST_LOG_REMOTE: &str = "RUST_LOG_REMOTE";
/// Default size of log write channel, if the channel is full, logs will be dropped
pub const CHANNEL_SIZE: usize = 10000;
const NUM_SEND_RETRIES: u8 = 1;

/// A single log entry emitted by a logging macro with associated metadata
#[derive(Debug, Serialize)]
pub struct LogEntry {
    #[serde(flatten)]
    metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_name: Option<String>,
    /// The program backtrace taken when the event occurred. Backtraces are
    /// only supported for errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    backtrace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<&'static str>,
    timestamp: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    data: BTreeMap<&'static str, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl LogEntry {
    fn new(event: &Event, thread_name: Option<&str>) -> Self {
        use crate::{Key, Value, Visitor};

        struct JsonVisitor<'a>(&'a mut BTreeMap<&'static str, serde_json::Value>);

        impl<'a> Visitor for JsonVisitor<'a> {
            fn visit_pair(&mut self, key: Key, value: Value<'_>) {
                let v = match value {
                    Value::Debug(d) => serde_json::Value::String(format!("{:?}", d)),
                    Value::Display(d) => serde_json::Value::String(d.to_string()),
                    Value::Serde(s) => match serde_json::to_value(s) {
                        Ok(value) => value,
                        Err(e) => {
                            eprintln!("error serializing structured log: {}", e);
                            return;
                        }
                    },
                };

                self.0.insert(key.as_str(), v);
            }
        }

        let metadata = *event.metadata();
        let thread_name = thread_name.map(ToOwned::to_owned);
        let message = event.message().map(fmt::format);

        static HOSTNAME: Lazy<Option<String>> = Lazy::new(|| {
            hostname::get()
                .ok()
                .and_then(|name| name.into_string().ok())
        });

        let hostname = HOSTNAME.as_deref();

        let backtrace = match metadata.level() {
            Level::Error => {
                let mut backtrace = Backtrace::new();
                let mut frames = backtrace.frames().to_vec();
                if frames.len() > 3 {
                    frames.drain(0..3); // Remove the first 3 unnecessary frames to simplify backtrace
                }
                backtrace = frames.into();
                Some(format!("{:?}", backtrace))
            }
            _ => None,
        };

        let mut data = BTreeMap::new();
        for schema in event.keys_and_values() {
            schema.visit(&mut JsonVisitor(&mut data));
        }

        Self {
            metadata,
            thread_name,
            backtrace,
            hostname,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            data,
            message,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn thread_name(&self) -> Option<&str> {
        self.thread_name.as_deref()
    }

    pub fn backtrace(&self) -> Option<&str> {
        self.backtrace.as_deref()
    }

    pub fn hostname(&self) -> Option<&str> {
        self.hostname
    }

    pub fn timestamp(&self) -> &str {
        self.timestamp.as_str()
    }

    pub fn data(&self) -> &BTreeMap<&'static str, serde_json::Value> {
        &self.data
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// A builder for a `DiemLogger`, configures what, where, and how to write logs.
pub struct DiemLoggerBuilder {
    channel_size: usize,
    level: Level,
    remote_level: Level,
    address: Option<String>,
    printer: Option<Box<dyn Writer>>,
    is_async: bool,
    custom_format: Option<fn(&LogEntry) -> Result<String, fmt::Error>>,
}

impl DiemLoggerBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            channel_size: CHANNEL_SIZE,
            level: Level::Info,
            remote_level: Level::Info,
            address: None,
            printer: Some(Box::new(StderrWriter)),
            is_async: false,
            custom_format: None,
        }
    }

    pub fn address(&mut self, address: String) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn read_env(&mut self) -> &mut Self {
        if let Ok(address) = env::var("STRUCT_LOG_TCP_ADDR") {
            self.address(address);
        }
        self
    }

    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    pub fn remote_level(&mut self, level: Level) -> &mut Self {
        self.remote_level = level;
        self
    }

    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    pub fn printer(&mut self, printer: Box<dyn Writer + Send + Sync + 'static>) -> &mut Self {
        self.printer = Some(printer);
        self
    }

    pub fn is_async(&mut self, is_async: bool) -> &mut Self {
        self.is_async = is_async;
        self
    }

    pub fn custom_format(
        &mut self,
        format: fn(&LogEntry) -> Result<String, fmt::Error>,
    ) -> &mut Self {
        self.custom_format = Some(format);
        self
    }

    pub fn init(&mut self) {
        self.build();
    }

    pub fn build(&mut self) -> Arc<DiemLogger> {
        let filter = {
            let local_filter = {
                let mut filter_builder = Filter::builder();

                if env::var(RUST_LOG).is_ok() {
                    filter_builder.with_env(RUST_LOG);
                } else {
                    filter_builder.filter_level(self.level.into());
                }

                filter_builder.build()
            };
            let remote_filter = {
                let mut filter_builder = Filter::builder();

                if self.is_async && self.address.is_some() {
                    if env::var(RUST_LOG_REMOTE).is_ok() {
                        filter_builder.with_env(RUST_LOG_REMOTE);
                    } else if env::var(RUST_LOG).is_ok() {
                        filter_builder.with_env(RUST_LOG);
                    } else {
                        filter_builder.filter_level(self.remote_level.into());
                    }
                } else {
                    filter_builder.filter_level(LevelFilter::Off);
                }

                filter_builder.build()
            };

            DiemFilter {
                local_filter,
                remote_filter,
            }
        };

        let logger = if self.is_async {
            let (sender, receiver) = mpsc::sync_channel(self.channel_size);
            let logger = Arc::new(DiemLogger {
                sender: Some(sender),
                printer: None,
                filter: RwLock::new(filter),
                formatter: self.custom_format.take().unwrap_or(default_format),
            });
            let service = LoggerService {
                receiver,
                address: self.address.clone(),
                printer: self.printer.take(),
                facade: logger.clone(),
            };

            thread::spawn(move || service.run());
            logger
        } else {
            Arc::new(DiemLogger {
                sender: None,
                printer: self.printer.take(),
                filter: RwLock::new(filter),
                formatter: self.custom_format.take().unwrap_or(default_format),
            })
        };

        crate::logger::set_global_logger(logger.clone());
        logger
    }
}

/// A combination of `Filter`s to control where logs are written
struct DiemFilter {
    /// The local printer `Filter` to control what is logged in text output
    local_filter: Filter,
    /// The remote logging `Filter` to control what is sent to external logging
    remote_filter: Filter,
}

impl DiemFilter {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.local_filter.enabled(metadata) || self.remote_filter.enabled(metadata)
    }
}

pub struct DiemLogger {
    sender: Option<SyncSender<LoggerServiceEvent>>,
    printer: Option<Box<dyn Writer>>,
    filter: RwLock<DiemFilter>,
    pub(crate) formatter: fn(&LogEntry) -> Result<String, fmt::Error>,
}

impl DiemLogger {
    pub fn builder() -> DiemLoggerBuilder {
        DiemLoggerBuilder::new()
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> DiemLoggerBuilder {
        Self::builder()
    }

    pub fn init_for_testing() {
        if env::var(RUST_LOG).is_err() {
            return;
        }

        Self::builder()
            .is_async(false)
            .printer(Box::new(StderrWriter))
            .build();
    }

    pub fn set_filter(&self, filter: Filter) {
        self.filter.write().local_filter = filter;
    }

    pub fn set_remote_filter(&self, filter: Filter) {
        self.filter.write().remote_filter = filter;
    }

    fn send_entry(&self, entry: LogEntry) {
        if let Some(printer) = &self.printer {
            let s = (self.formatter)(&entry).expect("Unable to format");
            printer.write(s);
        }

        if let Some(sender) = &self.sender {
            if let Err(e) = sender.try_send(LoggerServiceEvent::LogEntry(entry)) {
                STRUCT_LOG_QUEUE_ERROR_COUNT.inc();
                eprintln!("Failed to send structured log: {}", e);
            }
        }
    }
}

impl Logger for DiemLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.read().enabled(metadata)
    }

    fn record(&self, event: &Event) {
        let entry = LogEntry::new(event, ::std::thread::current().name());

        self.send_entry(entry)
    }

    fn flush(&self) {
        if let Some(sender) = &self.sender {
            let (oneshot_sender, oneshot_receiver) = mpsc::sync_channel(1);
            sender
                .send(LoggerServiceEvent::Flush(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
    }
}
#[allow(clippy::large_enum_variant)]
enum LoggerServiceEvent {
    LogEntry(LogEntry),
    Flush(SyncSender<()>),
}

/// A service for running a log listener, that will continually export logs through a local printer
/// or to a `DiemLogger` for external logging.
struct LoggerService {
    receiver: Receiver<LoggerServiceEvent>,
    address: Option<String>,
    printer: Option<Box<dyn Writer>>,
    facade: Arc<DiemLogger>,
}

impl LoggerService {
    pub fn run(mut self) {
        let mut writer = self.address.take().map(TcpWriter::new);

        for event in self.receiver {
            match event {
                LoggerServiceEvent::LogEntry(entry) => {
                    PROCESSED_STRUCT_LOG_COUNT.inc();

                    if let Some(printer) = &self.printer {
                        if self
                            .facade
                            .filter
                            .read()
                            .local_filter
                            .enabled(&entry.metadata)
                        {
                            let s = (self.facade.formatter)(&entry).expect("Unable to format");
                            printer.write(s)
                        }
                    }

                    if let Some(writer) = &mut writer {
                        if self
                            .facade
                            .filter
                            .read()
                            .remote_filter
                            .enabled(&entry.metadata)
                        {
                            Self::write_to_logstash(writer, entry);
                        }
                    }
                }
                LoggerServiceEvent::Flush(sender) => {
                    // This is just to notify the other side, the logger doesn't actually care if
                    // the listener is still listening
                    let _ = sender.send(());
                }
            }
        }
    }

    /// Writes a log line into json_lines logstash format, which has a newline at the end
    fn write_to_logstash(stream: &mut TcpWriter, mut entry: LogEntry) {
        // XXX Temporary hack to ensure that log lines don't show up empty in kibana when the
        // "message" field isn't set.
        if entry.message.is_none() {
            entry.message = Some(serde_json::to_string(&entry.data).unwrap());
        }

        let message = if let Ok(json) = serde_json::to_string(&entry) {
            json
        } else {
            STRUCT_LOG_PARSE_ERROR_COUNT.inc();
            return;
        };

        let message = message + "\n";
        let bytes = message.as_bytes();
        let message_length = bytes.len();

        // Attempt to write the log up to NUM_SEND_RETRIES + 1, and then drop it
        // Each `write_all` call will attempt to open a connection if one isn't open
        let mut result = stream.write_all(bytes);
        for _ in 0..NUM_SEND_RETRIES {
            if result.is_ok() {
                break;
            } else {
                result = stream.write_all(bytes);
            }
        }

        if let Err(e) = result {
            STRUCT_LOG_SEND_ERROR_COUNT.inc();
            eprintln!(
                "[Logging] Error while sending data to logstash({}): {}",
                stream.endpoint(),
                e
            );
        } else {
            SENT_STRUCT_LOG_COUNT.inc();
            SENT_STRUCT_LOG_BYTES.inc_by(message_length as u64);
        }
    }
}

/// An trait encapsulating the operations required for writing logs.
pub trait Writer: Send + Sync {
    /// Write the log.
    fn write(&self, log: String);
}

/// A struct for writing logs to stderr
struct StderrWriter;

impl Writer for StderrWriter {
    /// Write log to stderr
    fn write(&self, log: String) {
        eprintln!("{}", log);
    }
}

/// A struct for writing logs to a file
pub struct FileWriter {
    log_file: RwLock<std::fs::File>,
}

impl FileWriter {
    pub fn new(log_file: std::path::PathBuf) -> Self {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Unable to open log file");
        Self {
            log_file: RwLock::new(file),
        }
    }
}

impl Writer for FileWriter {
    /// Write to file
    fn write(&self, log: String) {
        if let Err(err) = writeln!(self.log_file.write(), "{}", log) {
            eprintln!("Unable to write to log file: {}", err);
        }
    }
}

/// Converts a record into a string representation:
/// UNIX_TIMESTAMP LOG_LEVEL [thread_name] FILE:LINE MESSAGE JSON_DATA
/// Example:
/// 2020-03-07 05:03:03 INFO [thread_name] common/diem-logger/src/lib.rs:261 Hello { "world": true }
fn default_format(entry: &LogEntry) -> Result<String, fmt::Error> {
    use std::fmt::Write;

    let mut w = String::new();
    write!(w, "{}", entry.timestamp)?;

    if let Some(thread_name) = &entry.thread_name {
        write!(w, " [{}]", thread_name)?;
    }

    write!(
        w,
        " {} {}",
        entry.metadata.level(),
        entry.metadata.location()
    )?;

    if let Some(message) = &entry.message {
        write!(w, " {}", message)?;
    }

    if !entry.data.is_empty() {
        write!(w, " {}", serde_json::to_string(&entry.data).unwrap())?;
    }

    Ok(w)
}

#[cfg(test)]
mod tests {
    use super::LogEntry;
    use crate::{
        debug, error, info, logger::Logger, trace, warn, Event, Key, KeyValue, Level, Metadata,
        Schema, Value, Visitor,
    };
    use chrono::{DateTime, Utc};
    use serde_json::Value as JsonValue;
    use std::{
        sync::{
            mpsc::{self, Receiver, SyncSender},
            Arc,
        },
        thread,
    };

    #[derive(serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    enum Enum {
        FooBar,
    }

    struct TestSchema<'a> {
        foo: usize,
        bar: &'a Enum,
    }

    impl Schema for TestSchema<'_> {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_pair(Key::new("foo"), Value::from_serde(&self.foo));
            visitor.visit_pair(Key::new("bar"), Value::from_serde(&self.bar));
        }
    }

    struct LogStream(SyncSender<LogEntry>);

    impl LogStream {
        fn new() -> (Self, Receiver<LogEntry>) {
            let (sender, receiver) = mpsc::sync_channel(1024);
            (Self(sender), receiver)
        }
    }

    impl Logger for LogStream {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Debug
        }

        fn record(&self, event: &Event) {
            let entry = LogEntry::new(event, ::std::thread::current().name());
            self.0.send(entry).unwrap();
        }

        fn flush(&self) {}
    }

    fn set_test_logger() -> Receiver<LogEntry> {
        let (logger, receiver) = LogStream::new();
        let logger = Arc::new(logger);
        crate::logger::set_global_logger(logger);
        receiver
    }

    // TODO: Find a better mechanism for testing that allows setting the logger not globally
    #[test]
    fn basic() {
        let receiver = set_test_logger();
        let number = 12345;

        // Send an info log
        let before = Utc::now();
        info!(
            TestSchema {
                foo: 5,
                bar: &Enum::FooBar
            },
            test = true,
            category = "name",
            KeyValue::new("display", Value::from_display(&number)),
            "This is a log"
        );
        let after = Utc::now();

        let entry = receiver.recv().unwrap();

        // Ensure standard fields are filled
        assert_eq!(entry.metadata.level(), Level::Info);
        assert_eq!(
            entry.metadata.target(),
            module_path!().split("::").next().unwrap()
        );
        assert_eq!(entry.metadata.module_path(), module_path!());
        assert_eq!(entry.metadata.file(), file!());
        assert_eq!(entry.message.as_deref(), Some("This is a log"));
        assert!(entry.backtrace.is_none());

        // Log time should be the time the structured log entry was created
        let timestamp = DateTime::parse_from_rfc3339(&entry.timestamp).unwrap();
        let timestamp: DateTime<Utc> = DateTime::from(timestamp);
        assert!(before <= timestamp && timestamp <= after);

        // Ensure data stored is the right type
        assert_eq!(entry.data.get("foo").and_then(JsonValue::as_u64), Some(5));
        assert_eq!(
            entry.data.get("bar").and_then(JsonValue::as_str),
            Some("foo_bar")
        );
        assert_eq!(
            entry.data.get("display").and_then(JsonValue::as_str),
            Some(format!("{}", number)).as_deref(),
        );
        assert_eq!(
            entry.data.get("test").and_then(JsonValue::as_bool),
            Some(true),
        );
        assert_eq!(
            entry.data.get("category").and_then(JsonValue::as_str),
            Some("name"),
        );

        // Test error logs contain backtraces
        error!("This is an error log");
        let entry = receiver.recv().unwrap();
        assert!(entry.backtrace.is_some());

        // Test all log levels work properly
        // Tracing should be skipped because the Logger was setup to skip Tracing events
        trace!("trace");
        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");

        let levels = &[Level::Debug, Level::Info, Level::Warn, Level::Error];

        for level in levels {
            let entry = receiver.recv().unwrap();
            assert_eq!(entry.metadata.level(), *level);
        }

        // Verify that the thread name is properly included
        let handler = thread::Builder::new()
            .name("named thread".into())
            .spawn(|| info!("thread"))
            .unwrap();

        handler.join().unwrap();
        let entry = receiver.recv().unwrap();
        assert_eq!(entry.thread_name.as_deref(), Some("named thread"));

        // Test Debug and Display inputs
        let debug_struct = DebugStruct {};
        let display_struct = DisplayStruct {};

        error!(identifier = ?debug_struct, "Debug test");
        error!(identifier = ?debug_struct, other = "value", "Debug2 test");
        error!(identifier = %display_struct, "Display test");
        error!(identifier = %display_struct, other = "value", "Display2 test");
        error!("Literal" = ?debug_struct, "Debug test");
        error!("Literal" = ?debug_struct, other = "value", "Debug test");
        error!("Literal" = %display_struct, "Display test");
        error!("Literal" = %display_struct, other = "value", "Display2 test");
        error!("Literal" = %display_struct, other = "value", identifier = ?debug_struct, "Mixed test");
    }

    struct DebugStruct {}

    impl std::fmt::Debug for DebugStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "DebugStruct!")
        }
    }

    struct DisplayStruct {}

    impl std::fmt::Display for DisplayStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "DisplayStruct!")
        }
    }
}
