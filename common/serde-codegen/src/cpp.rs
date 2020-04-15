// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;

pub fn output(registry: &RegistryOwned) {
    println!("{}", output_preambule());
    for (name, format) in registry {
        print!("{}", output_container_forward_definition(name, format));
    }
    println!();
    for (name, format) in registry {
        println!("{}", output_container(name, format));
    }
    println!();
    for (name, format) in registry {
        println!("{}", output_container_traits(name, format));
    }
}

fn output_preambule() -> String {
    "#include \"serde.hpp\"\n".into()
}

fn output_type(format: &Format, fixed_size: bool) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if fixed_size {
                format!("std::unique_ptr<{}>", x)
            } else {
                x.to_string()
            }
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

        Option(format) => format!("std:optional<{}>", output_type(format, fixed_size)),
        Seq(format) => format!("std::vector<{}>", output_type(format, false)),
        Map { key, value } => format!(
            "std::map<{}, {}>",
            output_type(key, false),
            output_type(value, false)
        ),
        Tuple(formats) => format!("std::tuple<{}>", output_types(formats, fixed_size)),
        TupleArray { content, size } => format!(
            "std::array<{}, {}>",
            output_type(content, fixed_size),
            *size
        ),

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

fn output_fields(indentation: usize, fields: &[Named<Format>]) -> String {
    let mut result = String::new();
    let tab = " ".repeat(indentation);
    for field in fields {
        result += &format!(
            "{}{} {};\n",
            tab,
            output_type(&field.value, true),
            field.name
        );
    }
    result
}

fn output_variant(base: &str, name: &str, index: u32, variant: &VariantFormat) -> String {
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
            output_type(format, true)
        ),
        Tuple(formats) => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n\n    std::tuple<{}> value;\n}};",
            base,
            name,
            base,
            index,
            output_types(formats, true)
        ),
        Struct(fields) => format!(
            "struct {}_{}: {} {{\n    const uint32_t index = {};\n\n{}}};",
            base,
            name,
            base,
            index,
            output_fields(4, fields)
        ),
        _ => panic!("incorrect value"),
    }
}

fn output_variants(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) -> String {
    let mut result = String::new();
    for (index, variant) in variants {
        result += &format!(
            "{}\n",
            output_variant(base, &variant.name, *index, &variant.value)
        );
    }
    result
}

fn output_variants_forward_definition(
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> String {
    let mut result = String::new();
    for variant in variants.values() {
        result += &format!("struct {}_{};\n", base, &variant.name,);
    }
    result
}

fn output_container_forward_definition(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct | NewTypeStruct(_) | TupleStruct(_) | Struct(_) => format!("struct {};\n", name),
        Enum(variants) => format!(
            "struct {};\n{}",
            name,
            output_variants_forward_definition(name, variants),
        ),
    }
}

fn output_container(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct => format!("struct {} {{}};\n", name),
        NewTypeStruct(format) => format!(
            "struct {} {{\n    {} value;\n}};\n",
            name,
            output_type(format, true)
        ),
        TupleStruct(formats) => format!(
            "struct {} {{\n    std::tuple<{}> value;\n}};\n",
            name,
            output_types(formats, true)
        ),
        Struct(fields) => format!("struct {} {{\n{}}};\n", name, output_fields(4, fields)),
        Enum(variants) => format!("struct {} {{}};\n{}", name, output_variants(name, variants),),
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
