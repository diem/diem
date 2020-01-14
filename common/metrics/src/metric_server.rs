// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use crate::json_metrics::get_json_metrics;
use crate::public_metrics::PUBLIC_METRICS;
use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use prometheus::{proto::MetricFamily, Encoder, TextEncoder};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use tokio::runtime;

fn encode_metrics(encoder: impl Encoder, whitelist: &'static [&'static str]) -> Vec<u8> {
    let mut metric_families = prometheus::gather();
    if !whitelist.is_empty() {
        metric_families = whitelist_metrics(metric_families, whitelist);
    }
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}

// filtering metrics from the prometheus collections
// only return the whitelisted metrics
fn whitelist_metrics(
    metric_families: Vec<MetricFamily>,
    whitelist: &'static [&'static str],
) -> Vec<MetricFamily> {
    let mut whitelist_metrics = Vec::new();
    for mf in metric_families {
        let name = mf.get_name();
        if whitelist.contains(&name) {
            whitelist_metrics.push(mf.clone());
        }
    }
    whitelist_metrics
}

// filtering metrics from the Json format metrics
// only return the whitelisted metrics
fn whitelist_json_metrics(
    json_metrics: HashMap<String, String>,
    whitelist: &'static [&'static str],
) -> HashMap<&'static str, String> {
    let mut whitelist_metrics: HashMap<&'static str, String> = HashMap::new();
    for key in whitelist {
        if let Some(metric) = json_metrics.get(*key) {
            whitelist_metrics.insert(key, metric.clone());
        }
    }
    whitelist_metrics
}

async fn serve_metrics(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            //Prometheus server expects metrics to be on host:port/metrics
            let encoder = TextEncoder::new();
            let buffer = encode_metrics(encoder, &[]);
            *resp.body_mut() = Body::from(buffer);
        }
        // expose non-numeric metrics to host:port/json_metrics
        (&Method::GET, "/json_metrics") => {
            let json_metrics = get_json_metrics();
            let encoded_metrics = serde_json::to_string(&json_metrics).unwrap();
            *resp.body_mut() = Body::from(encoded_metrics);
        }
        (&Method::GET, "/counters") => {
            // Json encoded libra_metrics;
            let encoder = JsonEncoder;
            let buffer = encode_metrics(encoder, &[]);
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(resp)
}

async fn serve_public_metrics(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();
            // encode public metrics defined in common/metrics/src/public_metrics.rs
            let buffer = encode_metrics(encoder, PUBLIC_METRICS);
            *resp.body_mut() = Body::from(buffer);
        }
        (&Method::GET, "/json_metrics") => {
            let json_metrics = get_json_metrics();
            let whitelist_json_metrics = whitelist_json_metrics(json_metrics, PUBLIC_METRICS);
            let encoded_metrics = serde_json::to_string(&whitelist_json_metrics).unwrap();
            *resp.body_mut() = Body::from(encoded_metrics);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(resp)
}

pub fn start_server(host: String, port: u16, public_metric: bool) {
    // Only called from places that guarantee that host is parsable, but this must be assumed.
    let addr: SocketAddr = (host.as_str(), port)
        .to_socket_addrs()
        .unwrap_or_else(|_| unreachable!("Failed to parse {}:{} as address", host, port))
        .next()
        .unwrap();

    if public_metric {
        thread::spawn(move || {
            let make_service = make_service_fn(|_| {
                future::ok::<_, hyper::Error>(service_fn(serve_public_metrics))
            });

            let mut rt = runtime::Builder::new()
                .basic_scheduler()
                .enable_io()
                .build()
                .unwrap();
            rt.block_on(async {
                let server = Server::bind(&addr).serve(make_service);
                server.await
            })
            .unwrap();
        });
    } else {
        thread::spawn(move || {
            let make_service =
                make_service_fn(|_| future::ok::<_, hyper::Error>(service_fn(serve_metrics)));

            let mut rt = runtime::Builder::new()
                .basic_scheduler()
                .enable_io()
                .build()
                .unwrap();
            rt.block_on(async {
                let server = Server::bind(&addr).serve(make_service);
                server.await
            })
            .unwrap();
        });
    }
}
