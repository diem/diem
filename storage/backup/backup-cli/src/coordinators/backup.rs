// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::backup::{EpochEndingBackupController, EpochEndingBackupOpt},
        state_snapshot::backup::{StateSnapshotBackupController, StateSnapshotBackupOpt},
        transaction::backup::{TransactionBackupController, TransactionBackupOpt},
    },
    metadata,
    metadata::cache::MetadataCacheOpt,
    storage::BackupStorage,
    utils::{backup_service_client::BackupServiceClient, GlobalBackupOpt},
};
use anyhow::{anyhow, ensure, Result};
use futures::{stream, Future, StreamExt};
use libra_logger::prelude::*;
use libra_types::transaction::Version;
use libradb::backup::backup_handler::DbState;
use std::{fmt::Debug, sync::Arc};
use structopt::StructOpt;
use tokio::{
    sync::watch,
    time::{interval, Duration},
};

#[derive(StructOpt)]
pub struct BackupCoordinatorOpt {
    #[structopt(flatten)]
    pub metadata_cache_opt: MetadataCacheOpt,
    // Assuming epoch doesn't bump frequently, we default to backing epoch ending info up as soon
    // as possible.
    #[structopt(long, default_value = "1")]
    pub epoch_ending_batch_size: usize,
    // We replay transactions on top of a state snapshot at about 2000 tps, having a state snapshot
    // every 2000 * 3600 * 2 = 14.4 Mil versions will guarantee that to achieve any state in
    // history, it won't take us more than two hours in transaction replaying. Defaulting to 10 Mil
    // here to make it less than two, and easier for eyes.
    #[structopt(long, default_value = "10000000")]
    pub state_snapshot_interval: usize,
    // Assuming the network runs at 100 tps, it's 100 * 3600 = 360k transactions per hour, we don't
    // want the backups to lag behind too much. Defaulting to 100k here in case the network is way
    // slower than expected.
    #[structopt(long, default_value = "100000")]
    pub transaction_batch_size: usize,
}

impl BackupCoordinatorOpt {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.epoch_ending_batch_size > 0
                && self.state_snapshot_interval > 0
                && self.transaction_batch_size > 0,
            "Backup interval and batch sizes must be greater than 0"
        );
        Ok(())
    }
}

pub struct BackupCoordinator {
    client: Arc<BackupServiceClient>,
    storage: Arc<dyn BackupStorage>,
    global_opt: GlobalBackupOpt,
    metadata_cache_opt: MetadataCacheOpt,
    epoch_ending_batch_size: usize,
    state_snapshot_interval: usize,
    transaction_batch_size: usize,
}

impl BackupCoordinator {
    pub fn new(
        opt: BackupCoordinatorOpt,
        global_opt: GlobalBackupOpt,
        client: Arc<BackupServiceClient>,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        opt.validate().unwrap();
        Self {
            client,
            storage,
            global_opt,
            metadata_cache_opt: opt.metadata_cache_opt,
            epoch_ending_batch_size: opt.epoch_ending_batch_size,
            state_snapshot_interval: opt.state_snapshot_interval,
            transaction_batch_size: opt.transaction_batch_size,
        }
    }
    pub async fn run(&self) -> Result<()> {
        // Connect to both the local Libra node and the backup storage.
        let backup_state =
            metadata::cache::sync_and_load(&self.metadata_cache_opt, Arc::clone(&self.storage))
                .await?
                .get_storage_state();
        let db_state = self
            .client
            .get_db_state()
            .await?
            .ok_or_else(|| anyhow!("DB not bootstrapped."))?;
        let (tx, rx) = watch::channel(db_state);

        // Schedule work streams.
        let watch_db_state = interval(Duration::from_secs(1))
            .then(|_| self.try_refresh_db_state(&tx))
            .boxed_local();

        let backup_epoch_endings = self
            .backup_work_stream(
                backup_state.latest_epoch_ending_epoch,
                &rx,
                Self::backup_epoch_endings,
            )
            .boxed_local();
        let backup_state_snapshots = self
            .backup_work_stream(
                backup_state.latest_state_snapshot_version,
                &rx,
                Self::backup_state_snapshot,
            )
            .boxed_local();
        let backup_transactions = self
            .backup_work_stream(
                backup_state.latest_transaction_version,
                &rx,
                Self::backup_transactions,
            )
            .boxed_local();

        info!("Backup coordinator started.");
        let mut all_work = stream::select_all(vec![
            watch_db_state,
            backup_epoch_endings,
            backup_state_snapshots,
            backup_transactions,
        ]);

        loop {
            all_work
                .next()
                .await
                .ok_or_else(|| anyhow!("Must be a bug: we never returned None."))?
        }
    }
}

impl BackupCoordinator {
    async fn try_refresh_db_state(&self, db_state_broadcast: &watch::Sender<DbState>) {
        match self.client.get_db_state().await {
            Ok(s) => db_state_broadcast
                .broadcast(s.expect("Db should have been bootstrapped."))
                .map_err(|e| anyhow!("Receivers should not be cancelled: {}", e))
                .unwrap(),
            Err(e) => warn!(
                "Failed pulling DbState from local Libra node: {}. Will keep trying.",
                e
            ),
        };
    }

    async fn backup_epoch_endings(
        &self,
        last_epoch_ending_epoch_in_backup: Option<u64>,
        db_state: DbState,
    ) -> Result<Option<u64>> {
        let (first, last) = get_batch_range(
            last_epoch_ending_epoch_in_backup,
            self.epoch_ending_batch_size,
        );

        // <= because `db_state.epoch` hasn't ended yet
        if db_state.epoch <= last {
            // wait for the next db_state update
            return Ok(last_epoch_ending_epoch_in_backup);
        }

        EpochEndingBackupController::new(
            EpochEndingBackupOpt {
                start_epoch: first,
                end_epoch: last + 1,
            },
            self.global_opt.clone(),
            Arc::clone(&self.client),
            Arc::clone(&self.storage),
        )
        .run()
        .await?;

        Ok(Some(last))
    }

    async fn backup_state_snapshot(
        &self,
        last_snapshot_version_in_backup: Option<Version>,
        db_state: DbState,
    ) -> Result<Option<Version>> {
        let next_snapshot_version = get_next_snapshot(
            last_snapshot_version_in_backup,
            db_state,
            self.state_snapshot_interval,
        );

        if db_state.committed_version < next_snapshot_version {
            // wait for the next db_state update
            return Ok(last_snapshot_version_in_backup);
        }

        StateSnapshotBackupController::new(
            StateSnapshotBackupOpt {
                version: next_snapshot_version,
            },
            self.global_opt.clone(),
            Arc::clone(&self.client),
            Arc::clone(&self.storage),
        )
        .run()
        .await?;

        Ok(Some(next_snapshot_version))
    }

    async fn backup_transactions(
        &self,
        last_transaction_version_in_backup: Option<Version>,
        db_state: DbState,
    ) -> Result<Option<u64>> {
        let (first, last) = get_batch_range(
            last_transaction_version_in_backup,
            self.transaction_batch_size,
        );

        if db_state.committed_version < last {
            // wait for the next db_state update
            return Ok(last_transaction_version_in_backup);
        }

        TransactionBackupController::new(
            TransactionBackupOpt {
                start_version: first,
                num_transactions: (last + 1 - first) as usize,
            },
            self.global_opt.clone(),
            Arc::clone(&self.client),
            Arc::clone(&self.storage),
        )
        .run()
        .await?;

        Ok(Some(last))
    }

    fn backup_work_stream<'a, S, W, Fut>(
        &'a self,
        initial_state: S,
        db_state_rx: &'a watch::Receiver<DbState>,
        worker: W,
    ) -> impl StreamExt<Item = ()> + 'a
    where
        S: Copy + Debug + 'a,
        W: Worker<'a, S, Fut> + Copy + 'static,
        Fut: Future<Output = Result<S>> + 'a,
    {
        stream::unfold(
            (initial_state, db_state_rx.clone()),
            move |(s, mut rx)| async move {
                let db_state = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow!("The broadcaster has been dropped."))
                    .unwrap();
                let next_state = worker(self, s, db_state).await.unwrap_or_else(|e| {
                    warn!("backup failed: {}. Keep trying with state {:?}.", e, s);
                    s
                });
                Some(((), (next_state, rx)))
            },
        )
    }
}

trait Worker<'a, S, Fut: Future<Output = Result<S>> + 'a>:
    Fn(&'a BackupCoordinator, S, DbState) -> Fut
{
}

impl<'a, T, S, Fut> Worker<'a, S, Fut> for T
where
    T: Fn(&'a BackupCoordinator, S, DbState) -> Fut,
    Fut: Future<Output = Result<S>> + 'a,
{
}

fn get_batch_range(last_in_backup: Option<u64>, batch_size: usize) -> (u64, u64) {
    // say, 5 is already in backup, and we target batches of size 10, we will return (6, 9) in this
    // case, so 6, 7, 8, 9 will be in this batch, and next time the backup worker will pass in 9,
    // and we will return (10, 19)
    let start = last_in_backup.map_or(0, |n| n + 1);
    let next_batch_start = (start / batch_size as u64 + 1) * batch_size as u64;

    (start, next_batch_start - 1)
}

fn get_next_snapshot(last_in_backup: Option<u64>, db_state: DbState, interval: usize) -> u64 {
    // We don't try to guarantee snapshots are taken at each applicable interval: when the backup
    // progress can't keep up with the ledger growth, we favor timeliness over completeness.
    // For example, with interval 100, when we finished taking a snapshot at version 700, if we
    // found the latest version is already 1250, the next snapshot we take will be at 1200, not 800.

    let next_for_storage = match last_in_backup {
        Some(last) => (last / interval as u64 + 1) * interval as u64,
        None => 0,
    };

    let last_for_db: u64 = db_state.committed_version / interval as u64 * interval as u64;

    std::cmp::max(next_for_storage, last_for_db)
}

#[cfg(test)]
mod tests {
    use crate::coordinators::backup::{get_batch_range, get_next_snapshot};
    use libradb::backup::backup_handler::DbState;

    #[test]
    fn test_get_batch_range() {
        assert_eq!(get_batch_range(None, 100), (0, 99));
        assert_eq!(get_batch_range(Some(99), 50), (100, 149));
        assert_eq!(get_batch_range(Some(149), 100), (150, 199));
        assert_eq!(get_batch_range(Some(199), 100), (200, 299));
    }

    #[test]
    fn test_get_next_snapshot() {
        let _state = |v| DbState {
            epoch: 0,
            committed_version: v,
            synced_version: v,
        };

        assert_eq!(get_next_snapshot(None, _state(90), 100), 0);
        assert_eq!(get_next_snapshot(Some(0), _state(90), 100), 100);
        assert_eq!(get_next_snapshot(Some(0), _state(190), 100), 100);
        assert_eq!(get_next_snapshot(Some(0), _state(200), 100), 200);
        assert_eq!(get_next_snapshot(Some(0), _state(250), 100), 200);
        assert_eq!(get_next_snapshot(Some(200), _state(250), 100), 300);
    }
}
