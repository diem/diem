// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    cmp::Ordering,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use channel::libra_channel;
use consensus_types::{
    common::{Author, Payload, Round},
    epoch_retrieval::EpochRetrievalRequest,
};
use futures::{select, StreamExt};
use libra_config::config::{ConsensusConfig, ConsensusProposerType, NodeConfig};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    epoch_change::EpochChangeProof,
    epoch_info::EpochInfo,
    on_chain_config::{OnChainConfigPayload, ValidatorSet},
};
use network::protocols::network::Event;
use safety_rules::SafetyRulesManager;

use crate::{
    block_storage::BlockStore,
    counters,
    event_processor::{EventProcessor, SyncProcessor, UnverifiedEvent, VerifiedEvent},
    liveness::{
        leader_reputation::{ActiveInactiveHeuristic, LeaderReputation, LibraDBBackend},
        multi_proposer_election::MultiProposer,
        pacemaker::{ExponentialTimeInterval, Pacemaker},
        proposal_generator::ProposalGenerator,
        proposer_election::ProposerElection,
        rotating_proposer_election::{choose_leader, RotatingProposer},
    },
    network::{IncomingBlockRetrievalRequest, NetworkReceivers, NetworkSender},
    network_interface::{ConsensusMsg, ConsensusNetworkSender},
    persistent_liveness_storage::{LedgerRecoveryData, PersistentLivenessStorage, RecoveryData},
    state_replication::{StateComputer, TxnManager},
    util::time_service::TimeService,
};

pub struct SimpleOnChainSender {
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}


fn validator_set_to_connectivity_request(validator_set: ValidatorSet) -> ConnectivityRequest::UpdateEligibleNodes {
    let validator_keys = validator_set.payload().to_vec();
    ConnectivityRequest::UpdateEligibleNodes(
        validators
            .into_iter()
            .map(|validator| {
                (
                    *validator.account_address(),
                    NetworkPublicKeys {
                        identity_public_key: validator.network_identity_public_key(),
                        signing_public_key: validator.network_signing_public_key().clone(),
                    },
                )
            })
            .collect(),
    )
}

impl SimpleOnChainSender {
    //the actual procssing of the onchainconfig payload event
    pub async fn process_payload(&mut self, payload: OnChainConfigPayload) {
        let validator_set: ValidatorSet = payload
            .get()
            .expect("failed to get ValidatorSet from payload");

        let update_eligible_nodes_request = validator_set_to_connectivity_request(validator_set);
        info!("Update Network about new validators");
        self.conn_mgr_reqs_tx.send(update_eligible_nodes_request)
            .await
            .expect("Unable to update network's eligible peers");
    }

    pub async fn start(
        mut self,
        mut reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    ) {
        loop {
            let pre_select_instant = Instant::now();
            let idle_duration;
            select! {
                payload = reconfig_events.select_next_some() => {
                    idle_duration = pre_select_instant.elapsed();
                    self.process_payload(payload).await
            }

            counters::EVENT_PROCESSING_LOOP_BUSY_DURATION_S
                .observe_duration(pre_select_instant.elapsed() - idle_duration);
            counters::EVENT_PROCESSING_LOOP_IDLE_DURATION_S.observe_duration(idle_duration);
            }
        }
    }
}