// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::{BTreeMap, HashSet};

pub fn output(registry: &RegistryOwned) {
    let dependencies = analyzer::get_dependency_map(registry).unwrap();
    let entries = analyzer::best_effort_topological_sort(&dependencies).unwrap();

    println!("{}", output_preambule());
    let mut known_sizes = HashSet::new();
    for name in entries {
        let format = &registry[name];
        println!("{}", output_container(name, format, &known_sizes));
        known_sizes.insert(name);
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

fn output_type(format: &Format, known_sizes: Option<&HashSet<&str>>) -> String {
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

        Option(format) => format!("Option<{}>", output_type(format, known_sizes)),
        Seq(format) => format!("Vec<{}>", output_type(format, None)),
        Map { key, value } => format!(
            "BTreeMap<{}, {}>",
            output_type(key, None),
            output_type(value, None)
        ),
        Tuple(formats) => format!("({})", output_types(formats, known_sizes)),
        TupleArray { content, size } => {
            format!("[{}; {}]", output_type(content, known_sizes), *size)
        }

        Unknown => panic!("unexpected value"),
    }
}

fn output_types(formats: &[Format], known_sizes: Option<&HashSet<&str>>) -> String {
    formats
        .iter()
        .map(|x| output_type(x, known_sizes))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(
    indentation: usize,
    fields: &[Named<Format>],
    is_pub: bool,
    known_sizes: &HashSet<&str>,
) -> String {
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
            output_type(&field.value, Some(known_sizes)),
        );
    }
    result
}

fn output_variant(name: &str, variant: &VariantFormat, known_sizes: &HashSet<&str>) -> String {
    use VariantFormat::*;
    match variant {
        Unit => name.to_string(),
        NewType(format) => format!("{}({})", name, output_type(format, Some(known_sizes))),
        Tuple(formats) => format!("{}({})", name, output_types(formats, Some(known_sizes))),
        Struct(fields) => format!(
            "{} {{\n{}    }}",
            name,
            output_fields(8, fields, false, known_sizes)
        ),
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    known_sizes: &HashSet<&str>,
) -> String {
    let mut result = String::new();
    for (expected_index, (index, variant)) in variants.iter().enumerate() {
        assert_eq!(*index, expected_index as u32);
        result += &format!(
            "    {},\n",
            output_variant(&variant.name, &variant.value, known_sizes)
        );
    }
    result
}

pub fn output_container(
    name: &str,
    format: &ContainerFormat,
    known_sizes: &HashSet<&str>,
) -> String {
    use ContainerFormat::*;
    let traits = "#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]\n";
    match format {
        UnitStruct => format!("{}pub struct {};\n", traits, name),
        NewTypeStruct(format) => format!(
            "{}pub struct {}({});\n",
            traits,
            name,
            output_type(format, Some(known_sizes))
        ),
        TupleStruct(formats) => format!(
            "{}pub struct {}({});\n",
            traits,
            name,
            output_types(formats, Some(known_sizes))
        ),
        Struct(fields) => format!(
            "{}pub struct {} {{\n{}}}\n",
            traits,
            name,
            output_fields(4, fields, true, known_sizes)
        ),
        Enum(variants) => format!(
            "{}pub enum {} {{\n{}}}\n",
            traits,
            name,
            output_variants(variants, known_sizes)
        ),
    }
}
