// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use serde_yaml;
use std::{collections::BTreeMap, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra Serde code generator",
    about = "Generate code for Serde containers"
)]
struct Options {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let options = Options::from_args();
    let content =
        std::fs::read_to_string(options.input.as_os_str()).expect("input file must be readable");
    let registry = serde_yaml::from_str::<RegistryOwned>(content.as_str()).unwrap();

    println!("{}", python3_preambule());
    for (name, format) in &registry {
        println!("{}", python3_container(name, format));
    }
}

fn python3_preambule() -> String {
    r#"
from dataclasses import dataclass
import numpy as np
from typing import *
"#
    .into()
}

fn python3_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => format!("'{}'", x), // Need quotes because of circular dependencies.
        Unit => "None".into(),
        Bool => "np.bool".into(),
        I8 => "np.int8".into(),
        I16 => "np.int16".into(),
        I32 => "np.int32".into(),
        I64 => "np.int64".into(),
        I128 => "Tuple(np.int64, np.int64)".into(),
        U8 => "np.uint8".into(),
        U16 => "np.uint16".into(),
        U32 => "np.uint32".into(),
        U64 => "np.uint64".into(),
        U128 => "Tuple(np.uint64, np.uint64)".into(),
        F32 => "np.float32".into(),
        F64 => "np.float64".into(),
        Char => "char".into(),
        Str => "str".into(),
        Bytes => "bytes".into(),

        Option(format) => format!("Optional[{}]", python3_type(format)),
        Seq(format) => format!("Sequence[{}]", python3_type(format)),
        Map { key, value } => format!("Dict[{}, {}]", python3_type(key), python3_type(value)),
        Tuple(formats) => format!("Tuple[{}]", python3_types(formats)),
        TupleArray { content, size } => format!(
            "Tuple[{}]",
            python3_types(&vec![content.as_ref().clone(); *size])
        ), // Sadly, there are no fixed-size arrays in python.

        _ => panic!("unexpected value"),
    }
}

fn python3_types(formats: &[Format]) -> String {
    formats
        .iter()
        .map(python3_type)
        .collect::<Vec<_>>()
        .join(", ")
}

fn python3_fields(indentation: usize, fields: &[Named<Format>]) -> String {
    let mut result = String::new();
    let tab = " ".repeat(indentation);
    for field in fields {
        result += &format!("{}{}: {}\n", tab, field.name, python3_type(&field.value));
    }
    result
}

fn python3_variant(base: &str, name: &str, index: u32, variant: &VariantFormat) -> String {
    use VariantFormat::*;
    match variant {
        Unit => format!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX={}\n",
            base, name, base, index,
        ),
        NewType(format) => format!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX={}\n    value: {}\n",
            base,
            name,
            base,
            index,
            python3_type(format)
        ),
        Tuple(formats) => format!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX={}\n    value: Tuple[{}]\n",
            base,
            name,
            base,
            index,
            python3_types(formats)
        ),
        Struct(fields) => format!(
            "@dataclass\nclass _{}_{}({}):\n    INDEX={}\n{}",
            base,
            name,
            base,
            index,
            python3_fields(4, fields)
        ),
        _ => panic!("incorrect value"),
    }
}

fn python3_variants(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) -> String {
    let mut result = String::new();
    for (index, variant) in variants {
        result += &format!(
            "{}\n",
            python3_variant(base, &variant.name, *index, &variant.value)
        );
    }
    result
}

fn python3_variant_aliases(base: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) -> String {
    let mut result = String::new();
    for variant in variants.values() {
        result += &format!("{}.{} = _{}_{}\n", base, &variant.name, base, &variant.name);
    }
    result
}

fn python3_container(name: &str, format: &ContainerFormat) -> String {
    use ContainerFormat::*;
    match format {
        UnitStruct => format!("@dataclass\nclass {}:\n    pass\n", name,),
        NewTypeStruct(format) => format!(
            "@dataclass\nclass {}:\n    value: {}\n",
            name,
            python3_type(format)
        ),
        TupleStruct(formats) => format!(
            "@dataclass\nclass {}:\n    value: Tuple[{}]\n",
            name,
            python3_types(formats)
        ),
        Struct(fields) => format!("@dataclass\nclass {}:\n{}", name, python3_fields(4, fields)),
        Enum(variants) => format!(
            "class {}:\n    pass\n\n{}{}{}.VARIANTS = [{}]\n",
            name,
            python3_variants(name, variants),
            python3_variant_aliases(name, variants),
            name,
            variants
                .iter()
                .map(|(_, v)| format!("{}.{}", name, v.name))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
