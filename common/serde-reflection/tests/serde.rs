// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bincode;
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_json;
use serde_reflection::{
    ContainerFormat, Error, Format, Named, SerializationRecords, Tracer, Value, VariantFormat,
};
use serde_yaml;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
enum E {
    Unit,
    Newtype(u16),
    Tuple(u16, Option<bool>),
    Struct { a: u32 },
    NewTupleArray((u16, u16, u16)),
}

fn test_variant(tracer: &mut Tracer, expr: E, expected_value: Value) {
    let mut records = SerializationRecords::new();
    let (format, value) = tracer.trace_value(&mut records, &expr).unwrap();
    // Check the local result of tracing.
    assert_eq!(format, Format::TypeName("E".into()));
    assert_eq!(value, expected_value);
    // Check that the value deserializes back into the Rust value `expr`.
    assert_eq!(expr, E::deserialize(value.into_deserializer()).unwrap());
}

#[test]
fn test_tracers() {
    let mut tracer = Tracer::new(/* is_human_readable */ false);

    test_variant(
        &mut tracer,
        E::Unit,
        Value::Variant(0, Box::new(Value::Unit)),
    );
    test_variant(
        &mut tracer,
        E::Newtype(1),
        Value::Variant(1, Box::new(Value::U16(1))),
    );
    test_variant(
        &mut tracer,
        E::Tuple(1, Some(true)),
        Value::Variant(
            2,
            Box::new(Value::Seq(vec![
                Value::U16(1),
                Value::Option(Some(Box::new(Value::Bool(true)))),
            ])),
        ),
    );
    test_variant(
        &mut tracer,
        E::Struct { a: 1 },
        Value::Variant(3, Box::new(Value::Seq(vec![Value::U32(1)]))),
    );
    test_variant(
        &mut tracer,
        E::NewTupleArray((1, 2, 3)),
        Value::Variant(
            4,
            Box::new(Value::Seq(vec![
                Value::U16(1),
                Value::U16(2),
                Value::U16(3),
            ])),
        ),
    );

    let registry = tracer.registry().unwrap();
    let format = registry.get("E").unwrap();
    let variants = match format {
        ContainerFormat::Enum(variants) => variants,
        _ => {
            unreachable!();
        }
    };

    // User provided identifier were scraped.
    assert_eq!(variants.len(), 5);
    assert_eq!(variants.get(&0).unwrap().name, "Unit");
    assert_eq!(variants.get(&1).unwrap().name, "Newtype");
    assert_eq!(variants.get(&2).unwrap().name, "Tuple");
    assert_eq!(variants.get(&3).unwrap().name, "Struct");
    assert_eq!(variants.get(&4).unwrap().name, "NewTupleArray");

    // Variant definitions were scraped.
    assert_eq!(variants.get(&0).unwrap().value, VariantFormat::Unit);
    assert_eq!(
        variants.get(&1).unwrap().value,
        VariantFormat::NewType(Box::new(Format::U16))
    );
    assert_eq!(
        variants.get(&2).unwrap().value,
        VariantFormat::Tuple(vec![Format::U16, Format::Option(Box::new(Format::Bool))])
    );
    assert_eq!(
        variants.get(&3).unwrap().value,
        VariantFormat::Struct(vec![Named {
            name: "a".into(),
            value: Format::U32
        }])
    );
    assert_eq!(
        variants.get(&4).unwrap().value,
        VariantFormat::NewType(Box::new(Format::TupleArray {
            content: Box::new(Format::U16),
            size: 3
        })),
    );

    // Format values can serialize and deserialize in text and binary encodings.
    let data = serde_json::to_string_pretty(format).unwrap();
    println!("{}\n", data);
    let format2 = serde_json::from_str::<ContainerFormat>(&data).unwrap();
    assert_eq!(*format, format2);

    let data = serde_yaml::to_string(format).unwrap();
    println!("{}\n", data);
    let format3 = serde_yaml::from_str::<ContainerFormat>(&data).unwrap();
    assert_eq!(*format, format3);

    let data = bincode::serialize(format).unwrap();
    let format4 = bincode::deserialize(&data).unwrap();
    assert_eq!(*format, format4);

    // Tracing deserialization
    let records = SerializationRecords::new();
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    let (ident, samples) = tracer.trace_type::<E>(&records).unwrap();
    assert_eq!(ident, Format::TypeName("E".into()));
    assert_eq!(tracer.registry().unwrap().get("E").unwrap(), format);
    assert_eq!(
        samples,
        vec![
            E::Unit,
            E::Newtype(0),
            E::Tuple(0, Some(false)),
            E::Struct { a: 0 },
            E::NewTupleArray((0, 0, 0))
        ]
    );
}

#[derive(Serialize, PartialEq, Eq, Debug, Clone)]
struct Name(String);

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        // Make sure to wrap our value in a container with the same name
        // as the original type.
        #[derive(Deserialize)]
        #[serde(rename = "Name")]
        struct InternalValue(String);

        let value = InternalValue::deserialize(deserializer)?.0;
        // Enforce some custom invariant
        if value.len() >= 2 && value.chars().all(char::is_alphabetic) {
            Ok(Name(value))
        } else {
            Err(<D::Error as ::serde::de::Error>::custom(format!(
                "Invalid name {}",
                value
            )))
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
enum Person {
    NickName(Name),
    FullName { first: Name, last: Name },
}

#[test]
fn test_trace_deserialization_with_custom_invariants() {
    let mut records = SerializationRecords::new();
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    // Type trace alone cannot guess a valid value for `Name`.
    assert_eq!(
        tracer.trace_type::<Person>(&records).unwrap_err(),
        Error::Custom(format!("Invalid name {}", "")),
    );

    // Let's trace a sample Rust value first. We obtain an abstract value as a side effect.
    let bob = Name("Bob".into());
    let (format, value) = tracer.trace_value(&mut records, &bob).unwrap();
    assert_eq!(format, Format::TypeName("Name".into()));
    assert_eq!(value, Value::Str("Bob".into()));
    assert_eq!(records.value("Name"), Some(&value));

    // Now try again.
    let (format, samples) = tracer.trace_type::<Person>(&records).unwrap();
    assert_eq!(format, Format::TypeName("Person".into()));
    assert_eq!(
        samples,
        vec![
            Person::NickName(bob.clone()),
            Person::FullName {
                first: bob.clone(),
                last: bob,
            }
        ]
    );

    // We now have a description of all Serde formats under `Person`.
    let registry = tracer.registry().unwrap();
    assert_eq!(
        registry.get("Name").unwrap(),
        &ContainerFormat::NewTypeStruct(Box::new(Format::Str))
    );
    let mut variants: BTreeMap<_, _> = BTreeMap::new();
    variants.insert(
        0,
        Named {
            name: "NickName".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("Name".into()))),
        },
    );
    variants.insert(
        1,
        Named {
            name: "FullName".into(),
            value: VariantFormat::Struct(vec![
                Named {
                    name: "first".into(),
                    value: Format::TypeName("Name".into()),
                },
                Named {
                    name: "last".into(),
                    value: Format::TypeName("Name".into()),
                },
            ]),
        },
    );
    assert_eq!(
        registry.get("Person").unwrap(),
        &ContainerFormat::Enum(variants)
    );
}

mod foo {
    #[derive(super::Serialize)]
    pub struct A;
}

mod bar {
    #[derive(super::Serialize)]
    pub struct A(pub u32);
}

#[test]
fn test_name_clash_not_suported() {
    let mut records = SerializationRecords::new();
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    tracer.trace_value(&mut records, &foo::A).unwrap();
    // Repeating names is fine.
    assert!(tracer.trace_value(&mut records, &foo::A).is_ok());
    // but format have to match.
    assert!(tracer.trace_value(&mut records, &bar::A(0)).is_err());
}

#[test]
fn test_borrowed_slice() {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct Borrowed<'a>(&'a [u8]);

    let bytes = [1u8; 4];
    let mut records = SerializationRecords::new();
    let mut tracer = Tracer::new(/* is_human_readable */ false);

    let (format, value) = tracer.trace_value(&mut records, &Borrowed(&bytes)).unwrap();
    assert_eq!(format, Format::TypeName("Borrowed".into()));
    // Slice was traced and serialized as a sequence.
    assert_eq!(value, Value::Seq(vec![Value::U8(1); 4]));

    // Unfortunately, borrowed slices can only de-serialize as bytes.
    assert_eq!(
        tracer.trace_type::<Borrowed>(&records),
        Err(Error::UnexpectedDeserializationFormat(
            "Borrowed",
            ContainerFormat::NewTypeStruct(Box::new(Format::Seq(Box::new(Format::U8)))),
            "bytes"
        )),
    );
}

#[test]
fn test_borrowed_bytes() {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct Borrowed<'a>(#[serde(with = "serde_bytes")] &'a [u8]);

    let bytes = [1u8; 4];
    let mut records = SerializationRecords::new();
    let mut tracer = Tracer::new(/* is_human_readable */ false);

    let (format, value) = tracer.trace_value(&mut records, &Borrowed(&bytes)).unwrap();
    assert_eq!(format, Format::TypeName("Borrowed".into()));
    // Value was traced and serialized as a bytes.
    assert_eq!(value, Value::Bytes(bytes.to_vec()));

    let (format, samples) = tracer.trace_type::<Borrowed>(&records).unwrap();
    assert_eq!(format, Format::TypeName("Borrowed".into()));
    assert_eq!(samples, vec![Borrowed(&bytes),]);

    let registry = tracer.registry().unwrap();
    assert_eq!(
        registry.get("Borrowed").unwrap(),
        &ContainerFormat::NewTypeStruct(Box::new(Format::Bytes))
    );
}
