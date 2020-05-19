// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod backup;
pub mod restore;
pub mod storage;

#[cfg(test)]
mod tests;

use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::convert::TryInto;
use tokio::io::{AsyncRead, AsyncReadExt};

#[async_trait]
trait ReadRecordBytes {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()>;
    async fn read_record_bytes(&mut self) -> Result<Option<Bytes>>;
}

#[async_trait]
impl<R: AsyncRead + Send + Unpin> ReadRecordBytes for R {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()> {
        assert_eq!(buf.len(), 0);
        let n_expected = buf.capacity();

        loop {
            let n_read = self.read_buf(buf).await?;
            let n_read_total = buf.len();
            if n_read_total == n_expected {
                return Ok(());
            }
            if n_read == 0 {
                if n_read_total == 0 {
                    return Ok(());
                } else {
                    bail!(
                        "Hit EOF before filling the whole buffer, read {}, expected {}",
                        n_read_total,
                        n_expected
                    );
                }
            }
        }
    }

    async fn read_record_bytes(&mut self) -> Result<Option<Bytes>> {
        // read record size
        let mut size_buf = BytesMut::with_capacity(4);
        self.read_full_buf_or_none(&mut size_buf).await?;
        if size_buf.is_empty() {
            return Ok(None);
        }

        // read record
        let record_size = u32::from_be_bytes(size_buf.as_ref().try_into()?) as usize;
        let mut record_buf = BytesMut::with_capacity(record_size);
        self.read_full_buf_or_none(&mut record_buf).await?;
        if record_buf.is_empty() {
            bail!("Hit EOF when reading record.")
        }

        Ok(Some(record_buf.freeze()))
    }
}
