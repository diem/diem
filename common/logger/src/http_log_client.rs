// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Send log to a http end point. Error will be propagate to call side.
//!
//! ## Usage
//!
//! let client = crate::logger::http_log_client::HttpLogClient{
//!     use_https: false,
//!     destination: "http://localhost:1234".to_string(),
//! };
use failure::Result;
use futures::{future::Future, stream::Stream};
use hyper::{header, Body, Client, Request, Uri};
use slog::{slog_error, slog_trace};
pub use slog_scope::{debug, error, trace};
use std::thread;
use tokio::runtime::current_thread;

#[derive(Debug)]
pub struct HttpLogClient {
    // Destination string for this client.
    // Https is not supported and will result an error.
    destination: String,
}

impl HttpLogClient {
    pub fn new(destination: String) -> Result<Self> {
        let uri = destination.parse::<Uri>()?;
        if let Some(schema) = uri.scheme_part() {
            if schema.as_str() == "https" {
                error!("Https is not supported, uri {}", destination)
            }
        }

        Ok(HttpLogClient { destination })
    }

    pub fn send_log(&self, logs: String) -> Result<()> {
        let client = Client::builder().build_http();
        let body = format!("json={}", logs);
        let req = Request::post(self.destination.clone())
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .header(header::CONTENT_LENGTH, body.len() as u64)
            .body(Body::from(body))?;

        let fut = client
            .request(req)
            .and_then(|res| {
                let status = res.status();
                res.into_body().concat2().and_then(move |body| {
                    trace!(
                        "Log sent, status: {}, response: {}",
                        status,
                        String::from_utf8_lossy(&body).into_owned()
                    );
                    Ok(())
                })
            })
            .map_err(|e| error!("error sending log: {}", e));

        // TODO Verify whether we need to spawn thread for each call or slog_async (queue + 1
        // worker) is sufficient.
        thread::spawn(move || current_thread::Runtime::new().unwrap().block_on(fut));
        Ok(())
    }
}
