// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// FIXME(aldenhu): doc
//! This module provides `Pruner` which manages a thread pruning old data in the background and is
//! meant to be triggered by other threads as they commit new data to the DB.

use crate::{
    metrics::{
        DIEM_STORAGE_OTHER_TIMERS_SECONDS, DIEM_STORAGE_PRUNER_LEAST_READABLE_STATE_VERSION,
        DIEM_STORAGE_PRUNE_WINDOW,
    },
    schema::bsmt_update_log::{BsmtKey, BsmtUpdateLogSchema},
};
use anyhow::Result;
use diem_infallible::Mutex;
use diem_types::transaction::Version;
use schemadb::{
    schema::{KeyCodec, Schema},
    SchemaBatch, DB,
};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
    time::Instant,
};

/// The `Pruner` is meant to be part of a `DiemDB` instance and runs in the background to prune old
/// data.
///
/// It creates a worker thread on construction and joins it on destruction. When destructed, it
/// quits the worker thread eagerly without waiting for all pending work to be done.
#[derive(Debug)]
pub(crate) struct Pruner {
    /// Other than the latest version, how many historical versions to keep being readable. For
    /// example, this being 0 means keep only the latest version.
    historical_versions_to_keep: u64,
    /// The worker thread handle, created upon Pruner instance construction and joined upon its
    /// destruction. It only becomes `None` after joined in `drop()`.
    worker_thread: Option<JoinHandle<()>>,
    /// The sender side of the channel talking to the worker thread.
    command_sender: Mutex<Sender<Command>>,
    /// (For tests) A way for the worker thread to inform the `Pruner` the pruning progress. If it
    /// sets this atomic value to `V`, all versions before `V` can no longer be accessed.
    #[allow(dead_code)]
    worker_progress: Arc<AtomicU64>,
}

impl Pruner {
    /// Creates a worker thread that waits on a channel for pruning commands.
    pub fn new(db: Arc<DB>, historical_versions_to_keep: u64) -> Self {
        let (command_sender, command_receiver) = channel();

        let worker_progress = Arc::new(AtomicU64::new(0));
        let worker_progress_clone = Arc::clone(&worker_progress);

        DIEM_STORAGE_PRUNE_WINDOW.set(historical_versions_to_keep as i64);
        let worker_thread = std::thread::Builder::new()
            .name("diemdb_pruner".into())
            .spawn(move || Worker::new(db, command_receiver, worker_progress_clone).work())
            .expect("Creating pruner thread should succeed.");

        Self {
            historical_versions_to_keep,
            worker_thread: Some(worker_thread),
            command_sender: Mutex::new(command_sender),
            worker_progress,
        }
    }

    /// Sends pruning command to the worker thread when necessary.
    pub fn wake(&self, latest_version: Version) {
        if latest_version > self.historical_versions_to_keep {
            let least_readable_version = latest_version - self.historical_versions_to_keep;
            self.command_sender
                .lock()
                .send(Command::Prune {
                    least_readable_version,
                })
                .expect("Receiver should not destruct prematurely.");
        }
    }

    /*
    /// (For tests only.) Notifies the worker thread and waits for it to finish its job by polling
    /// an internal counter.
    #[cfg(test)]
    pub fn wake_and_wait(&self, latest_version: Version) -> Result<()> {
        self.wake(latest_version);

        if latest_version > self.historical_versions_to_keep {
            let least_readable_version = latest_version - self.historical_versions_to_keep;
            // Assuming no big pruning chunks will be issued by a test.
            const TIMEOUT: Duration = Duration::from_secs(10);
            let end = Instant::now() + TIMEOUT;

            while Instant::now() < end {
                if self.worker_progress.load(Ordering::Relaxed) >= least_readable_version {
                    return Ok(());
                }
                sleep(Duration::from_millis(1));
            }
            anyhow::bail!("Timeout waiting for pruner worker.");
        }
        Ok(())
    }
     */
}

impl Drop for Pruner {
    fn drop(&mut self) {
        self.command_sender
            .lock()
            .send(Command::Quit)
            .expect("Receiver should not destruct.");
        self.worker_thread
            .take()
            .expect("Worker thread must exist.")
            .join()
            .expect("Worker thread should join peacefully.");
    }
}

enum Command {
    Quit,
    Prune { least_readable_version: Version },
}

struct Worker {
    db: Arc<DB>,
    command_receiver: Receiver<Command>,
    target_least_readable_version: Version,
    /// Keeps a record of the pruning progress. If this equals to version `V`, we know versions
    /// smaller than `V` are no longer readable.
    /// This being an atomic value is to communicate the info with the Pruner thread (for tests).
    least_readable_version: Arc<AtomicU64>,
    /// Indicates if there's NOT any pending work to do currently, to hint
    /// `Self::receive_commands()` to `recv()` blocking-ly.
    blocking_recv: bool,
    index_min_nonpurged_version: Version,
    index_purged_at: Instant,
}

impl Worker {
    const MAX_ITEMS_TO_PRUNE_PER_BATCH: usize = 10_000;

    fn new(
        db: Arc<DB>,
        command_receiver: Receiver<Command>,
        least_readable_version: Arc<AtomicU64>,
    ) -> Self {
        Self {
            db,
            command_receiver,
            least_readable_version,
            target_least_readable_version: 0,
            blocking_recv: true,
            index_min_nonpurged_version: 0,
            index_purged_at: Instant::now(),
        }
    }

    fn work(self) {
        self.work_impl().unwrap();
    }

    fn work_impl(mut self) -> Result<()> {
        // self.initialize();
        let mut target_version = 0;
        let mut last_pass_finished = true;
        // FIXME(aldenhu): deal with error
        let db = self.db.clone();
        let mut log_iter = db.iter::<BsmtUpdateLogSchema>(Default::default())?;

        while self.receive_commands() {
            // Process a reasonably small batch of work before trying to receive commands again,
            // in case `Command::Quit` is received (that's when we should quit.)

            // TODO(aldenhu): record_progress
            // TODO(aldenhu): set self.blocking_recv
            // TODO(aldenhu): purge log

            if last_pass_finished {
                // Pruning too frequently is not efficient in that we batch delete versions of
                // frequently updated nodes -- it's preferable to delete once after a node is
                // updated several times
                if self.target_least_readable_version > target_version + 10_000 {
                    target_version = self.target_least_readable_version;
                    // FIXME(aldenhu): do we need to make a new one here?
                    log_iter.seek(&target_version)?;
                    last_pass_finished = false;
                    self.blocking_recv = false;
                } else {
                    // no work to do, switch to blocking receive
                    self.blocking_recv = true;
                    continue;
                }
            }

            let _timer = DIEM_STORAGE_OTHER_TIMERS_SECONDS
                .with_label_values(&["bsmt_prune_batch"])
                .start_timer();
            let mut items = Vec::new();
            for _ in 0..Self::MAX_ITEMS_TO_PRUNE_PER_BATCH {
                match log_iter.next() {
                    Some(ret) => {
                        let ((ver, bsmt_key), _) = ret?;
                        items.push((ver, bsmt_key));
                    }
                    None => {
                        break;
                    }
                }
            }
            if items.is_empty() {
                last_pass_finished = true;
                self.record_progress(target_version);
                continue;
            }

            let mut batch = SchemaBatch::new();
            for item in items {
                // FIXME(aldenhu): delete these items in batch with range_delete()
                batch.delete::<BsmtUpdateLogSchema>(&item)?;
                let (_ver, bsmt_key) = item;

                match bsmt_key {
                    BsmtKey::Node(pos) => {
                        Self::delete_old_nodes(pos, target_version, &db, &mut batch)?;
                    }
                    BsmtKey::Value(key) => {
                        Self::delete_old_nodes(key, target_version, &db, &mut batch)?;
                    }
                }
            }
            self.db.write_schemas(batch)?;
        }

        Ok(())
    }

    fn delete_old_nodes<S, K>(
        key: K,
        target_version: Version,
        db: &Arc<DB>,
        batch: &mut SchemaBatch,
    ) -> Result<()>
    where
        S: Schema<Key = (K, Version)>,
        (K, Version): KeyCodec<S>,
        K: PartialEq,
    {
        let mut iter = db.iter::<S>(Default::default())?;
        let pair = (key, target_version);
        iter.seek(&pair)?;
        let (key, _) = pair;

        if let Some(ret) = iter.next() {
            let ((k, _ver), _) = ret?;
            if k == key {
                // There is a node at `pos` that's created at or before `target_version`

                // TODO(aldenhu): range delete instead of looping; need range_delete_inclusive()
                // And there might exist more nodes at `pos` before that, all of
                // which is no longer useful.
                while let Some(ret) = iter.next() {
                    let ((k, ver), _) = ret?;
                    if k == key {
                        batch.delete::<S>(&(k, ver))?;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Log the progress.
    fn record_progress(&mut self, least_readable_version: Version) {
        self.least_readable_version
            .store(least_readable_version, Ordering::Relaxed);
        DIEM_STORAGE_PRUNER_LEAST_READABLE_STATE_VERSION.set(least_readable_version as i64);
    }

    /// Tries to receive all pending commands, blocking waits for the next command if no work needs
    /// to be done, otherwise quits with `true` to allow the outer loop to do some work before
    /// getting back here.
    ///
    /// Returns `false` if `Command::Quit` is received, to break the outer loop and let
    /// `work_loop()` return.
    fn receive_commands(&mut self) -> bool {
        loop {
            let command = if self.blocking_recv {
                // Worker has nothing to do, blocking wait for the next command.
                self.command_receiver
                    .recv()
                    .expect("Sender should not destruct prematurely.")
            } else {
                // Worker has pending work to do, non-blocking recv.
                match self.command_receiver.try_recv() {
                    Ok(command) => command,
                    // Channel has drained, yield control to the outer loop.
                    Err(_) => return true,
                }
            };

            match command {
                // On `Command::Quit` inform the outer loop to quit by returning `false`.
                Command::Quit => return false,
                Command::Prune {
                    least_readable_version,
                } => {
                    if least_readable_version > self.target_least_readable_version {
                        self.target_least_readable_version = least_readable_version;
                        // Switch to non-blocking to allow some work to be done after the
                        // channel has drained.
                        self.blocking_recv = false;
                    }
                }
            }
        }
    }
}
