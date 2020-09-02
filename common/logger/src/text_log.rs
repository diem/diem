// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Event, Level, Metadata};
use chrono::Utc;

use std::{
    fmt,
    fmt::Write,
    sync::{
        mpsc::{self, Receiver, RecvError, SyncSender, TrySendError},
        Arc,
    },
    thread,
};

pub const CHANNEL_SIZE: usize = 256;
pub const DEFAULT_TARGET: &str = "libra";

/// Logging framework for Libra that encapsulates a minimal dependency logger with support for
/// environmental variable (RUST_LOG) and asynchronous logging.
/// Note: only a single logger can be instantiated at a time. Repeated instantiates of the loggers
/// will only adjust the global logging level but will not change the initial filter.
pub struct Logger {
    /// Channel size for sending logs to async handler.
    channel_size: usize,
    /// Only instantiate a logger if the environment is properly set
    environment_only: bool,
    /// Use a dedicated thread for logging.
    is_async: bool,
    /// The default logging level.
    level: Level,
    /// Override RUST_LOG even if set
    override_rust_log: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            channel_size: CHANNEL_SIZE,
            environment_only: false,
            is_async: false,
            level: Level::Info,
            override_rust_log: false,
        }
    }

    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    pub fn environment_only(&mut self, environment_only: bool) -> &mut Self {
        self.environment_only = environment_only;
        self
    }

    pub fn is_async(&mut self, is_async: bool) -> &mut Self {
        self.is_async = is_async;
        self
    }

    pub fn level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    pub fn override_rust_log(&mut self, override_rust_log: bool) -> &mut Self {
        self.override_rust_log = override_rust_log;
        self
    }

    pub fn init(&mut self) {
        self.internal_init(StderrWriter {});
    }

    fn internal_init<W: 'static + Writer>(&mut self, writer: W) {
        if self.environment_only && ::std::env::var("RUST_LOG").is_err() {
            return;
        }

        if self.is_async {
            let (sender, receiver) = mpsc::sync_channel(self.channel_size);

            let client = AsyncLogClient { sender };
            crate::logger::set_global_logger(Arc::new(client));

            let service = AsyncLogService { receiver, writer };

            thread::spawn(move || service.log_handler());
        } else {
            let logger = SyncLogger { writer };
            crate::logger::set_global_logger(Arc::new(logger));
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Provies the log::Log for Libra's synchronous logger
struct SyncLogger<W> {
    writer: W,
}

impl<W: Writer + 'static> crate::logger::Logger for SyncLogger<W> {
    /// Determines if a log message with the specified metadata would be logged.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() < Level::Trace
    }

    /// Logs the provided record but first evaluates the filters and then writes it.
    fn record(&self, record: &Event) {
        match format(record) {
            Ok(formatted) => self.writer.write(formatted),
            Err(e) => self
                .writer
                .write(format!("Unable to format log {:?}: {}", record, e)),
        };
    }
}

/// Operations between the AsyncLogClient and AsyncLogService
enum LogOp {
    /// Send a line to log to the writer.
    Log(String),
}

/// Provides the backend to the asynchronous interface for logging. This part actually writes the
/// log to the writer.
struct AsyncLogService<W> {
    receiver: Receiver<LogOp>,
    writer: W,
}

impl<W: Writer> AsyncLogService<W> {
    /// Loop for handling logging
    fn log_handler(mut self) {
        loop {
            if let Err(e) = self.handle_recv() {
                // TODO write an error record and then exit
                self.writer.write(format!("Unrecoverable error: {}", e));
                return;
            }
        }
    }

    fn handle_recv(&mut self) -> Result<(), RecvError> {
        match self.receiver.recv()? {
            LogOp::Log(line) => self.writer.write(line),
        };
        Ok(())
    }
}

/// Provides the log::Log interface for Libra's asynchronous logger
struct AsyncLogClient {
    sender: SyncSender<LogOp>,
}

impl crate::logger::Logger for AsyncLogClient {
    /// Determines if a log message with the specified metadata would be logged.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() < Level::Trace
    }

    /// Logs the provided record but first evaluates the filters and then sending it to the
    /// AsyncLogService via a SyncSender.
    fn record(&self, record: &Event) {
        let formatted = format(record)
            .unwrap_or_else(|e| format!("Unable to format log {:?} due to {}", record, e));
        if let Err(e) = self.sender.try_send(LogOp::Log(formatted)) {
            match e {
                TrySendError::Disconnected(_) => {
                    eprintln!("Unable to log {:?} due to {}", record, e)
                }
                TrySendError::Full(_) => eprintln!("Unable to log, queue full"),
            }
        };
    }
}

/// Converts a record into a string representation:
/// UNIX_TIMESTAMP LOG_LEVEL FILE:LINE MESSAGE
/// Example:
/// 2020-03-07 05:03:03 INFO common/libra-logger/src/lib.rs:261 Hello
fn format(record: &Event) -> Result<String, fmt::Error> {
    let mut buffer = String::new();

    write!(buffer, "{} ", record.metadata().level())?;
    write!(buffer, "{} ", Utc::now().format("%F %T"))?;

    write!(buffer, "{}", record.metadata().file())?;
    write!(buffer, ":{}", record.metadata().line())?;
    write!(buffer, " ")?;

    if let Some(message) = record.message() {
        write!(buffer, "{}", message)?;
    }

    Ok(buffer)
}

/// An trait encapsulating the operations required for writing logs.
trait Writer: Send + Sync {
    /// Write the log.
    fn write(&self, log: String);
}

/// A struct for writing logs to stderr
struct StderrWriter {}

impl Writer for StderrWriter {
    /// Write log to stderr
    fn write(&self, log: String) {
        eprintln!("{}", log);
    }
}

#[cfg(test)]
mod tests {
    use crate::{text_log::*, *};
    use std::sync::{Arc, RwLock};

    #[derive(Default)]
    struct VecWriter {
        logs: Arc<RwLock<Vec<String>>>,
    }

    impl Writer for VecWriter {
        fn write(&self, log: String) {
            self.logs.write().unwrap().push(log)
        }
    }

    #[test]
    fn verify_end_to_end() {
        let writer = VecWriter::default();
        let logs = writer.logs.clone();
        Logger::new().override_rust_log(true).internal_init(writer);

        assert_eq!(logs.read().unwrap().len(), 0);
        info!("Hello");
        assert_eq!(logs.read().unwrap().len(), 1);
        let string = logs.write().unwrap().remove(0);
        assert!(string.contains("INFO"));
        assert!(string.ends_with("Hello"));
    }
}
