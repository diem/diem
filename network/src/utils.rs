// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::NetworkError;
use bytes::Bytes;
use futures::{io::AsyncRead, stream::StreamExt};
pub use libra_prost_ext::MessageExt;
use netcore::compat::IoCompat;
use std::io;
use tokio::codec::{Framed, LengthDelimitedCodec};

pub async fn read_proto<T, TSubstream>(
    substream: &mut Framed<IoCompat<TSubstream>, LengthDelimitedCodec>,
) -> Result<T, NetworkError>
where
    T: prost::Message + Default,
    TSubstream: AsyncRead + Unpin,
{
    // Read from stream.
    let data: Bytes = substream.next().await.map_or_else(
        || Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
        |data| Ok(data?.freeze()),
    )?;
    // Parse to message.
    let msg = T::decode(data)?;
    Ok(msg)
}
