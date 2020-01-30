// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::event_processor::EventProcessorPrelude;
use crate::chained_bft::persistent_storage::RemoteRecoveryData;
use crate::{
    chained_bft::{network::NetworkSender, persistent_storage::PersistentStorage},
    counters,
    state_replication::{StateComputer, TxnManager},
    util::time_service::ClockTimeService,
};
use consensus_types::{
    common::{Author, Payload, Round},
    epoch_retrieval::EpochRetrievalRequest,
};
use futures::executor::block_on;
use libra_config::config::ConsensusConfig;
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{EpochInfo, LedgerInfoWithSignatures},
};
use network::{
    proto::{ConsensusMsg, ConsensusMsg_oneof},
    validator_network::{ConsensusNetworkSender, Event},
};
use safety_rules::SafetyRulesManager;
use std::{
    cmp::Ordering,
    convert::TryInto,
    sync::{Arc, RwLock},
};

// Manager the components that shared across epoch and spawn per-epoch EventProcessor with
// epoch-specific input.
pub struct EpochManager<T> {
    author: Author,
    epoch_info: Arc<RwLock<EpochInfo>>,
    config: ConsensusConfig,
    time_service: Arc<ClockTimeService>,
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
    network_sender: ConsensusNetworkSender,
    timeout_sender: channel::Sender<Round>,
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
    storage: Arc<dyn PersistentStorage<T>>,
    safety_rules_manager: SafetyRulesManager<T>,
}

impl<T: Payload> EpochManager<T> {
    pub fn new(
        author: Author,
        epoch_info: Arc<RwLock<EpochInfo>>,
        config: ConsensusConfig,
        time_service: Arc<ClockTimeService>,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
        network_sender: ConsensusNetworkSender,
        timeout_sender: channel::Sender<Round>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
        storage: Arc<dyn PersistentStorage<T>>,
        safety_rules_manager: SafetyRulesManager<T>,
    ) -> Self {
        Self {
            author,
            epoch_info,
            config,
            time_service,
            self_sender,
            network_sender,
            timeout_sender,
            txn_manager,
            state_computer,
            storage,
            safety_rules_manager,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch_info.read().unwrap().epoch
    }

    pub async fn process_epoch_retrieval(
        &mut self,
        request: EpochRetrievalRequest,
        peer_id: AccountAddress,
    ) {
        let proof = match self
            .state_computer
            .get_epoch_proof(request.start_epoch, request.end_epoch)
            .await
        {
            Ok(proof) => proof,
            Err(e) => {
                warn!("Failed to get epoch proof from storage: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::EpochChange(proof.into())),
        };
        if let Err(e) = self.network_sender.send_to(peer_id, msg).await {
            warn!(
                "Failed to send a epoch retrieval to peer {}: {:?}",
                peer_id, e
            );
        };
    }

    pub async fn process_different_epoch(&mut self, different_epoch: u64, peer_id: AccountAddress) {
        match different_epoch.cmp(&self.epoch()) {
            // We try to help nodes that have lower epoch than us
            Ordering::Less => {
                self.process_epoch_retrieval(
                    EpochRetrievalRequest {
                        start_epoch: different_epoch,
                        end_epoch: self.epoch(),
                    },
                    peer_id,
                )
                .await
            }
            // We request proof to join higher epoch
            Ordering::Greater => {
                let request = EpochRetrievalRequest {
                    start_epoch: self.epoch(),
                    end_epoch: different_epoch,
                };
                let msg = match request.try_into() {
                    Ok(bytes) => ConsensusMsg {
                        message: Some(ConsensusMsg_oneof::RequestEpoch(bytes)),
                    },
                    Err(e) => {
                        warn!("Fail to serialize EpochRetrievalRequest: {:?}", e);
                        return;
                    }
                };
                if let Err(e) = self.network_sender.send_to(peer_id, msg).await {
                    warn!(
                        "Failed to send a epoch retrieval to peer {}: {:?}",
                        peer_id, e
                    );
                }
            }
            Ordering::Equal => {
                warn!("Same epoch should not come to process_different_epoch");
            }
        }
    }

    pub async fn start_new_epoch(
        &mut self,
        ledger_info: LedgerInfoWithSignatures,
    ) -> EventProcessorPrelude<T> {
        // make sure storage is on this ledger_info too, it should be no-op if it's already committed
        if let Err(e) = self.state_computer.sync_to(ledger_info.clone()).await {
            error!("State sync to new epoch {} failed with {:?}, we'll try to start from current libradb", ledger_info, e);
        }
        let remote_recovery_data = self.storage.recover_from_ledger().await;
        *self.epoch_info.write().unwrap() = EpochInfo {
            epoch: remote_recovery_data.epoch(),
            verifier: remote_recovery_data.validators(),
        };
        self.start_epoch(remote_recovery_data)
    }

    pub fn start_epoch(
        &mut self,
        remote_recovery_data: RemoteRecoveryData<T>,
    ) -> EventProcessorPrelude<T> {
        let validators = remote_recovery_data.validators();
        let epoch = self.epoch();
        counters::EPOCH.set(epoch as i64);
        counters::CURRENT_EPOCH_VALIDATORS.set(validators.len() as i64);
        counters::CURRENT_EPOCH_QUORUM_SIZE.set(validators.quorum_voting_power() as i64);
        info!(
            "Start EventProcessor prelude with epoch {} , validators {}",
            epoch, validators,
        );
        block_on(
            self.network_sender
                .update_eligible_nodes(remote_recovery_data.validator_keys()),
        )
        .expect("Unable to update network's eligible peers");
        let network_sender = NetworkSender::new(
            self.author,
            self.network_sender.clone(),
            self.self_sender.clone(),
            validators.clone(),
        );

        EventProcessorPrelude::new(
            self.storage.clone(),
            network_sender,
            self.state_computer.clone(),
            self.config.clone(),
            self.safety_rules_manager.client(),
            self.author.clone(),
            self.txn_manager.clone(),
            self.time_service.clone(),
            self.timeout_sender.clone(),
        )
    }
}
