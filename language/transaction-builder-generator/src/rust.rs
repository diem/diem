// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Rust for the given ABIs.
/// If `local_types` is true, we generate a file suitable for the Libra codebase itself
/// rather than using serde-generated, standalone definitions.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI], local_types: bool) -> Result<()> {
    output_preamble(out, local_types)?;
    for abi in abis {
        output_builder(out, abi, local_types)?;
    }
    Ok(())
}

fn output_preamble(out: &mut dyn Write, local_types: bool) -> Result<()> {
    let preamble = if local_types {
        r#"
// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To re-generate this code, run: `(cd language/stdlib && cargo run --release)`

use move_core_types::language_storage::TypeTag;
use libra_types::transaction::{Script, TransactionArgument};
use libra_types::account_address::AccountAddress;
"#
    } else {
        r#"
use libra_types::{AccountAddress, TypeTag, Script, TransactionArgument};
"#
    };
    writeln!(out, "{}", preamble)
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI, local_types: bool) -> Result<()> {
    write!(out, "\n{}", quote_doc(abi.doc()))?;
    writeln!(
        out,
        "pub fn encode_{}_script({}) -> Script {{",
        abi.name(),
        [
            quote_type_parameters(abi.ty_args()),
            quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ")
    )?;
    if local_types {
        writeln!(
            out,
            r#"    Script::new(
        {},
        vec![{}],
        vec![{}],
    )"#,
            quote_code(abi.code()),
            quote_type_arguments(abi.ty_args()),
            quote_arguments(abi.args()),
        )?;
    } else {
        writeln!(
            out,
            r#"    Script {{
        code: {},
        ty_args: vec![{}],
        args: vec![{}],
    }}"#,
            quote_code(abi.code()),
            quote_type_arguments(abi.ty_args()),
            quote_arguments(abi.args()),
        )?;
    }
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
        .map(|ty_arg| format!("{}: TypeTag", ty_arg.name()))
        .collect()
}

fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
    args.iter()
        .map(|arg| format!("{}: {}", arg.name(), quote_type(arg.type_tag())))
        .collect()
}

fn quote_code(code: &[u8]) -> String {
    format!(
        "vec![{}]",
        code.iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn quote_type_arguments(ty_args: &[TypeArgumentABI]) -> String {
    ty_args
        .iter()
        .map(|ty_arg| ty_arg.name().to_string())
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
        U8 => "u8".into(),
        U64 => "u64".into(),
        U128 => "u128".into(),
        Address => "AccountAddress".into(),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => "Vec<u8>".into(),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn make_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => format!("TransactionArgument::Bool({})", name),
        U8 => format!("TransactionArgument::U8({})", name),
        U64 => format!("TransactionArgument::U64({})", name),
        U128 => format!("TransactionArgument::U128({})", name),
        Address => format!("TransactionArgument::Address({})", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("TransactionArgument::U8Vector({})", name),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

pub struct Installer {
    install_dir: PathBuf,
    libra_types_version: String,
}

impl Installer {
    pub fn new(install_dir: PathBuf, libra_types_version: String) -> Self {
        Installer {
            install_dir,
            libra_types_version,
        }
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_transaction_builders(
        &self,
        name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        let dir_path = self.install_dir.join(name);
        std::fs::create_dir_all(&dir_path)?;
        let mut cargo = std::fs::File::create(&dir_path.join("Cargo.toml"))?;
        write!(
            cargo,
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2018"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
libra_types = "{}"
"#,
            name, self.libra_types_version,
        )?;
        std::fs::create_dir(dir_path.join("src"))?;
        let source_path = dir_path.join("src/lib.rs");
        let mut source = std::fs::File::create(&source_path)?;
        output(&mut source, abis, /* local_types */ false)?;
        Ok(())
    }
}
