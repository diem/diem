// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct StateSyncConfig {
    // Size of chunk to request for state synchronization
    pub chunk_limit: u64,
    // The timeout of the state sync client to process a commit notification (in milliseconds)
    pub client_commit_timeout_ms: u64,
    // default timeout used for long polling to remote peer
    pub long_poll_timeout_ms: u64,
    // valid maximum chunk limit for sanity check
    pub max_chunk_limit: u64,
    // valid maximum timeout limit for sanity check
    pub max_timeout_ms: u64,
    // The timeout of the state sync coordinator to receive a commit ack from mempool (in milliseconds)
    pub mempool_commit_timeout_ms: u64,
    // default timeout to make state sync progress by sending chunk requests to a certain number of networks
    // if no progress is made by sending chunk requests to a number of networks,
    // the next sync request will be multicasted, i.e. sent to more networks
    pub multicast_timeout_ms: u64,
    // The timeout for ensuring sync requests are making progress (i.e., the maximum time between
    // commits when processing a sync request).
    pub sync_request_timeout_ms: u64,
    // interval used for checking state synchronization progress
    pub tick_interval_ms: u64,

    // Everything above belongs to state sync v1 and will be removed in the future.
    pub data_streaming_service: DataStreamingServiceConfig,
    pub diem_data_client: DiemDataClientConfig,
    pub state_sync_driver: StateSyncDriverConfig,
    pub storage_service: StorageServiceConfig,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            chunk_limit: 1000,
            client_commit_timeout_ms: 5_000,
            long_poll_timeout_ms: 10_000,
            max_chunk_limit: 1000,
            max_timeout_ms: 120_000,
            mempool_commit_timeout_ms: 5_000,
            multicast_timeout_ms: 30_000,
            sync_request_timeout_ms: 60_000,
            tick_interval_ms: 100,
            data_streaming_service: DataStreamingServiceConfig::default(),
            diem_data_client: DiemDataClientConfig::default(),
            state_sync_driver: StateSyncDriverConfig::default(),
            storage_service: StorageServiceConfig::default(),
        }
    }
}

/// The bootstrapping mode determines how the node will bootstrap to the latest
/// blockchain state, e.g., directly download the latest account states.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum BootstrappingMode {
    ApplyTransactionOutputsFromGenesis, // Applies transaction outputs (starting at genesis)
    DownloadLatestAccountStates,        // Downloads the account states (at the latest version)
    ExecuteTransactionsFromGenesis,     // Executes transactions (starting at genesis)
}

/// The continuous syncing mode determines how the node will stay up-to-date
/// once it has bootstrapped and the blockchain continues to grow, e.g.,
/// continuously executing all transactions.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ContinuousSyncingMode {
    ExecuteTransactions,     // Executes transactions to stay up-to-date
    ApplyTransactionOutputs, // Applies transaction outputs to stay up-to-date
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StateSyncDriverConfig {
    pub bootstrapping_mode: BootstrappingMode, // The mode by which to bootstrap
    pub enable_state_sync_v2: bool,            // If the node should sync with state sync v2
    pub continuous_syncing_mode: ContinuousSyncingMode, // The mode by which to sync after bootstrapping
    pub progress_check_interval_ms: u64, // The interval (ms) at which to check state sync progress
}

/// The default state sync driver config will be the one that gets (and keeps)
/// the node up-to-date as quickly and cheaply as possible.
impl Default for StateSyncDriverConfig {
    fn default() -> Self {
        Self {
            bootstrapping_mode: BootstrappingMode::DownloadLatestAccountStates,
            enable_state_sync_v2: false,
            continuous_syncing_mode: ContinuousSyncingMode::ApplyTransactionOutputs,
            progress_check_interval_ms: 100,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct StorageServiceConfig {
    pub max_account_states_chunk_sizes: u64, // Max num of accounts per chunk
    pub max_concurrent_requests: u64,        // Max num of concurrent storage server tasks
    pub max_epoch_chunk_size: u64,           // Max num of epoch ending ledger infos per chunk
    pub max_transaction_chunk_size: u64,     // Max num of transactions per chunk
    pub max_transaction_output_chunk_size: u64, // Max num of transaction outputs per chunk
}

impl Default for StorageServiceConfig {
    fn default() -> Self {
        Self {
            max_account_states_chunk_sizes: 3000,
            max_concurrent_requests: 50,
            max_epoch_chunk_size: 100,
            max_transaction_chunk_size: 3000,
            max_transaction_output_chunk_size: 3000,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct DataStreamingServiceConfig {
    // The interval (milliseconds) at which to refresh the global data summary.
    pub global_summary_refresh_interval_ms: u64,

    // Maximum number of concurrent data client requests (per stream).
    pub max_concurrent_requests: u64,

    // Maximum channel sizes for each data stream listener. If messages are not
    // consumed, they will be dropped (oldest messages first). The remaining
    // messages will be retrieved using FIFO ordering.
    pub max_data_stream_channel_sizes: u64,

    // Maximum number of retries for a single client request before a data
    // stream will terminate.
    pub max_request_retry: u64,

    // Maximum number of notification ID to response context mappings held in
    // memory. Once the number grows beyond this value, garbage collection occurs.
    pub max_notification_id_mappings: u64,

    // The interval (milliseconds) at which to check the progress of each stream.
    pub progress_check_interval_ms: u64,
}

impl Default for DataStreamingServiceConfig {
    fn default() -> Self {
        Self {
            global_summary_refresh_interval_ms: 300,
            max_concurrent_requests: 5,
            max_data_stream_channel_sizes: 1000,
            max_request_retry: 5,
            max_notification_id_mappings: 2000,
            progress_check_interval_ms: 100,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct DiemDataClientConfig {
    pub response_timeout_ms: u64, // Timeout (in milliseconds) when waiting for a response
    pub summary_poll_interval_ms: u64, // Interval (in milliseconds) between data summary polls
}

impl Default for DiemDataClientConfig {
    fn default() -> Self {
        Self {
            response_timeout_ms: 3_000,
            summary_poll_interval_ms: 300,
        }
    }
}
