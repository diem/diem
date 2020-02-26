// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytes::Bytes;
use prost::{EncodeError, Message};

impl<T: ?Sized> MessageExt for T where T: Message {}

pub trait MessageExt: Message {
    fn to_bytes(&self) -> Result<Bytes, EncodeError>
    where
        Self: Sized,
    {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        self.encode(&mut bytes)?;
        Ok(bytes.into())
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

pub mod test_helpers {
    use super::MessageExt;
    use std::{convert::TryFrom, fmt::Debug};

    /// Assert that protobuf encoding and decoding roundtrips correctly.
    ///
    /// This is meant to be used for `prost::Message` instances.
    pub fn assert_protobuf_encode_decode<P, T>(object: &T)
    where
        T: TryFrom<P> + Into<P> + Clone + Debug + Eq,
        T::Error: Debug,
        P: prost::Message + Default,
    {
        let proto: P = object.clone().into();
        let proto_bytes = proto.to_bytes().unwrap();
        let from_proto = T::try_from(proto).unwrap();
        let from_proto_bytes = T::try_from(P::decode(proto_bytes).unwrap()).unwrap();
        assert_eq!(*object, from_proto);
        assert_eq!(*object, from_proto_bytes);
    }
}
