// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use libra_crypto::{hash::CryptoHash, traits::SigningKey};
use libra_json_rpc_types::response::JsonRpcResponse;
use libra_types::{
    account_address::AccountAddress,
    account_config::{
        lbr_type_tag, testnet_dd_account_address, treasury_compliance_account_address, LBR_NAME,
    },
    chain_id::ChainId,
    transaction::SignedTransaction,
};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct Env {
    pub url: String,
    pub tc: Account,
    pub dd: Account,
    pub vasps: Vec<Account>,
    pub client: reqwest::blocking::Client,
    allow_execution_failures: bool,
}

impl Env {
    pub fn gen(root_private_key: libra_crypto::ed25519::Ed25519PrivateKey, url: String) -> Self {
        Self {
            url,
            tc: Account::new_with_address(
                treasury_compliance_account_address(),
                root_private_key.clone(),
            ),
            dd: Account::new_with_address(testnet_dd_account_address(), root_private_key),
            vasps: vec![],
            client: reqwest::blocking::Client::new(),
            allow_execution_failures: false,
        }
    }

    pub fn init_vasps(&mut self, num: usize) {
        for i in 0..num {
            self.create_parent_vasp();
            self.transfer_coins_to_vasp(i, 1_000_000_000_000);
        }
        for i in 0..num {
            self.create_child_vasp(i, 3_000_000_000);
        }
    }

    pub fn gen_vasps_transfers(&mut self) {
        let num = self.vasps.len();
        for i in 0..num {
            for j in 0..num {
                if i == j {
                    continue;
                }
                let amount = ((1 + i + j) * 1_000_000) as u64;
                self.transfer_coins((i as usize, j as usize), (j as usize, i as usize), amount);
            }
        }
    }

    pub fn create_parent_vasp(&mut self) {
        let vasp = Account::gen();
        let script =
            transaction_builder_generated::stdlib::encode_create_parent_vasp_account_script(
                lbr_type_tag(),
                0, // sliding nonce
                vasp.address,
                vasp.auth_key().prefix().to_vec(),
                format!("Novi {}", self.vasps.len()).as_bytes().to_owned(),
                false, /* add all currencies */
            );
        let txn = self.create_txn(&self.tc, script);
        self.submit_and_wait(txn);
        self.vasps.push(vasp);
    }

    pub fn create_child_vasp(&mut self, parent_vasp_index: usize, amount: u64) {
        let child = Account::gen();
        let script = transaction_builder_generated::stdlib::encode_create_child_vasp_account_script(
            lbr_type_tag(),
            child.address,
            child.auth_key().prefix().to_vec(),
            false, /* add all currencies */
            amount,
        );
        let txn = self.create_txn(&self.vasps[parent_vasp_index], script);
        self.submit_and_wait(txn);
        self.vasps[parent_vasp_index].children.push(child);
    }

    pub fn transfer_coins_to_vasp(&mut self, index: usize, amount: u64) {
        let script =
            transaction_builder_generated::stdlib::encode_peer_to_peer_with_metadata_script(
                lbr_type_tag(),
                self.vasps[index].address,
                amount,
                vec![],
                vec![],
            );
        let txn = self.create_txn(&self.dd, script);
        self.submit_and_wait(txn);
    }

    pub fn transfer_coins(
        &mut self,
        sender: (usize, usize),
        receiver: (usize, usize),
        amount: u64,
    ) -> SignedTransaction {
        let txn = self.transfer_coins_txn(sender, receiver, amount);
        self.submit_and_wait(txn.clone());
        txn
    }

    pub fn transfer_coins_txn(
        &mut self,
        sender: (usize, usize),
        receiver: (usize, usize),
        amount: u64,
    ) -> SignedTransaction {
        let (rid, rcid) = receiver;
        let receiver_address = self.vasps[rid].children[rcid].address;
        let script =
            transaction_builder_generated::stdlib::encode_peer_to_peer_with_metadata_script(
                lbr_type_tag(),
                receiver_address,
                amount,
                // todo: add metadata
                vec![],
                vec![],
            );
        self.create_txn(&self.vasps[sender.0].children[sender.1], script)
    }

    pub fn get_account_sequence(&self, address: String) -> Result<u64> {
        let resp = self.send("get_account", json!([address]));
        if let Some(result) = resp.result {
            if result.is_object() {
                return Ok(result["sequence_number"].as_u64().unwrap());
            }
        }
        Err(format_err!("account not found: {}", address))
    }

    pub fn create_txn(
        &self,
        account: &Account,
        script: libra_types::transaction::Script,
    ) -> SignedTransaction {
        let seq = self
            .get_account_sequence(account.address.to_string())
            .expect("account should exist onchain for create transaction");
        libra_types::transaction::helpers::create_user_txn(
            account,
            libra_types::transaction::TransactionPayload::Script(script),
            account.address,
            seq,
            1_000_000,
            0,
            LBR_NAME.to_owned(),
            30,
            ChainId::test(),
        )
        .expect("user signed transaction")
    }

    pub fn allow_execution_failures<T>(&mut self, f: fn(&mut Self) -> T) -> T {
        self.allow_execution_failures = true;
        let ret = f(self);
        self.allow_execution_failures = false;
        ret
    }

    pub fn submit_and_wait(&mut self, txn: SignedTransaction) -> Value {
        self.submit(&txn);
        self.wait_for_txn(&txn)
    }

    pub fn submit(&self, txn: &SignedTransaction) -> JsonRpcResponse {
        let txn_hex = hex::encode(lcs::to_bytes(txn).expect("lcs txn failed"));
        self.send("submit", json!([txn_hex]))
    }

    pub fn get_account_transaction(
        &self,
        address: &AccountAddress,
        seq_num: u64,
        include_events: bool,
    ) -> JsonRpcResponse {
        self.send(
            "get_account_transaction",
            json!([hex::encode(address), seq_num, include_events]),
        )
    }

    pub fn wait_for_txn(&self, txn: &SignedTransaction) -> Value {
        let txn_hash = libra_types::transaction::Transaction::UserTransaction(txn.clone())
            .hash()
            .to_hex();
        for _i in 0..60 {
            let resp = self.get_account_transaction(&txn.sender(), txn.sequence_number(), true);
            if let Some(result) = resp.result {
                if result.is_object() {
                    if !self.allow_execution_failures {
                        assert_eq!(result["vm_status"]["type"], "executed", "{:#}", result);
                    }
                    assert_eq!(result["hash"], txn_hash, "{:#}", result);
                    return result;
                }
            }
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
        panic!("transaction not executed?");
    }

    pub fn send(&self, method: &'static str, params: Value) -> JsonRpcResponse {
        let request = json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1});
        let resp = self
            .client
            .post(self.url.as_str())
            .json(&request)
            .send()
            .expect("request success");
        assert_eq!(resp.status(), 200);

        let json: serde_json::Value = resp.json().unwrap();
        if !self.allow_execution_failures {
            assert_eq!(json.get("error"), None);
        }
        serde_json::from_value(json).expect("should be valid JsonRpcResponse")
    }
}

impl std::panic::UnwindSafe for Env {}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub private_key: libra_crypto::ed25519::Ed25519PrivateKey,
    pub public_key: libra_crypto::ed25519::Ed25519PublicKey,
    pub address: libra_types::account_address::AccountAddress,
    pub seq: u64,
    pub children: Vec<Account>,
}

impl Account {
    pub fn new(private_key: libra_crypto::ed25519::Ed25519PrivateKey) -> Self {
        let public_key: libra_crypto::ed25519::Ed25519PublicKey = (&private_key).into();
        Account {
            private_key,
            public_key: public_key.clone(),
            address: libra_types::account_address::from_public_key(&public_key),
            seq: 0,
            children: vec![],
        }
    }

    pub fn new_with_address(
        address: libra_types::account_address::AccountAddress,
        private_key: libra_crypto::ed25519::Ed25519PrivateKey,
    ) -> Self {
        let public_key = (&private_key).into();
        Account {
            private_key,
            public_key,
            address,
            seq: 0,
            children: vec![],
        }
    }

    pub fn gen() -> Self {
        Self::new(generate_key::generate_key())
    }

    pub fn hex_address(&self) -> String {
        hex::encode(self.address)
    }

    pub fn auth_key(&self) -> libra_types::transaction::authenticator::AuthenticationKey {
        libra_types::transaction::authenticator::AuthenticationKey::ed25519(&self.public_key)
    }
}

impl libra_types::transaction::helpers::TransactionSigner for Account {
    fn sign_txn(
        &self,
        raw_txn: libra_types::transaction::RawTransaction,
    ) -> Result<SignedTransaction> {
        let signature = self.private_key.sign(&raw_txn);
        Ok(SignedTransaction::new(
            raw_txn,
            libra_crypto::ed25519::Ed25519PublicKey::from(&self.private_key),
            signature,
        ))
    }
}
