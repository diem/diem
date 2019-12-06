// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use libra_crypto::hash::CryptoHash;
use libra_crypto::HashValue;
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::ValidatorSet;
use libra_types::crypto_proxies::{
    ValidatorChangeEventWithProof, ValidatorSigner, ValidatorVerifier,
};
use libra_types::{
    account_address::AccountAddress, crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo, test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::Transaction,
};
use std::collections::{BTreeMap, HashMap};
use transaction_builder::encode_transfer_script;
use vm_genesis::GENESIS_KEYPAIR;

pub struct MockStorage {
    // some mock transactions in the storage
    transactions: Vec<Transaction>,
    // latest ledger info per epoch
    ledger_infos: HashMap<u64, LedgerInfoWithSignatures>,
    // latest epoch number (starts with 1)
    epoch_num: u64,
    // Validator signer of the latest epoch
    // All epochs are built s.t. a single signature is enough for quorum cert
    signer: ValidatorSigner,
    // A validator verifier of the latest epoch
    verifier: ValidatorVerifier,
}

impl MockStorage {
    pub fn new(genesis_li: LedgerInfoWithSignatures, signer: ValidatorSigner) -> Self {
        let verifier = genesis_li
            .ledger_info()
            .next_validator_set()
            .unwrap()
            .into();
        let epoch_num = genesis_li.ledger_info().epoch() + 1;
        let mut ledger_infos = HashMap::new();
        ledger_infos.insert(0, genesis_li);
        Self {
            transactions: vec![],
            ledger_infos,
            epoch_num,
            signer,
            verifier,
        }
    }

    pub fn version(&self) -> u64 {
        self.transactions.len() as u64
    }

    pub fn epoch_num(&self) -> u64 {
        self.epoch_num
    }

    pub fn highest_local_li(&self) -> LedgerInfoWithSignatures {
        let cur_epoch = self.epoch_num();
        let epoch_with_li = if self.ledger_infos.contains_key(&cur_epoch) {
            cur_epoch
        } else {
            cur_epoch - 1
        };
        self.ledger_infos.get(&epoch_with_li).unwrap().clone()
    }

    pub fn get_local_storage_state(&self) -> SynchronizerState {
        SynchronizerState::new(
            self.highest_local_li(),
            self.version(),
            self.verifier.clone(),
        )
    }

    pub fn get_epoch_changes(&self, known_epoch: u64) -> ValidatorChangeEventWithProof {
        let mut epoch_change_lis = vec![];
        for epoch_num in known_epoch..self.epoch_num() {
            epoch_change_lis.push(self.ledger_infos.get(&epoch_num).unwrap().clone());
        }
        ValidatorChangeEventWithProof::new(epoch_change_lis)
    }

    pub fn get_chunk(
        &self,
        start_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Vec<Transaction> {
        let mut version = start_version;
        let mut res = vec![];
        let limit = std::cmp::min(limit, target_version - start_version + 1);
        while version - 1 < self.transactions.len() as u64 && version - start_version < limit {
            res.push(self.transactions[(version - 1) as usize].clone());
            version += 1;
        }
        res
    }

    pub fn add_txns_with_li(
        &mut self,
        mut transactions: Vec<Transaction>,
        li: LedgerInfoWithSignatures,
    ) {
        assert_eq!(self.epoch_num, li.ledger_info().epoch());
        self.transactions.append(&mut transactions);
        self.ledger_infos.insert(self.epoch_num(), li.clone());
        if let Some(next_validator_set) = li.ledger_info().next_validator_set() {
            self.epoch_num += 1;
            self.verifier = next_validator_set.into();
        }
    }

    // Generate new dummy txns and updates the LI
    // with the version corresponding to the new transactions, signed by this storage signer.
    pub fn commit_new_txns(&mut self, num_txns: u64) {
        for _ in 0..num_txns {
            self.transactions.push(Self::gen_mock_user_txn());
        }
        self.add_li(None);
    }

    fn gen_mock_user_txn() -> Transaction {
        let sender = AccountAddress::random();
        let receiver = AccountAddress::random();
        let program = encode_transfer_script(&receiver, 1);
        Transaction::UserTransaction(get_test_signed_txn(
            sender,
            0, // sequence number
            GENESIS_KEYPAIR.0.clone(),
            GENESIS_KEYPAIR.1.clone(),
            Some(program),
        ))
    }

    // add the LI to the current highest version and sign it
    fn add_li(&mut self, validator_set: Option<ValidatorSet>) {
        let ledger_info = LedgerInfo::new(
            BlockInfo::new(
                self.epoch_num(),
                self.version(),
                HashValue::zero(),
                HashValue::zero(),
                self.version(),
                0,
                validator_set,
            ),
            HashValue::zero(),
        );
        let signature = self.signer.sign_message(ledger_info.hash()).unwrap();
        let mut signatures = BTreeMap::new();
        signatures.insert(self.signer.author(), signature);
        self.ledger_infos.insert(
            self.epoch_num(),
            LedgerInfoWithSignatures::new(ledger_info, signatures),
        );
    }

    // This function is applying the LedgerInfo with the next validator set to the existing version
    // (yes, it's different from reality, we're not adding any real reconfiguration txn,
    // just adding a new LedgerInfo).
    // The validator set is different only in the consensus public / private keys, network data
    // remains the same.
    pub fn move_to_next_epoch(&mut self, signer: ValidatorSigner, validator_set: ValidatorSet) {
        self.add_li(Some(validator_set.clone()));
        self.epoch_num += 1;
        self.signer = signer;
        self.verifier = self
            .highest_local_li()
            .ledger_info()
            .next_validator_set()
            .unwrap()
            .into();
    }
}
