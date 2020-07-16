// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    errors::JsonRpcError,
    methods::{build_registry, JsonRpcRequest, JsonRpcService, RpcRegistry},
};
use futures::future::join_all;
use libra_config::config::{NodeConfig, RoleType};
use libra_json_rpc_types::views::{
    JSONRPC_LIBRA_LEDGER_TIMESTAMPUSECS, JSONRPC_LIBRA_LEDGER_VERSION,
};
use libra_mempool::MempoolClientSender;
use libra_types::ledger_info::LedgerInfoWithSignatures;
use serde_json::{map::Map, Value};
use std::{net::SocketAddr, sync::Arc};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Runtime};
use warp::{
    reject::{self, Reject},
    Filter,
};

// Counter labels for runtime metrics
const LABEL_FAIL: &str = "fail";
const LABEL_INVALID_FORMAT: &str = "invalid_format";
const LABEL_INVALID_METHOD: &str = "invalid_method";
const LABEL_INVALID_PARAMS: &str = "invalid_params";
const LABEL_MISSING_METHOD: &str = "method_not_found";
const LABEL_SUCCESS: &str = "success";

/// Creates HTTP server (warp-based) that serves JSON RPC requests
/// Returns handle to corresponding Tokio runtime
pub fn bootstrap(
    address: SocketAddr,
    libra_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
    role: RoleType,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("rpc-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[rpc] failed to create runtime");

    let registry = Arc::new(build_registry());
    let service = JsonRpcService::new(libra_db, mp_sender, role);

    let base_route = warp::any()
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::json())
        .and(warp::any().map(move || service.clone()))
        .and(warp::any().map(move || Arc::clone(&registry)))
        .and_then(rpc_endpoint);

    // For now we still allow user to use "/", but user should start to move to "/v1" soon
    let route_root = warp::path::end().and(base_route.clone());

    let route_v1 = warp::path::path("v1")
        .and(warp::path::end())
        .and(base_route);

    let full_route = route_v1.or(route_root);

    // Ensure that we actually bind to the socket first before spawning the
    // server tasks. This helps in tests to prevent races where a client attempts
    // to make a request before the server task is actually listening on the
    // socket.
    //
    // Note: we need to enter the runtime context first to actually bind, since
    //       tokio TcpListener can only be bound inside a tokio context.
    let server = runtime.enter(move || warp::serve(full_route).bind(address));
    runtime.handle().spawn(server);
    runtime
}

/// Creates JSON RPC endpoint by given node config
pub fn bootstrap_from_config(
    config: &NodeConfig,
    libra_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
) -> Runtime {
    bootstrap(config.rpc.address, libra_db, mp_sender, config.base.role)
}

/// JSON RPC entry point
/// Handles all incoming rpc requests
/// Performs routing based on methods defined in `registry`
async fn rpc_endpoint(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // take snapshot of latest version of DB to be used across all requests, especially for batched requests
    let ledger_info = service
        .get_latest_ledger_info()
        .map_err(|_| reject::custom(DatabaseError))?;

    let resp = Ok(if let Value::Array(requests) = data {
        // batch API call
        let futures = requests.into_iter().map(|req| {
            rpc_request_handler(
                req,
                service.clone(),
                Arc::clone(&registry),
                ledger_info.clone(),
            )
        });
        let responses = join_all(futures).await;
        warp::reply::json(&Value::Array(responses))
    } else {
        // single API call
        let resp = rpc_request_handler(data, service, registry, ledger_info).await;
        warp::reply::json(&resp)
    });

    Ok(Box::new(resp) as Box<dyn warp::Reply>)
}

/// Handler of single RPC request
/// Performs validation and executes corresponding rpc handler
async fn rpc_request_handler(
    req: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
    ledger_info: LedgerInfoWithSignatures,
) -> Value {
    let request: Map<String, Value>;
    let mut response = Map::new();
    let version = ledger_info.ledger_info().version();
    let timestamp = ledger_info.ledger_info().timestamp_usecs();

    // set defaults: protocol version to 2.0, request id to null
    response.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    response.insert("id".to_string(), Value::Null);
    response.insert(
        JSONRPC_LIBRA_LEDGER_VERSION.to_string(),
        Value::Number(version.into()),
    );
    response.insert(
        JSONRPC_LIBRA_LEDGER_TIMESTAMPUSECS.to_string(),
        Value::Number(timestamp.into()),
    );

    match req {
        Value::Object(data) => {
            request = data;
        }
        _ => {
            set_response_error(
                &mut response,
                JsonRpcError::invalid_request(),
                Some(LABEL_INVALID_FORMAT),
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
            set_response_error(&mut response, err, Some(LABEL_INVALID_FORMAT));
            return Value::Object(response);
        }
    };

    // verify protocol version
    if let Err(err) = verify_protocol(&request) {
        set_response_error(&mut response, err, Some(LABEL_INVALID_FORMAT));
        return Value::Object(response);
    }

    // parse parameters
    let params;
    match request.get("params") {
        Some(Value::Array(parameters)) => {
            params = parameters.to_vec();
        }
        _ => {
            set_response_error(
                &mut response,
                JsonRpcError::invalid_params(),
                Some(LABEL_INVALID_PARAMS),
            );
            return Value::Object(response);
        }
    }

    let request_params = JsonRpcRequest {
        ledger_info,
        params,
    };
    // get rpc handler
    match request.get("method") {
        Some(Value::String(name)) => match registry.get(name) {
            Some(handler) => match handler(service, request_params).await {
                Ok(result) => {
                    response.insert("result".to_string(), result);
                    counters::REQUESTS
                        .with_label_values(&[name, LABEL_SUCCESS])
                        .inc();
                }
                Err(err) => {
                    // check for custom error
                    if let Some(custom_error) = err.downcast_ref::<JsonRpcError>() {
                        set_response_error(&mut response, custom_error.clone(), None);
                    } else {
                        set_response_error(
                            &mut response,
                            JsonRpcError::internal_error(err.to_string()),
                            None,
                        );
                    }
                    counters::REQUESTS
                        .with_label_values(&[name, LABEL_FAIL])
                        .inc();
                }
            },
            None => {
                set_response_error(
                    &mut response,
                    JsonRpcError::method_not_found(),
                    Some(LABEL_MISSING_METHOD),
                );
            }
        },
        _ => {
            set_response_error(
                &mut response,
                JsonRpcError::invalid_request(),
                Some(LABEL_INVALID_METHOD),
            );
        }
    }

    Value::Object(response)
}

// Sets the JSON RPC error value for a given response.
// If a counter label is supplied, also increments the invalid request counter using the label,
fn set_response_error(response: &mut Map<String, Value>, error: JsonRpcError, label: Option<&str>) {
    response.insert("error".to_string(), error.serialize());

    if let Some(label) = label {
        counters::INVALID_REQUESTS.with_label_values(&[label]).inc();
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

/// Warp rejection types
#[derive(Debug)]
struct DatabaseError;

impl Reject for DatabaseError {}
