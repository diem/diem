// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::{
        PROCESSED_STRUCT_LOG_COUNT, SENT_STRUCT_LOG_COUNT, STRUCT_LOG_PARSE_ERROR_COUNT,
        STRUCT_LOG_QUEUE_ERROR_COUNT, STRUCT_LOG_SEND_ERROR_COUNT,
    },
    logger::Logger,
    struct_log::TcpWriter,
    Event, Level, Metadata,
};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use std::{
    collections::HashMap,
    env, fmt,
    io::Write,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
};

pub const CHANNEL_SIZE: usize = 10000;
const NUM_SEND_RETRIES: u8 = 1;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    #[serde(flatten)]
    metadata: Metadata,
    timestamp: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    data: HashMap<&'static str, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl LogEntry {
    fn new(event: &Event) -> Self {
        use crate::{Key, Value, Visitor};

        struct JsonVisitor<'a>(&'a mut HashMap<&'static str, serde_json::Value>);

        impl<'a> Visitor for JsonVisitor<'a> {
            fn visit_pair(&mut self, key: Key, value: Value<'_>) {
                let v = match value {
                    Value::Debug(d) => serde_json::Value::String(format!("{:?}", d)),
                    Value::Display(d) => serde_json::Value::String(d.to_string()),
                    Value::Serde(s) => serde_json::to_value(s).unwrap(),
                };

                self.0.insert(key.as_str(), v);
            }
        }

        let metadata = *event.metadata();
        let message = event.message().map(fmt::format);

        let mut data = HashMap::new();
        for schema in event.keys_and_values() {
            schema.visit(&mut JsonVisitor(&mut data));
        }

        Self {
            metadata,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            data,
            message,
        }
    }
}

pub struct LibraLoggerBuilder {
    channel_size: usize,
    level: Level,
    address: Option<String>,
    printer: Option<Box<dyn Writer>>,
    is_async: bool,
}

impl LibraLoggerBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            channel_size: CHANNEL_SIZE,
            level: Level::Debug,
            address: None,
            printer: Some(Box::new(StderrWriter)),
            is_async: false,
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

    pub fn init(&mut self) {
        self.build()
    }

    pub fn build(&mut self) {
        let logger = if self.is_async {
            let (sender, receiver) = mpsc::sync_channel(self.channel_size);
            let service = LoggerService {
                receiver,
                address: self.address.clone(),
                printer: self.printer.take(),
            };
            let logger = Arc::new(LibraLogger {
                sender: Some(sender),
                printer: None,
                level: self.level,
            });

            thread::spawn(move || service.run());
            logger
        } else {
            Arc::new(LibraLogger {
                sender: None,
                printer: self.printer.take(),
                level: self.level,
            })
        };

        crate::logger::set_global_logger(logger);
    }
}

pub struct LibraLogger {
    sender: Option<SyncSender<LogEntry>>,
    printer: Option<Box<dyn Writer>>,
    level: Level,
}

impl LibraLogger {
    pub fn builder() -> LibraLoggerBuilder {
        LibraLoggerBuilder::new()
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> LibraLoggerBuilder {
        Self::builder()
    }

    pub fn init_for_testing() {
        if env::var("RUST_LOG").is_err() {
            return;
        }

        Self::builder()
            .is_async(false)
            .printer(Box::new(StderrWriter))
            .build()
    }

    fn send_entry(&self, entry: LogEntry) {
        if let Some(printer) = &self.printer {
            let s = format(&entry).expect("Unable to format");
            printer.write(s);
        }

        if let Some(sender) = &self.sender {
            if let Err(e) = sender.try_send(entry) {
                STRUCT_LOG_QUEUE_ERROR_COUNT.inc();
                eprintln!("Failed to send structured log: {}", e);
            }
        }
    }
}

impl Logger for LibraLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn record(&self, event: &Event) {
        let entry = LogEntry::new(event);

        self.send_entry(entry)
    }
}

struct LoggerService {
    receiver: Receiver<LogEntry>,
    address: Option<String>,
    printer: Option<Box<dyn Writer>>,
}

impl LoggerService {
    pub fn run(mut self) {
        let mut writer = self.address.take().map(TcpWriter::new);

        for entry in self.receiver {
            PROCESSED_STRUCT_LOG_COUNT.inc();

            if let Some(writer) = &mut writer {
                Self::write_to_logstash(writer, &entry);
            }

            if let Some(printer) = &self.printer {
                let s = format(&entry).expect("Unable to format");
                printer.write(s)
            }
        }
    }

    /// Writes a log line into json_lines logstash format, which has a newline at the end
    fn write_to_logstash(stream: &mut TcpWriter, entry: &LogEntry) {
        let message = if let Ok(json) = serde_json::to_string(entry) {
            json
        } else {
            STRUCT_LOG_PARSE_ERROR_COUNT.inc();
            return;
        };

        let message = message + "\n";
        let bytes = message.as_bytes();

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
            SENT_STRUCT_LOG_COUNT.inc()
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

/// Converts a record into a string representation:
/// UNIX_TIMESTAMP LOG_LEVEL FILE:LINE MESSAGE
/// Example:
/// 2020-03-07 05:03:03 INFO common/libra-logger/src/lib.rs:261 Hello
fn format(entry: &LogEntry) -> Result<String, fmt::Error> {
    use std::fmt::Write;

    let mut w = String::new();
    write!(
        w,
        "{} {} {}",
        entry.metadata.level(),
        entry.timestamp,
        entry.metadata.location()
    )?;

    if let Some(message) = &entry.message {
        write!(w, " {}", message)?;
    }

    Ok(w)
}
