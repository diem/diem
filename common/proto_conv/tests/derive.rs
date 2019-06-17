// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

mod proto;

use proptest::prelude::*;
use proptest_derive::Arbitrary;
use proto_conv::{test_helper::assert_protobuf_encode_decode, FromProto, IntoProto};

macro_rules! test_conversion {
    ($struct_name: ident, $test_name: ident, $field_type: ty) => {
        #[derive(Arbitrary, Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
        #[ProtoType(crate::proto::test::$struct_name)]
        struct $struct_name {
            value: $field_type,
        }

        proptest! {
            #[test]
            fn $test_name(rust_object in any::<$struct_name>()) {
                let proto_object = rust_object.clone().into_proto();
                prop_assert_eq!(rust_object.clone().value, proto_object.get_value());

                let rust_object2 = $struct_name::from_proto(proto_object)
                    .expect("Converting Protobuf object to Rust object should work.");
                prop_assert_eq!(rust_object, rust_object2);
            }
        }
    };
}

test_conversion!(Int32, test_convert_int32, i32);
test_conversion!(Int64, test_convert_int64, i64);
test_conversion!(UInt32, test_convert_uint32, u32);
test_conversion!(UInt64, test_convert_uint64, u64);
test_conversion!(SInt32, test_convert_sint32, i32);
test_conversion!(SInt64, test_convert_sint64, i64);
test_conversion!(Fixed32, test_convert_fixed32, u32);
test_conversion!(Fixed64, test_convert_fixed64, u64);
test_conversion!(Boolean, test_convert_boolean, bool);
test_conversion!(Strings, test_convert_strings, String);
test_conversion!(Bytes, test_convert_bytes, Vec<u8>);

#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[ProtoType(crate::proto::test::Structs)]
struct Structs {
    a: Int32,
    b: UInt32,
    c: SInt32,
    d: Fixed32,
    e: Boolean,
    f: Strings,
    g: Bytes,
    h: Vec<Int32>,
}

proptest! {
    #[test]
    fn test_convert_vecs(rust_object in any::<Vec<Int32>>()) {
        assert_protobuf_encode_decode(&rust_object);
    }

    #[test]
    fn test_convert_structs(rust_object in any::<Structs>()) {
        let proto_object = rust_object.clone().into_proto();
        prop_assert_eq!(rust_object.clone().a.value, proto_object.get_a().get_value());
        prop_assert_eq!(rust_object.clone().b.value, proto_object.get_b().get_value());
        prop_assert_eq!(rust_object.clone().c.value, proto_object.get_c().get_value());
        prop_assert_eq!(rust_object.clone().d.value, proto_object.get_d().get_value());
        prop_assert_eq!(rust_object.clone().e.value, proto_object.get_e().get_value());
        prop_assert_eq!(rust_object.clone().f.value, proto_object.get_f().get_value());
        prop_assert_eq!(rust_object.clone().g.value, proto_object.get_g().get_value());

        let rust_object2 = Structs::from_proto(proto_object)
            .expect("Converting Protobuf object to Rust object should work.");
        prop_assert_eq!(rust_object, rust_object2);
    }
}
