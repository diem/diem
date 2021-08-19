// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{
    ArgumentABI, ScriptABI, ScriptFunctionABI, TransactionScriptABI, TypeArgumentABI,
};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, TypeTag},
};
use serde_generate::indent::{IndentConfig, IndentedWriter};

use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Output a header-only library providing C++ transaction builders for the given ABIs.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI], namespace: Option<&str>) -> Result<()> {
    let mut emitter = CppEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        namespace,
        inlined_definitions: true,
    };
    emitter.output_preamble()?;
    emitter.output_open_namespace()?;
    emitter.output_using_namespaces()?;
    for abi in abis {
        match abi {
            ScriptABI::TransactionScript(abi) => {
                emitter.output_transaction_script_builder_definition(abi)?
            }
            ScriptABI::ScriptFunction(abi) => {
                emitter.output_script_function_builder_definition(abi)?
            }
        };
    }
    emitter.output_close_namespace()
}

/// Output the headers of a library providing C++ transaction builders for the given ABIs.
pub fn output_library_header(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    namespace: Option<&str>,
) -> Result<()> {
    let mut emitter = CppEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        namespace,
        inlined_definitions: true,
    };
    emitter.output_preamble()?;
    emitter.output_open_namespace()?;
    emitter.output_using_namespaces()?;
    for abi in abis {
        emitter.output_builder_declaration(abi)?;
    }
    emitter.output_close_namespace()
}

/// Output the function definitions of a library providing C++ transaction builders for the given ABIs.
pub fn output_library_body(
    out: &mut dyn Write,
    abis: &[ScriptABI],
    library_name: &str,
    namespace: Option<&str>,
) -> Result<()> {
    let mut emitter = CppEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        namespace,
        inlined_definitions: false,
    };
    writeln!(emitter.out, "#include \"{}.hpp\"\n", library_name)?;
    emitter.output_open_namespace()?;
    emitter.output_using_namespaces()?;
    for abi in abis {
        match abi {
            ScriptABI::TransactionScript(abi) => {
                emitter.output_transaction_script_builder_definition(abi)?
            }
            ScriptABI::ScriptFunction(abi) => {
                emitter.output_script_function_builder_definition(abi)?
            }
        };
    }
    emitter.output_close_namespace()
}

/// Shared state for the Cpp code generator.
struct CppEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Name of the package owning the generated definitions (e.g. "com.my_org.my_package")
    namespace: Option<&'a str>,
    /// Whether function definitions should be prefixed with "inlined"
    inlined_definitions: bool,
}

impl<'a, T> CppEmitter<'a, T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"#pragma once

#include "diem_types.hpp"
"#
        )
    }

    fn output_using_namespaces(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
using namespace serde;
using namespace diem_types;
"#
        )
    }

    fn output_open_namespace(&mut self) -> Result<()> {
        if let Some(name) = self.namespace {
            writeln!(self.out, "namespace {} {{\n", name)?;
        }
        Ok(())
    }

    fn output_close_namespace(&mut self) -> Result<()> {
        if let Some(name) = self.namespace {
            writeln!(self.out, "\n}} // end of namespace {}", name)?;
        }
        Ok(())
    }

    fn output_builder_declaration(&mut self, abi: &ScriptABI) -> Result<()> {
        self.output_doc(abi.doc())?;
        let parameters = [
            Self::quote_type_parameters(abi.ty_args()),
            Self::quote_parameters(abi.args()),
        ]
        .concat()
        .join(", ");
        match abi {
            ScriptABI::TransactionScript(abi) => writeln!(
                self.out,
                "Script encode_{}_script({});",
                abi.name(),
                parameters,
            )?,
            ScriptABI::ScriptFunction(abi) => writeln!(
                self.out,
                "TransactionPayload encode_{}_script_function({});",
                abi.name(),
                parameters,
            )?,
        };
        Ok(())
    }

    fn output_transaction_script_builder_definition(
        &mut self,
        abi: &TransactionScriptABI,
    ) -> Result<()> {
        if self.inlined_definitions {
            self.output_doc(abi.doc())?;
        }
        writeln!(
            self.out,
            "{}Script encode_{}_script({}) {{",
            if self.inlined_definitions {
                "inline "
            } else {
                ""
            },
            abi.name(),
            [
                Self::quote_type_parameters(abi.ty_args()),
                Self::quote_parameters(abi.args()),
            ]
            .concat()
            .join(", ")
        )?;
        writeln!(
            self.out,
            r#"    return Script {{
                {},
                std::vector<TypeTag> {{{}}},
                std::vector<TransactionArgument> {{{}}},
            }};"#,
            Self::quote_code(abi.code()),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments_for_script(abi.args()),
        )?;
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_script_function_builder_definition(&mut self, abi: &ScriptFunctionABI) -> Result<()> {
        if self.inlined_definitions {
            self.output_doc(abi.doc())?;
        }
        writeln!(
            self.out,
            "{}TransactionPayload encode_{}_script_function({}) {{",
            if self.inlined_definitions {
                "inline "
            } else {
                ""
            },
            abi.name(),
            [
                Self::quote_type_parameters(abi.ty_args()),
                Self::quote_parameters(abi.args()),
            ]
            .concat()
            .join(", ")
        )?;
        writeln!(
            self.out,
            r#"    return TransactionPayload {{
                TransactionPayload::ScriptFunction {{
                    {},
                    {},
                    std::vector<TypeTag> {{{}}},
                    std::vector<std::vector<uint8_t>> {{{}}},
                }}
            }};"#,
            Self::quote_module_id(abi.module_name()),
            Self::quote_identifier(abi.name()),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_doc(&mut self, doc: &str) -> Result<()> {
        let doc = crate::common::prepare_doc_string(doc);
        let text = textwrap::indent(&doc, "/// ").replace("\n\n", "\n///\n");
        write!(self.out, "\n{}\n", text)
    }

    fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
        ty_args
            .iter()
            .map(|ty_arg| format!("TypeTag {}", ty_arg.name()))
            .collect()
    }

    fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
        args.iter()
            .map(|arg| format!("{} {}", Self::quote_type(arg.type_tag()), arg.name()))
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

    fn quote_identifier(ident: &str) -> String {
        format!("Identifier {{ \"{}\" }}", ident)
    }

    fn quote_address(address: &AccountAddress) -> String {
        format!(
            "std::array<uint8_t, 16>{{ {} }}",
            address
                .to_vec()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    fn quote_module_id(module_id: &ModuleId) -> String {
        format!(
            "ModuleId {{ {}, {} }}",
            Self::quote_address(module_id.address()),
            Self::quote_identifier(module_id.name().as_str())
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
            .map(|arg| Self::quote_transaction_argument(arg.type_tag(), arg.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn quote_arguments_for_script(args: &[ArgumentABI]) -> String {
        args.iter()
            .map(|arg| Self::quote_transaction_argument_for_script(arg.type_tag(), arg.name()))
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
                Vector(type_tag) => {
                    if type_tag.as_ref() == &U8 {
                        "std::vector<std::vector<uint8_t>>".into()
                    } else {
                        common::type_not_allowed(type_tag)
                    }
                }
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        match Self::bcs_primitive_type_name(type_tag) {
            None => format!("{}.bcsSerialize()", name),
            Some(type_name) => format!(
                r#"({{
            auto s = BcsSerializer();
            Serializable<{}>::serialize({}, s);
            std::move(s).bytes();
            }})"#,
                type_name, name
            ),
        }
    }

    fn quote_transaction_argument_for_script(type_tag: &TypeTag, name: &str) -> String {
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
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    // - if a `type_tag` is a primitive type in BCS, we can call
    //   `Serializable<name>::serialize(arg, &s)` and `Deserializable<name>::deserialize(arg, &d)`
    //   to convert into and from `std::vector<uint8_t>`.
    // - otherwise, we can use `<arg>.bcsSerialize()`, `<arg>.bcsDeserialize()` to do the work.
    fn bcs_primitive_type_name(type_tag: &TypeTag) -> Option<&'static str> {
        use TypeTag::*;
        match type_tag {
            Bool => Some("bool"),
            U8 => Some("uint8_t"),
            U64 => Some("uint64_t"),
            U128 => Some("uint128_t"),
            Address => None,
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => Some("std::vector<uint8_t>"),
                Vector(type_tag) => {
                    if type_tag.as_ref() == &U8 {
                        Some("std::vector<std::vector<uint8_t>>")
                    } else {
                        common::type_not_allowed(type_tag)
                    }
                }
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
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
