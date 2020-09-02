// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::StructLogSink;
use crate::{
    prelude::*, struct_log::InitLoggerError, Key, LoggingField, Schema, StructuredLogEntry, Value,
    Visitor,
};
use chrono::{DateTime, Utc};
use serde_json::{Map, Value as JsonValue};
use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc,
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
    pub fn start_new() -> (Self, Receiver<StructuredLogEntry>) {
        let (sender, receiver) = mpsc::sync_channel(1024);
        (Self { sender }, receiver)
    }
}

impl crate::logger::Logger for StreamStructLog {
    fn enabled(&self, metadata: &crate::Metadata) -> bool {
        metadata.level() <= crate::Level::Debug
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

        self.sender.send(entry).unwrap();
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
    let (logger, receiver) = StreamStructLog::start_new();
    let logger = Arc::new(logger);
    crate::logger::set_global_logger(logger);
    Ok(receiver)
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
    info!(StructuredLogEntry::new_named("category", "name")
        .data("test", true)
        .data_display("display", number)
        .schema(TestSchema {
            foo: 5,
            bar: &Enum::FooBar,
        })
        .field(u64_field, &number)
        .message(log_message));
    let after = Utc::now();

    let map = recieve_one_event(&receiver);

    // Ensure standard fields are filled
    assert_eq!("INFO", map.string("level").as_str());
    assert_eq!(
        "libra_logger::struct_log::tests",
        map.string("module").as_str()
    );
    assert!(map
        .string("location")
        .starts_with("common/logger/src/struct_log/tests.rs"));

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
    trace!(StructuredLogEntry::new_named("a", "b"));
    debug!(StructuredLogEntry::new_named("a", "b"));
    info!(StructuredLogEntry::new_named("a", "b"));
    warn!(StructuredLogEntry::new_named("a", "b"));
    error!(StructuredLogEntry::new_named("a", "b"));

    // TODO: Fix this as it's fragile based on what the log level is set at
    let vals = vec!["DEBUG", "INFO", "WARN", "ERROR"];

    for val in &vals {
        let map = recieve_one_event(&receiver);
        assert_eq!(val, &map.string("level").as_str());
    }
}
