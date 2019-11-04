// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{mem, sync::mpsc};
use threadpool;

#[derive(Clone)]
pub struct ThreadPoolExecutor {
    inner: threadpool::ThreadPool,
}

impl ThreadPoolExecutor {
    pub fn new(thread_name: String) -> Self {
        let inner = threadpool::Builder::new().thread_name(thread_name).build();
        Self { inner }
    }

    /// Executes jobs, wait for them to complete and return results
    /// Note: Results in vector do not match order of input jobs
    pub fn execute_jobs<'a, R, J>(&self, jobs: Vec<J>) -> Vec<R>
    where
        R: Send + 'a,
        J: FnOnce() -> R + Send + 'a,
    {
        let (sender, recv) = mpsc::channel();
        let size = jobs.len();
        for job in jobs {
            let sender = sender.clone();
            let closure = move || {
                let r = job();
                sender
                    .send(r)
                    .expect("main execute_jobs thread terminated before worker");
            };
            let closure: Box<dyn FnOnce() + Send + 'a> = Box::new(closure);
            // Using mem::transmute to cast from 'a to 'static lifetime
            // This is safe because we ensure lifetime of current stack frame
            // is longer then lifetime of closure
            // Even if one of worker threads panics, we still going to wait in recv loop below
            // until every single thread completes
            let closure: Box<dyn FnOnce() + Send + 'static> = unsafe { mem::transmute(closure) };
            self.inner.execute(closure);
        }
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let r = recv.recv().expect("One of job threads had panic");
            result.push(r);
        }
        result
    }
}
