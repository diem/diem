// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bincode;
use libra_serde_reflection::{DTracer, Format, Named, Tracer, VariantFormat};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
enum E {
    Unit,
    Newtype(u16),
    Tuple(u16, u16),
    Struct { a: u32 },
    NewTupleArray((u16, u16, u16)),
}

#[test]
fn test_tracers() {
    let mut tracer = Tracer::new(/* is_human_readable */ false);
    let ident = Format::TypeName("E".into());

    let u = E::Unit;
    assert_eq!(tracer.trace(&u).unwrap(), ident);

    let n = E::Newtype(1);
    assert_eq!(tracer.trace(&n).unwrap(), ident);

    let t = E::Tuple(1, 2);
    assert_eq!(tracer.trace(&t).unwrap(), ident);

    let s = E::Struct { a: 1 };
    assert_eq!(tracer.trace(&s).unwrap(), ident);

    let r = E::NewTupleArray((1, 2, 3));
    assert_eq!(tracer.trace(&r).unwrap(), ident);

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
        VariantFormat::Tuple(vec![Format::U16, Format::U16])
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

    // DTracer
    let mut dtracer = DTracer::new(/* is_human_readable */ false);
    let ident = Format::TypeName("E".into());

    assert_eq!(dtracer.trace::<E>().unwrap(), ident);
    assert_eq!(dtracer.registry().unwrap().get("E").unwrap(), format);

    // Sampling values with DTracer
    let mut dtracer = DTracer::new(/* is_human_readable */ false);
    let values = dtracer.sample::<E>().unwrap();
    assert_eq!(
        values,
        vec![
            E::Unit,
            E::Newtype(0),
            E::Tuple(0, 0),
            E::Struct { a: 0 },
            E::NewTupleArray((0, 0, 0))
        ]
    );
}
