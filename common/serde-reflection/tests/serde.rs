// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bincode;
use libra_serde_reflection::{Format, Named, Tracer, Value, VariantFormat};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_json;
use serde_yaml;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
enum E {
    Unit,
    Newtype(u16),
    Tuple(u16, Option<bool>),
    Struct { a: u32 },
    NewTupleArray((u16, u16, u16)),
}

fn test_variant(tracer: &mut Tracer, expr: E, expected_value: Value) {
    let (format, value) = tracer.trace_value(&expr).unwrap();
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
        Format::Variant(variants) => variants,
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
    let format2 = serde_json::from_str::<Format>(&data).unwrap();
    assert_eq!(*format, format2);

    let data = serde_yaml::to_string(format).unwrap();
    println!("{}\n", data);
    let format3 = serde_yaml::from_str::<Format>(&data).unwrap();
    assert_eq!(*format, format3);

    let data = bincode::serialize(format).unwrap();
    let format4 = bincode::deserialize(&data).unwrap();
    assert_eq!(*format, format4);

    // Tracing deserialization
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    assert_eq!(
        tracer.trace_type::<E>().unwrap(),
        Format::TypeName("E".into())
    );
    assert_eq!(tracer.registry().unwrap().get("E").unwrap(), format);

    // Sampling values with DTracer
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    let values = tracer.sample_type::<E>().unwrap();
    assert_eq!(
        values,
        vec![
            E::Unit,
            E::Newtype(0),
            E::Tuple(0, Some(false)),
            E::Struct { a: 0 },
            E::NewTupleArray((0, 0, 0))
        ]
    );
}
