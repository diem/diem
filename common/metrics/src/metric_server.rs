// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use crate::public_metrics::PUBLIC_METRICS;
use futures::future;
use hyper::{
    rt::{self, Future},
    service::service_fn,
    Body, Method, Request, Response, Server, StatusCode,
};
use libra_logger::prelude::*;
use prometheus::{proto::MetricFamily, Encoder, TextEncoder};
use std::net::{SocketAddr, ToSocketAddrs};

fn encode_metrics(encoder: impl Encoder, whitelist: Vec<String>) -> Vec<u8> {
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
    whitelist: Vec<String>,
) -> Vec<MetricFamily> {
    let mut whitelist_metrics = Vec::new();
    for mf in metric_families {
        let name = mf.get_name();
        if whitelist.contains(&name.to_string()) {
            whitelist_metrics.push(mf.clone());
        }
    }
    whitelist_metrics
}

fn serve_metrics(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            //Prometheus server expects metrics to be on host:port/metrics
            let encoder = TextEncoder::new();
            let buffer = encode_metrics(encoder, Vec::new());
            *resp.body_mut() = Body::from(buffer);
        }
        (&Method::GET, "/counters") => {
            // Json encoded libra_metrics;
            let encoder = JsonEncoder;
            let buffer = encode_metrics(encoder, Vec::new());
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    future::ok(resp)
}

fn serve_public_metrics(
    req: Request<Body>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();
            // encode public metrics defined in common/metrics/src/public_metrics.rs
            let buffer = encode_metrics(encoder, PUBLIC_METRICS.to_vec());
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    future::ok(resp)
}

pub fn start_server(host: String, port: u16, public_metric: bool) {
    let addr: SocketAddr = (host.as_str(), port)
        .to_socket_addrs()
        .unwrap_or_else(|_| panic!("Failed to parse {}:{} as address", host, port))
        .next()
        .unwrap();

    if public_metric {
        rt::run(rt::lazy(move || {
            match Server::try_bind(&addr) {
                Ok(srv) => {
                    let srv = srv
                        .serve(|| service_fn(serve_public_metrics))
                        .map_err(|e| eprintln!("server error: {}", e));
                    info!("Metric server listening on http://{}", addr);
                    rt::spawn(srv);
                }
                Err(e) => error!("Metric server bind error: {}", e),
            };

            Ok(())
        }));
    } else {
        rt::run(rt::lazy(move || {
            match Server::try_bind(&addr) {
                Ok(srv) => {
                    let srv = srv
                        .serve(|| service_fn(serve_metrics))
                        .map_err(|e| eprintln!("server error: {}", e));
                    info!("Metric server listening on http://{}", addr);
                    rt::spawn(srv);
                }
                Err(e) => error!("Metric server bind error: {}", e),
            };

            Ok(())
        }));
    }
}
