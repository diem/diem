// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;

pub fn output(registry: &RegistryOwned) {
    println!("{}", output_preambule());

    println!("public class Test {{");
    println!(
        r#"
public static class Unit {{}};
public static class Tuple1<T1> {{ public T1 field1; }};
public static class Tuple2<T1, T2> {{ public T1 field1; public T2 field2; }};
public static class Tuple3<T1, T2, T3> {{ public T1 field1; public T2 field2; public T3 field3; }};
public static class Tuple4<T1, T2, T3, T4> {{ public T1 field1; public T2 field2; public T3 field3; public T4 field4; }};
public static class Int128 {{ public Long high; public Long low; }};
"#
    );
    for (name, format) in registry {
        println!("{}", output_container(name, format));
    }
    println!("}}");
}

fn output_preambule() -> String {
    r#"
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;
import java.lang.Class;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

@Target({ElementType.TYPE_USE})
@interface Unsigned {}

@Target({ElementType.TYPE_USE})
@interface FixedLength {
    int length();
}

@Target({ElementType.TYPE_USE})
@interface Enum {
    Class<?>[] variants();
}

@Target({ElementType.TYPE_USE})
@interface Variant {
    long index();
}
"#
    .into()
}

fn output_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => x.to_string(),
        Unit => "Unit".into(),
        Bool => "Boolean".into(),
        I8 => "Byte".into(),
        I16 => "Short".into(),
        I32 => "Int".into(),
        I64 => "Long".into(),
        I128 => "Integer128".into(),
        U8 => "@Unsigned Byte".into(),
        U16 => "@Unsigned Short".into(),
        U32 => "@Unsigned Int".into(),
        U64 => "@Unsigned Long".into(),
        U128 => "@Unsigned Int128".into(),
        F32 => "Float".into(),
        F64 => "Double".into(),
        Char => "Char".into(),
        Str => "String".into(),
        Bytes => "ByteBuffer".into(),

        Option(format) => format!("Optional<{}>", output_type(format)),
        Seq(format) => format!("Vector<{}>", output_type(format)),
        Map { key, value } => format!("SortedMap<{}, {}>", output_type(key), output_type(value)),
        Tuple(formats) => format!("Tuple{}<{}>", formats.len(), output_types(formats)),
        TupleArray { content, size } => format!(
            "@FixedLength(length={}) Vector<{}>",
            size,
            output_type(content)
        ),
        Unknown => panic!("unexpected value"),
    }
}

fn output_types(formats: &[Format]) -> String {
    formats
        .iter()
        .map(output_type)
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(indentation: usize, fields: &[Named<Format>]) -> String {
    let mut result = String::new();
    let tab = " ".repeat(indentation);
    for field in fields {
        result += &format!(
            "{} public {} {};\n",
            tab,
            output_type(&field.value),
            field.name
        );
    }
    result
}

fn output_variant(base: &str, name: &str, index: u32, variant: &VariantFormat) -> String {
    use VariantFormat::*;
    let annotation = format!("@Variant(index = {})\n", index);
    let class = format!("public static class {}_{} extends {}", base, name, base);
    match variant {
        Unit => format!("{}{} {{}};", annotation, class),
        NewType(format) => format!(
            "{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            output_type(format),
        ),
        Tuple(formats) => format!(
            "{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            output_type(&Format::Tuple(formats.clone())),
        ),
        Struct(fields) => format!(
            "{}{} {{\n{}}};",
            annotation,
            class,
            output_fields(4, fields)
        ),
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) -> String {
    let mut result = String::new();
    for (index, variant) in variants {
        result += &format!(
            "\n{}\n",
            output_variant(base, &variant.name, *index, &variant.value)
        );
    }
    result
}

fn output_container(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct => format!("public static class {} {{}};\n", name),
        NewTypeStruct(format) => format!(
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            output_type(format)
        ),
        TupleStruct(formats) => format!(
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            output_type(&Format::Tuple(formats.clone()))
        ),
        Struct(fields) => format!(
            "public static class {} {{\n{}}};\n",
            name,
            output_fields(4, fields)
        ),
        Enum(variants) => format!(
            r#"@Enum(variants={{
    {}
}})
public abstract static class {} {{}};
{}"#,
            variants
                .iter()
                .map(|(_, v)| format!("{}_{}.class", name, v.name))
                .collect::<Vec<_>>()
                .join(",\n    "),
            name,
            output_variants(name, variants),
        ),
    }
}
