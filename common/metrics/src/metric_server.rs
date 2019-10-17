// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use futures::future;
use hyper::{
    rt::{self, Future},
    service::service_fn,
    Body, Method, Request, Response, Server, StatusCode,
};
use libra_logger::prelude::*;
use prometheus::{Encoder, TextEncoder};
use std::net::{SocketAddr, ToSocketAddrs};

fn encode_metrics(encoder: impl Encoder) -> Vec<u8> {
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}

fn serve_metrics(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            //Prometheus server expects metrics to be on host:port/metrics
            let encoder = TextEncoder::new();
            let buffer = encode_metrics(encoder);
            *resp.body_mut() = Body::from(buffer);
        }
        (&Method::GET, "/counters") => {
            // Json encoded libra_metrics;
            let encoder = JsonEncoder;
            let buffer = encode_metrics(encoder);
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    future::ok(resp)
}

pub fn start_server<T: ToSocketAddrs>(to_addr: T) {
    let addr: SocketAddr = to_addr
        .to_socket_addrs()
        .unwrap_or_else(|_| panic!("Failed to parse address"))
        .next()
        .unwrap();

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
