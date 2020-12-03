// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{
    executor::block_on,
    io::{AsyncReadExt, AsyncWriteExt},
    stream::StreamExt,
};
use memsocket::{MemoryListener, MemorySocket};
use std::io::Result;

#[test]
fn listener_bind() -> Result<()> {
    let listener = MemoryListener::bind(42)?;
    assert_eq!(listener.local_addr(), 42);

    Ok(())
}

#[test]
fn simple_connect() -> Result<()> {
    let mut listener = MemoryListener::bind(10)?;

    let mut dialer = MemorySocket::connect(10)?;
    let mut listener_socket = block_on(listener.incoming().next()).unwrap()?;

    block_on(dialer.write_all(b"foo"))?;
    block_on(dialer.flush())?;

    let mut buf = [0; 3];
    block_on(listener_socket.read_exact(&mut buf))?;
    assert_eq!(&buf, b"foo");

    Ok(())
}

#[test]
fn listen_on_port_zero() -> Result<()> {
    let mut listener = MemoryListener::bind(0)?;
    let listener_addr = listener.local_addr();

    let mut dialer = MemorySocket::connect(listener_addr)?;
    let mut listener_socket = block_on(listener.incoming().next()).unwrap()?;

    block_on(dialer.write_all(b"foo"))?;
    block_on(dialer.flush())?;

    let mut buf = [0; 3];
    block_on(listener_socket.read_exact(&mut buf))?;
    assert_eq!(&buf, b"foo");

    block_on(listener_socket.write_all(b"bar"))?;
    block_on(listener_socket.flush())?;

    let mut buf = [0; 3];
    block_on(dialer.read_exact(&mut buf))?;
    assert_eq!(&buf, b"bar");

    Ok(())
}

#[test]
fn listener_correctly_frees_port_on_drop() -> Result<()> {
    fn connect_on_port(port: u16) -> Result<()> {
        let mut listener = MemoryListener::bind(port)?;
        let mut dialer = MemorySocket::connect(port)?;
        let mut listener_socket = block_on(listener.incoming().next()).unwrap()?;

        block_on(dialer.write_all(b"foo"))?;
        block_on(dialer.flush())?;

        let mut buf = [0; 3];
        block_on(listener_socket.read_exact(&mut buf))?;
        assert_eq!(&buf, b"foo");

        Ok(())
    }

    connect_on_port(9)?;
    connect_on_port(9)?;

    Ok(())
}
