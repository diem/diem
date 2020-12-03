// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::counters::{STRUCT_LOG_CONNECT_ERROR_COUNT, STRUCT_LOG_TCP_CONNECT_COUNT};
use std::{
    io,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

const WRITE_TIMEOUT_MS: u64 = 2000;
const CONNECTION_TIMEOUT_MS: u64 = 5000;

/// A wrapper for `TcpStream` that handles reconnecting to the endpoint automatically
///
/// `TcpWriter::write()` will block on the message until it is connected.
pub(crate) struct TcpWriter {
    /// The DNS name or IP address logs are being sent to
    endpoint: String,
    /// The `TCPStream` to write to, which will be `None` when disconnected
    stream: Option<TcpStream>,
}

impl TcpWriter {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            stream: None,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Ensure that we get a connection, no matter how long it takes
    /// This will block until there is a connection
    fn refresh_connection(&mut self) {
        loop {
            match self.connect() {
                Ok(stream) => {
                    self.stream = Some(stream);
                    return;
                }
                Err(e) => {
                    eprintln!("[Logging] Failed to connect: {}", e);
                    STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                }
            }

            // Sleep a second so this doesn't just spin as fast as possible
            std::thread::sleep(Duration::from_millis(1000));
        }
    }

    /// Connect and ensure the write timeout is set
    fn connect(&mut self) -> io::Result<TcpStream> {
        STRUCT_LOG_TCP_CONNECT_COUNT.inc();

        let mut last_error = io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to resolve and connect to {}", self.endpoint),
        );

        // resolve addresses to handle DNS names
        for addr in self.endpoint.to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(CONNECTION_TIMEOUT_MS)) {
                Ok(stream) => {
                    // Set the write timeout
                    if let Err(err) =
                        stream.set_write_timeout(Some(Duration::from_millis(WRITE_TIMEOUT_MS)))
                    {
                        STRUCT_LOG_CONNECT_ERROR_COUNT.inc();
                        eprintln!("[Logging] Failed to set write timeout: {}", err);
                        continue;
                    }
                    return Ok(stream);
                }
                Err(err) => last_error = err,
            }
        }

        Err(last_error)
    }
}

impl Write for TcpWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Refresh the connection if it's missing
        if self.stream.is_none() {
            self.refresh_connection();
        }

        // Attempt to write, and if it fails clear underlying stream
        // This doesn't guarantee a message cut off mid send will work, but it does guarantee that
        // we will connect first
        self.stream
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "No stream"))
            .and_then(|stream| stream.write(buf))
            .map_err(|e| {
                self.stream = None;
                e
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(mut stream) = self.stream.as_ref() {
            stream.flush()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Can't flush, not connected",
            ))
        }
    }
}
