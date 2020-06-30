// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Python for the given ABIs.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI]) -> Result<()> {
    output_preamble(out, None, None)?;
    for abi in abis {
        output_builder(out, abi)?;
    }
    Ok(())
}

fn output_with_optional_packages(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    serde_package_name: Option<String>,
    libra_package_name: Option<String>,
) -> Result<()> {
    output_preamble(out, serde_package_name, libra_package_name)?;
    for abi in abis {
        output_builder(out, abi)?;
    }
    Ok(())
}

fn quote_from_package(package_name: Option<String>) -> String {
    match package_name {
        None => "".to_string(),
        Some(name) => format!("from {} ", name),
    }
}

fn quote_from_package_and_module(package_name: Option<String>, module_name: &str) -> String {
    match package_name {
        None => format!("from {} ", module_name),
        Some(name) => format!("from {}.{} ", name, module_name),
    }
}

fn output_preamble(
    out: &mut dyn Write,
    serde_package_name: Option<String>,
    libra_package_name: Option<String>,
) -> Result<()> {
    writeln!(
        out,
        r#"import typing
{}import serde_types as st
{}import Script, TypeTag, AccountAddress, TransactionArgument__Bool, TransactionArgument__U8, TransactionArgument__U64, TransactionArgument__U128, TransactionArgument__Address, TransactionArgument__U8Vector
"#,
        quote_from_package(serde_package_name),
        quote_from_package_and_module(libra_package_name, "libra_types"),
    )
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    writeln!(
        out,
        "\ndef encode_{}_script({}) -> Script:",
        abi.name(),
        [
            quote_type_parameters(abi.ty_args()),
            quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ")
    )?;
    writeln!(out, "{}", quote_doc(abi.doc()))?;
    writeln!(
        out,
        r#"    return Script(
        code={},
        ty_args=[{}],
        args=[{}],
    )"#,
        quote_code(abi.code()),
        quote_type_arguments(abi.ty_args()),
        quote_arguments(abi.args()),
    )?;
    Ok(())
}

fn quote_doc(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    let s: Vec<_> = doc.splitn(2, |c| c == '.').collect();
    if s.len() <= 1 || s[1].is_empty() {
        format!("    \"\"\"{}.\"\"\"", s[0])
    } else {
        format!(
            r#"    """{}.

{}    """"#,
            s[0],
            textwrap::indent(&textwrap::fill(&textwrap::dedent(s[1]), 86), "    "),
        )
    }
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
        "b\"{}\"",
        code.iter()
            .map(|x| format!("\\x{:02x}", x))
            .collect::<Vec<_>>()
            .join("")
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
        Bool => "st.bool".into(),
        U8 => "st.uint8".into(),
        U64 => "st.uint64".into(),
        U128 => "st.uint128".into(),
        Address => "AccountAddress".into(),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => "bytes".into(),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn make_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => format!("TransactionArgument__Bool({})", name),
        U8 => format!("TransactionArgument__U8({})", name),
        U64 => format!("TransactionArgument__U64({})", name),
        U128 => format!("TransactionArgument__U128({})", name),
        Address => format!("TransactionArgument__Address({})", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("TransactionArgument__U8Vector({})", name),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

pub struct Installer {
    install_dir: PathBuf,
    serde_package_name: Option<String>,
    libra_package_name: Option<String>,
}

impl Installer {
    pub fn new(
        install_dir: PathBuf,
        serde_package_name: Option<String>,
        libra_package_name: Option<String>,
    ) -> Self {
        Installer {
            install_dir,
            serde_package_name,
            libra_package_name,
        }
    }

    fn open_module_init_file(&self, name: &str) -> Result<std::fs::File> {
        let dir_path = self.install_dir.join(name);
        std::fs::create_dir_all(&dir_path)?;
        std::fs::File::create(dir_path.join("__init__.py"))
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_transaction_builders(
        &self,
        name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        let mut file = self.open_module_init_file(name)?;
        output_with_optional_packages(
            &mut file,
            abis,
            self.serde_package_name.clone(),
            self.libra_package_name.clone(),
        )?;
        Ok(())
    }
}
