// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This provides a simple networking substrate between a client and server. It is assumed that all
//! operations are blocking and return only complete blocks of data. The intended use case has the
//! server blocking on read.  Upon receiving a payload during a read, the server should process the
//! payload, write a response, and then block on read again. The client should block on read after
//! performing a write. Note: if the server will attempt to accept or acquire a new client if one
//! does not exist prior to performing any operations.
//!
//! Internally both the client and server leverage a NetworkStream that communications in blocks
//! where a block is a length prefixed array of bytes.

use anyhow::{anyhow, ensure, Result};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread, time,
};

pub struct NetworkClient {
    stream: NetworkStream,
}

impl NetworkClient {
    pub fn connect(server: SocketAddr) -> Result<Self> {
        let mut stream = TcpStream::connect(server);

        let mut attempts = 0;
        let sleeptime = time::Duration::from_millis(100);

        while stream.is_err() && attempts < 5 {
            thread::sleep(sleeptime);
            attempts += 1;
            stream = TcpStream::connect(server);
        }
        let stream = stream?;
        stream.set_nodelay(true)?;

        Ok(Self {
            stream: NetworkStream::new(stream),
        })
    }

    /// Blocking read until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>> {
        self.stream.read()
    }

    /// Blocking read until able to successfully read an entire message
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write(data)
    }
}

pub struct NetworkServer {
    listener: TcpListener,
    stream: Option<NetworkStream>,
}

impl NetworkServer {
    pub fn new(listen: SocketAddr) -> Self {
        let listener = TcpListener::bind(listen).unwrap();
        Self {
            listener,
            stream: None,
        }
    }

    /// If there isn't already a downstream client, it accepts. Otherwise it
    /// blocks until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>> {
        let stream = self.client()?;
        stream.read()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.client()?;
        stream.write(data)
    }

    fn client(&mut self) -> Result<&mut NetworkStream> {
        if self.stream.is_none() {
            let (stream, _stream_addr) = self.listener.accept()?;
            stream.set_nodelay(true)?;
            self.stream = Some(NetworkStream::new(stream));
        }

        self.stream
            .as_mut()
            .ok_or_else(|| anyhow!("Unable to retrieve a stream"))
    }
}

struct NetworkStream {
    stream: TcpStream,
    buffer: Vec<u8>,
    temp_buffer: [u8; 1024],
}

impl NetworkStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
            temp_buffer: [0; 1024],
        }
    }

    /// Blocking read until able to successfully read an entire message
    pub fn read(&mut self) -> Result<Vec<u8>> {
        let result = self.read_buffer();
        if !result.is_empty() {
            return Ok(result);
        }

        loop {
            let read = self.stream.read(&mut self.temp_buffer)?;
            self.buffer.extend(self.temp_buffer[0..read].to_vec());
            let result = self.read_buffer();
            if !result.is_empty() {
                return Ok(result);
            }
        }
    }

    /// Blocking write until able to successfully send an entire message
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        let u32_max = u32::max_value() as usize;
        ensure!(
            data.len() <= u32_max,
            "Found data that is too large to decode: {}",
            data.len()
        );
        let data_len = data.len() as u32;
        self.write_all(&data_len.to_le_bytes())?;
        self.write_all(data)?;
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
        u32_bytes.copy_from_slice(&self.buffer[0..4]);
        let data_size = u32::from_le_bytes(u32_bytes) as usize;

        let remaining_data = &self.buffer[4..];
        if remaining_data.len() < data_size {
            return Vec::new();
        }

        let returnable_data = remaining_data[..data_size].to_vec();
        if remaining_data.len() > data_size {
            self.buffer = remaining_data[data_size + 1..].to_vec();
        } else {
            self.buffer = Vec::new();
        }
        returnable_data
    }

    /// Writing to a TCP socket will take in as much data as the underlying buffer has space for.
    /// This wraps around that buffer and blocks until all the data has been pushed.
    fn write_all(&mut self, data: &[u8]) -> Result<()> {
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

    #[test]
    fn test_ping() {
        let server_port = utils::get_available_port();
        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
        let mut server = NetworkServer::new(server_addr);
        let mut client = NetworkClient::connect(server_addr).unwrap();

        let data = vec![0, 1, 2, 3];
        client.write(&data).unwrap();
        let result = server.read().unwrap();
        assert_eq!(data, result);

        let data = vec![4, 5, 6, 7];
        server.write(&data).unwrap();
        let result = client.read().unwrap();
        assert_eq!(data, result);
    }
}
