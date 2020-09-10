// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This provides a simple networking substrate between a client and server. It is assumed that all
//! operations are blocking and return only complete blocks of data. The intended use case has the
//! server blocking on read.  Upon receiving a payload during a read, the server should process the
//! payload, write a response, and then block on read again. The client should block on read after
//! performing a write. Upon errors or remote disconnections, the call (read, write) will return an
//! error to let the caller know of the event. A follow up call will result in the service
//! attempting to either reconnect in the case of a client or accept a new client in the case of a
//! server.
//!
//! Internally both the client and server leverage a NetworkStream that communications in blocks
//! where a block is a length prefixed array of bytes.

use libra_logger::{debug, trace};
use std::{
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    thread, time,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Already called shutdown")]
    AlreadyShutdown,
    #[error("Found data that is too large to decode: {0}")]
    DataTooLarge(usize),
    #[error("Internal network error:")]
    NetworkError(#[from] std::io::Error),
    #[error("No active stream")]
    NoActiveStream,
    #[error("Remote stream cleanly closed")]
    RemoteStreamClosed,
}

pub struct NetworkClient {
    server: SocketAddr,
    stream: Option<NetworkStream>,
    /// Read, Write, Connect timeout in milliseconds.
    timeout_ms: u64,
}

impl NetworkClient {
    pub fn new(server: SocketAddr, timeout_ms: u64) -> Self {
        Self {
            server,
            stream: None,
            timeout_ms,
        }
    }

    /// Blocking read until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>, Error> {
        let stream = self.server()?;
        let result = stream.read();
        if result.is_err() {
            debug!("On read, upstream peer disconnected, setting stream to None");
            self.stream = None;
        }
        result
    }

    /// Shutdown the internal network stream
    pub fn shutdown(&mut self) -> Result<(), Error> {
        debug!("Shutdown called");
        let stream = self.stream.take().ok_or_else(|| Error::NoActiveStream)?;
        stream.shutdown()?;
        Ok(())
    }

    /// Blocking write until able to successfully send an entire message
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        let stream = self.server()?;
        let result = stream.write(data);
        if result.is_err() {
            debug!("On write, upstream peer disconnected, setting stream to None");
            self.stream = None;
        }
        result
    }

    fn server(&mut self) -> Result<&mut NetworkStream, Error> {
        if self.stream.is_none() {
            let timeout = std::time::Duration::from_millis(self.timeout_ms);
            debug!("Attempting to connect to upstream {}", self.server);
            let mut stream = TcpStream::connect_timeout(&self.server, timeout);

            let sleeptime = time::Duration::from_millis(100);
            while let Err(e) = stream {
                debug!("Failed to connect to upstream {} {:?}", self.server, e);
                thread::sleep(sleeptime);
                stream = TcpStream::connect_timeout(&self.server, timeout);
            }

            let stream = stream?;
            stream.set_nodelay(true)?;
            self.stream = Some(NetworkStream::new(stream, self.timeout_ms));
            debug!("Connection established to upstream {}", self.server);
        }

        self.stream.as_mut().ok_or_else(|| Error::NoActiveStream)
    }
}

pub struct NetworkServer {
    listener: Option<TcpListener>,
    stream: Option<NetworkStream>,
    /// Read, Write, Connect timeout in milliseconds.
    timeout_ms: u64,
}

impl NetworkServer {
    pub fn new(listen: SocketAddr, timeout_ms: u64) -> Self {
        let listener = TcpListener::bind(listen);
        Self {
            listener: Some(listener.unwrap()),
            stream: None,
            timeout_ms,
        }
    }

    /// If there isn't already a downstream client, it accepts. Otherwise it
    /// blocks until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>, Error> {
        let stream = self.client()?;
        let result = stream.read();
        if result.is_err() {
            debug!("On read, downstream peer disconnected, setting stream to None");
            self.stream = None;
        }
        result
    }

    /// Shutdown the internal network stream
    pub fn shutdown(&mut self) -> Result<(), Error> {
        debug!("Shutdown called");
        self.listener.take().ok_or_else(|| Error::AlreadyShutdown)?;
        let stream = self.stream.take().ok_or_else(|| Error::NoActiveStream)?;
        stream.shutdown()?;
        Ok(())
    }

    /// If there isn't already a downstream client, it accepts. Otherwise it
    /// blocks until it is able to successfully send an entire message.
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        let stream = self.client()?;
        let result = stream.write(data);
        if result.is_err() {
            debug!("On write, downstream peer disconnected, setting stream to None");
            self.stream = None;
        }
        result
    }

    fn client(&mut self) -> Result<&mut NetworkStream, Error> {
        if self.stream.is_none() {
            debug!("Waiting for downstream to connect");
            let listener = self
                .listener
                .as_mut()
                .ok_or_else(|| Error::AlreadyShutdown)?;
            let (stream, stream_addr) = listener.accept()?;
            debug!("Connection established with downstream {}", stream_addr);
            stream.set_nodelay(true)?;
            self.stream = Some(NetworkStream::new(stream, self.timeout_ms));
        }

        self.stream.as_mut().ok_or_else(|| Error::NoActiveStream)
    }
}

struct NetworkStream {
    stream: TcpStream,
    buffer: Vec<u8>,
    temp_buffer: [u8; 1024],
}

impl NetworkStream {
    pub fn new(stream: TcpStream, timeout_ms: u64) -> Self {
        let timeout = Some(std::time::Duration::from_millis(timeout_ms));
        // These only fail if a duration of 0 is passed in.
        stream.set_read_timeout(timeout).unwrap();
        stream.set_write_timeout(timeout).unwrap();

        Self {
            stream,
            buffer: Vec::new(),
            temp_buffer: [0; 1024],
        }
    }

    /// Blocking read until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>, Error> {
        let result = self.read_buffer();
        if !result.is_empty() {
            return Ok(result);
        }

        loop {
            trace!("Attempting to read from stream");
            let read = self.stream.read(&mut self.temp_buffer)?;
            trace!("Read {} bytes from stream", read);
            if read == 0 {
                return Err(Error::RemoteStreamClosed);
            }
            self.buffer.extend(self.temp_buffer[..read].to_vec());
            let result = self.read_buffer();
            if !result.is_empty() {
                trace!("Found a message in the stream");
                return Ok(result);
            }
            trace!("Did not find a message yet, reading again");
        }
    }

    /// Terminate the socket
    pub fn shutdown(&self) -> Result<(), Error> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    /// Blocking write until able to successfully send an entire message
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        let u32_max = u32::max_value() as usize;
        if u32_max <= data.len() {
            return Err(Error::DataTooLarge(data.len()));
        }
        let data_len = data.len() as u32;
        trace!("Attempting to write length, {},  to the stream", data_len);
        self.write_all(&data_len.to_le_bytes())?;
        trace!("Attempting to write data, {},  to the stream", data_len);
        self.write_all(data)?;
        trace!(
            "Successfully wrote length, {}, and data to the stream",
            data_len
        );
        Ok(())
    }

    /// Data sent on a TCP socket may not necessarily be delivered at the exact time. So a read may
    /// only include a subset of what was sent. This wraps around the TCP read buffer to ensure
    /// that only full messages are received.
    fn read_buffer(&mut self) -> Vec<u8> {
        if self.buffer.len() < 4 {
            return Vec::new();
        }

        let mut u32_bytes = [0; 4];
        u32_bytes.copy_from_slice(&self.buffer[..4]);
        let data_size = u32::from_le_bytes(u32_bytes) as usize;

        let remaining_data = &self.buffer[4..];
        if remaining_data.len() < data_size {
            return Vec::new();
        }

        let returnable_data = remaining_data[..data_size].to_vec();
        self.buffer = remaining_data[data_size..].to_vec();
        returnable_data
    }

    /// Writing to a TCP socket will take in as much data as the underlying buffer has space for.
    /// This wraps around that buffer and blocks until all the data has been pushed.
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        let mut unwritten = &data[..];
        let mut total_written = 0;

        while !unwritten.is_empty() {
            let written = self.stream.write(unwritten)?;
            total_written += written;
            unwritten = &data[total_written..];
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libra_config::utils;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    /// Read, Write, Connect timeout in milliseconds.
    const TIMEOUT: u64 = 5_000;

    #[test]
    fn test_ping() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);

        let data = vec![0, 1, 2, 3];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);

        let data = vec![4, 5, 6, 7];
        server.write(&data).unwrap();
        let result = client.read().unwrap();
        assert_eq!(data, result);
    }

    #[test]
    fn test_client_shutdown() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);

        let data = vec![0, 1, 2, 3];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);

        client.shutdown().unwrap();
        let mut client = NetworkClient::new(server_addr, TIMEOUT);
        assert!(server.read().is_err());

        let data = vec![4, 5, 6, 7];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);
    }

    #[test]
    fn test_server_shutdown() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);

        let data = vec![0, 1, 2, 3];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);

        server.shutdown().unwrap();
        let mut server = NetworkServer::new(server_addr, TIMEOUT);

        let data = vec![4, 5, 6, 7];
        // We aren't notified immediately that a server has shutdown, but it happens eventually
        while client.write(&data).is_ok() {}

        let data = vec![8, 9, 10, 11];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);
    }

    #[test]
    fn test_write_two_messages_buffered() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);
        let data1 = vec![0, 1, 2, 3];
        let data2 = vec![4, 5, 6, 7];
        client.write(&data1).unwrap();
        client.write(&data2).unwrap();
        let result1 = server.read().unwrap();
        let result2 = server.read().unwrap();
        assert_eq!(data1, result1);
        assert_eq!(data2, result2);
    }

    #[test]
    fn test_server_timeout() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);
        let data1 = vec![0, 1, 2, 3];
        let data2 = vec![4, 5, 6, 7];

        // First client, success
        client.write(&data1).unwrap();
        let result1 = server.read().unwrap();
        assert_eq!(data1, result1);

        // Timedout
        server.read().unwrap_err();

        // New client, success, note the previous client connection is still active, the server is
        // actively letting it go due to lack of activity.
        let mut client2 = NetworkClient::new(server_addr, TIMEOUT);
        client2.write(&data2).unwrap();
        let result2 = server.read().unwrap();
        assert_eq!(data2, result2);
    }

    #[test]
    fn test_client_timeout() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr, TIMEOUT);
        let mut client = NetworkClient::new(server_addr, TIMEOUT);
        let data1 = vec![0, 1, 2, 3];
        let data2 = vec![4, 5, 6, 7];

        // First server success
        client.write(&data1).unwrap();
        let result1 = server.read().unwrap();
        assert_eq!(data1, result1);

        // Timedout, it is hard to simulate a client receiving a write timeout
        client.read().unwrap_err();

        // Clean up old Server listener but keep the stream online. Start a new server, which will
        // be the one the client now connects to.
        server.listener = None;
        let mut server2 = NetworkServer::new(server_addr, TIMEOUT);

        // Client starts a new stream, success
        client.write(&data2).unwrap();
        let result2 = server2.read().unwrap();
        assert_eq!(data2, result2);
    }
}
