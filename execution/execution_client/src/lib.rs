// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::ed25519::*;
use execution_proto::{
    proto::{execution::CommitBlockRequest, execution_grpc},
    ExecuteBlockRequest, ExecuteBlockResponse,
};
use failure::{bail, Result};
use grpcio::{ChannelBuilder, Environment};
use proto_conv::{FromProto, IntoProto};
use std::sync::Arc;
use types::ledger_info::LedgerInfoWithSignatures;

pub struct ExecutionClient {
    client: execution_grpc::ExecutionClient,
}

impl ExecutionClient {
    pub fn new(env: Arc<Environment>, host: &str, port: u16) -> Self {
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", host, port));
        let client = execution_grpc::ExecutionClient::new(channel);
        ExecutionClient { client }
    }

    pub fn execute_block(&self, request: ExecuteBlockRequest) -> Result<ExecuteBlockResponse> {
        let proto_request = request.into_proto();
        match self.client.execute_block(&proto_request) {
            Ok(proto_response) => Ok(ExecuteBlockResponse::from_proto(proto_response)?),
            Err(err) => bail!("GRPC error: {}", err),
        }
    }

    pub fn commit_block(
        &self,
        ledger_info_with_sigs: LedgerInfoWithSignatures<Ed25519Signature>,
    ) -> Result<()> {
        let proto_ledger_info = ledger_info_with_sigs.into_proto();
        let mut request = CommitBlockRequest::new();
        request.set_ledger_info_with_sigs(proto_ledger_info);
        match self.client.commit_block(&request) {
            Ok(_proto_response) => Ok(()),
            Err(err) => bail!("GRPC error: {}", err),
        }
    }
}
