// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{bail, Result};
use executor_types::ExecutedTrees;
use libra_crypto::HashValue;
#[cfg(test)]
use libra_types::{
    account_address::AccountAddress,
    account_config::lbr_type_tag,
    block_info::BlockInfo,
    ledger_info::LedgerInfo,
    on_chain_config::ValidatorSet,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{authenticator::AuthenticationKey, SignedTransaction},
};
use libra_types::{
    epoch_state::EpochState, ledger_info::LedgerInfoWithSignatures, transaction::Transaction,
    validator_signer::ValidatorSigner,
};
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::HashMap;
#[cfg(test)]
use transaction_builder::encode_peer_to_peer_with_metadata_script;
#[cfg(test)]
use vm_genesis::GENESIS_KEYPAIR;

#[derive(Clone)]
pub struct MockStorage {
    // some mock transactions in the storage
    transactions: Vec<Transaction>,
    // the executed trees after applying the txns above.
    synced_trees: ExecutedTrees,
    // latest ledger info per epoch
    ledger_infos: HashMap<u64, LedgerInfoWithSignatures>,
    // latest epoch number (starts with 1)
    epoch_num: u64,
    // Validator signer of the latest epoch
    // All epochs are built s.t. a single signature is enough for quorum cert
    signer: ValidatorSigner,
    // A validator verifier of the latest epoch
    epoch_state: EpochState,
}

impl MockStorage {
    pub fn new(genesis_li: LedgerInfoWithSignatures, signer: ValidatorSigner) -> Self {
        let epoch_state = genesis_li.ledger_info().next_epoch_state().unwrap().clone();
        let epoch_num = genesis_li.ledger_info().epoch() + 1;
        let mut ledger_infos = HashMap::new();
        ledger_infos.insert(0, genesis_li);
        Self {
            transactions: vec![],
            synced_trees: ExecutedTrees::new_empty(),
            ledger_infos,
            epoch_num,
            signer,
            epoch_state,
        }
    }

    fn add_txns(&mut self, txns: &mut Vec<Transaction>) {
        self.transactions.append(txns);
        let num_leaves = self.transactions.len() + 1;
        let frozen_subtree_roots = vec![HashValue::zero(); num_leaves.count_ones() as usize];
        self.synced_trees = ExecutedTrees::new(
            HashValue::zero(), /* dummy_state_root */
            frozen_subtree_roots,
            num_leaves as u64,
        );
    }

    pub fn version(&self) -> u64 {
        self.transactions.len() as u64
    }

    pub fn synced_trees(&self) -> &ExecutedTrees {
        &self.synced_trees
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
            self.synced_trees().clone(),
            self.epoch_state.clone(),
        )
    }

    pub fn get_epoch_changes(&self, known_epoch: u64) -> Result<LedgerInfoWithSignatures> {
        match self.ledger_infos.get(&known_epoch) {
            None => bail!("[mock storage] missing epoch change li"),
            Some(li) => Ok(li.clone()),
        }
    }

    pub fn get_chunk(
        &self,
        start_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Vec<Transaction> {
        let mut res = vec![];
        if target_version < start_version || start_version == 0 {
            return res;
        }
        let mut version = start_version;
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
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) {
        self.add_txns(&mut transactions);
        if let Some(li) = intermediate_end_of_epoch_li {
            self.epoch_num = li.ledger_info().epoch() + 1;
            self.ledger_infos.insert(li.ledger_info().epoch(), li);
            return;
        }
        if verified_target_li.ledger_info().epoch() != self.epoch_num() {
            return;
        }

        // store ledger info only if version matches last tx
        if verified_target_li.ledger_info().version() == self.version() {
            self.ledger_infos.insert(
                verified_target_li.ledger_info().epoch(),
                verified_target_li.clone(),
            );
            if let Some(next_epoch_state) = verified_target_li.ledger_info().next_epoch_state() {
                self.epoch_num = next_epoch_state.epoch;
                self.epoch_state = next_epoch_state.clone();
            }
        }
    }

    // Generate new dummy txns and updates the LI
    // with the version corresponding to the new transactions, signed by this storage signer.
    #[cfg(test)]
    pub fn commit_new_txns(&mut self, num_txns: u64) -> (Vec<Transaction>, Vec<SignedTransaction>) {
        let mut committed_txns = vec![];
        let mut signed_txns = vec![];
        for _ in 0..num_txns {
            let txn = Self::gen_mock_user_txn();
            self.add_txns(&mut vec![txn.clone()]);
            committed_txns.push(txn.clone());
            if let Transaction::UserTransaction(signed_txn) = txn {
                signed_txns.push(signed_txn);
            }
        }
        self.add_li(None);
        (committed_txns, signed_txns)
    }

    #[cfg(test)]
    fn gen_mock_user_txn() -> Transaction {
        let sender = AccountAddress::random();
        let receiver = AuthenticationKey::random();
        let program = encode_peer_to_peer_with_metadata_script(
            lbr_type_tag(),
            receiver.derived_address(),
            1,
            vec![],
            vec![],
        );
        Transaction::UserTransaction(get_test_signed_txn(
            sender,
            0, // sequence number
            &GENESIS_KEYPAIR.0,
            GENESIS_KEYPAIR.1.clone(),
            Some(program),
        ))
    }

    // add the LI to the current highest version and sign it
    #[cfg(test)]
    fn add_li(&mut self, validator_set: Option<ValidatorSet>) {
        let epoch_state = validator_set.map(|set| EpochState {
            epoch: self.epoch_num() + 1,
            verifier: (&set).into(),
        });
        let ledger_info = LedgerInfo::new(
            BlockInfo::new(
                self.epoch_num(),
                self.version(),
                HashValue::zero(),
                HashValue::zero(),
                self.version(),
                0,
                epoch_state,
            ),
            HashValue::zero(),
        );
        let signature = self.signer.sign(&ledger_info);
        let mut signatures = BTreeMap::new();
        signatures.insert(self.signer.author(), signature);
        self.ledger_infos.insert(
            self.epoch_num(),
            LedgerInfoWithSignatures::new(ledger_info, signatures),
        );
    }

    // This function is applying the LedgerInfo with the next epoch info to the existing version
    // (yes, it's different from reality, we're not adding any real reconfiguration txn,
    // just adding a new LedgerInfo).
    // The validator set is different only in the consensus public / private keys, network data
    // remains the same.
    #[cfg(test)]
    pub fn move_to_next_epoch(&mut self, signer: ValidatorSigner, validator_set: ValidatorSet) {
        self.add_li(Some(validator_set));
        self.epoch_num += 1;
        self.signer = signer;
        self.epoch_state = self
            .highest_local_li()
            .ledger_info()
            .next_epoch_state()
            .unwrap()
            .clone();
    }

    // Find LedgerInfo for an epoch boundary version.
    pub fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        for li in self.ledger_infos.values() {
            if li.ledger_info().version() == version && li.ledger_info().ends_epoch() {
                return Ok(li.clone());
            }
        }
        bail!("No LedgerInfo found for version {}", version);
    }
}
