// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(missing_docs)]

use ::libra_types::proto::*;
use libra_mempool_shared_proto::proto::mempool_status;

pub mod mempool {
    tonic::include_proto!("mempool");
}

pub mod mempool_client {
    use super::mempool::{
        mempool_client::MempoolClient, AddTransactionWithValidationRequest,
        AddTransactionWithValidationResponse, CommitTransactionsRequest,
        CommitTransactionsResponse, GetBlockRequest, GetBlockResponse, HealthCheckRequest,
        HealthCheckResponse,
    };

    #[tonic::async_trait]
    pub trait MempoolClientTrait: Clone + Send + Sync {
        async fn add_transaction_with_validation(
            &mut self,
            _request: AddTransactionWithValidationRequest,
        ) -> Result<AddTransactionWithValidationResponse, tonic::Status> {
            unimplemented!();
        }

        async fn health_check(
            &mut self,
            _request: HealthCheckRequest,
        ) -> Result<HealthCheckResponse, tonic::Status> {
            unimplemented!();
        }
    }

    #[tonic::async_trait]
    impl MempoolClientTrait for MempoolClientWrapper {
        async fn add_transaction_with_validation(
            &mut self,
            request: AddTransactionWithValidationRequest,
        ) -> Result<AddTransactionWithValidationResponse, tonic::Status> {
            self.add_transaction_with_validation(request).await
        }

        async fn health_check(
            &mut self,
            request: HealthCheckRequest,
        ) -> Result<HealthCheckResponse, tonic::Status> {
            self.health_check(request).await
        }
    }

    // Allow for lazily creating a Client
    #[derive(Clone)]
    pub struct MempoolClientWrapper {
        addr: String,
        client: Option<MempoolClient<tonic::transport::Channel>>,
    }

    impl MempoolClientWrapper {
        pub fn new<A: AsRef<str>>(address: A, port: u16) -> Self {
            let addr = format!("http://{}:{}", address.as_ref(), port);

            Self { client: None, addr }
        }

        async fn client(
            &mut self,
        ) -> Result<&mut MempoolClient<tonic::transport::Channel>, tonic::Status> {
            if self.client.is_none() {
                self.client = Some(
                    super::mempool_client::MempoolClient::connect(self.addr.clone())
                        .await
                        .map_err(|e| tonic::Status::new(tonic::Code::Unavailable, e.to_string()))?,
                );
            }

            // client is guaranteed to be populated by the time we reach here
            Ok(self.client.as_mut().unwrap())
        }

        pub async fn add_transaction_with_validation(
            &mut self,
            request: AddTransactionWithValidationRequest,
        ) -> Result<AddTransactionWithValidationResponse, tonic::Status> {
            self.client()
                .await?
                .add_transaction_with_validation(request)
                .await
                .map(tonic::Response::into_inner)
        }

        pub async fn get_block(
            &mut self,
            request: GetBlockRequest,
        ) -> Result<GetBlockResponse, tonic::Status> {
            self.client()
                .await?
                .get_block(request)
                .await
                .map(tonic::Response::into_inner)
        }

        pub async fn commit_transactions(
            &mut self,
            request: CommitTransactionsRequest,
        ) -> Result<CommitTransactionsResponse, tonic::Status> {
            self.client()
                .await?
                .commit_transactions(request)
                .await
                .map(tonic::Response::into_inner)
        }

        pub async fn health_check(
            &mut self,
            request: HealthCheckRequest,
        ) -> Result<HealthCheckResponse, tonic::Status> {
            self.client()
                .await?
                .health_check(request)
                .await
                .map(tonic::Response::into_inner)
        }
    }
}
