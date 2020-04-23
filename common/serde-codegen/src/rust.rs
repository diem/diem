// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::{BTreeMap, HashSet};

pub fn output(registry: &RegistryOwned) {
    let dependencies = analyzer::get_dependency_map(registry).unwrap();
    let entries = analyzer::best_effort_topological_sort(&dependencies).unwrap();

    output_preambule();
    let mut known_sizes = HashSet::new();
    for name in entries {
        let format = &registry[name];
        output_container(name, format, &known_sizes);
        known_sizes.insert(name);
    }
}

fn output_preambule() {
    println!(
        r#"
#![allow(unused_imports)]
use serde::{{Serialize, Deserialize}};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
"#
    );
}

fn quote_type(format: &Format, known_sizes: Option<&HashSet<&str>>) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if let Some(set) = known_sizes {
                if !set.contains(x.as_str()) {
                    return format!("Box<{}>", x);
                }
            }
            x.to_string()
        }
        Unit => "()".into(),
        Bool => "bool".into(),
        I8 => "i8".into(),
        I16 => "i16".into(),
        I32 => "i32".into(),
        I64 => "i64".into(),
        I128 => "i28".into(),
        U8 => "u8".into(),
        U16 => "u16".into(),
        U32 => "u32".into(),
        U64 => "u64".into(),
        U128 => "u128".into(),
        F32 => "f32".into(),
        F64 => "f64".into(),
        Char => "char".into(),
        Str => "String".into(),
        Bytes => "ByteBuf".into(),

        Option(format) => format!("Option<{}>", quote_type(format, known_sizes)),
        Seq(format) => format!("Vec<{}>", quote_type(format, None)),
        Map { key, value } => format!(
            "BTreeMap<{}, {}>",
            quote_type(key, None),
            quote_type(value, None)
        ),
        Tuple(formats) => format!("({})", quote_types(formats, known_sizes)),
        TupleArray { content, size } => {
            format!("[{}; {}]", quote_type(content, known_sizes), *size)
        }

        Unknown => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format], known_sizes: Option<&HashSet<&str>>) -> String {
    formats
        .iter()
        .map(|x| quote_type(x, known_sizes))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(
    indentation: usize,
    fields: &[Named<Format>],
    is_pub: bool,
    known_sizes: &HashSet<&str>,
) {
    let mut tab = " ".repeat(indentation);
    if is_pub {
        tab += " pub ";
    }
    for field in fields {
        println!(
            "{}{}: {},",
            tab,
            field.name,
            quote_type(&field.value, Some(known_sizes)),
        );
    }
}

fn output_variant(name: &str, variant: &VariantFormat, known_sizes: &HashSet<&str>) {
    use VariantFormat::*;
    match variant {
        Unit => println!("    {},", name),
        NewType(format) => println!("    {}({}),", name, quote_type(format, Some(known_sizes))),
        Tuple(formats) => println!("    {}({}),", name, quote_types(formats, Some(known_sizes))),
        Struct(fields) => {
            println!("    {} {{", name);
            output_fields(8, fields, false, known_sizes);
            println!("    }},");
        }
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(variants: &BTreeMap<u32, Named<VariantFormat>>, known_sizes: &HashSet<&str>) {
    for (expected_index, (index, variant)) in variants.iter().enumerate() {
        assert_eq!(*index, expected_index as u32);
        output_variant(&variant.name, &variant.value, known_sizes);
    }
}

pub fn output_container(name: &str, format: &ContainerFormat, known_sizes: &HashSet<&str>) {
    use ContainerFormat::*;
    let traits = "#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]\n";
    match format {
        UnitStruct => println!("{}pub struct {};\n", traits, name),
        NewTypeStruct(format) => println!(
            "{}pub struct {}({});\n",
            traits,
            name,
            quote_type(format, Some(known_sizes))
        ),
        TupleStruct(formats) => println!(
            "{}pub struct {}({});\n",
            traits,
            name,
            quote_types(formats, Some(known_sizes))
        ),
        Struct(fields) => {
            println!("{}pub struct {} {{", traits, name);
            output_fields(4, fields, true, known_sizes);
            println!("}}\n");
        }
        Enum(variants) => {
            println!("{}pub enum {} {{", traits, name,);
            output_variants(variants, known_sizes);
            println!("}}\n");
        }
    }
}
