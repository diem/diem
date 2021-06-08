// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    CouldNotStringifyMessage(String),
    ClientAlreadyClosed(u64),
    ClientWantsToDisconnect,
    TransportError(String),
}

impl Error {
    fn message(&self) -> String {
        match self {
            Error::CouldNotStringifyMessage(s) => format!("Could not convert {} to string", s),
            Error::ClientAlreadyClosed(client_id) => format!("Client#{} is closed", client_id),
            Error::ClientWantsToDisconnect => {
                "Received disconnect request message from client".to_string()
            }
            Error::TransportError(s) => format!("Transport error: {}", s),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl error::Error for Error {}
