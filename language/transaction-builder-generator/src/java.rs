// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::type_not_allowed;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Java for the given ABIs.
pub fn output(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    package: Option<&str>,
    class_name: &str,
) -> Result<()> {
    output_preamble(out, package)?;
    writeln!(out, "\npublic final class {} {{\n", class_name)?;
    for abi in abis {
        output_builder(out, abi)?;
    }
    writeln!(out, "\n}}\n")?;
    Ok(())
}

fn output_preamble(out: &mut dyn Write, package: Option<&str>) -> Result<()> {
    if let Some(name) = package {
        writeln!(out, "package {};\n", name)?;
    }
    writeln!(
        out,
        r#"
import java.math.BigInteger;
import java.util.Arrays;
import org.libra.types.AccountAddress;
import org.libra.types.Script;
import org.libra.types.TransactionArgument;
import org.libra.types.TypeTag;
import com.facebook.serde.Int128;
import com.facebook.serde.Unsigned;
import com.facebook.serde.Bytes;
"#,
    )?;
    Ok(())
}

fn output_builder(out: &mut dyn Write, abi: &ScriptABI) -> Result<()> {
    writeln!(
        out,
        "\n{}public static Script encode_{}_script({}) {{",
        quote_doc(abi.doc()),
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
        r#"    Script.Builder builder = new Script.Builder();
    builder.code = new Bytes(new byte[]{});
    builder.ty_args = Arrays.asList({});
    builder.args = Arrays.asList({});
    return builder.build();
}}"#,
        quote_code(abi.code()),
        quote_type_arguments(abi.ty_args()),
        quote_arguments(abi.args()),
    )?;
    Ok(())
}

fn quote_doc(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    let text = textwrap::fill(&doc, 86);
    format!("/**\n{} */\n", textwrap::indent(&text, " * "))
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
        "{{{}}}",
        code.iter()
            .map(|x| format!("{}", *x as i8))
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
        Bool => "Boolean".into(),
        U8 => "@Unsigned Byte".into(),
        U64 => "@Unsigned Long".into(),
        U128 => "@Unsigned @Int128 BigInteger".into(),
        Address => "AccountAddress".into(),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => "Bytes".into(),
            _ => type_not_allowed(type_tag),
        },

        Struct(_) | Signer => type_not_allowed(type_tag),
    }
}

fn make_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
    use TypeTag::*;
    match type_tag {
        Bool => format!("new TransactionArgument.Bool({})", name),
        U8 => format!("new TransactionArgument.U8({})", name),
        U64 => format!("new TransactionArgument.U64({})", name),
        U128 => format!("new TransactionArgument.U128({})", name),
        Address => format!("new TransactionArgument.Address({})", name),
        Vector(type_tag) => match type_tag.as_ref() {
            U8 => format!("new TransactionArgument.U8Vector({})", name),
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
        let parts = name.split('.').collect::<Vec<_>>();
        let mut dir_path = self.install_dir.clone();
        let mut package_name = None;
        for part in &parts[..parts.len() - 1] {
            dir_path = dir_path.join(part);
            package_name = match package_name {
                Some(previous) => Some(format!("{}.{}", previous, part)),
                None => Some(part.to_string()),
            };
        }
        let class_name = parts.last().unwrap().to_string();

        std::fs::create_dir_all(&dir_path)?;

        let mut file = std::fs::File::create(dir_path.join(class_name.clone() + ".java"))?;
        output(&mut file, abis, package_name.as_deref(), &class_name)?;
        Ok(())
    }
}
