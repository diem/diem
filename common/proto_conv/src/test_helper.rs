// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{FromProto, FromProtoBytes, IntoProto, IntoProtoBytes};
use std::fmt::Debug;

/// Assert that protobuf encoding and decoding roundtrips correctly.
///
/// This is meant to be used for `protobuf::Message` instances. For non-message instances, see
/// `assert_protobuf_encode_decode_non_message`.
pub fn assert_protobuf_encode_decode<P, T>(object: &T)
where
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
    P: protobuf::Message,
{
    test_into_from_proto(object);
    test_into_from_proto_bytes(object);
}

/// Assert that protobuf encoding and decoding roundtrips correctly (non-message version).
pub fn assert_protobuf_encode_decode_non_message<P, T>(object: &T)
where
    T: FromProto<ProtoType = P> + IntoProto<ProtoType = P> + Clone + Debug + Eq,
{
    test_into_from_proto(object);
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
