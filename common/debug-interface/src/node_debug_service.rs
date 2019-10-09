// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use crate::{
    json_log,
    proto::{
        Event, GetEventsRequest, GetEventsResponse, GetNodeDetailsRequest, GetNodeDetailsResponse,
        NodeDebugInterface,
    },
};
use futures::Future;
use logger::prelude::*;
use metrics::counters::COUNTER_ADMISSION_CONTROL_CANNOT_SEND_REPLY;

#[derive(Clone, Default)]
pub struct NodeDebugService {}

impl NodeDebugService {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeDebugInterface for NodeDebugService {
    fn get_node_details(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        _req: GetNodeDetailsRequest,
        sink: ::grpcio::UnarySink<GetNodeDetailsResponse>,
    ) {
        info!("[GRPC] get_node_details");
        let mut response = GetNodeDetailsResponse::default();
        response.stats = metrics::get_all_metrics();
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger))
    }

    fn get_events(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        _req: GetEventsRequest,
        sink: ::grpcio::UnarySink<GetEventsResponse>,
    ) {
        let mut response = GetEventsResponse::default();
        for event in json_log::pop_last_entries() {
            let mut response_event = Event::default();
            response_event.name = event.name.to_string();
            response_event.timestamp = event.timestamp as i64;
            let serialized_event =
                serde_json::to_string(&event.json).expect("Failed to serialize event to json");
            response_event.json = serialized_event;
            response.events.push(response_event);
        }
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger))
    }
}

fn default_reply_error_logger<T: ::std::fmt::Debug>(e: T) {
    COUNTER_ADMISSION_CONTROL_CANNOT_SEND_REPLY.inc();
    error!("Failed to reply error due to {:?}", e)
}
