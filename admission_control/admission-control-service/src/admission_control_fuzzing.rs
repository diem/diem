// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::AdmissionControlService;
use crate::{
    admission_control_service::SubmitTransactionRequest,
    mocks::local_mock_mempool::LocalMockMempool,
};
use futures::channel::mpsc;
use libra_proptest_helpers::ValueGenerator;
use libra_types::transaction::SignedTransaction;
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

    let mut bytes = bytes::BytesMut::with_capacity(req.encoded_len());
    req.encode(&mut bytes).unwrap();
    bytes.to_vec()
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
    let (upstream_proxy_sender, _) = mpsc::unbounded();

    // create service to receive it
    let ac_service = AdmissionControlService::new(
        Some(Arc::new(LocalMockMempool::new())),
        Arc::new(MockStorageReadClient),
        Arc::new(MockVMValidator),
        false,
        upstream_proxy_sender,
    );

    // process the request
    let res = ac_service.submit_transaction_inner(req);
    if cfg!(test) && res.is_err() {
        panic!();
    }
}
