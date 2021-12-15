// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::Context, index, tests::pretty};
use bytes::Bytes;
use diem_api_types::{
    mime_types, HexEncodedBytes, TransactionOnChainData, X_DIEM_CHAIN_ID, X_DIEM_LEDGER_TIMESTAMP,
    X_DIEM_LEDGER_VERSION,
};
use diem_config::config::{ApiConfig, JsonRpcConfig, RoleType};
use diem_crypto::{hash::HashValue, SigningKey};
use diem_genesis_tool::validator_builder::{RootKeys, ValidatorBuilder};
use diem_global_constants::OWNER_ACCOUNT;
use diem_mempool::mocks::MockSharedMempool;
use diem_sdk::{
    transaction_builder::{Currency, TransactionFactory},
    types::{
        account_config::{diem_root_address, treasury_compliance_account_address},
        transaction::SignedTransaction,
        AccountKey, LocalAccount,
    },
};
use diem_secure_storage::KVStorage;
use diem_temppath::TempPath;
use diem_types::{
    account_address::AccountAddress,
    account_config::testnet_dd_account_address,
    block_info::BlockInfo,
    block_metadata::BlockMetadata,
    chain_id::ChainId,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::VMPublishingOption,
    transaction::{Transaction, TransactionStatus},
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor::db_bootstrapper;
use executor_types::BlockExecutorTrait;
use hyper::Response;
use mempool_notifications::MempoolNotificationSender;
use storage_interface::DbReaderWriter;

use executor::block_executor::BlockExecutor;
use rand::{Rng, SeedableRng};
use serde_json::{json, Value};
use std::{boxed::Box, collections::BTreeMap, sync::Arc, time::SystemTime};
use vm_validator::vm_validator::VMValidator;
use warp::http::header::CONTENT_TYPE;

pub fn new_test_context() -> TestContext {
    let tmp_dir = TempPath::new();
    tmp_dir.create_as_dir().unwrap();

    let mut rng = ::rand::rngs::StdRng::from_seed(rand::rngs::OsRng.gen());
    let builder = ValidatorBuilder::new(
        &tmp_dir,
        diem_framework_releases::current_module_blobs().to_vec(),
    )
    .publishing_option(VMPublishingOption::open());

    let (root_keys, genesis, genesis_waypoint, validators) = builder.build(&mut rng).unwrap();
    let validator_owner = validators[0].storage().get(OWNER_ACCOUNT).unwrap().value;

    let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(&tmp_dir));
    let ret =
        db_bootstrapper::maybe_bootstrap::<DiemVM>(&db_rw, &genesis, genesis_waypoint).unwrap();
    assert!(ret);

    let mempool = MockSharedMempool::new_in_runtime(&db_rw, VMValidator::new(db.clone()));

    TestContext::new(
        Context::new(
            ChainId::test(),
            db.clone(),
            mempool.ac_client.clone(),
            RoleType::Validator,
            JsonRpcConfig::default(),
            ApiConfig::default(),
        ),
        rng,
        root_keys,
        validator_owner,
        Box::new(BlockExecutor::<DiemVM>::new(db_rw)),
        mempool,
        db,
    )
}

#[derive(Clone)]
pub struct TestContext {
    pub context: Context,
    pub validator_owner: AccountAddress,
    pub mempool: Arc<MockSharedMempool>,
    pub db: Arc<DiemDB>,
    rng: rand::rngs::StdRng,
    root_keys: Arc<RootKeys>,
    executor: Arc<dyn BlockExecutorTrait>,
    expect_status_code: u16,
}

impl TestContext {
    pub fn new(
        context: Context,
        rng: rand::rngs::StdRng,
        root_keys: RootKeys,
        validator_owner: AccountAddress,
        executor: Box<dyn BlockExecutorTrait>,
        mempool: MockSharedMempool,
        db: Arc<DiemDB>,
    ) -> Self {
        Self {
            context,
            rng,
            root_keys: Arc::new(root_keys),
            validator_owner,
            executor: executor.into(),
            mempool: Arc::new(mempool),
            expect_status_code: 200,
            db,
        }
    }

    pub fn rng(&mut self) -> &mut rand::rngs::StdRng {
        &mut self.rng
    }

    pub fn transaction_factory(&self) -> TransactionFactory {
        TransactionFactory::new(self.context.chain_id())
    }

    pub fn tc_account(&self) -> LocalAccount {
        LocalAccount::new(
            treasury_compliance_account_address(),
            self.root_keys.root_key.clone(),
            0,
        )
    }

    pub fn dd_account(&self) -> LocalAccount {
        LocalAccount::new(
            testnet_dd_account_address(),
            self.root_keys.root_key.clone(),
            0,
        )
    }

    pub fn root_account(&self) -> LocalAccount {
        LocalAccount::new(diem_root_address(), self.root_keys.root_key.clone(), 0)
    }

    pub fn gen_account(&mut self) -> LocalAccount {
        LocalAccount::generate(self.rng())
    }

    pub fn create_parent_vasp(&self, account: &LocalAccount) -> SignedTransaction {
        let mut tc = self.tc_account();
        self.create_parent_vasp_by_account(&mut tc, account)
    }

    pub fn create_parent_vasp_by_account(
        &self,
        creator: &mut LocalAccount,
        account: &LocalAccount,
    ) -> SignedTransaction {
        let factory = self.transaction_factory();
        creator.sign_with_transaction_builder(factory.create_parent_vasp_account(
            Currency::XUS,
            0,
            account.authentication_key(),
            "vasp",
            true,
        ))
    }

    pub fn create_invalid_signature_transaction(&mut self) -> SignedTransaction {
        let factory = self.transaction_factory();
        let tc_account = self.tc_account();
        let txn = factory
            .create_recovery_address()
            .sender(tc_account.address())
            .sequence_number(tc_account.sequence_number())
            .build();
        let invalid_key = AccountKey::generate(self.rng());
        txn.sign(invalid_key.private_key(), tc_account.public_key().clone())
            .unwrap()
            .into_inner()
    }

    pub fn get_latest_ledger_info(&self) -> diem_api_types::LedgerInfo {
        self.context.get_latest_ledger_info().unwrap()
    }

    pub fn get_transactions(&self, start: u64, limit: u16) -> Vec<TransactionOnChainData> {
        self.context
            .get_transactions(start, limit, self.get_latest_ledger_info().version())
            .unwrap()
    }

    pub fn expect_status_code(&self, status_code: u16) -> TestContext {
        let mut ret = self.clone();
        ret.expect_status_code = status_code;
        ret
    }

    pub async fn commit_mempool_txns(&self, size: u64) {
        let txns = self.mempool.get_txns(size);
        self.commit_block(&txns).await;
        for txn in txns {
            self.mempool.remove_txn(&txn);
        }
    }

    pub async fn commit_block(&self, signed_txns: &[SignedTransaction]) {
        let metadata = self.new_block_metadata();
        let timestamp = metadata.timestamp_usec();
        let txns: Vec<Transaction> = std::iter::once(Transaction::BlockMetadata(metadata.clone()))
            .chain(
                signed_txns
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            )
            .chain(std::iter::once(Transaction::StateCheckpoint))
            .collect();

        let parent_id = self.executor.committed_block_id();
        let result = self
            .executor
            .execute_block((metadata.id(), txns.clone()), parent_id)
            .unwrap();

        let mut compute_status = result.compute_status().clone();
        assert_eq!(compute_status.len(), txns.len(), "{:?}", result);
        if matches!(compute_status.last(), Some(TransactionStatus::Retry)) {
            // a state checkpoint txn can be Retry if prefixed by a write set txn
            compute_status.pop();
        }
        for st in &compute_status {
            match st {
                TransactionStatus::Discard(st) => panic!("transaction is discarded: {:?}", st),
                TransactionStatus::Retry => panic!("should not retry"),
                TransactionStatus::Keep(_) => (),
            }
        }
        self.executor
            .commit_blocks(
                vec![metadata.id()],
                self.new_ledger_info(&metadata, result.root_hash(), compute_status.len()),
            )
            .unwrap();

        self.mempool
            .mempool_notifier
            .notify_new_commit(txns, timestamp, 1000)
            .await
            .unwrap();
    }

    pub async fn api_get_account_resource(
        &self,
        account: &LocalAccount,
        type_name: String,
    ) -> serde_json::Value {
        let resources = self
            .get(&format!(
                "/accounts/{}/resources",
                account.address().to_hex_literal()
            ))
            .await;
        let vals: Vec<serde_json::Value> = serde_json::from_value(resources).unwrap();
        vals.into_iter().find(|v| v["type"] == type_name).unwrap()
    }

    pub async fn api_execute_script_function(
        &self,
        account: &mut LocalAccount,
        func_id: &str,
        type_args: serde_json::Value,
        args: serde_json::Value,
    ) {
        self.api_execute_txn(
            account,
            json!({
                "type": "script_function_payload",
                "function": format!("{}::{}", account.address().to_hex_literal(), func_id),
                "type_arguments": type_args,
                "arguments": args
            }),
        )
        .await;
    }

    pub async fn api_publish_module(&self, account: &mut LocalAccount, code: HexEncodedBytes) {
        self.api_execute_txn(
            account,
            json!({
                "type": "module_bundle_payload",
                "modules" : [
                    {"bytecode": code},
                ],
            }),
        )
        .await;
    }

    pub async fn api_execute_txn(&self, account: &mut LocalAccount, payload: Value) {
        let mut request = json!({
            "sender": account.address(),
            "sequence_number": account.sequence_number().to_string(),
            "gas_unit_price": "0",
            "max_gas_amount": "1000000",
            "gas_currency_code": "XUS",
            "expiration_timestamp_secs": "16373698888888",
            "payload": payload,
        });
        let resp = self
            .post("/transactions/signing_message", request.clone())
            .await;

        let signing_msg: HexEncodedBytes = resp["message"].as_str().unwrap().parse().unwrap();

        let sig = account
            .private_key()
            .sign_arbitrary_message(signing_msg.inner());

        request["signature"] = json!({
            "type": "ed25519_signature",
            "public_key": HexEncodedBytes::from(account.public_key().to_bytes().to_vec()),
            "signature": HexEncodedBytes::from(sig.to_bytes().to_vec()),
        });

        self.expect_status_code(202)
            .post("/transactions", request)
            .await;
        self.commit_mempool_txns(1).await;
        *account.sequence_number_mut() += 1;
    }

    pub async fn get(&self, path: &str) -> Value {
        self.execute(warp::test::request().method("GET").path(path))
            .await
    }

    pub async fn post(&self, path: &str, body: Value) -> Value {
        self.execute(warp::test::request().method("POST").path(path).json(&body))
            .await
    }

    pub async fn post_bcs_txn(&self, path: &str, body: impl AsRef<[u8]>) -> Value {
        self.execute(
            warp::test::request()
                .method("POST")
                .path(path)
                .header(CONTENT_TYPE, mime_types::BCS_SIGNED_TRANSACTION)
                .body(body),
        )
        .await
    }

    pub async fn reply(&self, req: warp::test::RequestBuilder) -> Response<Bytes> {
        req.reply(&index::routes(self.context.clone())).await
    }

    pub async fn execute(&self, req: warp::test::RequestBuilder) -> Value {
        let resp = self.reply(req).await;

        let headers = resp.headers();
        assert_eq!(headers[CONTENT_TYPE], mime_types::JSON);

        let body = serde_json::from_slice(resp.body()).expect("response body is JSON");
        assert_eq!(
            self.expect_status_code,
            resp.status(),
            "\nresponse: {}",
            pretty(&body)
        );

        if self.expect_status_code < 300 {
            let ledger_info = self.get_latest_ledger_info();
            assert_eq!(headers[X_DIEM_CHAIN_ID], "4");
            assert_eq!(
                headers[X_DIEM_LEDGER_VERSION],
                ledger_info.version().to_string()
            );
            assert_eq!(
                headers[X_DIEM_LEDGER_TIMESTAMP],
                ledger_info.timestamp().to_string()
            );
        }

        body
    }

    fn new_block_metadata(&self) -> BlockMetadata {
        let round = 1;
        let id = HashValue::random();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        BlockMetadata::new(id, round, timestamp, vec![], self.validator_owner)
    }

    fn new_ledger_info(
        &self,
        metadata: &BlockMetadata,
        root_hash: HashValue,
        block_size: usize,
    ) -> LedgerInfoWithSignatures {
        let parent = self
            .context
            .get_latest_ledger_info_with_signatures()
            .unwrap();
        let epoch = parent.ledger_info().epoch();
        let version = parent.ledger_info().version() + (block_size as u64);
        let info = LedgerInfo::new(
            BlockInfo::new(
                epoch,
                metadata.round(),
                metadata.id(),
                root_hash,
                version,
                metadata.timestamp_usec(),
                None,
            ),
            HashValue::random(),
        );
        LedgerInfoWithSignatures::new(info, BTreeMap::new())
    }
}
