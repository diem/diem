// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

use std::io::{Result, Write};

pub fn output(out: &mut dyn Write, abis: &[ScriptABI]) -> Result<()> {
    output_preamble(out)?;
    for abi in abis {
        output_builder(out, abi)?;
    }
    Ok(())
}

fn output_preamble(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"# pyre-ignore-all-errors
import libra_types as libra
import typing
import serde_types as st"#
    )
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    writeln!(
        out,
        "\ndef encode_{}_script({}) -> libra.Script:",
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
        r#"    return libra.Script(
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
        .map(|ty_arg| format!("{}: libra.TypeTag", ty_arg.name()))
        .collect()
}

fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
    args.iter()
        .map(|arg| format!("{}: {}", arg.name(), quote_type(arg.type_tag())))
        .collect()
}

fn quote_code(code: &[u8]) -> String {
    format!(
        "bytes([{}])",
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
        Bool => "st.bool".into(),
        U8 => "st.uint8".into(),
        U64 => "st.uint64".into(),
        U128 => "st.uint128".into(),
        Address => "libra.AccountAddress".into(),
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
        Bool => format!("libra.TransactionArgument.Bool({})", name),
        U8 => format!("libra.TransactionArgument.U8({})", name),
        U64 => format!("libra.TransactionArgument.U64({})", name),
        U128 => format!("libra.TransactionArgument.U128({})", name),
        Address => format!("libra.TransactionArgument.Address({})", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("libra.TransactionArgument.U8Vector({})", name),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}
