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
use failure::prelude::*;
use logger::prelude::*;
use schemadb::{ReadOptions, SchemaBatch, DB};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};
use types::transaction::Version;

use failure::_core::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64;
#[cfg(test)]
use std::thread::sleep;
#[cfg(test)]
use std::time::{Duration, Instant};

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
            bail!("Timeout waiting for pruner worker.");
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
    least_readable_version: Version,
    /// (For tests) a way for the worker thread to inform the `Pruner` the pruning progress. If we
    /// set this atomic value to `V`, all versions before `V` can no longer be accessed.
    progress: Arc<AtomicU64>,
    /// indicates if there's NOT any pending work to do currently, to hint
    /// `Self::receive_commands()` to `recv()` blocking-ly.
    blocking_recv: bool,
}

impl Worker {
    const BATCH_SIZE: usize = 1024;

    fn new(db: Arc<DB>, command_receiver: Receiver<Command>, progress: Arc<AtomicU64>) -> Self {
        Self {
            db,
            command_receiver,
            progress,
            least_readable_version: 0,
            blocking_recv: true,
        }
    }

    fn work_loop(mut self) {
        while self.receive_commands() {
            // Process a reasonably small batch of work before trying to receive commands again,
            // in case `Command::Quit` is received (that's when we should quit.)
            match prune_state(
                Arc::clone(&self.db),
                self.progress.load(Ordering::Relaxed),
                self.least_readable_version,
                Self::BATCH_SIZE,
            ) {
                Ok((num_pruned, last_seen_version)) => {
                    // Make next recv() blocking if all done.
                    self.blocking_recv = num_pruned < Self::BATCH_SIZE;

                    // Log the progress.
                    self.progress.store(last_seen_version, Ordering::Relaxed);
                    OP_COUNTER.set(
                        "pruner.least_readable_state_version",
                        last_seen_version as usize,
                    );
                }
                Err(e) => {
                    crit!("Error purging db records. {:?}", e);
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
                    if least_readable_version > self.least_readable_version {
                        self.least_readable_version = least_readable_version;
                        // Switch to non-blocking to allow some work to be done after the
                        // channel has drained.
                        self.blocking_recv = false;
                    }
                }
            }
        }
    }
}

pub fn prune_state(
    db: Arc<DB>,
    // `iter.seek_to_first()` can be costly if there are a lot of deletes at the beginning of the
    // whole key range which have not been compacted away yet. We let the call site hint us where
    // the first undeleted key is if possible. Pass in `0` to essentially seek to the first record.
    max_pruned_version_hint: Version,
    least_readable_version: Version,
    limit: usize,
) -> Result<(usize, Version)> {
    let mut batch = SchemaBatch::new();
    let mut num_pruned = 0;
    let mut iter = db.iter::<StaleNodeIndexSchema>(ReadOptions::default())?;
    iter.seek(&max_pruned_version_hint)?;

    // Collect records to prune, as many as `limit`.
    let mut iter = iter.take(limit);
    let mut last_seen_version = 0;
    while let Some((index, _)) = iter.next().transpose()? {
        // Only records that have retired before or at version `least_readable_version` can be
        // pruned in order to keep that version still readable after pruning.
        if index.stale_since_version > least_readable_version {
            break;
        }
        last_seen_version = index.stale_since_version;
        batch.delete::<JellyfishMerkleNodeSchema>(&index.node_key)?;
        batch.delete::<StaleNodeIndexSchema>(&index)?;
        num_pruned += 1;
    }

    // Persist.
    if num_pruned > 0 {
        db.write_schemas(batch)?;
    }

    Ok((num_pruned, last_seen_version))
}

#[cfg(test)]
mod test;
