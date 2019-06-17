// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Debug interface to access information in a specific node.

use crate::proto::{
    node_debug_interface::{
        DumpJemallocHeapProfileRequest, DumpJemallocHeapProfileResponse, GetNodeDetailsRequest,
        GetNodeDetailsResponse,
    },
    node_debug_interface_grpc::NodeDebugInterface,
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
        let mut response = GetNodeDetailsResponse::new();
        response.stats = metrics::get_all_metrics();
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger))
    }

    fn dump_jemalloc_heap_profile(
        &mut self,
        ctx: ::grpcio::RpcContext<'_>,
        _request: DumpJemallocHeapProfileRequest,
        sink: ::grpcio::UnarySink<DumpJemallocHeapProfileResponse>,
    ) {
        trace!("[GRPC] dump_jemalloc_heap_profile");
        let status_code = match jemalloc::dump_jemalloc_memory_profile() {
            Ok(_) => 0,
            Err(err_code) => err_code,
        };
        let mut resp = DumpJemallocHeapProfileResponse::new();
        resp.status_code = status_code;
        let f = sink.success(resp).map_err(default_reply_error_logger);
        ctx.spawn(f)
    }
}

fn default_reply_error_logger<T: ::std::fmt::Debug>(e: T) {
    COUNTER_ADMISSION_CONTROL_CANNOT_SEND_REPLY.inc();
    error!("Failed to reply error due to {:?}", e)
}
