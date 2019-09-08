// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::config::NodeConfig;
use execution_proto::{CommitBlockRequest, ExecuteBlockRequest, ExecuteChunkRequest};
use executor::Executor;
use failure::Result;
use futures01::future::Future;
use futures03::{
    channel::oneshot,
    future::{FutureExt, TryFutureExt},
};
use grpc_helpers::default_reply_error_logger;
use grpcio::{RpcStatus, RpcStatusCode};
use proto_conv::{FromProto, IntoProto};
use std::sync::Arc;
use storage_client::{StorageRead, StorageWrite};
use vm_runtime::MoveVM;

#[derive(Clone)]
pub struct ExecutionService {
    /// `ExecutionService` simply contains an `Executor` and uses it to process requests. We wrap
    /// it in `Arc` because `ExecutionService` has to implement `Clone`.
    executor: Arc<Executor<MoveVM>>,
}

impl ExecutionService {
    /// Constructs an `ExecutionService`.
    pub fn new(
        storage_read_client: Arc<dyn StorageRead>,
        storage_write_client: Arc<dyn StorageWrite>,
        config: &NodeConfig,
    ) -> Self {
        let executor = Arc::new(Executor::new(
            storage_read_client,
            storage_write_client,
            config,
        ));
        ExecutionService { executor }
    }
}

impl execution_proto::proto::execution_grpc::Execution for ExecutionService {
    fn execute_block(
        &mut self,
        ctx: grpcio::RpcContext,
        request: execution_proto::proto::execution::ExecuteBlockRequest,
        sink: grpcio::UnarySink<execution_proto::proto::execution::ExecuteBlockResponse>,
    ) {
        match ExecuteBlockRequest::from_proto(request) {
            Ok(req) => {
                let fut = process_response(
                    self.executor.execute_block(
                        req.transactions,
                        req.parent_block_id,
                        req.block_id,
                    ),
                    sink,
                )
                .boxed()
                .unit_error()
                .compat();
                ctx.spawn(fut);
            }
            Err(err) => {
                let fut = process_conversion_error(err, sink);
                ctx.spawn(fut);
            }
        }
    }

    fn commit_block(
        &mut self,
        ctx: grpcio::RpcContext,
        request: execution_proto::proto::execution::CommitBlockRequest,
        sink: grpcio::UnarySink<execution_proto::proto::execution::CommitBlockResponse>,
    ) {
        match CommitBlockRequest::from_proto(request) {
            Ok(req) => {
                let fut =
                    process_response(self.executor.commit_block(req.ledger_info_with_sigs), sink)
                        .boxed()
                        .unit_error()
                        .compat();
                ctx.spawn(fut);
            }
            Err(err) => {
                let fut = process_conversion_error(err, sink);
                ctx.spawn(fut);
            }
        }
    }

    fn execute_chunk(
        &mut self,
        ctx: grpcio::RpcContext,
        request: execution_proto::proto::execution::ExecuteChunkRequest,
        sink: grpcio::UnarySink<execution_proto::proto::execution::ExecuteChunkResponse>,
    ) {
        match ExecuteChunkRequest::from_proto(request) {
            Ok(req) => {
                let fut = process_response(
                    self.executor
                        .execute_chunk(req.txn_list_with_proof, req.ledger_info_with_sigs),
                    sink,
                )
                .boxed()
                .unit_error()
                .compat();
                ctx.spawn(fut);
            }
            Err(err) => {
                let fut = process_conversion_error(err, sink);
                ctx.spawn(fut);
            }
        }
    }
}

async fn process_response<T>(
    resp: oneshot::Receiver<Result<T>>,
    sink: grpcio::UnarySink<<T as IntoProto>::ProtoType>,
) where
    T: IntoProto,
{
    match resp.await {
        Ok(Ok(response)) => {
            sink.success(response.into_proto());
        }
        Ok(Err(err)) => {
            set_failure_message(
                RpcStatusCode::Unknown,
                format!("Failed to process request: {}", err),
                sink,
            );
        }
        Err(oneshot::Canceled) => {
            set_failure_message(
                RpcStatusCode::Internal,
                "Executor Internal error: sender is dropped.".to_string(),
                sink,
            );
        }
    }
}

fn process_conversion_error<T>(
    err: failure::Error,
    sink: grpcio::UnarySink<T>,
) -> impl Future<Item = (), Error = ()> {
    set_failure_message(
        RpcStatusCode::InvalidArgument,
        format!("Failed to convert request from Protobuf: {}", err),
        sink,
    )
    .map_err(default_reply_error_logger)
}

fn set_failure_message<T>(
    status_code: RpcStatusCode,
    details: String,
    sink: grpcio::UnarySink<T>,
) -> grpcio::UnarySinkResult {
    let status = RpcStatus::new(status_code, Some(details));
    sink.fail(status)
}
