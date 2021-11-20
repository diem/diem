// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{driver::DriverConfiguration, error::Error, storage_synchronizer::StorageStateSummary};
use data_streaming_service::streaming_client::StreamingServiceClient;
use diem_config::config::{BootstrappingMode, StateSyncDriverConfig};
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    stream::FusedStream,
    SinkExt, Stream,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// A simple component that manages the bootstrapping of the node
pub struct Bootstrapper {
    // The channel used to notify a listener of successful bootstrapping
    bootstrap_notifier_channel: Option<oneshot::Sender<Result<(), Error>>>,

    // If the node has already been bootstrapped
    bootstrapped: bool,

    // The config of the state sync driver
    driver_configuration: DriverConfiguration,

    // The client through which to stream data from the Diem network
    streaming_service_client: StreamingServiceClient,
}

impl Bootstrapper {
    pub fn new(
        driver_configuration: DriverConfiguration,
        streaming_service_client: StreamingServiceClient,
    ) -> Self {
        Self {
            bootstrap_notifier_channel: None,
            bootstrapped: false,
            driver_configuration,
            streaming_service_client,
        }
    }

    /// Returns true iff the node has already completed bootsrapping
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// Notifies the given channel once the node has completed bootstrapping
    pub fn add_bootstrap_notifier_channel(
        &mut self,
        bootstrap_notifier_channel: oneshot::Sender<Result<(), Error>>,
    ) {
        self.bootstrap_notifier_channel = Some(bootstrap_notifier_channel);
    }

    // TODO(joshlind): improve this so that bootstrapping is not just checking
    // the local state against the waypoint version. Bootstrapping should be
    // based on the network around us.
    /// Checks if the node has successfully bootstrapped
    pub fn check_if_bootstrapped(
        &mut self,
        latest_storage_summary: &StorageStateSummary,
    ) -> Result<(), Error> {
        let latest_committed_version = latest_storage_summary
            .latest_ledger_info
            .ledger_info()
            .version();
        if self.driver_configuration.waypoint.version() <= latest_committed_version {
            self.bootstrapped = true;
            if let Some(notifier_channel) = self.bootstrap_notifier_channel.take() {
                let notification_message = Ok(());
                if let Err(error) = notifier_channel.send(notification_message) {
                    return Err(Error::CallbackSendFailed(format!(
                        "Bootstrap notification error: {:?}",
                        error
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn check_progress(&mut self) {
        match self.driver_configuration.config.bootstrapping_mode {
            BootstrappingMode::ExecuteTransactionsFromGenesis => {
                // TODO(joshlind): fetch the data!
            }
            mode => unimplemented!("Bootstrapping mode {:?} is not yet supported!", mode),
        }
    }
}
