// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::methods::{build_registry, RpcRegistry};
use futures::future::join_all;
use libra_config::config::NodeConfig;
use libradb::LibraDB;
use serde::Serialize;
use serde_json::{map::Map, Value};
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};
use warp::Filter;

/// Creates HTTP server (warp-based) that serves JSON RPC requests
/// Returns handle to corresponding Tokio runtime
pub fn bootstrap(address: SocketAddr, libra_db: Arc<LibraDB>) -> Runtime {
    let runtime = Builder::new()
        .thread_name("rpc-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[rpc] failed to create runtime");

    let registry = Arc::new(build_registry());

    let handler = warp::any()
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::json())
        .and(warp::any().map(move || Arc::clone(&libra_db)))
        .and(warp::any().map(move || Arc::clone(&registry)))
        .and_then(rpc_endpoint);

    let server = warp::serve(handler).run(address);
    runtime.handle().spawn(server);
    runtime
}

/// Creates JSON RPC endpoint by given node config
pub fn bootstrap_from_config(config: &NodeConfig, libra_db: Arc<LibraDB>) -> Runtime {
    bootstrap(config.rpc.address, libra_db)
}

/// JSON RPC entry point
/// Handles all incoming rpc requests
/// Performs routing based on methods defined in `registry`
async fn rpc_endpoint(
    data: Value,
    libra_db: Arc<LibraDB>,
    registry: Arc<RpcRegistry>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if let Value::Array(requests) = data {
        // batch API call
        let futures = requests
            .into_iter()
            .map(|req| rpc_request_handler(req, Arc::clone(&libra_db), Arc::clone(&registry)));
        let responses = join_all(futures).await;
        Ok(Box::new(warp::reply::json(&Value::Array(responses))))
    } else {
        // single API call
        let resp = rpc_request_handler(data, libra_db, registry).await;
        Ok(Box::new(warp::reply::json(&resp)))
    }
}

/// Handler of single RPC request
/// Performs validation and executes corresponding rpc handler
async fn rpc_request_handler(
    req: Value,
    libra_db: Arc<LibraDB>,
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
            return Value::Object(response);
        }
    };

    // verify protocol version
    if let Err(err) = verify_protocol(&request) {
        response.insert("error".to_string(), err.serialize());
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
            return Value::Object(response);
        }
    }

    // get rpc handler
    match request.get("method") {
        Some(Value::String(name)) => match registry.get(name) {
            Some(handler) => match handler(libra_db, params).await {
                Ok(result) => {
                    response.insert("result".to_string(), result);
                }
                Err(err) => {
                    response.insert(
                        "error".to_string(),
                        JsonRpcError::internal_error(err.to_string()).serialize(),
                    );
                }
            },
            None => {
                response.insert(
                    "error".to_string(),
                    JsonRpcError::method_not_found().serialize(),
                );
            }
        },
        _ => {
            response.insert(
                "error".to_string(),
                JsonRpcError::invalid_request().serialize(),
            );
        }
    }

    Value::Object(response)
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i16,
    message: String,
}

impl JsonRpcError {
    fn serialize(self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
        }
    }

    fn invalid_params() -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
        }
    }

    fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
        }
    }

    fn internal_error(message: String) -> Self {
        Self {
            code: -32000,
            message: format!("Server error: {}", message),
        }
    }
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
