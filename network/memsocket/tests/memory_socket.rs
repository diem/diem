// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{
    executor::block_on,
    io::{AsyncReadExt, AsyncWriteExt},
};
use memsocket::MemorySocket;
use std::io::Result;

#[test]
fn simple_write_read() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"hello world"))?;
    block_on(a.flush())?;
    drop(a);

    let mut v = Vec::new();
    block_on(b.read_to_end(&mut v))?;
    assert_eq!(v, b"hello world");

    Ok(())
}

#[test]
fn partial_read() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"foobar"))?;
    block_on(a.flush())?;

    let mut buf = [0; 3];
    block_on(b.read_exact(&mut buf))?;
    assert_eq!(&buf, b"foo");
    block_on(b.read_exact(&mut buf))?;
    assert_eq!(&buf, b"bar");

    Ok(())
}

#[test]
fn partial_read_write_both_sides() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"foobar"))?;
    block_on(a.flush())?;
    block_on(b.write_all(b"stormlight"))?;
    block_on(b.flush())?;

    let mut buf_a = [0; 5];
    let mut buf_b = [0; 3];
    block_on(a.read_exact(&mut buf_a))?;
    assert_eq!(&buf_a, b"storm");
    block_on(b.read_exact(&mut buf_b))?;
    assert_eq!(&buf_b, b"foo");

    block_on(a.read_exact(&mut buf_a))?;
    assert_eq!(&buf_a, b"light");
    block_on(b.read_exact(&mut buf_b))?;
    assert_eq!(&buf_b, b"bar");

    Ok(())
}

#[test]
fn many_small_writes() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"words"))?;
    block_on(a.write_all(b" "))?;
    block_on(a.write_all(b"of"))?;
    block_on(a.write_all(b" "))?;
    block_on(a.write_all(b"radiance"))?;
    block_on(a.flush())?;
    drop(a);

    let mut buf = [0; 17];
    block_on(b.read_exact(&mut buf))?;
    assert_eq!(&buf, b"words of radiance");

    Ok(())
}

#[test]
fn read_zero_bytes() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"way of kings"))?;
    block_on(a.flush())?;

    let mut buf = [0; 12];
    block_on(b.read_exact(&mut buf[..0]))?;
    assert_eq!(buf, [0; 12]);
    block_on(b.read_exact(&mut buf))?;
    assert_eq!(&buf, b"way of kings");

    Ok(())
}

#[test]
fn read_bytes_with_large_buffer() -> Result<()> {
    let (mut a, mut b) = MemorySocket::new_pair();

    block_on(a.write_all(b"way of kings"))?;
    block_on(a.flush())?;

    let mut buf = [0; 20];
    let bytes_read = block_on(b.read(&mut buf))?;
    assert_eq!(bytes_read, 12);
    assert_eq!(&buf[..12], b"way of kings");

    Ok(())
}
