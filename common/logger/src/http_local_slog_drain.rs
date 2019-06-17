// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Slog::Drain implementation to send log to specified http end point using POST.
//! A PlainKVSerializer is used to collect KVs from both Record and OwnedKVList.
//! The log sent will be plain json.
//!
//! ## Usage
//!
//! use slog::{o, Drain, Logger, *};
//! let client = crate::logger::http_log_client::HttpLogClient{
//!     use_https: false,
//!     destination: "http://localhost:1234".to_string(),
//! };
//! let drain = crate::logger::http_local_slog_drain::HttpLocalSlogDrain { client };
//! let logger = Logger::root(drain.fuse(), o!("component" => "admission_control"));
//! slog_info!(logger, "test info log"; "log-key" => true);
use crate::{collector_serializer::PlainKVSerializer, http_log_client::HttpLogClient};
use serde_json::json;
use slog::{slog_error, Drain, OwnedKVList, Record, KV};
pub use slog_scope::error;
use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct HttpLocalSlogDrain {
    client: HttpLogClient,
}
impl Drain for HttpLocalSlogDrain {
    type Ok = ();
    type Err = slog::Never;
    fn log(&self, record: &Record, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let ret = self.log_impl(record, values);
        if let Some(e) = ret.err() {
            // The error from logging should not be cascading, but we have to log it
            // somewhere for troubleshooting.
            error!("Error sending log using http client: {}", e);
        }
        Ok(())
    }
}

impl HttpLocalSlogDrain {
    pub fn new(client: HttpLogClient) -> Self {
        HttpLocalSlogDrain { client }
    }
    fn log_impl(&self, record: &Record, values: &OwnedKVList) -> Result<(), Box<Error>> {
        let mut serializer = PlainKVSerializer::new();
        values.serialize(record, &mut serializer)?;
        record.kv().serialize(record, &mut serializer)?;
        let mut kvs = serializer.into_inner();
        kvs.insert("msg", format!("{}", record.msg()));
        kvs.insert("level", record.level().as_str().to_string());
        kvs.insert("current_mod", module_path!().to_string());
        kvs.insert(
            "time",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current timestamp is earlier than UNIX epoch")
                .as_secs()
                .to_string(),
        );

        self.client.send_log(json!(kvs).to_string())?;
        Ok(())
    }
}
