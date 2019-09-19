// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_manager::PeerManagerError;
use failure::{err_msg, Backtrace, Context, Fail};
use futures::channel::mpsc;
use protobuf::error::ProtobufError;
use std::{
    fmt::{self, Display},
    io,
};
use tokio::timer;
use types::validator_verifier::VerifyError;

/// Errors propagated from the network module.
#[derive(Debug)]
pub struct NetworkError {
    inner: Context<NetworkErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum NetworkErrorKind {
    #[fail(display = "IO error")]
    IoError,

    #[fail(display = "Error parsing protobuf message")]
    ProtobufParseError,

    #[fail(display = "Invalid signature error")]
    SignatureError,

    #[fail(display = "Failed to parse multiaddrs")]
    MultiaddrError,

    #[fail(display = "Error sending on mpsc channel")]
    MpscSendError,

    #[fail(display = "Error setting timeout")]
    TimerError,

    #[fail(display = "Operation timed out")]
    TimedOut,

    #[fail(display = "Unknown tokio::timer Error variant")]
    UnknownTimerError,

    #[fail(display = "PeerManager error")]
    PeerManagerError,

    #[fail(display = "Parsing error")]
    ParsingError,

    #[fail(display = "Peer not connected")]
    NotConnected,
}

impl Fail for NetworkError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl NetworkError {
    pub fn kind(&self) -> NetworkErrorKind {
        *self.inner.get_context()
    }
}

impl From<NetworkErrorKind> for NetworkError {
    fn from(kind: NetworkErrorKind) -> NetworkError {
        NetworkError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<NetworkErrorKind>> for NetworkError {
    fn from(inner: Context<NetworkErrorKind>) -> NetworkError {
        NetworkError { inner }
    }
}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> NetworkError {
        err.context(NetworkErrorKind::IoError).into()
    }
}

impl From<VerifyError> for NetworkError {
    fn from(err: VerifyError) -> NetworkError {
        err.context(NetworkErrorKind::SignatureError).into()
    }
}

impl From<ProtobufError> for NetworkError {
    fn from(err: ProtobufError) -> NetworkError {
        err.context(NetworkErrorKind::ProtobufParseError).into()
    }
}

impl From<parity_multiaddr::Error> for NetworkError {
    fn from(err: parity_multiaddr::Error) -> NetworkError {
        err.context(NetworkErrorKind::MultiaddrError).into()
    }
}

impl From<mpsc::SendError> for NetworkError {
    fn from(err: mpsc::SendError) -> NetworkError {
        err.context(NetworkErrorKind::MpscSendError).into()
    }
}

impl From<PeerManagerError> for NetworkError {
    fn from(err: PeerManagerError) -> NetworkError {
        match err {
            PeerManagerError::IoError(_) => err.context(NetworkErrorKind::IoError).into(),
            PeerManagerError::NotConnected(_) => err.context(NetworkErrorKind::NotConnected).into(),
            err => err.context(NetworkErrorKind::PeerManagerError).into(),
        }
    }
}

impl From<timer::timeout::Error<NetworkError>> for NetworkError {
    fn from(err: timer::timeout::Error<NetworkError>) -> NetworkError {
        if err.is_elapsed() {
            Context::new(NetworkErrorKind::TimedOut).into()
        } else if err.is_timer() {
            err.into_timer()
                .unwrap()
                .context(NetworkErrorKind::TimerError)
                .into()
        } else if err.is_inner() {
            err.into_inner().unwrap()
        } else {
            err_msg(err)
                .context(NetworkErrorKind::UnknownTimerError)
                .into()
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use failure::AsFail;

    // This test demos a causal error chain that can be created using the `context` method of `Fail`
    // types.
    #[test]
    fn causal_chain() {
        let base_error = ::failure::err_msg("First error");
        let first_level_error = base_error.context(NetworkErrorKind::TimedOut);
        let second_level_error = first_level_error.context(NetworkErrorKind::PeerManagerError);
        let network_error: NetworkError = second_level_error.into();
        // When called without RUST_BACKTRACE=1, the debug mode should print the following:
        // NetworkError { inner: ErrorMessage { msg: "First error" }
        // Operation timed out
        // PeerManager error }
        eprintln!("{:?}", network_error);
        // The display mode output is just the outermost error:
        // PeerManager error
        eprintln!("{}", network_error);
        // Alternatively, we can iterate over the individual failures in the causal chain to get
        // the following output:
        // Error: PeerManager error
        // Error: Operation timed out
        // Error: First error
        for e in network_error.as_fail().iter_chain() {
            eprintln!("Error: {}", e);
        }
    }
}
