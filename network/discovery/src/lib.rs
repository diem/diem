// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters::DISCOVERY_COUNTS;
use channel::{diem_channel, diem_channel::Receiver};
use diem_config::{config::PeerSet, network_id::NetworkContext};
use diem_crypto::x25519;
use diem_logger::prelude::*;
use diem_types::on_chain_config::{OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY};
use futures::{Stream, StreamExt};
use network::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    counters::inc_by_with_context,
    logging::NetworkSchema,
};
use std::pin::Pin;
use subscription_service::ReconfigSubscription;
use tokio::runtime::Handle;

mod counters;
mod file;
mod validator_set;

use crate::{file::FileStream, validator_set::ValidatorSetStream};
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::Storage;
use diem_time_service::TimeService;
use std::{
    path::Path,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

/// A union type for all implementations of `DiscoveryChangeListenerTrait`
pub struct DiscoveryChangeListener {
    discovery_source: DiscoverySource,
    network_context: Arc<NetworkContext>,
    update_channel: channel::Sender<ConnectivityRequest>,
    source_stream: DiscoveryChangeStream,
}

enum DiscoveryChangeStream {
    ValidatorSet(ValidatorSetStream),
    File(FileStream),
}

impl Stream for DiscoveryChangeStream {
    type Item = PeerSet;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::ValidatorSet(stream) => Pin::new(stream).poll_next(cx),
            Self::File(stream) => Pin::new(stream).poll_next(cx),
        }
    }
}

impl DiscoveryChangeListener {
    pub fn validator_set(
        network_context: Arc<NetworkContext>,
        update_channel: channel::Sender<ConnectivityRequest>,
        expected_pubkey: x25519::PublicKey,
        encryptor: Encryptor<Storage>,
        reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    ) -> Self {
        let source_stream = DiscoveryChangeStream::ValidatorSet(ValidatorSetStream::new(
            network_context.clone(),
            expected_pubkey,
            encryptor,
            reconfig_events,
        ));
        DiscoveryChangeListener {
            discovery_source: DiscoverySource::OnChainValidatorSet,
            network_context,
            update_channel,
            source_stream,
        }
    }

    pub fn file(
        network_context: Arc<NetworkContext>,
        update_channel: channel::Sender<ConnectivityRequest>,
        file_path: &Path,
        interval_duration: Duration,
        time_service: TimeService,
    ) -> Self {
        let source_stream = DiscoveryChangeStream::File(FileStream::new(
            network_context.clone(),
            file_path,
            interval_duration,
            time_service,
        ));
        DiscoveryChangeListener {
            discovery_source: DiscoverySource::File,
            network_context,
            update_channel,
            source_stream,
        }
    }

    pub fn start(self, executor: &Handle) {
        executor.spawn(Box::pin(self).run());
    }

    async fn run(mut self: Pin<Box<Self>>) {
        info!(
            NetworkSchema::new(&self.network_context),
            "{} Starting {} Discovery", self.network_context, self.discovery_source
        );

        while let Some(update) = self.source_stream.next().await {
            trace!(
                NetworkSchema::new(&self.network_context),
                "{} Sending update: {:?}",
                self.network_context,
                update
            );
            let request = ConnectivityRequest::UpdateDiscoveredPeers(self.discovery_source, update);
            if let Err(error) = self.update_channel.try_send(request) {
                inc_by_with_context(&DISCOVERY_COUNTS, &self.network_context, "send_failure", 1);
                warn!(
                    NetworkSchema::new(&self.network_context),
                    "{} Failed to send update {:?}", self.network_context, error
                );
            }
        }
        warn!(
            NetworkSchema::new(&self.network_context),
            "{} {} Discovery actor terminated", &self.network_context, self.discovery_source
        );
    }

    pub fn discovery_source(&self) -> DiscoverySource {
        self.discovery_source
    }
}

pub fn gen_simple_discovery_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all("network", ON_CHAIN_CONFIG_REGISTRY.to_vec(), vec![])
}
