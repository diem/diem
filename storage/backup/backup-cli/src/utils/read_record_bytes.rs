// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::error_notes::ErrorNotes;
use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::convert::TryInto;
use tokio::io::{AsyncRead, AsyncReadExt};

#[async_trait]
pub trait ReadRecordBytes {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()>;
    async fn read_record_bytes(&mut self) -> Result<Option<Bytes>>;
}

#[async_trait]
impl<R: AsyncRead + Send + Unpin> ReadRecordBytes for R {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()> {
        assert_eq!(buf.len(), 0);
        let n_expected = buf.capacity();

        loop {
            let n_read = self.read_buf(buf).await.err_notes("")?;
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

        // empty record
        let record_size = u32::from_be_bytes(size_buf.as_ref().try_into()?) as usize;
        if record_size == 0 {
            return Ok(Some(Bytes::new()));
        }

        // read record
        let mut record_buf = BytesMut::with_capacity(record_size);
        self.read_full_buf_or_none(&mut record_buf).await?;
        if record_buf.is_empty() {
            bail!("Hit EOF when reading record.")
        }

        Ok(Some(record_buf.freeze()))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::read_record_bytes::ReadRecordBytes;
    use tokio::runtime::Runtime;

    #[test]
    fn test_read_record_bytes() {
        Runtime::new().unwrap().block_on(async {
            let data = b"abc";
            let size = (data.len() as u32).to_be_bytes();

            let mut good_record = size.to_vec();
            good_record.extend_from_slice(data);

            assert_eq!(
                good_record
                    .as_slice()
                    .read_record_bytes()
                    .await
                    .unwrap()
                    .unwrap(),
                &data[..],
            );

            let mut eof: &[u8] = &[];
            assert!(eof.read_record_bytes().await.unwrap().is_none());

            let mut empty = &0u32.to_be_bytes()[..];
            assert_eq!(empty.read_record_bytes().await.unwrap().unwrap(), &[][..]);

            let mut data_missing = &1u32.to_be_bytes()[..];
            assert!(data_missing.read_record_bytes().await.is_err());

            let mut bad_len = 10u32.to_be_bytes().to_vec();
            bad_len.pop();
            assert!(bad_len.as_slice().read_record_bytes().await.is_err());

            let mut bad_data = 10u32.to_be_bytes().to_vec();
            bad_data.push(0u8);
            assert!(bad_data.as_slice().read_record_bytes().await.is_err());
        })
    }
}
