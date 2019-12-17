// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

use ::libra_types::proto::*;
use libra_mempool_shared_proto::proto::mempool_status;
use tokio::runtime::{Builder, Runtime};

pub mod admission_control {
    tonic::include_proto!("admission_control");
}

pub use self::admission_control::{
    admission_control_client::AdmissionControlClient, AdmissionControlMsg,
    SubmitTransactionRequest, SubmitTransactionResponse,
};

pub struct AdmissionControlClientBlocking {
    // Currently the runtime but be ordered before the tonic client to ensure that the runtime is
    // dropped last when this struct is dropped.
    // See https://github.com/tokio-rs/tokio/issues/1948 for more info.
    rt: Runtime,
    addr: String,
    client: Option<AdmissionControlClient<tonic::transport::Channel>>,
}

impl AdmissionControlClientBlocking {
    pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
        let rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        let addr = format!("http://{}:{}", address.as_ref(), port);

        Self {
            client: None,
            addr,
            rt,
        }
    }

    fn client(
        &mut self,
    ) -> Result<
        (
            &mut Runtime,
            &mut AdmissionControlClient<tonic::transport::Channel>,
        ),
        tonic::Status,
    > {
        if self.client.is_none() {
            self.client = Some(
                self.rt
                    .block_on(AdmissionControlClient::connect(self.addr.clone()))
                    .map_err(|e| tonic::Status::new(tonic::Code::Unavailable, e.to_string()))?,
            );
        }

        // client is guaranteed to be populated by the time we reach here
        Ok((&mut self.rt, self.client.as_mut().unwrap()))
    }

    pub fn submit_transaction(
        &mut self,
        request: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, tonic::Status> {
        let (rt, client) = self.client()?;
        rt.block_on(client.submit_transaction(request))
            .map(tonic::Response::into_inner)
    }

    pub fn update_to_latest_ledger(
        &mut self,
        request: types::UpdateToLatestLedgerRequest,
    ) -> Result<types::UpdateToLatestLedgerResponse, tonic::Status> {
        let (rt, client) = self.client()?;
        rt.block_on(async {
            tokio::time::timeout(
                std::time::Duration::from_millis(5000),
                client.update_to_latest_ledger(request),
            )
            .await
        })
        .map_err(|_| tonic::Status::new(tonic::Code::DeadlineExceeded, ""))?
        .map(tonic::Response::into_inner)
    }
}
