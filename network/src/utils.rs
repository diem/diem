// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::NetworkError;
use bytes::Bytes;
use futures::{
    compat::{Compat, Compat01As03Sink},
    io::AsyncRead,
    stream::StreamExt,
};
pub use prost_ext::MessageExt;
use std::io;
use tokio::codec::Framed;
use unsigned_varint::codec::UviBytes;

pub async fn read_proto<T, TSubstream>(
    substream: &mut Compat01As03Sink<Framed<Compat<TSubstream>, UviBytes<Bytes>>, Bytes>,
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
