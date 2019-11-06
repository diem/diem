use crypto::HashValue;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use super::ensure_slice_len_eq;
use failure::prelude::*;
use std::mem::size_of;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Write;
use super::BLOCK_INDEX_CF_NAME;

define_schema!(
    BlockIndexSchema,
    Height,
    BlockIndex,
    BLOCK_INDEX_CF_NAME
);

type Height = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockIndex {
    pub id: HashValue,
    pub parent_block_id: HashValue,
}

impl ValueCodec<BlockIndexSchema> for BlockIndex {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let mut encode_value = Vec::with_capacity(HashValue::LENGTH + HashValue::LENGTH);
        encode_value.write_all(&self.id.to_vec().as_slice())?;
        encode_value.write_all(&self.parent_block_id.to_vec().as_slice())?;
        Ok(encode_value)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let block_id = HashValue::from_slice(&data[..HashValue::LENGTH])?;
        let parent_block_id = HashValue::from_slice(&data[HashValue::LENGTH..])?;

        Ok(BlockIndex{id:block_id, parent_block_id})
    }
}

impl KeyCodec<BlockIndexSchema> for Height {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Height>())?;
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}