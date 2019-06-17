// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FromProto, FromProtoBytes, IntoProto, IntoProtoBytes};
use std::fmt::Debug;

pub fn assert_protobuf_encode_decode<P, T>(object: &T)
where
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
{
    object.assert_protobuf_encode_decode()
}

trait ProtobufEncodeDecodeTest<P>:
    FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq
{
    /// The default implementation tests conversion roundtrip via `{Into/From}Proto`. If the type
    /// implements `protobuf::Message`, also tests conversion roundtrip via
    /// `{Into/From}ProtoBytes`.
    fn assert_protobuf_encode_decode(&self);
}

impl<P, T> ProtobufEncodeDecodeTest<P> for T
where
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
{
    default fn assert_protobuf_encode_decode(&self) {
        test_into_from_proto(self);
    }
}

impl<P, T> ProtobufEncodeDecodeTest<P> for T
where
    P: protobuf::Message,
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
{
    fn assert_protobuf_encode_decode(&self) {
        test_into_from_proto(self);
        test_into_from_proto_bytes(self);
    }
}

/// Tests conversion roundtrip via `{Into/From}Proto`.
fn test_into_from_proto<P, T>(object: &T)
where
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
{
    let proto = object.clone().into_proto();
    let from_proto = T::from_proto(proto).expect("Should convert.");
    assert_eq!(*object, from_proto);
}

/// Tests conversion roundtrip via `{Into/From}ProtoBytes`.
fn test_into_from_proto_bytes<P, T>(object: &T)
where
    T: FromProtoBytes<P> + IntoProtoBytes<P> + Clone + Debug + Eq,
{
    let proto_bytes = object.clone().into_proto_bytes().expect("Should convert.");
    let from_proto_bytes = T::from_proto_bytes(&proto_bytes).expect("Should convert.");
    assert_eq!(*object, from_proto_bytes);
}
