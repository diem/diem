// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    indent::{IndentConfig, IndentedWriter},
    typescript, CodeGeneratorConfig,
};

use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};
/// Output transaction builders and decoders in TypeScript for the given ABIs.
pub fn output(out: &mut dyn Write, abis: &[ScriptABI]) -> Result<()> {
    write_script_calls(out, abis)?;
    write_helpers(out, abis)
}

fn write_stdlib_helper_interfaces(emitter: &mut TypeScriptEmitter<&mut dyn Write>) -> Result<()> {
    writeln!(
        emitter.out,
        r#"
export interface TypeTagDef {{
  type: Types;
  arrayType?: TypeTagDef;
  name?: string;
  module?: string;
  address?: string;
  typeParams?: TypeTagDef[];
}}

export interface ArgDef {{
  readonly name: string;
  readonly type: TypeTagDef;
  readonly choices?: string[];
  readonly mandatory?: boolean;
}}

export interface ScriptDef {{
  readonly stdlibEncodeFunction: (...args: any[]) => DiemTypes.Script;
  readonly stdlibDecodeFunction: (script: DiemTypes.Script) => ScriptCall;
  readonly codeName: string;
  readonly description: string;
  readonly typeArgs: string[];
  readonly args: ArgDef[];
}}

export enum Types {{
  Boolean,
  U8,
  U64,
  U128,
  Address,
  Array,
  Struct
}}
"#
    )?;

    Ok(())
}

/// Output transaction helper functions for the given ABIs.
fn write_helpers(out: &mut dyn Write, abis: &[ScriptABI]) -> Result<()> {
    let mut emitter = TypeScriptEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(2)),
    };
    emitter.output_preamble()?;
    write_stdlib_helper_interfaces(&mut emitter)?;
    writeln!(emitter.out, "\nexport class Stdlib {{")?;
    emitter.out.indent();
    writeln!(emitter.out, "private static fromHexString(hexString: string): Uint8Array {{ return new Uint8Array(hexString.match(/.{{1,2}}/g)!.map((byte) => parseInt(byte, 16)));}}")?;

    for abi in abis {
        emitter.output_script_encoder_function(abi)?;
    }
    for abi in abis {
        emitter.output_script_decoder_function(abi)?;
    }

    for abi in abis {
        emitter.output_code_constant(abi)?;
    }
    writeln!(
        emitter.out,
        "\nstatic ScriptArgs: {{[name: string]: ScriptDef}} = {{"
    )?;
    emitter.out.indent();
    for abi in abis {
        emitter.output_script_args_definition(abi)?;
    }
    emitter.out.unindent();
    writeln!(emitter.out, "}}")?;

    emitter.out.unindent();
    writeln!(emitter.out, "\n}}\n")?;

    writeln!(emitter.out, "\nexport type ScriptDecoders = {{")?;
    emitter.out.indent();
    writeln!(emitter.out, "User: {{")?;
    emitter.out.indent();
    for abi in abis {
        emitter.output_script_args_callbacks(abi)?;
    }
    writeln!(
        emitter.out,
        "default: (type: keyof ScriptDecoders['User']) => void;"
    )?;
    emitter.out.unindent();
    writeln!(emitter.out, "}};")?;
    emitter.out.unindent();
    writeln!(emitter.out, "}};")
}

fn write_script_calls(out: &mut dyn Write, abis: &[ScriptABI]) -> Result<()> {
    let external_definitions = crate::common::get_external_definitions("diemTypes");
    let script_registry: BTreeMap<_, _> = vec![(
        "ScriptCall".to_string(),
        common::make_abi_enum_container(abis),
    )]
    .into_iter()
    .collect();
    let mut comments: BTreeMap<_, _> = abis
        .iter()
        .map(|abi| {
            let mut paths = Vec::new();
            paths.push("ScriptCall".to_string());
            paths.push(abi.name().to_camel_case());
            (paths, crate::common::prepare_doc_string(abi.doc()))
        })
        .collect();
    comments.insert(
        vec!["ScriptCall".to_string()],
        "Structured representation of a call into a known Move script.".into(),
    );

    let config = CodeGeneratorConfig::new("StdLib".to_string())
        .with_comments(comments)
        .with_external_definitions(external_definitions)
        .with_serialization(false);
    typescript::CodeGenerator::new(&config)
        .output(out, &script_registry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
    Ok(())
}

/// Shared state for the TypeScript code generator.
struct TypeScriptEmitter<T> {
    /// Writer.
    out: IndentedWriter<T>,
}

impl<T> TypeScriptEmitter<T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
import {{ BcsSerializer }} from '../bcs/bcsSerializer';

"#,
        )?;
        Ok(())
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\n{}static encode{}Script({}): DiemTypes.Script {{",
            Self::quote_doc(abi.doc()),
            abi.name().to_camel_case(),
            [
                Self::quote_type_parameters(abi.ty_args()),
                Self::quote_parameters(abi.args()),
            ]
            .concat()
            .join(", ")
        )?;
        self.out.indent();
        writeln!(
            self.out,
            r#"const code = Stdlib.{}_CODE;
const tyArgs: Seq<DiemTypes.TypeTag> = [{}];
const args: Seq<DiemTypes.TransactionArgument> = [{}];
return new DiemTypes.Script(code, tyArgs, args);"#,
            abi.name().to_shouty_snake_case(),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_decoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nstatic decode{}Script({}script: DiemTypes.Script): ScriptCallVariant{0} {{",
            abi.name().to_camel_case(),
            // prevent warning "unused variable"
            if abi.ty_args().is_empty() && abi.args().is_empty() {
                "_"
            } else {
                ""
            }
        )?;
        let mut all_args: Vec<String> = Vec::new();
        all_args.extend(
            abi.ty_args()
                .iter()
                .enumerate()
                .map(|(idx, _)| format!("script.ty_args[{}]", idx))
                .collect::<Vec<_>>(),
        );
        all_args.extend(
            abi.args()
                .iter()
                .enumerate()
                .map(|(idx, arg)| {
                    format!(
                        "(script.args[{}] as {}).value",
                        idx,
                        Self::quote_transaction_argument_type(arg.type_tag())
                    )
                })
                .collect::<Vec<_>>(),
        );
        self.out.indent();
        writeln!(
            self.out,
            "return new ScriptCallVariant{}(",
            abi.name().to_camel_case()
        )?;
        self.out.indent();
        writeln!(self.out, "{}", all_args.join(",\n"))?;
        self.out.unindent();
        writeln!(self.out, ");",)?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_code_constant(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nstatic {}_CODE = Stdlib.fromHexString('{}');",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("{:02x}", *x as i8))
                .collect::<Vec<_>>()
                .join("")
        )?;
        Ok(())
    }

    fn output_script_args_definition(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(self.out, "{}: {{", abi.name().to_camel_case())?;
        writeln!(
            self.out,
            "  stdlibEncodeFunction: Stdlib.encode{}Script,",
            abi.name().to_camel_case()
        )?;
        writeln!(
            self.out,
            "  stdlibDecodeFunction: Stdlib.decode{}Script,",
            abi.name().to_camel_case()
        )?;
        writeln!(
            self.out,
            "  codeName: '{}',",
            abi.name().to_shouty_snake_case()
        )?;
        writeln!(
            self.out,
            "  description: \"{}\",",
            abi.doc().replace("\"", "\\\"").replace("\n", "\" + \n \"")
        )?;
        writeln!(
            self.out,
            "  typeArgs: [{}],",
            abi.ty_args()
                .iter()
                .map(|ty_arg| format!("\"{}\"", ty_arg.name()))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(self.out, "  args: [")?;
        writeln!(
            self.out,
            "{}",
            abi.args()
                .iter()
                .map(|arg| format!(
                    "{{name: \"{}\", type: {}}}",
                    arg.name(),
                    Self::quote_script_arg_type(arg.type_tag())
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        writeln!(self.out, "  ]")?;
        writeln!(self.out, "}},")?;
        Ok(())
    }

    fn output_script_args_callbacks(&mut self, abi: &ScriptABI) -> Result<()> {
        let mut args_with_types = abi
            .ty_args()
            .iter()
            .map(|ty_arg| {
                format!(
                    "{}: DiemTypes.TypeTagVariantStruct",
                    ty_arg.name().to_mixed_case()
                )
            })
            .collect::<Vec<_>>();
        args_with_types.extend(
            abi.args()
                .iter()
                .map(|arg| {
                    format!(
                        "{}: {}",
                        arg.name().to_mixed_case(),
                        Self::quote_transaction_argument_type(arg.type_tag())
                    )
                })
                .collect::<Vec<_>>(),
        );
        writeln!(
            self.out,
            "{}: (type: string, {}) => void;",
            abi.name().to_camel_case(),
            args_with_types.join(", ")
        )?;
        Ok(())
    }

    fn quote_doc(doc: &str) -> String {
        let doc = crate::common::prepare_doc_string(doc);
        let text = textwrap::indent(&doc, " * ").replace("\n\n", "\n *\n");
        format!("/**\n{}\n */\n", text)
    }

    fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
        ty_args
            .iter()
            .map(|ty_arg| format!("{}: DiemTypes.TypeTag", ty_arg.name()))
            .collect()
    }

    fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
        args.iter()
            .map(|arg| format!("{}: {}", arg.name(), Self::quote_type(arg.type_tag())))
            .collect()
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
            .map(|arg| Self::quote_transaction_argument(arg.type_tag(), arg.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn quote_type(type_tag: &TypeTag) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => "boolean".into(),
            U8 => "number".into(),
            U64 => "BigInt".into(),
            U128 => "BigInt".into(),
            Address => "DiemTypes.AccountAddress".into(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => "Uint8Array".into(),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument_type(type_tag: &TypeTag) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => "DiemTypes.TransactionArgumentVariantBool".to_string(),
            U8 => "DiemTypes.TransactionArgumentVariantU8".to_string(),
            U64 => "DiemTypes.TransactionArgumentVariantU64".to_string(),
            U128 => "DiemTypes.TransactionArgumentVariantU128".to_string(),
            Address => "DiemTypes.TransactionArgumentVariantAddress".to_string(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => "DiemTypes.TransactionArgumentVariantU8Vector".to_string(),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        format!(
            "new {}({})",
            Self::quote_transaction_argument_type(type_tag),
            name
        )
    }

    fn quote_script_arg_type(type_tag: &TypeTag) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => "{type: Types.Boolean}".to_string(),
            U8 => "{type: Types.U8}".to_string(),
            U64 => "{type: Types.U64}".to_string(),
            U128 => "{type: Types.U128}".to_string(),
            Address => "{type: Types.Address}".to_string(),
            Vector(type_tag) => format!("{{type: Types.Array, arrayType: {}}}", Self::quote_script_arg_type(type_tag)),
            Struct(struct_tag) => format!("{{type: Types.Struct, name: \"{}\", module: \"{}\", address: \"{}\", typeParams: [{}]}}",
                                          struct_tag.name,
                                          struct_tag.module,
                                          struct_tag.address,
                                          struct_tag.type_params.iter().map(|tt| Self::quote_script_arg_type(tt)).collect::<Vec<_>>().join(", ")),
            Signer => common::type_not_allowed(type_tag),
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
        let dir_path = self.install_dir.join(name);
        std::fs::create_dir_all(&dir_path)?;
        let mut file = std::fs::File::create(dir_path.join("index.ts"))?;
        output(&mut file, abis)?;
        Ok(())
    }
}
