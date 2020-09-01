// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::counters::{
    PROCESSED_STRUCT_LOG_COUNT, SENT_STRUCT_LOG_COUNT, STRUCT_LOG_CONNECT_ERROR_COUNT,
    STRUCT_LOG_PARSE_ERROR_COUNT, STRUCT_LOG_QUEUE_ERROR_COUNT, STRUCT_LOG_SEND_ERROR_COUNT,
    STRUCT_LOG_TCP_CONNECT_COUNT,
};
use chrono::{SecondsFormat, Utc};
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{File, OpenOptions},
    io,
    io::Write,
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
const WRITE_CHANNEL_SIZE: usize = 10000; // MAX_LOG_LINE_SIZE * 10000 = max buffer size
const WRITE_TIMEOUT_MS: u64 = 2000;
const CONNECTION_TIMEOUT_MS: u64 = 5000;
const NUM_SEND_RETRIES: u8 = 1;

// Fields to keep when over size
static FIELDS_TO_KEEP: &[&str] = &["error"];

#[derive(Debug, Serialize)]
pub struct StructuredLogEntry {
    /// log message set by macros like info!
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
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
    /// time of the log
    timestamp: String,
    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<log::Level>,
    /// arbitrary data that can be logged
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    data: HashMap<&'static str, Value>,
}

impl Default for StructuredLogEntry {
    fn default() -> Self {
        Self {
            message: None,
            pattern: None,
            category: None,
            name: None,
            module: None,
            location: None,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            level: None,
            data: Default::default(),
        }
    }
}

impl StructuredLogEntry {
    /// Specifically for text based conversion logs
    pub fn new_text() -> Self {
        let mut ret = Self::default();
        ret.category = Some("text");
        ret
    }

    /// Creates a log with a category and a name.  This should be preferred
    pub fn new_named(category: &'static str, name: &'static str) -> Self {
        let mut ret = Self::default();
        ret.category = Some(category);
        ret.name = Some(name);
        ret
    }

    fn clone_without_data(&self) -> Self {
        Self {
            message: self.message.clone(),
            pattern: self.pattern,
            category: self.category,
            name: self.name,
            module: self.module,
            location: self.location,
            timestamp: self.timestamp.clone(),
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
            if let Some(mut message) = entry.message {
                message.truncate(MAX_LOG_LINE_SIZE - LOG_INFO_OFFSET);
                // Leave 128 bytes for all the other info
                entry.message = Some(message);
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
    pub fn level(mut self, level: log::Level) -> Self {
        self.level = Some(level);
        self
    }

    pub(crate) fn json_data(mut self, key: &'static str, value: Value) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn schema<S: crate::Schema>(mut self, schema: S) -> Self {
        struct JsonVisitor<'a>(&'a mut HashMap<&'static str, serde_json::Value>);

        impl<'a> crate::Visitor for JsonVisitor<'a> {
            fn visit_pair(&mut self, key: crate::Key, value: crate::Value<'_>) {
                use crate::Value;

                let v = match value {
                    Value::Debug(d) => serde_json::Value::String(format!("{:?}", d)),
                    Value::Display(d) => serde_json::Value::String(d.to_string()),
                    Value::Serde(s) => serde_json::to_value(s).unwrap(),
                };

                self.0.insert(key.as_str(), v);
            }
        }

        schema.visit(&mut JsonVisitor(&mut self.data));

        self
    }

    /// Add a data field, with a given value, will serialize into JSON and be indexed accordingly
    pub fn data<D: Serialize>(self, key: &'static str, value: D) -> Self {
        self.json_data(
            key,
            serde_json::to_value(value).expect("Failed to serialize StructuredLogEntry key"),
        )
    }

    /// Used for errors and other types that don't serialize well
    pub fn data_display<D: Display>(self, key: &'static str, value: D) -> Self {
        self.data(key, value.to_string())
    }

    /// Add a typed data field to serialize, to ensure type and name consistency
    pub fn field<D: Serialize>(self, field: &LoggingField<D>, value: D) -> Self {
        self.data(field.0, value)
    }

    /// Sets the context log line used for text logs.  This is useful for migration of text logs
    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_message(&mut self, message: String) -> &mut Self {
        self.message = Some(message);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_category(&mut self, target: &'static str) -> &mut Self {
        if self.category.is_none() {
            self.category = Some(target);
        }
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_pattern(&mut self, pattern: &'static str) -> &mut Self {
        self.pattern = Some(pattern);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_module(&mut self, module: &'static str) -> &mut Self {
        self.module = Some(module);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_location(&mut self, location: &'static str) -> &mut Self {
        self.location = Some(location);
        self
    }

    #[doc(hidden)] // set from macro
    pub fn add_data<D: Serialize>(&mut self, key: &'static str, value: D) -> &mut Self {
        self.data.insert(
            key,
            serde_json::to_value(value).expect("Failed to serialize StructuredLogEntry key"),
        );
        self
    }

    // Use sl_level! macros instead of this method to populate extra meta information such as git rev and module name
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
///        sl_info!(StructuredLogEntry::new(...).field(&logging::MY_FIELD, 0))
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
    sender: SyncSender<String>,
}

impl TCPStructLog {
    pub fn start_new(endpoint: String) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(WRITE_CHANNEL_SIZE);
        let sink_thread = TCPStructLogThread::new(receiver, endpoint);
        thread::spawn(move || sink_thread.run());
        Ok(Self { sender })
    }
}

impl StructLogSink for TCPStructLog {
    fn send(&self, entry: StructuredLogEntry) {
        // Convert and trim before submitting to queue to ensure set log size
        // TODO: Will this have an impact on performance of the writing thread from serialization?
        if let Ok(json) = entry.to_json_string() {
            if let Err(e) = self.sender.try_send(json) {
                STRUCT_LOG_QUEUE_ERROR_COUNT.inc();
                // Use log crate macro to avoid generation of structured log in this case
                // Otherwise we will have infinite loop
                log::error!("[Logging] Failed to send structured log: {}", e);
            }
        } else {
            STRUCT_LOG_PARSE_ERROR_COUNT.inc();
        }
    }
}

/// Thread for sending logs to a Logging TCP endpoint
///
/// `write_log_line` Blocks on the oldest log in the `receiver` until it can properly connect to an endpoint.
///   It will drop any message that is disconnected or failed in the middle of transmission after `NUM_SEND_RETRIES` retries.
struct TCPStructLogThread {
    receiver: Receiver<String>,
    endpoint: String,
}

impl TCPStructLogThread {
    fn new(receiver: Receiver<String>, endpoint: String) -> TCPStructLogThread {
        TCPStructLogThread { receiver, endpoint }
    }

    /// Writes a log line into json_lines logstash format, which has a newline at the end
    fn write_log_line(stream: &mut TcpWriter, message: String) -> io::Result<()> {
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

        result
    }

    pub fn run(self) {
        let mut writer = TcpWriter::new(self.endpoint.clone());
        for entry in &self.receiver {
            PROCESSED_STRUCT_LOG_COUNT.inc();

            // Write single log lines, guaranteeing stream is ready first
            // Note: This does not guarantee delivery of a message cut off in the middle,
            //       if there is an error after retries the log will be dropped.
            if let Err(e) = Self::write_log_line(&mut writer, entry) {
                STRUCT_LOG_SEND_ERROR_COUNT.inc();
                log::error!(
                    "[Logging] Error while sending data to logstash({}): {}",
                    self.endpoint,
                    e
                );
            } else {
                SENT_STRUCT_LOG_COUNT.inc()
            }
        }
    }
}

/// A wrapper for `TcpStream` that handles reconnecting to the endpoint automatically
///
/// `TcpWriter::write()` will block on the message until it is connected.
struct TcpWriter {
    /// The DNS name or IP address logs are being sent to
    endpoint: String,
    /// The `TCPStream` to write to, which will be `None` when disconnected
    stream: Option<TcpStream>,
}

impl TcpWriter {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            stream: None,
        }
    }

    /// Ensure that we get a connection, no matter how long it takes
    /// This will block until there is a connection
    fn refresh_connection(&mut self) {
        loop {
            match self.connect() {
                Ok(stream) => {
                    self.stream = Some(stream);
                    return;
                }
                Err(e) => {
                    log::error!("[Logging] Failed to connect: {}", e);
                    STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                }
            }

            // Sleep a second so this doesn't just spin as fast as possible
            std::thread::sleep(Duration::from_millis(1000));
        }
    }

    /// Connect and ensure the write timeout is set
    fn connect(&mut self) -> io::Result<TcpStream> {
        STRUCT_LOG_TCP_CONNECT_COUNT.inc();

        let mut last_error = io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to resolve and connect to {}", self.endpoint),
        );

        // resolve addresses to handle DNS names
        for addr in self.endpoint.to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(CONNECTION_TIMEOUT_MS)) {
                Ok(stream) => {
                    // Set the write timeout
                    if let Err(err) =
                        stream.set_write_timeout(Some(Duration::from_millis(WRITE_TIMEOUT_MS)))
                    {
                        STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                        log::error!("[Logging] Failed to set write timeout: {}", err);
                        continue;
                    }
                    return Ok(stream);
                }
                Err(err) => last_error = err,
            }
        }

        Err(last_error)
    }
}

impl Write for TcpWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Refresh the connection if it's missing
        if self.stream.is_none() {
            self.refresh_connection();
        }

        // Attempt to write, and if it fails clear underlying stream
        // This doesn't guarantee a message cut off mid send will work, but it does guarantee that
        // we will connect first
        let mut stream = self.stream.as_ref().unwrap();
        let result = stream.write(buf);
        if result.is_err() {
            self.stream = None;
        }

        result
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(mut stream) = self.stream.as_ref() {
            stream.flush()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Can't flush, not connected",
            ))
        }
    }
}
