// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(missing_docs)]

use ::libra_types::proto::*;
use mempool_shared_proto::proto::mempool_status;

pub mod mempool {
    include!(concat!(env!("OUT_DIR"), "/mempool.rs"));
}

pub mod mempool_client {
    pub trait MempoolClientTrait: Clone + Send + Sync {
        fn add_transaction_with_validation(
            &self,
            _req: &super::mempool::AddTransactionWithValidationRequest,
        ) -> ::grpcio::Result<super::mempool::AddTransactionWithValidationResponse> {
            unimplemented!();
        }

        fn health_check(
            &self,
            _req: &super::mempool::HealthCheckRequest,
        ) -> ::grpcio::Result<super::mempool::HealthCheckResponse> {
            unimplemented!();
        }
    }

    impl MempoolClientTrait for super::mempool::MempoolClient {
        fn add_transaction_with_validation(
            &self,
            req: &super::mempool::AddTransactionWithValidationRequest,
        ) -> ::grpcio::Result<super::mempool::AddTransactionWithValidationResponse> {
            self.add_transaction_with_validation(req)
        }

        fn health_check(
            &self,
            req: &super::mempool::HealthCheckRequest,
        ) -> ::grpcio::Result<super::mempool::HealthCheckResponse> {
            self.health_check(req)
        }
    }
}
