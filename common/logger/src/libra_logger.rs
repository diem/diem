// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::{
        PROCESSED_STRUCT_LOG_COUNT, SENT_STRUCT_LOG_COUNT, STRUCT_LOG_PARSE_ERROR_COUNT,
        STRUCT_LOG_QUEUE_ERROR_COUNT, STRUCT_LOG_SEND_ERROR_COUNT,
    },
    struct_log::TcpWriter,
    Level, StructuredLogEntry,
};
use std::{
    env, fmt,
    io::Write,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
};

pub const CHANNEL_SIZE: usize = 10000; // MAX_LOG_LINE_SIZE * 10000 = max buffer size
const NUM_SEND_RETRIES: u8 = 1;

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
    sender: Option<SyncSender<StructuredLogEntry>>,
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

    fn send_entry(&self, entry: StructuredLogEntry) {
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

impl crate::logger::Logger for LibraLogger {
    fn enabled(&self, metadata: &crate::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn record(&self, event: &crate::Event) {
        let mut entry = StructuredLogEntry::default()
            .level(event.metadata().level())
            .schemas(event.keys_and_values());
        entry
            .add_category(event.metadata().target())
            .add_module(event.metadata().module_path())
            .add_location(event.metadata().location());

        if let Some(message) = event.message() {
            entry = entry.message(message.to_string());
        }

        self.send_entry(entry)
    }
}

struct LoggerService {
    receiver: Receiver<StructuredLogEntry>,
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
    fn write_to_logstash(stream: &mut TcpWriter, entry: &StructuredLogEntry) {
        let message = if let Ok(json) = entry.to_json_string() {
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
fn format(entry: &StructuredLogEntry) -> Result<String, fmt::Error> {
    use std::fmt::Write;

    let mut w = String::new();
    if let Some(level) = &entry.level {
        write!(w, "{} ", level)?;
    }

    write!(w, "{} ", entry.timestamp)?;

    if let Some(location) = &entry.location {
        write!(w, "{} ", location)?;
    }

    if let Some(message) = &entry.message {
        write!(w, "{}", message)?;
    }

    Ok(w)
}
