// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/*!
`ServiceMetrics` is a metric [`Collector`](prometheus::core::Collector) to capture key
metrics about a gRPC server.

For each method, the counters that are captured are:
- num_req: number of requests
- num_error: number of errors (can be used to calculate error rate)
- duration: duration (in units determined by the exporter) the request too, bucketed

Example use:
call `req` when when entering service method, and call `resp` on
exit, with a boolean flag to specify whether the request was
a success or a failure, to bump the counter for failures.
The call to `req` will provide a timer that handle time logging, as long
as it's in scope.

fn sample_service_method(ctx: RpcContext, params: Params) {
  let _timer = metrics.req(&ctx);
  // do business logic
  metrics.resp(&ctx, success_flag);
}
*/

use grpcio::RpcContext;
use libra_logger::prelude::*;
use prometheus::{
    core::{Collector, Desc},
    exponential_buckets,
    proto::MetricFamily,
    HistogramOpts, HistogramTimer, HistogramVec, IntCounterVec, Opts, Result,
};
use std::str;

#[derive(Clone)]
pub struct ServiceMetrics {
    // This struct is going to define a MetricFamily (per metric, this is how the library
    // works). We'll have a metric per counter, with the methods as dimensions (labels):
    // e.g., calc_service.req{method = "add"} = +1
    // e.g., calc_service.duration_sum{method="add"} = 6
    num_req: IntCounterVec,
    num_error: IntCounterVec,
    duration: HistogramVec,     // we want .avg and .p90 (but really p99)
    message_size: HistogramVec, // collect the size of messages, up to max-send-size
}

impl ServiceMetrics {
    pub fn default() -> ServiceMetrics {
        let message_size_buckets = exponential_buckets(2.0, 2.0, 22)
            .expect("Could not create buckets for message-size histogram");

        ServiceMetrics {
            num_req: IntCounterVec::new(Opts::new("num_req", "Number of requests"), &["method"])
                .unwrap(),
            num_error: IntCounterVec::new(Opts::new("num_error", "Number of errors"), &["method"])
                .unwrap(),
            duration: HistogramVec::new(
                //TODO: frumious: how to ensure units?
                HistogramOpts::new("duration", "Duration for a request, in units of time"),
                &["method"],
            )
            .unwrap(),

            message_size: HistogramVec::new(
                HistogramOpts::new("message_size", "gRPC message size, in bytes (or close to)")
                    .buckets(message_size_buckets),
                &["message"],
            )
            .unwrap(),
        }
    }

    pub fn new_and_registered() -> ServiceMetrics {
        let svc = ServiceMetrics::default();
        //TODO: should be OK to panic here
        let _res = prometheus::register(Box::new(svc.clone()));
        svc
    }

    pub fn req(&self, ctx: &RpcContext) -> Option<HistogramTimer> {
        // this should match a server interceptor; but it's going to be
        // a lot of conversions from [byte] to String
        let mut method_name = "unknown_method".to_string();
        if let Some(name) = path_from_ctx(ctx) {
            method_name = name;
        }

        self.num_req
            .with_label_values(&[method_name.as_str()])
            .inc();
        Some(
            self.duration
                .with_label_values(&[method_name.as_str()])
                .start_timer(),
        )
    }

    pub fn resp(&self, ctx: &RpcContext, success: bool) {
        // The reason for counting everything here, instead of doing the
        // if outside of the increment is that we could also compare
        // number of responses to number of requests
        if let Some(name) = path_from_ctx(ctx) {
            self.num_error
                .with_label_values(&[name.as_str()])
                .inc_by(if success { 0 } else { 1 });
        }
    }

    pub fn register_default(&self) -> Result<()> {
        prometheus::register(Box::new(self.clone()))
    }
}

impl Collector for ServiceMetrics {
    fn desc(&self) -> Vec<&Desc> {
        // order: num_req, num_error, duration
        vec![
            self.num_req.desc(),
            self.num_error.desc(),
            self.duration.desc(),
            self.message_size.desc(),
        ]
        .into_iter()
        .map(|m| m[0])
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        // families
        let vs = vec![
            self.num_req.collect(),
            self.num_error.collect(),
            self.duration.collect(),
            self.message_size.collect(),
        ];
        // The model here is annoying --
        // I'd like a single MetricFamily with a name (of the service) and
        // each metric with its own name.
        vs.into_iter().fold(vec![], |mut l, v| {
            l.extend(v);
            l
        })
    }
}

/// This method reads the full URI from gRpcContext (looks like:
/// `/{package}.{service_name}/{method}`
/// and converts it into a dot-delimited string, dropping the 1st `/`
fn path_from_ctx(ctx: &RpcContext) -> Option<String> {
    // The content of method() is: /{package}.{service}/{method}
    // 47 ascii = '/'
    let method = ctx.method();
    path_from_byte_slice(method)
}

/// This method reads the full URI from gRpcContext (looks like:
/// `/{package}.{service_name}/{method}`
/// and converts it into a dot-delimited string, dropping the 1st `/`
fn path_from_byte_slice(bytes: &[u8]) -> Option<String> {
    // The content of method() is expected to be:
    // `/{package}.{service}/{method}`
    // 47 ascii = '/'
    if bytes.len() < 5 || bytes[0] != 47u8 {
        // Incorrect structure: too short, or first char is not '/'
        info!("malformed request path: {:?}", bytes);
        return None;
    }

    let mut method_raw = vec![0u8; bytes.len() - 1];
    method_raw.copy_from_slice(&bytes[1..]);
    if let Ok(name) = str::from_utf8(&method_raw) {
        return Some(name.replace("/", "."));
    }
    info!("failed to convert byte slice to string: {:?}", &method_raw);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_from_bytes() {
        let too_short = vec![47u8, 65u8, 47u8];
        assert_eq!(path_from_byte_slice(&too_short), None);

        // first char is not '/'
        let malformed = vec![65u8, 46u8, 65u8, 47u8, 66u8];
        assert_eq!(path_from_byte_slice(&malformed), None);

        // /package.service/method
        let full_name = vec![47u8, 65u8, 46u8, 65u8, 47u8, 66u8];
        assert_eq!(
            path_from_byte_slice(&full_name),
            Some(String::from("A.A.B"))
        );

        // /service/method
        let no_package = vec![47u8, 65u8, 98u8, 47u8, 99u8];
        assert_eq!(
            path_from_byte_slice(&no_package),
            Some(String::from("Ab.c"))
        );
    }
}
