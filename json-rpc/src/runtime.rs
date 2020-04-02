// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    errors::JsonRpcError,
    methods::{build_registry, JsonRpcService, RpcRegistry},
};
use futures::future::join_all;
use libra_config::config::NodeConfig;
use libra_mempool::MempoolClientSender;
use libradb::DbReader;
use serde_json::{map::Map, Value};
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};
use warp::Filter;

/// Creates HTTP server (warp-based) that serves JSON RPC requests
/// Returns handle to corresponding Tokio runtime
pub fn bootstrap(
    address: SocketAddr,
    libra_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("rpc-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[rpc] failed to create runtime");

    let registry = Arc::new(build_registry());
    let service = JsonRpcService::new(libra_db, mp_sender);

    let handler = warp::any()
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::json())
        .and(warp::any().map(move || service.clone()))
        .and(warp::any().map(move || Arc::clone(&registry)))
        .and_then(rpc_endpoint);

    // Ensure that we actually bind to the socket first before spawning the
    // server tasks. This helps in tests to prevent races where a client attempts
    // to make a request before the server task is actually listening on the
    // socket.
    //
    // Note: we need to enter the runtime context first to actually bind, since
    //       tokio TcpListener can only be bound inside a tokio context.
    let server = runtime.enter(move || warp::serve(handler).bind(address));
    runtime.handle().spawn(server);
    runtime
}

/// Creates JSON RPC endpoint by given node config
pub fn bootstrap_from_config(
    config: &NodeConfig,
    libra_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
) -> Runtime {
    bootstrap(config.rpc.address, libra_db, mp_sender)
}

/// JSON RPC entry point
/// Handles all incoming rpc requests
/// Performs routing based on methods defined in `registry`
async fn rpc_endpoint(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if let Value::Array(requests) = data {
        // batch API call
        let futures = requests
            .into_iter()
            .map(|req| rpc_request_handler(req, service.clone(), Arc::clone(&registry)));
        let responses = join_all(futures).await;
        Ok(Box::new(warp::reply::json(&Value::Array(responses))))
    } else {
        // single API call
        let resp = rpc_request_handler(data, service, registry).await;
        Ok(Box::new(warp::reply::json(&resp)))
    }
}

/// Handler of single RPC request
/// Performs validation and executes corresponding rpc handler
async fn rpc_request_handler(
    req: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
) -> Value {
    let request: Map<String, Value>;
    let mut response = Map::new();

    // set defaults: protocol version to 2.0, request id to null
    response.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    response.insert("id".to_string(), Value::Null);

    match req {
        Value::Object(data) => {
            request = data;
        }
        _ => {
            response.insert(
                "error".to_string(),
                JsonRpcError::invalid_request().serialize(),
            );
            counters::INVALID_REQUESTS
                .with_label_values(&["invalid_format"])
                .inc();
            return Value::Object(response);
        }
    }

    // parse request id
    match parse_request_id(&request) {
        Ok(request_id) => {
            response.insert("id".to_string(), request_id);
        }
        Err(err) => {
            response.insert("error".to_string(), err.serialize());
            counters::INVALID_REQUESTS
                .with_label_values(&["invalid_format"])
                .inc();
            return Value::Object(response);
        }
    };

    // verify protocol version
    if let Err(err) = verify_protocol(&request) {
        response.insert("error".to_string(), err.serialize());
        counters::INVALID_REQUESTS
            .with_label_values(&["invalid_format"])
            .inc();
        return Value::Object(response);
    }

    // parse parameters
    let params;
    match request.get("params") {
        Some(Value::Array(parameters)) => {
            params = parameters.to_vec();
        }
        _ => {
            response.insert(
                "error".to_string(),
                JsonRpcError::invalid_params().serialize(),
            );
            counters::INVALID_REQUESTS
                .with_label_values(&["invalid_params"])
                .inc();
            return Value::Object(response);
        }
    }

    // get rpc handler
    match request.get("method") {
        Some(Value::String(name)) => match registry.get(name) {
            Some(handler) => match handler(service, params).await {
                Ok(result) => {
                    response.insert("result".to_string(), result);
                    counters::REQUESTS
                        .with_label_values(&[name, "success"])
                        .inc();
                }
                Err(err) => {
                    // check for custom error
                    if let Some(custom_error) = err.downcast_ref::<JsonRpcError>() {
                        response.insert("error".to_string(), custom_error.clone().serialize());
                    } else {
                        response.insert(
                            "error".to_string(),
                            JsonRpcError::internal_error(err.to_string()).serialize(),
                        );
                    }
                    counters::REQUESTS.with_label_values(&[name, "fail"]).inc();
                }
            },
            None => {
                response.insert(
                    "error".to_string(),
                    JsonRpcError::method_not_found().serialize(),
                );
                counters::INVALID_REQUESTS
                    .with_label_values(&["method_not_found"])
                    .inc();
            }
        },
        _ => {
            response.insert(
                "error".to_string(),
                JsonRpcError::invalid_request().serialize(),
            );
            counters::INVALID_REQUESTS
                .with_label_values(&["invalid_method"])
                .inc();
        }
    }

    Value::Object(response)
}

fn parse_request_id(request: &Map<String, Value>) -> Result<Value, JsonRpcError> {
    match request.get("id") {
        Some(req_id) => {
            if req_id.is_string() || req_id.is_number() || req_id.is_null() {
                Ok(req_id.clone())
            } else {
                Err(JsonRpcError::invalid_request())
            }
        }
        None => Ok(Value::Null),
    }
}

fn verify_protocol(request: &Map<String, Value>) -> Result<(), JsonRpcError> {
    if let Some(Value::String(protocol)) = request.get("jsonrpc") {
        if protocol == "2.0" {
            return Ok(());
        }
    }
    Err(JsonRpcError::invalid_request())
}
