// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backtrace::Backtrace;
use libra_logger::error;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde::Serialize;
use std::fmt::{self, Debug};

#[derive(Serialize)]
pub enum SecurityEvent {
    /// Admission Control received a transaction with an invalid signature
    InvalidTransactionAC,

    /// Mempool received a transaction with an invalid signature
    InvalidTransactionMP,

    /// Consensus received a transaction with an invalid signature
    InvalidTransactionConsensus,

    /// Executor received an invalid transactions chunk
    InvalidChunkExecutor,

    /// Mempool received an invalid network event
    InvalidNetworkEventMP,

    /// Consensus received an invalid vote
    DuplicateConsensusVote,

    /// Consensus received an invalid proposal
    InvalidConsensusProposal,

    /// Consensus received an invalid vote
    InvalidConsensusVote,

    /// Consensus received an invalid new round message
    InvalidConsensusRound,

    /// Consensus received an invalid sync info message
    InvalidSyncInfoMsg,

    /// HealthChecker received an invalid network event
    InvalidNetworkEventHC,

    /// HealthChecker received an invalid message
    InvalidHealthCheckerMsg,

    /// A received block is invalid
    InvalidRetrievedBlock,

    /// A block being committed or executed is invalid
    InvalidBlock,

    /// Network identified an invalid peer
    InvalidNetworkPeer,

    /// Network discovery received an invalid DiscoveryMsg
    InvalidDiscoveryMsg,

    /// Error for testing
    #[cfg(test)]
    TestError,
}

/// The `SecurityLog` struct is used to log security-sensitive operations, for instance when an
/// invalid signature is detected or when an unexpected event happens.
///
/// The `security_log()` function should be used to instantiate this struct. It can be decorated
/// with different type of metadata:
/// - `event` contains a pre-defined element in the `SecurityEvent` enum
/// - `error` can contain an error type provided by the application
/// - `data` can contain associated metadata related to the event
/// - `backtrace` can contain a backtrace of the current call stack
///
/// All these information can be defined by using the appropriate function with the same name.
///
/// The method `log()` needs to be called to ensure that the event is actually printed.
///
/// # Example:
/// ```rust
/// use libra_security_logger::{security_log, SecurityEvent};
/// use std::fmt::Debug;
///
/// #[derive(Debug)]
/// struct SampleData {
///     i: u8,
///     s: Vec<u8>,
/// }
///
/// #[derive(Debug)]
/// enum TestError {
///     Error,
/// }
///
///     security_log(SecurityEvent::InvalidTransactionAC)
///         .error(&TestError::Error)
///         .data(&SampleData {
///             i: 0xff,
///             s: vec![0x90, 0xcd, 0x80],
///         })
///         .data("additional payload")
///         .backtrace(100)
///         .log();
/// ```
/// In this example, `security_log()` logs an event of type `SecurityEvent::InvalidTransactionAC`,
/// having `TestError::Error` as application error, a `SimpleData` struct and a `String` as
/// additional metadata, and a backtrace that samples 100% of the times.

#[must_use = "must use `log()`"]
#[derive(Serialize)]
pub struct SecurityLog {
    event: SecurityEvent,
    error: Option<String>,
    data: Vec<String>,
    backtrace: Option<Backtrace>,
}

/// Creates a `SecurityLog` struct that can be decorated with additional data.
pub fn security_log(event: SecurityEvent) -> SecurityLog {
    SecurityLog::new(event)
}

impl SecurityLog {
    fn new(event: SecurityEvent) -> Self {
        SecurityLog {
            event,
            error: None,
            data: Vec::new(),
            backtrace: None,
        }
    }

    /// Adds additional metadata to the `SecurityLog` struct. The argument needs to implement the
    /// `std::fmt::Debug` trait.
    pub fn data<T: Debug>(mut self, data: T) -> Self {
        let data = format!("{:?}", data);
        if usize::checked_add(self.data.len(), 1).is_some() {
            self.data.push(data);
        }
        self
    }

    /// Adds an application error to the `SecurityLog` struct. The argument needs to implement the
    /// `std::fmt::Debug` trait.
    pub fn error<T: Debug>(mut self, error: T) -> Self {
        self.error = Some(format!("{:?}", error));
        self
    }

    /// Adds a backtrace to the `SecurityLog` struct.
    pub fn backtrace(mut self, sampling_rate: u8) -> Self {
        let sampling_rate = std::cmp::min(sampling_rate, 100);
        self.backtrace = {
            let mut rng = SmallRng::from_entropy();
            match rng.gen_range(0, 100) {
                x if x < sampling_rate => Some(Backtrace::new()),
                _ => None,
            }
        };
        self
    }

    /// Prints the `SecurityEvent` struct.
    pub fn log(self) {
        error!("[security] {}", self.to_string());
    }
}

impl fmt::Display for SecurityLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).unwrap_or_else(|e| e.to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct SampleData {
        i: u8,
        s: Vec<u8>,
    }

    #[derive(Debug)]
    enum TestError {
        Error,
    }

    #[test]
    fn test_log() {
        let s = security_log(SecurityEvent::TestError)
            .error(&TestError::Error)
            .data(&SampleData {
                i: 0xff,
                s: vec![0x90, 0xcd, 0x80],
            })
            .data("second_payload");
        assert_eq!(
            s.to_string(),
            r#"{"event":"TestError","error":"Error","data":["SampleData { i: 255, s: [144, 205, 128] }","\"second_payload\""],"backtrace":null}"#,
        );
    }
}
