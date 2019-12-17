// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{mocks::local_mock_mempool::LocalMockMempool, upstream_proxy};
use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use channel;
use futures::executor::block_on;
use libra_config::config::{AdmissionControlConfig, RoleType};
use libra_proptest_helpers::ValueGenerator;
use libra_prost_ext::MessageExt;
use libra_types::transaction::SignedTransaction;
use network::validator_network::AdmissionControlNetworkSender;
use proptest;
use prost::Message;
use std::sync::Arc;
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

#[test]
fn test_fuzzer() {
    let mut gen = ValueGenerator::new();
    let data = generate_corpus(&mut gen);
    fuzzer(&data);
}

/// generate_corpus produces an arbitrary SubmitTransactionRequest for admission control
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    // use proptest to generate a SignedTransaction
    let signed_txn = gen.generate(proptest::arbitrary::any::<SignedTransaction>());
    // wrap it in a SubmitTransactionRequest
    let mut req = SubmitTransactionRequest::default();
    req.transaction = Some(signed_txn.into());

    req.to_vec().unwrap()
}

/// fuzzer takes a serialized SubmitTransactionRequest an process it with an admission control
/// service
pub fn fuzzer(data: &[u8]) {
    // parse SubmitTransactionRequest
    let req = match SubmitTransactionRequest::decode(data) {
        Ok(value) => value,
        Err(_) => {
            if cfg!(test) {
                panic!();
            }
            return;
        }
    };

    let (network_reqs_tx, _) = channel::new_test(8);
    let network_sender = AdmissionControlNetworkSender::new(network_reqs_tx);

    let upstream_proxy_data = upstream_proxy::UpstreamProxyData::new(
        AdmissionControlConfig::default(),
        network_sender,
        RoleType::Validator,
        Some(LocalMockMempool::new()),
        Arc::new(MockStorageReadClient),
        Arc::new(MockVMValidator),
        false,
    );

    // process the request
    let res = block_on(upstream_proxy::submit_transaction_to_mempool(
        upstream_proxy_data,
        req,
    ));
    if cfg!(test) && res.is_err() {
        panic!();
    }
}
