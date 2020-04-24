// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;

pub fn output(registry: &RegistryOwned) {
    output_preambule();
    for (name, format) in registry {
        output_container(name, format);
    }
}

fn output_preambule() {
    println!(
        r#"# pyre-ignore-all-errors
from dataclasses import dataclass
import numpy as np
import typing
"#
    );
}

fn quote_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => format!("\"{}\"", x), // Need quotes because of circular dependencies.
        Unit => "None".into(),
        Bool => "np.bool".into(),
        I8 => "np.int8".into(),
        I16 => "np.int16".into(),
        I32 => "np.int32".into(),
        I64 => "np.int64".into(),
        I128 => "typing.Tuple[np.int64, np.int64]".into(),
        U8 => "np.uint8".into(),
        U16 => "np.uint16".into(),
        U32 => "np.uint32".into(),
        U64 => "np.uint64".into(),
        U128 => "typing.Tuple[np.uint64, np.uint64]".into(),
        F32 => "np.float32".into(),
        F64 => "np.float64".into(),
        Char => "char".into(),
        Str => "str".into(),
        Bytes => "bytes".into(),

        Option(format) => format!("typing.Optional[{}]", quote_type(format)),
        Seq(format) => format!("typing.Sequence[{}]", quote_type(format)),
        Map { key, value } => format!("typing.Dict[{}, {}]", quote_type(key), quote_type(value)),
        Tuple(formats) => format!("typing.Tuple[{}]", quote_types(formats)),
        TupleArray { content, size } => format!(
            "typing.Tuple[{}]",
            quote_types(&vec![content.as_ref().clone(); *size])
        ), // Sadly, there are no fixed-size arrays in python.

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
        println!("{}{}: {}", tab, field.name, quote_type(&field.value));
    }
}

fn output_variant(base: &str, name: &str, index: u32, variant: &VariantFormat) {
    use VariantFormat::*;
    match variant {
        Unit => println!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n",
            base, name, base, index,
        ),
        NewType(format) => println!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: {}\n",
            base,
            name,
            base,
            index,
            quote_type(format)
        ),
        Tuple(formats) => println!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: typing.Tuple[{}]\n",
            base,
            name,
            base,
            index,
            quote_types(formats)
        ),
        Struct(fields) => {
            println!(
                "@dataclass\nclass _{}_{}({}):\n    INDEX = {}",
                base, name, base, index
            );
            output_fields(4, fields);
            println!();
        }
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
    for (index, variant) in variants {
        output_variant(base, &variant.name, *index, &variant.value);
    }
}

fn output_variant_aliases(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
    for variant in variants.values() {
        println!("{}.{} = _{}_{}", base, &variant.name, base, &variant.name);
    }
}

fn output_container(name: &str, format: &ContainerFormat) {
    use ContainerFormat::*;
    match format {
        UnitStruct => println!("@dataclass\nclass {}:\n    pass\n", name),
        NewTypeStruct(format) => println!(
            "@dataclass\nclass {}:\n    value: {}\n",
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => println!(
            "@dataclass\nclass {}:\n    value: typing.Tuple[{}]\n",
            name,
            quote_types(formats)
        ),
        Struct(fields) => {
            println!("@dataclass\nclass {}:", name);
            output_fields(4, fields);
            println!();
        }
        Enum(variants) => {
            println!("class {}:\n    pass\n", name);
            output_variants(name, variants);
            output_variant_aliases(name, variants);
            println!(
                "{}.VARIANTS = [\n{}]\n",
                name,
                variants
                    .iter()
                    .map(|(_, v)| format!("    {}.{},\n", name, v.name))
                    .collect::<Vec<_>>()
                    .join("")
            );
        }
    }
}
