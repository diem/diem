// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::{BTreeMap, HashSet};

pub fn output(registry: &RegistryOwned) {
    let dependencies = analyzer::get_dependency_map(registry).unwrap();
    let entries = analyzer::best_effort_topological_sort(&dependencies).unwrap();

    println!("{}", output_preambule());
    let mut known_names = HashSet::new();
    let mut known_sizes = HashSet::new();
    for name in entries {
        for dependency in &dependencies[name] {
            if !known_names.contains(dependency) {
                println!("{}", output_container_forward_definition(*dependency));
                known_names.insert(*dependency);
            }
        }

        let format = &registry[name];
        println!("{}", output_container(name, format, &known_sizes));
        known_sizes.insert(name);
        known_names.insert(name);
    }
    println!();
    for (name, format) in registry {
        println!("{}", output_container_traits(name, format));
    }
}

fn output_preambule() -> String {
    "#include \"serde.hpp\"\n".into()
}

/// If known_size is present, we must try to return a type with a known size as well.
fn output_type(format: &Format, known_sizes: Option<&HashSet<&str>>) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if let Some(set) = known_sizes {
                if !set.contains(x.as_str()) {
                    return format!("std::unique_ptr<{}>", x);
                }
            }
            x.to_string()
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

        Option(format) => format!("std:optional<{}>", output_type(format, known_sizes)),
        Seq(format) => format!("std::vector<{}>", output_type(format, None)),
        Map { key, value } => format!(
            "std::map<{}, {}>",
            output_type(key, None),
            output_type(value, None)
        ),
        Tuple(formats) => format!("std::tuple<{}>", output_types(formats, known_sizes)),
        TupleArray { content, size } => format!(
            "std::array<{}, {}>",
            output_type(content, known_sizes),
            *size
        ),

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
    known_sizes: &HashSet<&str>,
) -> String {
    let mut result = String::new();
    let tab = " ".repeat(indentation);
    for field in fields {
        result += &format!(
            "{}{} {};\n",
            tab,
            output_type(&field.value, Some(known_sizes)),
            field.name
        );
    }
    result
}

fn output_variant(
    base: &str,
    name: &str,
    index: u32,
    variant: &VariantFormat,
    known_sizes: &HashSet<&str>,
) -> String {
    use VariantFormat::*;
    match variant {
        Unit => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n}};",
            base, name, base, index,
        ),
        NewType(format) => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n\n    {} value;\n}};",
            base,
            name,
            base,
            index,
            output_type(format, Some(known_sizes))
        ),
        Tuple(formats) => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n\n    std::tuple<{}> value;\n}};",
            base,
            name,
            base,
            index,
            output_types(formats, Some(known_sizes))
        ),
        Struct(fields) => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n\n{}}};",
            base,
            name,
            base,
            index,
            output_fields(4, fields, known_sizes)
        ),
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    known_sizes: &HashSet<&str>,
) -> String {
    let mut result = String::new();
    for (index, variant) in variants {
        result += &format!(
            "{}\n",
            output_variant(base, &variant.name, *index, &variant.value, known_sizes)
        );
    }
    result
}

fn output_container_forward_definition(name: &str) -> String {
    format!("struct {};\n", name)
}

fn output_container(name: &str, format: &ContainerFormat, known_sizes: &HashSet<&str>) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct => format!("struct {} {{}};\n", name),
        NewTypeStruct(format) => format!(
            "struct {} {{\n    {} value;\n}};\n",
            name,
            output_type(format, Some(known_sizes))
        ),
        TupleStruct(formats) => format!(
            "struct {} {{\n    std::tuple<{}> value;\n}};\n",
            name,
            output_types(formats, Some(known_sizes))
        ),
        Struct(fields) => format!(
            "struct {} {{\n{}}};\n",
            name,
            output_fields(4, fields, known_sizes)
        ),
        Enum(variants) => format!(
            "struct {} {{}};\n{}",
            name,
            output_variants(name, variants, known_sizes),
        ),
    }
}

fn output_struct_serializable(name: &str, fields: &[Named<Format>]) -> String {
    let mut fields_serializable = String::new();
    for field in fields {
        fields_serializable += &format!(
            "    Serializable<decltype(obj.{})>::serialize(obj.{}, serializer);\n",
            field.name, field.name,
        );
    }

    format!(
        r#"
template <>
template <typename Serializer>
void Serializable<{}>::serialize(const {} &obj, Serializer &serializer) {{
{}}}"#,
        name, name, fields_serializable,
    )
}

fn output_struct_deserializable(name: &str, fields: &[Named<Format>]) -> String {
    let mut fields_deserializable = String::new();
    for field in fields {
        fields_deserializable += &format!(
            "    obj.{} = Deserializable<decltype(obj.{})>::deserialize(deserializer);\n",
            field.name, field.name,
        );
    }

    format!(
        r#"
template <>
template <typename Deserializer>
{} Deserializable<{}>::deserialize(Deserializer &deserializer) {{
    {} obj;
{}    return obj;
}}"#,
        name, name, name, fields_deserializable,
    )
}

fn output_struct_traits(name: &str, fields: &[Named<Format>]) -> String {
    format!(
        "{}\n{}\n",
        output_struct_serializable(name, fields),
        output_struct_deserializable(name, fields)
    )
}

fn output_container_traits(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct => output_struct_traits(name, &[]),
        NewTypeStruct(format) => output_struct_traits(
            name,
            &[Named {
                name: "value".into(),
                value: format.as_ref().clone(),
            }],
        ),
        TupleStruct(formats) => output_struct_traits(
            name,
            &[Named {
                name: "value".into(),
                value: Format::Tuple(formats.clone()),
            }],
        ),
        Struct(fields) => output_struct_traits(name, fields),
        Enum(_variants) => String::new(), // TODO
    }
}
