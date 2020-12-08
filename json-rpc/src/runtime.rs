// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    errors::{is_internal_error, JsonRpcError},
    methods::{build_registry, JsonRpcRequest, JsonRpcService, RpcRegistry},
    response::{JsonRpcResponse, X_DIEM_CHAIN_ID, X_DIEM_TIMESTAMP_USEC_ID, X_DIEM_VERSION_ID},
};
use diem_config::config::{NodeConfig, RoleType};
use diem_logger::{debug, Level, Schema};
use diem_mempool::MempoolClientSender;
use diem_types::{chain_id::ChainId, ledger_info::LedgerInfoWithSignatures};
use futures::future::join_all;
use rand::{rngs::OsRng, RngCore};
use serde_json::{map::Map, Value};
use std::{net::SocketAddr, sync::Arc};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Runtime};
use warp::{
    http::header,
    reject::{self, Reject},
    Filter, Reply,
};

// Counter labels for runtime metrics
const LABEL_FAIL: &str = "fail";
const LABEL_SUCCESS: &str = "success";
const LABEL_BATCH: &str = "batch";
const LABEL_SINGLE: &str = "single";

#[derive(Schema)]
struct HttpRequestLog<'a> {
    #[schema(display)]
    remote_addr: Option<std::net::SocketAddr>,
    method: String,
    path: String,
    status: u16,
    referer: Option<&'a str>,
    user_agent: Option<&'a str>,
    #[schema(debug)]
    elapsed: std::time::Duration,
    forwarded: Option<&'a str>,
}

#[derive(Schema)]
struct RpcRequestLog {
    trace_id: String,
    request: Value,
}

#[derive(Schema)]
struct RpcResponseLog<'a> {
    trace_id: String,
    is_batch: bool,
    response_error: bool,
    response: &'a JsonRpcResponse,
}

#[macro_export]
macro_rules! log_response {
    ($trace_id: expr, $resp: expr, $is_batch: expr) => {
        let mut level = Level::Trace;
        if let Some(ref error) = $resp.error {
            if is_internal_error(&error.code) {
                level = Level::Error
            }
        }
        diem_logger::log!(
            level,
            RpcResponseLog {
                trace_id: $trace_id,
                is_batch: $is_batch,
                response_error: $resp.error.is_some(),
                response: $resp,
            }
        );
    };
}

/// Creates HTTP server (warp-based) that serves JSON RPC requests
/// Returns handle to corresponding Tokio runtime
pub fn bootstrap(
    address: SocketAddr,
    batch_size_limit: u16,
    page_size_limit: u16,
    content_len_limit: usize,
    diem_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
    role: RoleType,
    chain_id: ChainId,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("json-rpc")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[json-rpc] failed to create runtime");

    let registry = Arc::new(build_registry());
    let service = JsonRpcService::new(
        diem_db,
        mp_sender,
        role,
        chain_id,
        batch_size_limit,
        page_size_limit,
    );

    let base_route = warp::any()
        .and(warp::post())
        .and(warp::header::exact("content-type", "application/json"))
        .and(warp::body::content_length_limit(content_len_limit as u64))
        .and(warp::body::json())
        .and(warp::any().map(move || service.clone()))
        .and(warp::any().map(move || Arc::clone(&registry)))
        .and_then(rpc_endpoint)
        .with(warp::log::custom(|info| {
            debug!(HttpRequestLog {
                remote_addr: info.remote_addr(),
                method: info.method().to_string(),
                path: info.path().to_string(),
                status: info.status().as_u16(),
                referer: info.referer(),
                user_agent: info.user_agent(),
                elapsed: info.elapsed(),
                forwarded: info
                    .request_headers()
                    .get(header::FORWARDED)
                    .and_then(|v| v.to_str().ok())
            })
        }))
        // CORS is required for full node server to accept requests from different domain web pages.
        // It needs to be configured for the json-rpc request accepting method and headers.
        // Technically it's fine for any headers, but for simplicity we only set must have header
        // content-type.
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["POST"])
                .allow_headers(vec![header::CONTENT_TYPE]),
        );

    // For now we still allow user to use "/", but user should start to move to "/v1" soon
    let route_root = warp::path::end().and(base_route.clone());

    let route_v1 = warp::path::path("v1")
        .and(warp::path::end())
        .and(base_route);

    let health_route = warp::path!("-" / "healthy")
        .and(warp::path::end())
        .map(|| "diem-node:ok");

    let full_route = health_route.or(route_v1.or(route_root));

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
    chain_id: ChainId,
    diem_db: Arc<dyn DbReader>,
    mp_sender: MempoolClientSender,
) -> Runtime {
    bootstrap(
        config.json_rpc.address,
        config.json_rpc.batch_size_limit,
        config.json_rpc.page_size_limit,
        config.json_rpc.content_length_limit,
        diem_db,
        mp_sender,
        config.base.role,
        chain_id,
    )
}

/// JSON RPC entry point
/// Handles all incoming rpc requests
/// Performs routing based on methods defined in `registry`
pub(crate) async fn rpc_endpoint(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
) -> Result<warp::reply::Response, warp::Rejection> {
    let label = match data {
        Value::Array(_) => LABEL_BATCH,
        _ => LABEL_SINGLE,
    };
    counters::RPC_REQUESTS.with_label_values(&[label]).inc();
    let timer = counters::RPC_REQUEST_LATENCY
        .with_label_values(&[label])
        .start_timer();
    let ret = rpc_endpoint_without_metrics(data, service, registry).await;
    timer.stop_and_record();
    ret
}

async fn rpc_endpoint_without_metrics(
    data: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
) -> Result<warp::reply::Response, warp::Rejection> {
    // take snapshot of latest version of DB to be used across all requests, especially for batched requests
    let ledger_info = service
        .get_latest_ledger_info()
        .map_err(|_| reject::custom(DatabaseError))?;

    let mut rng = OsRng;
    let trace_id = format!("{:x}", rng.next_u64());
    debug!(RpcRequestLog {
        trace_id: trace_id.clone(),
        request: data.clone(),
    });
    let chain_id = service.chain_id();
    let latest_ledger_version = ledger_info.ledger_info().version();
    let latest_ledger_timestamp_usecs = ledger_info.ledger_info().timestamp_usecs();

    let resp = Ok(if let Value::Array(requests) = data {
        match service.validate_batch_size_limit(requests.len()) {
            Ok(_) => {
                // batch API call
                let futures = requests.into_iter().map(|req| {
                    rpc_request_handler(
                        req,
                        service.clone(),
                        Arc::clone(&registry),
                        ledger_info.clone(),
                        LABEL_BATCH,
                        trace_id.clone(),
                    )
                });
                let responses = join_all(futures).await;
                for resp in &responses {
                    log_response!(trace_id.clone(), &resp, true);
                }
                warp::reply::json(&responses)
            }
            Err(err) => {
                let mut resp = JsonRpcResponse::new(
                    chain_id,
                    latest_ledger_version,
                    latest_ledger_timestamp_usecs,
                );
                set_response_error(&mut resp, err, LABEL_BATCH, "unknown");
                log_response!(trace_id.clone(), &resp, true);

                warp::reply::json(&resp)
            }
        }
    } else {
        // single API call
        let resp = rpc_request_handler(
            data,
            service,
            registry,
            ledger_info,
            LABEL_SINGLE,
            trace_id.clone(),
        )
        .await;
        log_response!(trace_id, &resp, false);

        warp::reply::json(&resp)
    });

    let mut http_response = resp.into_response();
    let headers = http_response.headers_mut();

    headers.insert(
        X_DIEM_CHAIN_ID,
        header::HeaderValue::from_str(&chain_id.id().to_string()).unwrap(),
    );
    headers.insert(
        X_DIEM_VERSION_ID,
        header::HeaderValue::from_str(&latest_ledger_version.to_string()).unwrap(),
    );
    headers.insert(
        X_DIEM_TIMESTAMP_USEC_ID,
        header::HeaderValue::from_str(&latest_ledger_timestamp_usecs.to_string()).unwrap(),
    );

    Ok(http_response)
}

/// Handler of single RPC request
/// Performs validation and executes corresponding rpc handler
async fn rpc_request_handler(
    req: Value,
    service: JsonRpcService,
    registry: Arc<RpcRegistry>,
    ledger_info: LedgerInfoWithSignatures,
    request_type_label: &str,
    trace_id: String,
) -> JsonRpcResponse {
    let request: Map<String, Value>;
    let mut response = JsonRpcResponse::new(
        service.chain_id(),
        ledger_info.ledger_info().version(),
        ledger_info.ledger_info().timestamp_usecs(),
    );

    match req {
        Value::Object(data) => {
            request = data;
        }
        _ => {
            set_response_error(
                &mut response,
                JsonRpcError::invalid_format(),
                request_type_label,
                "unknown",
            );
            return response;
        }
    }

    // parse request id
    match parse_request_id(&request) {
        Ok(request_id) => {
            response.id = Some(request_id);
        }
        Err(err) => {
            set_response_error(&mut response, err, request_type_label, "unknown");
            return response;
        }
    };

    // verify protocol version
    if let Err(err) = verify_protocol(&request) {
        set_response_error(&mut response, err, request_type_label, "unknown");
        return response;
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
                JsonRpcError::invalid_params(None),
                request_type_label,
                "unknown",
            );
            return response;
        }
    }

    let request_params = JsonRpcRequest {
        trace_id,
        ledger_info,
        params,
    };
    // get rpc handler
    match request.get("method") {
        Some(Value::String(name)) => match registry.get(name) {
            Some(handler) => {
                let timer = counters::METHOD_LATENCY
                    .with_label_values(&[request_type_label, name])
                    .start_timer();
                match handler(service, request_params).await {
                    Ok(result) => {
                        response.result = Some(result);
                        counters::REQUESTS
                            .with_label_values(&[request_type_label, name, LABEL_SUCCESS])
                            .inc();
                    }
                    Err(err) => {
                        // check for custom error
                        set_response_error(
                            &mut response,
                            err.downcast_ref::<JsonRpcError>()
                                .cloned()
                                .unwrap_or_else(|| JsonRpcError::internal_error(err.to_string())),
                            request_type_label,
                            &name,
                        );
                        counters::REQUESTS
                            .with_label_values(&[request_type_label, name, LABEL_FAIL])
                            .inc();
                    }
                }
                timer.stop_and_record();
            }
            None => set_response_error(
                &mut response,
                JsonRpcError::method_not_found(),
                request_type_label,
                "not_found",
            ),
        },
        _ => set_response_error(
            &mut response,
            JsonRpcError::method_not_found(),
            request_type_label,
            "not_found",
        ),
    }

    response
}

// Sets the JSON RPC error value for a given response.
// If a counter label is supplied, also increments the invalid request counter using the label,
fn set_response_error(
    response: &mut JsonRpcResponse,
    error: JsonRpcError,
    request_type: &str,
    method: &str,
) {
    let err_code = error.code;
    if is_internal_error(&error.code) {
        counters::INTERNAL_ERRORS
            .with_label_values(&[request_type, method, &err_code.to_string()])
            .inc();
    } else {
        let label = match err_code {
            -32600 => "invalid_request",
            -32601 => "method_not_found",
            -32602 => "invalid_params",
            -32604 => "invalid_format",
            _ => "unexpected_code",
        };
        counters::INVALID_REQUESTS
            .with_label_values(&[request_type, method, label])
            .inc();
    }

    response.error = Some(error);
}

fn parse_request_id(request: &Map<String, Value>) -> Result<Value, JsonRpcError> {
    match request.get("id") {
        Some(req_id) => {
            if req_id.is_string() || req_id.is_number() || req_id.is_null() {
                Ok(req_id.clone())
            } else {
                Err(JsonRpcError::invalid_format())
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
