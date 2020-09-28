// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Trait used by the VM to log interesting data.
// Clients are responsible for the implementation.
pub trait Logger: Clone {
    // A critical error occurred. Typically an invariant violation, and generally anything
    // that can compromise the behavior of the system.
    fn crit(&self, message: &str);
    // An error occurred, severity is high. The error should be investigated but it
    // does not compromise the health of the system.
    fn error(&self, message: &str);
    // An error occurred, severity is low. The error is unexpected under normal conditions
    // but it may be expected in specific circumstances.
    fn warn(&self, message: &str);
    // Logged for conditions that are not frequent, and useful to understand the
    // overall behavior of the system.
    fn info(&self, message: &str);
    // Useful information for debugging. A client may choose (and likely will) to drop
    // those messages. But under critical conditions, and difficult investigations, those
    // messages may be reported to properly and quickly identify a problem.
    fn debug(&self, message: &str);
}

// A logger for the VM that prints to stderr
#[derive(Clone)]
pub struct StdErrLogger;

impl Logger for StdErrLogger {
    fn crit(&self, message: &str) {
        eprintln!("{}", format!("crit: {}", message).as_str());
    }

    fn error(&self, message: &str) {
        eprintln!("{}", format!("error: {}", message).as_str());
    }

    fn warn(&self, message: &str) {
        eprintln!("{}", format!("warn: {}", message).as_str());
    }

    fn info(&self, message: &str) {
        eprintln!("{}", format!("info: {}", message).as_str());
    }

    fn debug(&self, message: &str) {
        eprintln!("{}", format!("debug: {}", message).as_str());
    }
}
