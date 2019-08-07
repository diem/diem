// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::{
    state_synchronizer::{CommitTransactionsRequest, CommitTransactionsResponse},
    state_synchronizer_grpc::StateSynchronizer,
};
use futures::future::Future;
use grpc_helpers::default_reply_error_logger;
use logger::prelude::*;

#[derive(Clone)]
pub(crate) struct StateSyncService;

impl StateSynchronizer for StateSyncService {
    fn commit_transactions(
        &mut self,
        ctx: ::grpcio::RpcContext,
        _req: CommitTransactionsRequest,
        sink: ::grpcio::UnarySink<CommitTransactionsResponse>,
    ) {
        trace!("[GRPC] StateSynchronizer::commit_transactions");
        let response = CommitTransactionsResponse::new();
        ctx.spawn(sink.success(response).map_err(default_reply_error_logger));
    }
}
