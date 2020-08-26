// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Prover task runner that runs multiple instances of the prover task and returns
//! as soon as the fastest instance finishes.

use crate::cli::Options;
use async_trait::async_trait;
use futures::{future::FutureExt, pin_mut, select};
use rand::Rng;
use std::{
    process::Output,
    sync::mpsc::{channel, Sender},
};
use tokio::{
    process::Command,
    runtime::Runtime,
    sync::{broadcast, broadcast::Receiver},
};

#[derive(Debug, Clone)]
enum BroadcastMsg {
    Stop,
}

#[async_trait]
pub trait ProverTask {
    type TaskResult: Send + 'static;
    type TaskId: Send + Copy + 'static;

    /// Initialize the task runner given the number of instances.
    fn init(&mut self, num_instances: usize) -> Vec<Self::TaskId>;

    /// Run the task with task_id. This function will be called from one of the worker threads.
    async fn run(&mut self, task_id: Self::TaskId) -> Self::TaskResult;
}

pub struct ProverTaskRunner();

impl ProverTaskRunner {
    /// Run `num_instances` instances of the prover `task` and returns the task id
    /// as well as the result of the fastest running instance.
    pub fn run_tasks<T>(mut task: T, num_instances: usize) -> (T::TaskId, T::TaskResult)
    where
        T: ProverTask + Clone + Send + 'static,
    {
        let rt = Runtime::new().unwrap();

        // Create channels for communication.
        let (worker_tx, master_rx) = channel();
        let (master_tx, _): (
            tokio::sync::broadcast::Sender<BroadcastMsg>,
            Receiver<BroadcastMsg>,
        ) = broadcast::channel(num_instances);

        // Initialize the prover tasks.
        let task_ids = task.init(num_instances);
        for task_id in task_ids {
            let send_n = worker_tx.clone();
            let worker_rx = master_tx.subscribe();
            let cloned_task = task.clone();
            // Spawn a task worker for each task_id.
            rt.spawn(async move {
                Self::run_task_until_cancelled(cloned_task, task_id, send_n, worker_rx).await;
            });
        }
        // Listens until one of the workers finishes.
        loop {
            let res = master_rx.recv();
            if let Ok((task_id, result)) = res {
                // Result received from one worker. Broadcast to other workers
                // so they can stop working.
                if num_instances > 1 {
                    let _ = master_tx.send(BroadcastMsg::Stop);
                }
                return (task_id, result);
            }
        }
    }

    // Run two async tasks, listening on broadcast channel and running boogie, until
    // either boogie finishes running, or a stop message is received.
    async fn run_task_until_cancelled<T>(
        mut task: T,
        task_id: T::TaskId,
        tx: Sender<(T::TaskId, T::TaskResult)>,
        rx: Receiver<BroadcastMsg>,
    ) where
        T: ProverTask,
    {
        let boogie_fut = task.run(task_id).fuse();
        let watchdog_fut = Self::watchdog(rx).fuse();
        pin_mut!(boogie_fut, watchdog_fut);
        select! {
            _ = watchdog_fut => {
                // A stop message is received.
            }
            res = boogie_fut => {
                // Boogie finishes running, send the result to parent thread.
                let _ = tx.send((task_id, res));
            },
        }
    }

    /// Waits for a stop message from the parent thread.
    async fn watchdog(mut rx: Receiver<BroadcastMsg>) {
        let _ = rx.recv().await;
    }
}

#[derive(Debug, Clone)]
pub struct RunBoogieWithSeeds {
    pub options: Options,
    pub boogie_file: String,
}

#[async_trait]
impl ProverTask for RunBoogieWithSeeds {
    type TaskResult = Output;
    type TaskId = usize;

    fn init(&mut self, num_instances: usize) -> Vec<Self::TaskId> {
        // If we are running only one Boogie instance, use the default random seed.
        if num_instances == 1 {
            return vec![self.options.backend.random_seed];
        }
        let mut rng = rand::thread_rng();
        // Otherwise generate a list of random numbers to use as seeds.
        (0..num_instances)
            .map(|_| rng.gen::<u8>() as usize)
            .collect()
    }

    async fn run(&mut self, task_id: Self::TaskId) -> Self::TaskResult {
        let args = self.get_boogie_command(task_id);
        Command::new(&args[0])
            .args(&args[1..])
            .kill_on_drop(true)
            .output()
            .await
            .unwrap()
    }
}

impl RunBoogieWithSeeds {
    /// Returns command line to call boogie.
    pub fn get_boogie_command(&mut self, seed: usize) -> Vec<String> {
        self.options
            .backend
            .boogie_flags
            .push(format!("-proverOpt:O:smt.random_seed={}", seed));
        self.options.get_boogie_command(&self.boogie_file)
    }
}
