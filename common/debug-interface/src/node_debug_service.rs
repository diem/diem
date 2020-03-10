// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use crate::{
    json_log,
    json_log::JsonLogEntry,
    proto::{
        node_debug_interface_server::NodeDebugInterface, Event, GetEventsRequest,
        GetEventsResponse, GetNodeDetailsRequest, GetNodeDetailsResponse,
    },
};
use libra_logger::prelude::*;
use tonic::{Request, Response, Status};

#[derive(Clone, Default)]
pub struct NodeDebugService;

impl NodeDebugService {
    pub fn new() -> Self {
        Default::default()
    }
}

#[tonic::async_trait]
impl NodeDebugInterface for NodeDebugService {
    async fn get_node_details(
        &self,
        _request: Request<GetNodeDetailsRequest>,
    ) -> Result<Response<GetNodeDetailsResponse>, Status> {
        info!("[GRPC] get_node_details");

        let mut response = GetNodeDetailsResponse::default();
        response.stats = libra_metrics::get_all_metrics();
        Ok(Response::new(response))
    }

    async fn get_events(
        &self,
        _request: Request<GetEventsRequest>,
    ) -> Result<Response<GetEventsResponse>, Status> {
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
        Ok(Response::new(response))
    }
}

pub fn parse_events(events: Vec<Event>) -> Vec<JsonLogEntry> {
    let mut ret = vec![];
    for event in events.into_iter() {
        ret.push(parse_event(event));
    }
    ret
}

pub fn parse_event(event: Event) -> JsonLogEntry {
    let json = serde_json::from_str(&event.json).expect("Failed to parse json");
    JsonLogEntry {
        name: Box::leak(event.name.into_boxed_str()),
        timestamp: event.timestamp as u128,
        json,
    }
}
