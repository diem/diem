use crate::schema::{ensure_slice_len_eq, BLOCK_INDEX_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use libra_crypto::HashValue;
use libra_types::block_index::BlockIndex;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::io::Write;
use std::mem::size_of;

define_schema!(BlockIndexSchema, Height, BlockIndex, BLOCK_INDEX_CF_NAME);

type Height = u64;

impl ValueCodec<BlockIndexSchema> for BlockIndex {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let mut encode_value = Vec::with_capacity(HashValue::LENGTH + HashValue::LENGTH);
        encode_value.write_all(&self.id().to_vec().as_slice())?;
        encode_value.write_all(&self.parent_id().to_vec().as_slice())?;
        Ok(encode_value)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let block_id = HashValue::from_slice(&data[..HashValue::LENGTH])?;
        let parent_block_id = HashValue::from_slice(&data[HashValue::LENGTH..])?;

        Ok(BlockIndex::new(&block_id, &parent_block_id))
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
