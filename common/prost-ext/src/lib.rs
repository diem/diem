use bytes::{Bytes, BytesMut};
use prost::{EncodeError, Message};

impl<T: ?Sized> MessageExt for T where T: Message {}

pub trait MessageExt: Message {
    fn to_bytes(&self) -> Result<Bytes, EncodeError>
    where
        Self: Sized,
    {
        let mut bytes = BytesMut::with_capacity(self.encoded_len());
        self.encode(&mut bytes)?;
        Ok(bytes.freeze())
    }

    fn to_vec(&self) -> Result<Vec<u8>, EncodeError>
    where
        Self: Sized,
    {
        let mut vec = Vec::with_capacity(self.encoded_len());
        self.encode(&mut vec)?;
        Ok(vec)
    }
}
