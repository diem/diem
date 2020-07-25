// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output a header-only library providing C++ transaction builders for the given ABIs.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI], namespace: Option<&str>) -> Result<()> {
    output_preamble(out)?;
    output_open_namespace(out, namespace)?;
    output_using_namespaces(out)?;
    for abi in abis {
        output_builder_definition(out, abi, /* inlined */ true)?;
    }
    output_close_namespace(out, namespace)
}

/// Output the headers of a library providing C++ transaction builders for the given ABIs.
pub fn output_library_header(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    namespace: Option<&str>,
) -> Result<()> {
    output_preamble(out)?;
    output_open_namespace(out, namespace)?;
    output_using_namespaces(out)?;
    for abi in abis {
        output_builder_declaration(out, abi)?;
    }
    output_close_namespace(out, namespace)
}

/// Output the function definitions of a library providing C++ transaction builders for the given ABIs.
pub fn output_library_body(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    library_name: &str,
    namespace: Option<&str>,
) -> Result<()> {
    writeln!(out, "#include \"{}.hpp\"\n", library_name)?;
    output_open_namespace(out, namespace)?;
    output_using_namespaces(out)?;
    for abi in abis {
        output_builder_definition(out, abi, /* inlined */ false)?;
    }
    output_close_namespace(out, namespace)
}

fn output_preamble(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"
#pragma once

#include "libra_types.hpp"
"#
    )
}

fn output_using_namespaces(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"
using namespace serde;
using namespace libra_types;
"#
    )
}

fn output_open_namespace(out: &mut dyn std::io::Write, namespace: Option<&str>) -> Result<()> {
    if let Some(name) = namespace {
        writeln!(out, "namespace {} {{\n", name,)?;
    }
    Ok(())
}

fn output_close_namespace(out: &mut dyn std::io::Write, namespace: Option<&str>) -> Result<()> {
    if let Some(name) = namespace {
        writeln!(out, "\n}} // end of namespace {}", name,)?;
    }
    Ok(())
}

fn output_builder_declaration(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    write!(out, "\n{}", quote_doc(abi.doc()))?;
    writeln!(
        out,
        "Script encode_{}_script({});",
        abi.name(),
        [
            quote_type_parameters(abi.ty_args()),
            quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ")
    )?;
    Ok(())
}

fn output_builder_definition(out: &mut dyn Write, abi: &ScriptABI, inlined: bool) -> Result<()> {
    if inlined {
        write!(out, "\n{}", quote_doc(abi.doc()))?;
    }
    writeln!(
        out,
        "{}Script encode_{}_script({}) {{",
        if inlined { "inline " } else { "" },
        abi.name(),
        [
            quote_type_parameters(abi.ty_args()),
            quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ")
    )?;
    writeln!(
        out,
        r#"    return Script {{
        {},
        std::vector<TypeTag> {{{}}},
        std::vector<TransactionArgument> {{{}}},
    }};"#,
        quote_code(abi.code()),
        quote_type_arguments(abi.ty_args()),
        quote_arguments(abi.args()),
    )?;
    writeln!(out, "}}")?;
    Ok(())
}

fn quote_doc(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    let text = textwrap::fill(&doc, 86);
    textwrap::indent(&text, "/// ")
}

fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
    ty_args
        .iter()
        .map(|ty_arg| format!("TypeTag {}", ty_arg.name()))
        .collect()
}

fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
    args.iter()
        .map(|arg| format!("{} {}", quote_type(arg.type_tag()), arg.name()))
        .collect()
}

fn quote_code(code: &[u8]) -> String {
    format!(
        "std::vector<uint8_t> {{{}}}",
        code.iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn quote_type_arguments(ty_args: &[TypeArgumentABI]) -> String {
    ty_args
        .iter()
        .map(|ty_arg| format!("std::move({})", ty_arg.name()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn quote_arguments(args: &[ArgumentABI]) -> String {
    args.iter()
        .map(|arg| make_transaction_argument(arg.type_tag(), arg.name()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn quote_type(type_tag: &TypeTag) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => "bool".into(),
        U8 => "uint8_t".into(),
        U64 => "uint64_t".into(),
        U128 => "uint128_t".into(),
        Address => "AccountAddress".into(),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => "std::vector<uint8_t>".into(),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn make_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => format!("{{TransactionArgument::Bool {{{}}} }}", name),
        U8 => format!("{{TransactionArgument::U8 {{{}}} }}", name),
        U64 => format!("{{TransactionArgument::U64 {{{}}} }}", name),
        U128 => format!("{{TransactionArgument::U128 {{{}}} }}", name),
        // Adding std::move in the non-obvious cases to be future-proof.
        Address => format!("{{TransactionArgument::Address {{std::move({})}}}}", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("{{TransactionArgument::U8Vector {{std::move({})}}}}", name),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer { install_dir }
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_transaction_builders(
        &self,
        name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        let dir_path = &self.install_dir;
        std::fs::create_dir_all(dir_path)?;
        let header_path = dir_path.join(name.to_string() + ".hpp");
        let mut header = std::fs::File::create(&header_path)?;
        output_library_header(&mut header, abis, Some(name))?;
        let body_path = dir_path.join(name.to_string() + ".cpp");
        let mut body = std::fs::File::create(&body_path)?;
        output_library_body(&mut body, abis, name, Some(name))?;
        Ok(())
    }
}
