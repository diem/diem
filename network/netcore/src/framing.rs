// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::{convert::TryInto, io::Result};

/// Read a u16 length prefixed frame from `Stream` into `buf`.
pub async fn read_u16frame<'stream, 'buf, 'c, TSocket>(
    mut stream: &'stream mut TSocket,
    buf: &'buf mut BytesMut,
) -> Result<()>
where
    'stream: 'c,
    'buf: 'c,
    TSocket: AsyncRead + Unpin,
{
    let len = read_u16frame_len(&mut stream).await?;
    buf.resize(len as usize, 0);
    stream.read_exact(buf.as_mut()).await?;
    Ok(())
}

/// Read a u16 (encoded as BE bytes) from `Stream` and return the length.
async fn read_u16frame_len<TSocket>(stream: &mut TSocket) -> Result<u16>
where
    TSocket: AsyncRead + Unpin,
{
    let mut len_buf = [0, 0];
    stream.read_exact(&mut len_buf).await?;

    Ok(u16::from_be_bytes(len_buf))
}

/// Write the contents of `buf` to `stream` prefixed with a u16 length.
/// The length of `buf` must be less than or equal to u16::max_value().
///
/// Caller is responsible for flushing the write to `stream`.
pub async fn write_u16frame<'stream, 'buf, 'c, TSocket>(
    mut stream: &'stream mut TSocket,
    buf: &'buf [u8],
) -> Result<()>
where
    'stream: 'c,
    'buf: 'c,
    TSocket: AsyncWrite + Unpin,
{
    let len = buf
        .len()
        .try_into()
        // TODO Maybe use our own Error Type?
        .map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "Too big"))?;
    write_u16frame_len(&mut stream, len).await?;
    stream.write_all(buf).await?;

    Ok(())
}

/// Write a u16 `len` as BE bytes to `stream`.
///
/// Caller is responsible for flushing the write to `stream`.
async fn write_u16frame_len<TSocket>(stream: &mut TSocket, len: u16) -> Result<()>
where
    TSocket: AsyncWrite + Unpin,
{
    let len = u16::to_be_bytes(len);
    stream.write_all(&len).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{read_u16frame, read_u16frame_len, write_u16frame, write_u16frame_len};
    use bytes::BytesMut;
    use futures::{executor::block_on, io::AsyncWriteExt};
    use memsocket::MemorySocket;
    use std::io::Result;

    #[test]
    fn write_read_u16frame_len() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();

        block_on(write_u16frame_len(&mut a, 17))?;
        block_on(a.flush())?;
        let len = block_on(read_u16frame_len(&mut b))?;
        assert_eq!(len, 17);

        Ok(())
    }

    #[test]
    fn read_u16frame_len_eof() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();

        block_on(a.write_all(&[42]))?;
        block_on(a.flush())?;
        drop(a);

        let result = block_on(read_u16frame_len(&mut b));
        assert!(result.is_err(), true);

        Ok(())
    }

    #[test]
    fn write_u16frame_len_eof() -> Result<()> {
        let (mut a, b) = MemorySocket::new_pair();
        drop(b);

        let result = block_on(a.write_all(&[42]));
        assert!(result.is_err(), true);

        Ok(())
    }

    #[test]
    fn write_read_u16frame() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();

        block_on(write_u16frame(&mut a, b"The Name of the Wind"))?;
        block_on(a.flush())?;

        let mut buf = BytesMut::new();
        block_on(read_u16frame(&mut b, &mut buf))?;

        assert_eq!(buf.as_ref(), b"The Name of the Wind");

        Ok(())
    }

    #[test]
    fn write_read_multiple_u16frames() -> Result<()> {
        let (mut a, mut b) = MemorySocket::new_pair();

        block_on(write_u16frame(&mut a, b"The Name of the Wind"))?;
        block_on(write_u16frame(&mut b, b"The Wise Man's Fear"))?;
        block_on(b.flush())?;
        block_on(write_u16frame(&mut a, b"The Doors of Stone"))?;
        block_on(a.flush())?;

        let mut buf = BytesMut::new();
        block_on(read_u16frame(&mut b, &mut buf))?;
        assert_eq!(buf.as_ref(), b"The Name of the Wind");
        block_on(read_u16frame(&mut b, &mut buf))?;
        assert_eq!(buf.as_ref(), b"The Doors of Stone");
        block_on(read_u16frame(&mut a, &mut buf))?;
        assert_eq!(buf.as_ref(), b"The Wise Man's Fear");

        Ok(())
    }

    #[test]
    fn write_large_u16frame() -> Result<()> {
        let (mut a, _b) = MemorySocket::new_pair();

        let mut buf = Vec::new();
        buf.resize((u16::max_value() as usize) * 2, 0);

        let result = block_on(write_u16frame(&mut a, &buf));
        assert!(result.is_err(), true);

        Ok(())
    }
}
