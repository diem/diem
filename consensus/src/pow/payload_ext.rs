use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use failure::prelude::*;
use libra_types::transaction::SignedTransaction;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct BlockPayloadExt {
    pub txns: Vec<SignedTransaction>,
    pub nonce: u64,
    pub solve: Vec<u32>,
}

impl BlockPayloadExt {
    pub fn get_txns(&self) -> Vec<SignedTransaction> {
        self.txns.clone()
    }

    /// Dumps into a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        SimpleSerializer::<Vec<u8>>::serialize(self).expect("BlockPayloadExt serialization failed")
    }
}

impl CanonicalSerialize for BlockPayloadExt {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_vec(self.txns.as_ref())?
            .encode_u64(self.nonce)?
            .encode_vec(self.solve.as_ref())?;
        Ok(())
    }
}

impl fmt::Debug for BlockPayloadExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BlockPayloadExt {{ \n \
             {{ txns: {:#?}, \n \
             nonce: {:#?}, \n \
             solve: {:#?}, \n \
             }} \n \
             }}",
            self.txns, self.nonce, self.solve,
        )
    }
}

impl CanonicalDeserialize for BlockPayloadExt {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let txns = deserializer.decode_vec()?;
        let nonce = deserializer.decode_u64()?;
        let solve = deserializer.decode_vec()?;

        Ok(BlockPayloadExt { txns, nonce, solve })
    }
}
