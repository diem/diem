// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    prelude::*, set_struct_logger, struct_log::InitLoggerError, Key, LoggingField, Schema,
    StructLogSink, StructuredLogEntry, Value, Visitor,
};
use chrono::{DateTime, Utc};
use serde_json::{Map, Value as JsonValue};
use std::{
    io,
    sync::mpsc::{self, Receiver, SyncSender},
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

struct StreamStructLog {
    sender: SyncSender<StructuredLogEntry>,
}

impl StreamStructLog {
    pub fn start_new() -> io::Result<(Self, Receiver<StructuredLogEntry>)> {
        let (sender, receiver) = mpsc::sync_channel(1024);
        Ok((Self { sender }, receiver))
    }
}

impl StructLogSink for StreamStructLog {
    fn send(&self, entry: StructuredLogEntry) {
        if let Err(e) = self.sender.send(entry) {
            // Use log crate macro to avoid generation of structured log in this case
            // Otherwise we will have infinite loop
            log::error!("Failed to send structured log: {}", e);
        }
    }
}

fn set_test_struct_logger() -> Result<Receiver<StructuredLogEntry>, InitLoggerError> {
    let (logger, receiver) = StreamStructLog::start_new().map_err(InitLoggerError::IoError)?;
    let logger = Box::leak(Box::new(logger));
    set_struct_logger(logger).map(|_| receiver)
}

trait MapHelper {
    fn val(&self, key: &str) -> JsonValue;
    fn string(&self, key: &str) -> String;
}

impl MapHelper for Map<String, JsonValue> {
    fn val(&self, key: &str) -> JsonValue {
        self.get(key).unwrap().clone()
    }

    fn string(&self, key: &str) -> String {
        String::from(self.val(key).as_str().unwrap())
    }
}

fn as_object(value: JsonValue) -> Map<String, JsonValue> {
    value.as_object().unwrap().clone()
}

fn recieve_one_event(receiver: &Receiver<StructuredLogEntry>) -> Map<String, JsonValue> {
    let entry = receiver.recv().unwrap();
    let value = serde_json::to_value(entry).unwrap();
    as_object(value)
}

// TODO: Find a better mechanism for testing that allows setting the logger not globally
#[test]
fn test_structured_logs() {
    let log_message = String::from("This is a log");
    let number = 12345;
    let u64_field: &LoggingField<&u64> = &LoggingField::new("field");
    let receiver = set_test_struct_logger().unwrap();

    // Send an info log
    let before = Utc::now();
    sl_info!(StructuredLogEntry::new_named("category", "name")
        .data("test", true)
        .data_display("display", number)
        .schema(TestSchema {
            foo: 5,
            bar: &Enum::FooBar,
        })
        .field(u64_field, &number)
        .log(log_message.clone()));
    let after = Utc::now();

    let map = recieve_one_event(&receiver);

    // Ensure standard fields are filled
    assert_eq!("category", map.string("category").as_str());
    assert_eq!("name", map.string("name").as_str());
    assert_eq!("INFO", map.string("level").as_str());
    assert_eq!(
        "libra_logger::tests::struct_log",
        map.string("module").as_str()
    );
    assert!(map
        .string("location")
        .starts_with("common/logger/src/tests/struct_log.rs"));
    assert_eq!(log_message, map.string("log"));

    // Id should be random, as long as it's not empty, we shoudl be good
    assert!(!map.string("id").is_empty());

    // Log time should be the time the structured log entry was created
    let timestamp = DateTime::parse_from_rfc3339(&map.string("timestamp")).unwrap();
    let timestamp: DateTime<Utc> = DateTime::from(timestamp);
    assert!(before <= timestamp && timestamp <= after);

    // Ensure data stored is the right type
    let data = as_object(map.val("data"));
    assert_eq!(true, data.val("test").as_bool().unwrap());
    assert_eq!(data.val("foo").as_u64().unwrap(), 5);
    assert_eq!(data.val("bar").as_str().unwrap(), "foo_bar");

    // Data display always stores as a number rather than serializing
    assert_eq!(format!("{}", number), data.string("display"));

    // LoggingField keeps strong typing
    assert_eq!(number, data.val("field").as_u64().unwrap());

    // Test all log levels work properly
    sl_trace!(StructuredLogEntry::new_named("a", "b"));
    sl_debug!(StructuredLogEntry::new_named("a", "b"));
    sl_info!(StructuredLogEntry::new_named("a", "b"));
    sl_warn!(StructuredLogEntry::new_named("a", "b"));
    sl_error!(StructuredLogEntry::new_named("a", "b"));

    // TODO: Fix this as it's fragile based on what the log level is set at
    let vals = vec!["DEBUG", "INFO", "WARN", "ERROR"];

    for val in &vals {
        let map = recieve_one_event(&receiver);
        assert_eq!(val, &map.string("level").as_str());
    }

    // Ensure levels are copied over and logs are converted to structured logs
    let value = "value";
    trace!("NOT-ENABLED");
    debug!("DEBUG {:?}", "Debug this");
    info!("Info, alright! {}", value);
    warn!("Warning! Get serious {}", "Serious warning");
    error!("Error! I give up {} & {}", value, "match");

    for val in &vals {
        let map = recieve_one_event(&receiver);
        assert_eq!(val, &map.string("level").as_str());
    }

    // Test pattern conversion
    let pattern = "Let's try a pattern {} {}";
    let value = "value";
    info!("Let's try a pattern {} {}", "anonymous", key = value);

    // Check specifics to text conversion
    let map = recieve_one_event(&receiver);
    assert_eq!("text", map.string("category").as_str());
    assert_eq!(pattern, map.string("pattern").as_str());
    assert_eq!(
        format!("Let's try a pattern {} {}", "anonymous", value),
        map.string("log")
    );

    // Ensure data stored is the right type
    let data = as_object(map.val("data"));
    assert_eq!(format!("{:?}", value), data.string("key"));
    assert_eq!(format!("{:?}", "anonymous"), data.string("_0"));
}
