// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor_proxy::ExecutorProxyTrait, tests::mock_storage::MockStorage, SynchronizerState,
};
use anyhow::Result;
use executor_types::ExecutedTrees;
use libra_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, test_utils::TEST_SEED, x25519, Uniform};
use libra_network_address::{
    encrypted::{
        RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION,
    },
    NetworkAddress, RawNetworkAddress,
};
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures,
    on_chain_config::ValidatorSet, proof::TransactionListProof,
    transaction::TransactionListWithProof, validator_config::ValidatorConfig,
    validator_info::ValidatorInfo, validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier,
};
use rand::{rngs::StdRng, SeedableRng};
use std::{
    convert::TryFrom,
    str::FromStr,
    sync::{Arc, RwLock},
};

pub(crate) struct SynchronizerEnvHelper;

impl SynchronizerEnvHelper {
    pub(crate) fn default_handler() -> MockRpcHandler {
        Box::new(|resp| -> Result<TransactionListWithProof> { Ok(resp) })
    }

    // Returns the initial peers with their signatures
    pub(crate) fn initial_setup(
        count: usize,
    ) -> (
        Vec<ValidatorSigner>,
        Vec<ValidatorInfo>,
        Vec<x25519::PrivateKey>,
    ) {
        let (signers, _verifier) = random_validator_verifier(count, None, true);

        // Setup identity public keys.
        let mut rng = StdRng::from_seed(TEST_SEED);
        let network_keys: Vec<_> = (0..count)
            .map(|_| x25519::PrivateKey::generate(&mut rng))
            .collect();

        let mut validators_keys = vec![];
        // The voting power of peer 0 is enough to generate an LI that passes validation.
        for (idx, signer) in signers.iter().enumerate() {
            let voting_power = if idx == 0 { 1000 } else { 1 };
            let addr = NetworkAddress::from_str("/memory/0").unwrap();
            let raw_addr = RawNetworkAddress::try_from(&addr).unwrap();
            let enc_addr = raw_addr.clone().encrypt(
                &TEST_SHARED_VAL_NETADDR_KEY,
                TEST_SHARED_VAL_NETADDR_KEY_VERSION,
                &signer.author(),
                0,
                0,
            );
            let raw_enc_addr = RawEncNetworkAddress::try_from(&enc_addr).unwrap();

            let validator_config = ValidatorConfig::new(
                signer.public_key(),
                network_keys[idx].public_key(),
                raw_enc_addr,
                network_keys[idx].public_key(),
                raw_addr,
            );
            let validator_info =
                ValidatorInfo::new(signer.author(), voting_power, validator_config);
            validators_keys.push(validator_info);
        }
        (signers, validators_keys, network_keys)
    }

    pub(crate) fn genesis_li(validators: &[ValidatorInfo]) -> LedgerInfoWithSignatures {
        LedgerInfoWithSignatures::genesis(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            ValidatorSet::new(validators.to_vec()),
        )
    }
}

pub(crate) type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof> + Send + Sync + 'static,
>;

pub(crate) struct MockExecutorProxy {
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    pub(crate) fn new(handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self { handler, storage }
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_local_storage_state(&self) -> Result<SynchronizerState> {
        Ok(self.storage.read().unwrap().get_local_storage_state())
    }

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        _synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        self.storage.write().unwrap().add_txns_with_li(
            txn_list_with_proof.transactions,
            ledger_info_with_sigs,
            intermediate_end_of_epoch_li,
        );
        Ok(())
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof> {
        let txns = self
            .storage
            .read()
            .unwrap()
            .get_chunk(known_version + 1, limit, target_version);
        let first_txn_version = txns.first().map(|_| known_version + 1);
        let txns_with_proof = TransactionListWithProof::new(
            txns,
            None,
            first_txn_version,
            TransactionListProof::new_empty(),
        );
        (self.handler)(txns_with_proof)
    }

    fn get_epoch_proof(&self, epoch: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage.read().unwrap().get_epoch_changes(epoch)
    }

    fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage
            .read()
            .unwrap()
            .get_epoch_ending_ledger_info(version)
    }

    fn load_on_chain_configs(&mut self) -> Result<()> {
        Ok(())
    }

    fn publish_on_chain_config_updates(&mut self, _events: Vec<ContractEvent>) -> Result<()> {
        Ok(())
    }
}
