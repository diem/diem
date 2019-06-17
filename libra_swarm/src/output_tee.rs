// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    io::{BufRead, BufReader, Read, Write},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tools::output_capture::OutputCapture;

/// This tee takes stdout/stderr stream of spawned process and tees it into two destinations:
/// log file and stdout of main thread with given prefix
pub struct OutputTee {
    capture: OutputCapture,
    log_file: Arc<Mutex<dyn Write + Send>>,
    child_stdout: Box<dyn Read + Send>,
    child_stderr: Box<dyn Read + Send>,
    prefix: String,
}

impl OutputTee {
    pub fn new(
        capture: OutputCapture,
        log_file: Box<dyn Write + Send>,
        child_stdout: Box<dyn Read + Send>,
        child_stderr: Box<dyn Read + Send>,
        prefix: String,
    ) -> OutputTee {
        OutputTee {
            capture,
            log_file: Arc::new(Mutex::new(log_file)),
            child_stdout,
            child_stderr,
            prefix,
        }
    }

    /// Start new threads for teeing output of stdout/err streams
    /// Threads will terminate when streams are closed
    pub fn start(self) -> OutputTeeGuard {
        let capture = self.capture;
        let log_file = self.log_file;
        let prefix = self.prefix;

        let stdout_handle = TeeThread {
            capture: capture.clone(),
            log_file: log_file.clone(),
            prefix: prefix.clone(),
            stream: self.child_stdout,
        }
        .start();

        let stderr_handle = TeeThread {
            capture: capture.clone(),
            log_file: log_file.clone(),
            prefix: prefix.clone(),
            stream: self.child_stderr,
        }
        .start();

        OutputTeeGuard {
            stdout_handle,
            stderr_handle,
        }
    }
}

pub struct OutputTeeGuard {
    stdout_handle: JoinHandle<()>,
    stderr_handle: JoinHandle<()>,
}

impl OutputTeeGuard {
    // JoinHandle::join returns Result, which signals whether thread panicked or not
    // If it panicked, it will print panic anyway, there is no reason to process it here
    #[allow(unused_must_use)]
    pub fn join(self) {
        self.stdout_handle.join();
        self.stderr_handle.join();
    }
}

struct TeeThread {
    capture: OutputCapture,
    prefix: String,
    stream: Box<dyn Read + Send>,
    log_file: Arc<Mutex<dyn Write + Send>>,
}

impl TeeThread {
    pub fn start(self) -> JoinHandle<()> {
        std::thread::spawn(move || self.run())
    }

    pub fn run(self) {
        self.capture.apply();
        let buf_reader = BufReader::new(self.stream);
        for line in buf_reader.lines() {
            let line = match line {
                Err(e) => {
                    println!("Failed to read line for tee: {}", e);
                    return;
                }
                Ok(line) => line,
            };
            println!("{}{}", self.prefix, line);
            if let Err(e) = writeln!(self.log_file.lock().unwrap(), "{}", line) {
                println!("Error teeing to file: {}", e);
            }
        }
    }
}
