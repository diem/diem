// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides `Pruner` which manages a thread pruning old data in the background and is
//! meant to be triggered by other threads as they commit new data to the DB.

use crate::{
    schema::{
        jellyfish_merkle_node::JellyfishMerkleNodeSchema, stale_node_index::StaleNodeIndexSchema,
    },
    OP_COUNTER,
};
use anyhow::Result;
use jellyfish_merkle::StaleNodeIndex;
use libra_logger::prelude::*;
use libra_types::transaction::Version;
use schemadb::{ReadOptions, SchemaBatch, SchemaIterator, DB};
#[cfg(test)]
use std::thread::sleep;
use std::{
    iter::Peekable,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

/// The `Pruner` is meant to be part of a `LibraDB` instance and runs in the background to prune old
/// data.
///
/// It creates a worker thread on construction and joins it on destruction. When destructed, it
/// quits the worker thread eagerly without waiting for all pending work to be done.
pub(crate) struct Pruner {
    /// Other than the latest version, how many historical versions to keep being readable. For
    /// example, this being 0 means keep only the latest version.
    num_historical_versions_to_keep: u64,
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
    pub fn new(db: Arc<DB>, num_historical_versions_to_keep: u64) -> Self {
        let (command_sender, command_receiver) = channel();
        let worker_progress = Arc::new(AtomicU64::new(0));
        let worker_progress_clone = Arc::clone(&worker_progress);

        let worker_thread = std::thread::Builder::new()
            .name("libradb_pruner".into())
            .spawn(move || Worker::new(db, command_receiver, worker_progress_clone).work_loop())
            .expect("Creating pruner thread should succeed.");

        Self {
            num_historical_versions_to_keep,
            worker_thread: Some(worker_thread),
            command_sender: Mutex::new(command_sender),
            worker_progress,
        }
    }

    /// Sends pruning command to the worker thread when necessary.
    pub fn wake(&self, latest_version: Version) {
        if latest_version > self.num_historical_versions_to_keep {
            let least_readable_version = latest_version - self.num_historical_versions_to_keep;
            self.command_sender
                .lock()
                .expect("command_sender to pruner thread should lock.")
                .send(Command::Prune {
                    least_readable_version,
                })
                .expect("Receiver should not destruct prematurely.");
        }
    }

    /// (For tests only.) Notifies the worker thread and waits for it to finish its job by polling
    /// an internal counter.
    #[cfg(test)]
    pub fn wake_and_wait(&self, latest_version: Version) -> Result<()> {
        self.wake(latest_version);

        if latest_version > self.num_historical_versions_to_keep {
            let least_readable_version = latest_version - self.num_historical_versions_to_keep;
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
}

impl Drop for Pruner {
    fn drop(&mut self) {
        self.command_sender
            .lock()
            .expect("Locking command_sender should not fail.")
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
    const MAX_VERSIONS_TO_PRUNE_PER_BATCH: usize = 100;

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

    fn work_loop(mut self) {
        while self.receive_commands() {
            // Process a reasonably small batch of work before trying to receive commands again,
            // in case `Command::Quit` is received (that's when we should quit.)
            match prune_state(
                Arc::clone(&self.db),
                self.least_readable_version.load(Ordering::Relaxed),
                self.target_least_readable_version,
                Self::MAX_VERSIONS_TO_PRUNE_PER_BATCH,
            ) {
                Ok(least_readable_version) => {
                    // Make next recv() blocking if all done.
                    self.blocking_recv =
                        least_readable_version == self.target_least_readable_version;

                    // Log the progress.
                    self.least_readable_version
                        .store(least_readable_version, Ordering::Relaxed);
                    OP_COUNTER.set(
                        "pruner.least_readable_state_version",
                        least_readable_version as usize,
                    );

                    // Try to purge the log.
                    if let Err(e) = self.maybe_purge_index() {
                        crit!("Failed purging state state node index, ignored. Err: {}", e);
                    }
                }
                Err(e) => {
                    crit!("Error pruning stale state nodes. {:?}", e);
                    // On error, stop retrying vigorously by making next recv() blocking.
                    self.blocking_recv = true;
                }
            }
        }
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

    /// Purge the stale node index so that after restart not too much already pruned stuff is dealt
    /// with again (although no harm is done deleting those then non-existent things.)
    ///
    /// We issue (range) deletes on the index only periodically instead of after every pruning batch
    /// to avoid sending too many deletions to the DB, which takes disk space and slows it down.
    fn maybe_purge_index(&mut self) -> Result<()> {
        const MIN_INTERVAL: Duration = Duration::from_secs(60);
        const MIN_VERSIONS: u64 = 60000;

        // A deletion is issued at most once in one minute and when the pruner has progressed by at
        // least 60000 versions (assuming the pruner deletes as slow as 1000 versions per second,
        // this imposes at most one minute of work in vain after restarting.)
        let now = Instant::now();
        if now - self.index_purged_at > MIN_INTERVAL {
            let least_readable_version = self.least_readable_version.load(Ordering::Relaxed);

            if least_readable_version - self.index_min_nonpurged_version + 1 > MIN_VERSIONS {
                let new_min_non_purged_version = least_readable_version + 1;
                self.db.range_delete::<StaleNodeIndexSchema, Version>(
                    &self.index_min_nonpurged_version,
                    &new_min_non_purged_version, // end is exclusive
                )?;
                self.index_min_nonpurged_version = new_min_non_purged_version;
                self.index_purged_at = now;
            }
        }

        Ok(())
    }
}

struct StaleNodeIndicesByVersionIterator<'a> {
    inner: Peekable<SchemaIterator<'a, StaleNodeIndexSchema>>,
    target_least_readable_version: Version,
}

impl<'a> StaleNodeIndicesByVersionIterator<'a> {
    fn new(
        db: &'a DB,
        least_readable_version: Version,
        target_least_readable_version: Version,
    ) -> Result<Self> {
        let mut iter = db.iter::<StaleNodeIndexSchema>(ReadOptions::default())?;
        iter.seek(&least_readable_version)?;

        Ok(Self {
            inner: iter.peekable(),
            target_least_readable_version,
        })
    }

    fn next_result(&mut self) -> Result<Option<Vec<StaleNodeIndex>>> {
        match self.inner.next().transpose()? {
            None => Ok(None),
            Some((index, _)) => {
                let version = index.stale_since_version;
                if version > self.target_least_readable_version {
                    return Ok(None);
                }

                let mut indices = vec![index];
                while let Some(res) = self.inner.peek() {
                    if let Ok((index_ref, _)) = res {
                        if index_ref.stale_since_version != version {
                            break;
                        }
                    }

                    let (index, _) = self.inner.next().transpose()?.expect("Should be Some.");
                    indices.push(index);
                }

                Ok(Some(indices))
            }
        }
    }
}

impl<'a> Iterator for StaleNodeIndicesByVersionIterator<'a> {
    type Item = Result<Vec<StaleNodeIndex>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_result().transpose()
    }
}

pub fn prune_state(
    db: Arc<DB>,
    least_readable_version: Version,
    target_least_readable_version: Version,
    max_versions: usize,
) -> Result<Version> {
    let indices = StaleNodeIndicesByVersionIterator::new(
        &db,
        least_readable_version,
        target_least_readable_version,
    )?
    .take(max_versions) // Iterator<Item = Result<Vec<StaleNodeIndex>>>
    .collect::<Result<Vec<_>>>()? // now Vec<Vec<StaleNodeIndex>>
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if indices.is_empty() {
        Ok(least_readable_version)
    } else {
        let new_least_readable_version = indices.last().expect("Should exist.").stale_since_version;
        let mut batch = SchemaBatch::new();
        indices
            .into_iter()
            .map(|index| batch.delete::<JellyfishMerkleNodeSchema>(&index.node_key))
            .collect::<Result<_>>()?;
        db.write_schemas(batch)?;
        Ok(new_least_readable_version)
    }
}

#[cfg(test)]
mod test;
