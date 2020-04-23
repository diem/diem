// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::{BTreeMap, HashSet};

// TODO:
// * optional namespace
// * optionally use `using` for newtype/tuple structs and variants, as well as enums

pub fn output(registry: &RegistryOwned) {
    output_preambule();

    let dependencies = analyzer::get_dependency_map(registry).unwrap();
    let entries = analyzer::best_effort_topological_sort(&dependencies).unwrap();
    let mut known_names = HashSet::new();
    let mut known_sizes = HashSet::new();
    for name in entries {
        for dependency in &dependencies[name] {
            if !known_names.contains(dependency) {
                output_container_forward_definition(*dependency);
                known_names.insert(*dependency);
            }
        }
        let format = &registry[name];
        output_container(name, format, &known_sizes);
        known_sizes.insert(name);
        known_names.insert(name);
    }

    println!();
    for (name, format) in registry {
        output_container_traits(name, format);
    }
}

fn output_preambule() {
    println!("#include \"serde.hpp\"\n");
}

/// If known_sizes is present, we must try to return a type with a known size as well.
fn quote_type(format: &Format, known_sizes: Option<&HashSet<&str>>, namespace: &str) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if let Some(set) = known_sizes {
                if !set.contains(x.as_str()) {
                    return format!("std::unique_ptr<{}{}>", namespace, x);
                }
            }
            format!("{}{}", namespace, x)
        }
        Unit => "void".into(),
        Bool => "bool".into(),
        I8 => "int8_t".into(),
        I16 => "int16_t".into(),
        I32 => "int32_t".into(),
        I64 => "int64_t".into(),
        I128 => "int128_t".into(),
        U8 => "uint8_t".into(),
        U16 => "uint16_t".into(),
        U32 => "uint32_t".into(),
        U64 => "uint64_t".into(),
        U128 => "uint128_t".into(),
        F32 => "float".into(),
        F64 => "double".into(),
        Char => "char".into(),
        Str => "std::string".into(),
        Bytes => "std::vector<uint8_t>".into(),

        Option(format) => format!(
            "std:optional<{}>",
            quote_type(format, known_sizes, namespace)
        ),
        Seq(format) => format!("std::vector<{}>", quote_type(format, None, namespace)),
        Map { key, value } => format!(
            "std::map<{}, {}>",
            quote_type(key, None, namespace),
            quote_type(value, None, namespace)
        ),
        Tuple(formats) => format!(
            "std::tuple<{}>",
            quote_types(formats, known_sizes, namespace)
        ),
        TupleArray { content, size } => format!(
            "std::array<{}, {}>",
            quote_type(content, known_sizes, namespace),
            *size
        ),

        Unknown => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format], known_sizes: Option<&HashSet<&str>>, namespace: &str) -> String {
    formats
        .iter()
        .map(|x| quote_type(x, known_sizes, namespace))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(
    indentation: usize,
    fields: &[Named<Format>],
    known_sizes: &HashSet<&str>,
    namespace: &str,
) {
    let tab = " ".repeat(indentation);
    for field in fields {
        println!(
            "{}{} {};",
            tab,
            quote_type(&field.value, Some(known_sizes), namespace),
            field.name
        );
    }
}

fn output_variant(name: &str, variant: &VariantFormat, known_sizes: &HashSet<&str>) {
    use VariantFormat::*;
    match variant {
        Unit => println!("    struct {} {{}};", name),
        NewType(format) => println!(
            "    struct {} {{\n        {} value;\n    }};",
            name,
            quote_type(format, Some(known_sizes), "::")
        ),
        Tuple(formats) => println!(
            "    struct {} {{\n        std::tuple<{}> value;\n    }};",
            name,
            quote_types(formats, Some(known_sizes), "::")
        ),
        Struct(fields) => {
            println!("    struct {} {{", name);
            output_fields(8, fields, known_sizes, "::");
            println!("    }};");
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

fn output_container_forward_definition(name: &str) {
    println!("struct {};\n", name)
}

fn output_container(name: &str, format: &ContainerFormat, known_sizes: &HashSet<&str>) {
    use ContainerFormat::*;
    match format {
        UnitStruct => println!("struct {} {{}};\n", name),
        NewTypeStruct(format) => println!(
            "struct {} {{\n    {} value;\n}};\n",
            name,
            quote_type(format, Some(known_sizes), ""),
        ),
        TupleStruct(formats) => println!(
            "struct {} {{\n    std::tuple<{}> value;\n}};\n",
            name,
            quote_types(formats, Some(known_sizes), ""),
        ),
        Struct(fields) => {
            println!("struct {} {{", name);
            output_fields(4, fields, known_sizes, "");
            println!("}};\n");
        }
        Enum(variants) => {
            println!("struct {} {{", name);
            output_variants(variants, known_sizes);
            println!(
                "    std::variant<{}> value;\n}};\n",
                variants
                    .iter()
                    .map(|(_, v)| v.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
    }
}

fn output_struct_serializable(name: &str, fields: &[&str]) {
    println!(
        r#"
template <>
template <typename Serializer>
void Serializable<{}>::serialize(const {} &obj, Serializer &serializer) {{"#,
        name, name,
    );
    for field in fields {
        println!(
            "    Serializable<decltype(obj.{})>::serialize(obj.{}, serializer);",
            field, field,
        );
    }
    println!("}}");
}

fn output_struct_deserializable(name: &str, fields: &[&str]) {
    println!(
        r#"
template <>
template <typename Deserializer>
{} Deserializable<{}>::deserialize(Deserializer &deserializer) {{
    {} obj;"#,
        name, name, name,
    );
    for field in fields {
        println!(
            "    obj.{} = Deserializable<decltype(obj.{})>::deserialize(deserializer);",
            field, field,
        );
    }
    println!("    return obj;\n}}");
}

fn output_struct_traits(name: &str, fields: &[&str]) {
    output_struct_serializable(name, fields);
    output_struct_deserializable(name, fields);
}

fn get_variant_fields(format: &VariantFormat) -> Vec<&str> {
    use VariantFormat::*;
    match format {
        Unit => Vec::new(),
        NewType(_format) => vec!["value"],
        Tuple(_formats) => vec!["value"],
        Struct(fields) => fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        Unknown => panic!("incorrect value"),
    }
}

fn output_container_traits(name: &str, format: &ContainerFormat) {
    use ContainerFormat::*;
    match format {
        UnitStruct => output_struct_traits(name, &[]),
        NewTypeStruct(_format) => output_struct_traits(name, &["value"]),
        TupleStruct(_formats) => output_struct_traits(name, &["value"]),
        Struct(fields) => output_struct_traits(
            name,
            &fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
        ),
        Enum(variants) => {
            output_struct_traits(name, &["value"]);
            for variant in variants.values() {
                output_struct_traits(
                    &format!("{}::{}", name, variant.name),
                    &get_variant_fields(&variant.value),
                );
            }
        }
    }
}
