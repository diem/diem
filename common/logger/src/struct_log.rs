// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    counters::{
        PROCESSED_STRUCT_LOG_COUNT, SENT_STRUCT_LOG_COUNT, STRUCT_LOG_CONNECT_ERROR_COUNT,
        STRUCT_LOG_PARSE_ERROR_COUNT, STRUCT_LOG_QUEUE_ERROR_COUNT, STRUCT_LOG_SEND_ERROR_COUNT,
        STRUCT_LOG_TCP_CONNECT_COUNT,
    },
    LogLevel,
};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{File, OpenOptions},
    io,
    io::Write as IoWrite,
    marker::PhantomData,
    net::{TcpStream, ToSocketAddrs},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Duration,
};

pub trait StructLogSink: Sync {
    fn send(&self, entry: StructuredLogEntry);
}

// This is poor's man AtomicReference from crossbeam
// It have few unsafe lines, but does not require extra dependency
static NOP: NopStructLog = NopStructLog {};
static mut STRUCT_LOGGER: &'static dyn StructLogSink = &NOP;
static STRUCT_LOGGER_STATE: AtomicUsize = AtomicUsize::new(UNINITIALIZED);
const UNINITIALIZED: usize = 0;

const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

// Size configurations
const MAX_LOG_LINE_SIZE: usize = 10240; // 10KiB
const LOG_INFO_OFFSET: usize = 256;
const WRITE_CHANNEL_SIZE: usize = 1024;
const WRITE_TIMEOUT_MS: u64 = 2000;
const CONNECTION_TIMEOUT_MS: u64 = 5000;

// Fields to keep when over size
static FIELDS_TO_KEEP: &[&str] = &["error"];

#[derive(Default, Serialize)]
pub struct StructuredLogEntry {
    /// log message set by macros like info!
    #[serde(skip_serializing_if = "Option::is_none")]
    log: Option<String>,
    /// description of the log
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<&'static str>,
    /// category of the event
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'static str>,
    /// name of the event
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'static str>,
    /// rust module (e.g. consensus::round_manager)
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<&'static str>,
    /// filename + line (e.g. consensus/src/round_manager.rs:678)
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<&'static str>,
    /// git revision
    #[serde(skip_serializing_if = "Option::is_none")]
    git_rev: Option<&'static str>,
    /// time of the log
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    /// warning or critical TODO: Remove once alarms are migrated (https://github.com/libra/libra/issues/5484)
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<LogLevel>,
    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<LogLevel>,
    /// arbitrary data that can be logged
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    data: HashMap<&'static str, Value>,
}

#[must_use = "use send_struct_log! macro to send structured log"]
impl StructuredLogEntry {
    pub fn new_unnamed() -> Self {
        let mut ret = Self::default();
        ret.timestamp = Some(Utc::now().format("%F %T").to_string());
        ret
    }

    pub fn new_named(category: &'static str, name: &'static str) -> Self {
        let mut ret = Self::default();
        ret.category = Some(category);
        ret.name = Some(name);
        ret.timestamp = Some(Utc::now().format("%F %T").to_string());
        ret
    }

    fn clone_without_data(&self) -> Self {
        Self {
            log: self.log.clone(),
            pattern: self.pattern,
            category: self.category,
            name: self.name,
            module: self.module,
            location: self.location,
            git_rev: self.git_rev,
            timestamp: self.timestamp.clone(),
            severity: self.severity,
            level: self.level,
            data: HashMap::new(),
        }
    }

    fn to_json(&self) -> Result<Value, serde_json::Error> {
        let value = serde_json::to_value(self);
        if let Err(e) = &value {
            log::error!("[Logging] Failed to serialize struct log entry: {}", e);
        }
        value
    }

    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        let json = self.to_json()?;

        // If the message is too long, let's truncate the message and leave helpful info
        let mut json_string = json.to_string();
        if json_string.len() > MAX_LOG_LINE_SIZE {
            log::error!(
                "[Logging] Structured log is too long, shrinking below : {}",
                MAX_LOG_LINE_SIZE
            );

            let mut entry = self.clone_without_data();
            if let Some(mut log) = entry.log {
                log.truncate(MAX_LOG_LINE_SIZE - LOG_INFO_OFFSET);
                // Leave 128 bytes for all the other info
                entry.log = Some(log);
            }
            entry = entry
                .data(
                    "StructLogError",
                    format!("Message exceeded MAX_LOG_LINE_SIZE {}", MAX_LOG_LINE_SIZE),
                )
                .data("OriginalLength", json_string.len());

            // Keep important values around
            // TODO: We should work on decreasing log sizes, as this isn't sustainable
            for field in FIELDS_TO_KEEP {
                if let Some(value) = self.data.get(field) {
                    entry = entry.json_data(field, value.clone());
                }
            }

            json_string = entry.to_json()?.to_string();
        }

        Ok(json_string)
    }

    /// Sets the log level of the log
    pub fn level(mut self, level: LogLevel) -> Self {
        self.severity = Some(level);
        self.level = Some(level);
        self
    }

    // TODO: Remove in favor of level (https://github.com/libra/libra/issues/5484)
    pub fn critical(self) -> Self {
        self.level(crate::LogLevel::Critical)
    }

    pub fn warning(self) -> Self {
        self.level(crate::LogLevel::Warning)
    }

    pub fn json_data(mut self, key: &'static str, value: Value) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn data<D: Serialize>(mut self, key: &'static str, value: D) -> Self {
        self.data.insert(
            key,
            serde_json::to_value(value).expect("Failed to serialize StructuredLogEntry key"),
        );
        self
    }

    /// Used for errors and other types that don't serialize well
    pub fn data_display<D: Display>(self, key: &'static str, value: D) -> Self {
        self.data(key, value.to_string())
    }

    pub fn field<D: Serialize>(self, field: &LoggingField<D>, value: D) -> Self {
        self.data(field.0, value)
    }

    #[doc(hidden)] // set from macro
    pub fn log(&mut self, log: String) -> &mut Self {
        self.log = Some(log);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn pattern(&mut self, pattern: &'static str) -> &mut Self {
        self.pattern = Some(pattern);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn module(&mut self, module: &'static str) -> &mut Self {
        self.module = Some(module);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn location(&mut self, location: &'static str) -> &mut Self {
        self.location = Some(location);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn git_rev(&mut self, git_rev: Option<&'static str>) -> &mut Self {
        self.git_rev = git_rev;
        self
    }

    #[doc(hidden)] // set from macro
    pub fn data_mutref<D: Serialize>(&mut self, key: &'static str, value: D) -> &mut Self {
        self.data.insert(
            key,
            serde_json::to_value(value).expect("Failed to serialize StructuredLogEntry key"),
        );
        self
    }

    // Use send_struct_log! macro instead of this method to populate extra meta information such as git rev and module name
    #[doc(hidden)]
    pub fn send(self) {
        struct_logger().send(self);
    }
}

/// Field is similar to .data but restricts type of the value to a specific type.
///
/// Example:
///
/// mod logging {
///    pub const MY_FIELD:LoggingField<u64> = LoggingField::new("my_field");
/// }
///
/// mod my_code {
///    fn my_fn() {
///        send_struct_log!(StructuredLogEntry::new(...).field(&logging::MY_FIELD, 0))
///    }
/// }
pub struct LoggingField<D>(&'static str, PhantomData<D>);

impl<D> LoggingField<D> {
    pub const fn new(name: &'static str) -> Self {
        Self(name, PhantomData)
    }
}

// This is exact copy of similar function in log crate
/// Sets structured logger
pub fn set_struct_logger(logger: &'static dyn StructLogSink) -> Result<(), InitLoggerError> {
    unsafe {
        match STRUCT_LOGGER_STATE.compare_and_swap(UNINITIALIZED, INITIALIZING, Ordering::SeqCst) {
            UNINITIALIZED => {
                STRUCT_LOGGER = logger;
                STRUCT_LOGGER_STATE.store(INITIALIZED, Ordering::SeqCst);
                Ok(())
            }
            INITIALIZING => {
                while STRUCT_LOGGER_STATE.load(Ordering::SeqCst) == INITIALIZING {}
                Err(InitLoggerError::StructLoggerAlreadySet)
            }
            _ => Err(InitLoggerError::StructLoggerAlreadySet),
        }
    }
}

static STRUCT_LOG_LEVEL: Lazy<log::Level> = Lazy::new(|| {
    let level = env::var("STRUCT_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    log::Level::from_str(&level).expect("Failed to parse log level")
});

/// Checks if structured logging is enabled for level
pub fn struct_logger_enabled(level: log::Level) -> bool {
    struct_logger_set() && level <= *STRUCT_LOG_LEVEL
}

/// Checks if structured logging is enabled
pub fn struct_logger_set() -> bool {
    STRUCT_LOGGER_STATE.load(Ordering::SeqCst) == INITIALIZED
}

/// Initializes struct logger from STRUCT_LOG_FILE env var.
/// If STRUCT_LOG_FILE is set, STRUCT_LOG_TCP_ADDR will be ignored.
/// Can only be called once
pub fn init_struct_log_from_env() -> Result<(), InitLoggerError> {
    if let Ok(file) = env::var("STRUCT_LOG_FILE") {
        init_file_struct_log(file)
    } else if let Ok(address) = env::var("STRUCT_LOG_TCP_ADDR") {
        init_tcp_struct_log(address)
    } else if let Ok(address) = env::var("STRUCT_LOG_UDP_ADDR") {
        // Remove once all usages of STRUCT_LOG_UDP_ADDR are transferred over
        init_tcp_struct_log(address)
    } else {
        Ok(())
    }
}

/// Initializes struct logger sink that writes to specified file.
/// Can only be called once
pub fn init_file_struct_log(file_path: String) -> Result<(), InitLoggerError> {
    let logger = FileStructLog::start_new(file_path).map_err(InitLoggerError::IoError)?;
    let logger = Box::leak(Box::new(logger));
    set_struct_logger(logger)
}

/// Initializes struct logger sink that stream logs through TCP protocol.
/// Can only be called once
pub fn init_tcp_struct_log(address: String) -> Result<(), InitLoggerError> {
    let logger = TCPStructLog::start_new(address).map_err(InitLoggerError::IoError)?;
    let logger = Box::leak(Box::new(logger));
    set_struct_logger(logger)
}

/// Initialize struct logger sink that prints all structured logs to stdout
/// Can only be called once
pub fn init_println_struct_log() -> Result<(), InitLoggerError> {
    let logger = PrintStructLog {};
    let logger = Box::leak(Box::new(logger));
    set_struct_logger(logger)
}

#[derive(Debug)]
pub enum InitLoggerError {
    IoError(io::Error),
    StructLoggerAlreadySet,
}

// This is exact copy of similar function in log crate
fn struct_logger() -> &'static dyn StructLogSink {
    unsafe {
        if STRUCT_LOGGER_STATE.load(Ordering::SeqCst) != INITIALIZED {
            &NOP
        } else {
            STRUCT_LOGGER
        }
    }
}

struct NopStructLog {}

impl StructLogSink for NopStructLog {
    fn send(&self, _entry: StructuredLogEntry) {}
}

struct PrintStructLog {}

impl StructLogSink for PrintStructLog {
    fn send(&self, entry: StructuredLogEntry) {
        println!("{}", serde_json::to_string(&entry).unwrap());
    }
}

/// Sink that prints all structured logs to specified file
struct FileStructLog {
    sender: SyncSender<StructuredLogEntry>,
}

impl FileStructLog {
    /// Creates new FileStructLog and starts async thread to write results
    pub fn start_new(file_path: String) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(WRITE_CHANNEL_SIZE);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)?;
        let sink_thread = FileStructLogThread { receiver, file };
        thread::spawn(move || sink_thread.run());
        Ok(Self { sender })
    }
}

impl StructLogSink for FileStructLog {
    fn send(&self, entry: StructuredLogEntry) {
        if let Err(e) = self.sender.try_send(entry) {
            // Use log crate macro to avoid generation of structured log in this case
            // Otherwise we will have infinite loop
            log::error!("Failed to send structured log: {}", e);
        }
    }
}

struct FileStructLogThread {
    receiver: Receiver<StructuredLogEntry>,
    file: File,
}

impl FileStructLogThread {
    pub fn run(mut self) {
        for entry in self.receiver {
            let json_string = match entry.to_json_string() {
                Ok(json_string) => json_string,
                Err(_) => continue,
            };
            if let Err(e) = writeln!(&mut self.file, "{}", json_string) {
                log::error!("Failed to write struct log entry: {}", e);
            }
        }
    }
}

/// Sink that streams all structured logs to an address through TCP protocol
struct TCPStructLog {
    sender: SyncSender<StructuredLogEntry>,
}

impl TCPStructLog {
    pub fn start_new(address: String) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(WRITE_CHANNEL_SIZE);
        let sink_thread = TCPStructLogThread::new(receiver, address);
        thread::spawn(move || sink_thread.run());
        Ok(Self { sender })
    }
}

impl StructLogSink for TCPStructLog {
    fn send(&self, entry: StructuredLogEntry) {
        if let Err(e) = self.sender.try_send(entry) {
            STRUCT_LOG_QUEUE_ERROR_COUNT.inc();
            // Use log crate macro to avoid generation of structured log in this case
            // Otherwise we will have infinite loop
            log::error!("[Logging] Failed to send structured log: {}", e);
        }
    }
}

struct TCPStructLogThread {
    receiver: Receiver<StructuredLogEntry>,
    address: String,
}

impl TCPStructLogThread {
    fn new(receiver: Receiver<StructuredLogEntry>, address: String) -> TCPStructLogThread {
        TCPStructLogThread { receiver, address }
    }

    // Continually iterate over requests unless writing fails.
    fn process_requests(&self, stream: &mut TcpStream) {
        for entry in &self.receiver {
            PROCESSED_STRUCT_LOG_COUNT.inc();
            // Parse string, skipping over anything that can't be parsed
            let json_string = match entry.to_json_string() {
                Ok(json_string) => json_string,
                Err(_) => {
                    STRUCT_LOG_PARSE_ERROR_COUNT.inc();
                    continue;
                }
            };

            // If we fail to write, exit out and create a new stream
            if let Err(e) = Self::write_log_line(stream, json_string) {
                STRUCT_LOG_SEND_ERROR_COUNT.inc();
                log::error!(
                    "[Logging] Error while sending data to logstash({}): {}",
                    self.address,
                    e
                );

                // Start over stream
                return;
            } else {
                SENT_STRUCT_LOG_COUNT.inc()
            }
        }
    }

    pub fn connect(&mut self) -> io::Result<TcpStream> {
        // resolve addresses to handle DNS names
        let mut last_error = io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to resolve and connect to {}", self.address),
        );
        for addr in self.address.to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(CONNECTION_TIMEOUT_MS)) {
                Ok(stream) => return Ok(stream),
                Err(err) => last_error = err,
            }
        }

        Err(last_error)
    }

    /// Writes a log line into json_lines logstash format, which has a newline at the end
    fn write_log_line(stream: &mut TcpStream, message: String) -> io::Result<()> {
        let message = message + "\n";
        stream.write_all(message.as_bytes())
    }

    fn write_control_msg(stream: &mut TcpStream, msg: &'static str) -> io::Result<()> {
        let entry = StructuredLogEntry::new_named("logger", msg);
        let entry = entry
            .to_json_string()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        Self::write_log_line(stream, entry)
    }

    pub fn run(mut self) {
        loop {
            let mut maybe_stream = self.connect();
            STRUCT_LOG_TCP_CONNECT_COUNT.inc();

            // This is to ensure that we do actually connect before sending requests
            // If the request process loop ends, the stream is broken.  Reset and create a new one.
            match maybe_stream.as_mut() {
                Ok(mut stream) => {
                    // Set the write timeout
                    if let Err(err) =
                        stream.set_write_timeout(Some(Duration::from_millis(WRITE_TIMEOUT_MS)))
                    {
                        STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                        log::error!("[Logging] Failed to set write timeout: {}", err);
                        continue;
                    }

                    // Write a log signifying that the logger connected, and test the stream
                    if let Err(err) = Self::write_control_msg(stream, "connected") {
                        STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                        log::error!("[Logging] control message failed: {}", err);
                        continue;
                    }

                    self.process_requests(&mut stream)
                }
                Err(err) => {
                    STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                    log::error!(
                        "[Logging] Failed to connect to {}, cause {}",
                        self.address,
                        err
                    )
                }
            }
        }
    }
}
