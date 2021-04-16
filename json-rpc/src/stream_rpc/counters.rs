// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{register_int_counter_vec, IntCounterVec};
use once_cell::sync::Lazy;

pub enum RpcResult {
    Success,
    Error,
}

impl RpcResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            RpcResult::Success => "success",
            RpcResult::Error => "error",
        }
    }
}

pub static HTTP_WEBSOCKET_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_http_websocket_requests_count",
        "Cumulative number of http requests that the JSON RPC streaming service receives",
        &[
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver",  // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

pub static HTTP_WEBSOCKET_REQUEST_UPGRADES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_http_websocket_upgrades_count",
        "Cumulative number of http websocket requests which successfully get upgraded that the JSON RPC streaming service receives",
        &[
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
        .unwrap()
});

/// number of messages received. These are raw messages, before parsing
pub static MESSAGES_RECEIVED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_messages_received",
        "Cumulative number of messages that the Streaming RPC service receives",
        &[
            "transport", // transport used: ws, ipc, etc
            "sdk_lang",  // the language of the SDK: "java", "python", etc
            "sdk_ver",   // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// number of messages sent
pub static MESSAGES_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_messages_sent",
        "Cumulative number of messages that the Streaming RPC service sends",
        &[
            "transport", // transport used: ws, ipc, etc
            "sdk_lang",  // the language of the SDK: "java", "python", etc
            "sdk_ver",   // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// number of messages received. These are messages after parsing (subscription requests)
pub static SUBSCRIPTION_RPC_RECEIVED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_subscription_rpc_received",
        "Cumulative number of RPC messages that the Streaming RPC service receives, parses successfully, and responds to",
        &[
            "transport", // transport used: ws, ipc, etc
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "result", // result of request: "success", "fail"
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
        .unwrap()
});

/// number of messages sent. These are in response to subscriptions.
pub static SUBSCRIPTION_RPC_SENT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_subscription_rpc_sent",
        "Cumulative number of RPC messages that the Streaming RPC service sends as part of subscriptions",
        &[
            "transport", // transport used: ws, ipc, etc
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "result", // result of request: "success", "fail"
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// number of 'OK' messages sent. These are sent when a subscription is started.
pub static SUBSCRIPTION_OKS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_subscription_oks",
        "Cumulative number of 'OK' messages sent in response to starting a subscription",
        &[
            "transport", // transport used: ws, ipc, etc
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "result", // result of request: "success", "fail"
            "sdk_lang", // the language of the SDK: "java", "python", etc
            "sdk_ver", // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

/// Cumulative number of clients connected
pub static CLIENT_CONNECTED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_clients_count",
        "Cumulative number of connected clients",
        &[
            "transport", // transport used: ws, ipc, etc
            "sdk_lang",  // the language of the SDK: "java", "python", etc
            "sdk_ver",   // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});

pub static INVALID_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_streaming_rpc_invalid_requests_count",
        "Cumulative number of invalid requests that JSON RPC client service receives",
        &[
            "transport", // transport used: ws, ipc, etc
            "method", // method of request, matches JSON RPC method name (e.g. "submit", "get_account")
            "errortype", // categories of invalid requests: "invalid_format", "invalid_params", "invalid_method", "method_not_found"
            "sdk_lang",  // the language of the SDK: "java", "python", etc
            "sdk_ver",   // the version of the SDK: "0.0.11", "1.45.31", etc
        ]
    )
    .unwrap()
});
