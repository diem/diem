// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;

pub fn output(registry: &RegistryOwned) {
    output_preambule();

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
        output_container(name, format);
    }
    println!("}}");
}

fn output_preambule() {
    println!(
        r#"
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;
import java.lang.Class;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

@Target({{ElementType.TYPE_USE}})
@interface Unsigned {{}}

@Target({{ElementType.TYPE_USE}})
@interface FixedLength {{
    int length();
}}

@Target({{ElementType.TYPE_USE}})
@interface Enum {{
    Class<?>[] variants();
}}

@Target({{ElementType.TYPE_USE}})
@interface Variant {{
    long index();
}}
"#
    );
}

fn quote_type(format: &Format) -> String {
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

        Option(format) => format!("Optional<{}>", quote_type(format)),
        Seq(format) => format!("Vector<{}>", quote_type(format)),
        Map { key, value } => format!("SortedMap<{}, {}>", quote_type(key), quote_type(value)),
        Tuple(formats) => format!("Tuple{}<{}>", formats.len(), quote_types(formats)),
        TupleArray { content, size } => format!(
            "@FixedLength(length={}) Vector<{}>",
            size,
            quote_type(content)
        ),
        Unknown => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format]) -> String {
    formats
        .iter()
        .map(quote_type)
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(indentation: usize, fields: &[Named<Format>]) {
    let tab = " ".repeat(indentation);
    for field in fields {
        println!(
            "{} public {} {};",
            tab,
            quote_type(&field.value),
            field.name
        );
    }
}

fn output_variant(base: &str, name: &str, index: u32, variant: &VariantFormat) {
    use VariantFormat::*;
    let annotation = format!("@Variant(index = {})\n", index);
    let class = format!("public static class {}_{} extends {}", base, name, base);
    match variant {
        Unit => println!("\n{}{} {{}};", annotation, class),
        NewType(format) => println!(
            "\n{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            quote_type(format),
        ),
        Tuple(formats) => println!(
            "\n{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            quote_type(&Format::Tuple(formats.clone())),
        ),
        Struct(fields) => {
            println!("\n{}{} {{", annotation, class);
            output_fields(4, fields);
            println!("}};");
        }
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
    for (index, variant) in variants {
        output_variant(base, &variant.name, *index, &variant.value);
    }
}

fn output_container(name: &str, format: &ContainerFormat) {
    use ContainerFormat::*;
    match format {
        UnitStruct => println!("public static class {} {{}};\n", name),
        NewTypeStruct(format) => println!(
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => println!(
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            quote_type(&Format::Tuple(formats.clone()))
        ),
        Struct(fields) => {
            println!("public static class {} {{", name);
            output_fields(4, fields);
            println!("}};\n");
        }
        Enum(variants) => {
            println!(
                r#"@Enum(variants={{
    {}
}})
public abstract static class {} {{}};
"#,
                variants
                    .iter()
                    .map(|(_, v)| format!("{}_{}.class", name, v.name))
                    .collect::<Vec<_>>()
                    .join(",\n    "),
                name
            );
            output_variants(name, variants);
        }
    }
}
