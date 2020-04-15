// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;

pub fn output(registry: &RegistryOwned) {
    println!("{}", output_preambule());
    for (name, format) in registry {
        println!("{}", output_container(name, format));
    }
}

fn output_preambule() -> String {
    r#"
#![allow(unused_imports)]
use serde::{Serialize, Deserialize};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
"#
    .into()
}

fn output_type(format: &Format, fixed_size: bool) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if fixed_size {
                format!("Box<{}>", x)
            } else {
                x.to_string()
            }
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

        Option(format) => format!("Option<{}>", output_type(format, fixed_size)),
        Seq(format) => format!("Vec<{}>", output_type(format, false)),
        Map { key, value } => format!(
            "BTreeMap<{}, {}>",
            output_type(key, false),
            output_type(value, false)
        ),
        Tuple(formats) => format!("({})", output_types(formats, fixed_size)),
        TupleArray { content, size } => {
            format!("[{}; {}]", output_type(content, fixed_size), *size)
        }

        _ => panic!("unexpected value"),
    }
}

fn output_types(formats: &[Format], fixed_size: bool) -> String {
    formats
        .iter()
        .map(|x| output_type(x, fixed_size))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(indentation: usize, fields: &[Named<Format>], is_pub: bool) -> String {
    let mut result = String::new();
    let mut tab = " ".repeat(indentation);
    if is_pub {
        tab += " pub ";
    }
    for field in fields {
        result += &format!(
            "{}{}: {},\n",
            tab,
            field.name,
            output_type(&field.value, true),
        );
    }
    result
}

fn output_variant(name: &str, variant: &VariantFormat) -> String {
    use VariantFormat::*;
    match variant {
        Unit => name.to_string(),
        NewType(format) => format!("{}({})", name, output_type(format, true)),
        Tuple(formats) => format!("{}({})", name, output_types(formats, true)),
        Struct(fields) => format!("{} {{\n{}    }}", name, output_fields(8, fields, false)),
        _ => panic!("incorrect value"),
    }
}

fn output_variants(variants: &BTreeMap<u32, Named<VariantFormat>>) -> String {
    let mut result = String::new();
    for (expected_index, (index, variant)) in variants.iter().enumerate() {
        assert_eq!(*index, expected_index as u32);
        result += &format!("    {},\n", output_variant(&variant.name, &variant.value));
    }
    result
}

pub fn output_container(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    let traits = "#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]\n";
    match format {
        UnitStruct => format!("{}pub struct {};\n", traits, name),
        NewTypeStruct(format) => format!(
            "{}pub struct {}({});\n",
            traits,
            name,
            output_type(format, true)
        ),
        TupleStruct(formats) => format!(
            "{}pub struct {}({});\n",
            traits,
            name,
            output_types(formats, true)
        ),
        Struct(fields) => format!(
            "{}pub struct {} {{\n{}}}\n",
            traits,
            name,
            output_fields(4, fields, true)
        ),
        Enum(variants) => format!(
            "{}pub enum {} {{\n{}}}\n",
            traits,
            name,
            output_variants(variants)
        ),
    }
}
