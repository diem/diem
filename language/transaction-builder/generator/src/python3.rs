// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use heck::{CamelCase, ShoutySnakeCase};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    indent::{IndentConfig, IndentedWriter},
    python3, CodeGeneratorConfig,
};

use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders in Python for the given ABIs.
pub fn output(
    out: &mut dyn Write,
    serde_package_name: Option<String>,
    diem_package_name: Option<String>,
    abis: &[ScriptABI],
) -> Result<()> {
    let mut emitter = PythonEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        serde_package_name,
        diem_package_name,
    };
    emitter.output_script_call_enum_with_imports(abis)?;
    emitter.output_additional_imports()?;

    emitter.output_encode_method()?;
    emitter.output_decode_method()?;

    for abi in abis {
        emitter.output_script_encoder_function(abi)?;
    }
    for abi in abis {
        emitter.output_script_decoder_function(abi)?;
    }

    for abi in abis {
        emitter.output_code_constant(abi)?;
    }
    emitter.output_encoder_map(abis)?;
    emitter.output_decoder_map(abis)?;

    emitter.output_decoding_helpers(abis)?;

    Ok(())
}

/// Shared state for the Python code generator.
struct PythonEmitter<T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Package where to find the serde module (if any).
    serde_package_name: Option<String>,
    /// Package where to find the diem module (if any).
    diem_package_name: Option<String>,
}

impl<T> PythonEmitter<T>
where
    T: Write,
{
    fn output_additional_imports(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
from {}diem_types import (Script, TypeTag, AccountAddress, TransactionArgument, TransactionArgument__Bool, TransactionArgument__U8, TransactionArgument__U64, TransactionArgument__U128, TransactionArgument__Address, TransactionArgument__U8Vector)"#,
            match &self.diem_package_name {
                None => "".into(),
                Some(package) => package.clone() + ".",
            },
        )
    }

    fn output_encode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
def encode_script(call: ScriptCall) -> Script:
    """Build a Diem `Script` from a structured object `ScriptCall`.
    """
    helper = SCRIPT_ENCODER_MAP[call.__class__]
    return helper(call)
"#
        )
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
def decode_script(script: Script) -> ScriptCall:
    """Try to recognize a Diem `Script` and convert it into a structured object `ScriptCall`.
    """
    helper = SCRIPT_DECODER_MAP.get(script.code)
    if helper is None:
        raise ValueError("Unknown script bytecode")
    return helper(script)
"#
        )
    }

    fn output_script_call_enum_with_imports(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let diem_types_module = match &self.diem_package_name {
            None => "diem_types".into(),
            Some(package) => format!("{}.diem_types", package),
        };
        let external_definitions = crate::common::get_external_definitions(&diem_types_module);
        let script_registry: BTreeMap<_, _> = vec![(
            "ScriptCall".to_string(),
            crate::common::make_abi_enum_container(abis),
        )]
        .into_iter()
        .collect();
        let mut comments: BTreeMap<_, _> = abis
            .iter()
            .map(|abi| {
                (
                    vec![
                        "".to_string(),
                        "ScriptCall".to_string(),
                        abi.name().to_camel_case(),
                    ],
                    Self::prepare_doc_string(abi.doc()),
                )
            })
            .collect();
        comments.insert(
            vec!["".to_string(), "ScriptCall".to_string()],
            "Structured representation of a call into a known Move script.".into(),
        );
        let config = CodeGeneratorConfig::new("".to_string())
            .with_comments(comments)
            .with_external_definitions(external_definitions)
            .with_serialization(false);
        python3::CodeGenerator::new(&config)
            .with_serde_package_name(self.serde_package_name.clone())
            .output(&mut self.out, &script_registry)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
        Ok(())
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\ndef encode_{}_script({}) -> Script:",
            abi.name(),
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
            "\"\"\"{}\n\"\"\"",
            Self::prepare_doc_string(abi.doc())
        )?;
        writeln!(
            self.out,
            r#"return Script(
    code={}_CODE,
    ty_args=[{}],
    args=[{}],
)
"#,
            abi.name().to_shouty_snake_case(),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        self.out.unindent();
        Ok(())
    }

    fn output_script_decoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\ndef decode_{}_script({}script: Script) -> ScriptCall:",
            abi.name(),
            // prevent warning "unused variable"
            if abi.ty_args().is_empty() && abi.args().is_empty() {
                "_"
            } else {
                ""
            }
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "return ScriptCall__{0}(",
            abi.name().to_camel_case(),
        )?;
        self.out.indent();
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(self.out, "{}=script.ty_args[{}],", ty_arg.name(), index,)?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                "{}=decode_{}_argument(script.args[{}]),",
                arg.name(),
                common::mangle_type(arg.type_tag()),
                index,
            )?;
        }
        self.out.unindent();
        writeln!(self.out, ")\n")?;
        self.out.unindent();
        Ok(())
    }

    fn output_code_constant(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\n{}_CODE = b\"{}\"",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("\\x{:02x}", x))
                .collect::<Vec<_>>()
                .join(""),
        )
    }

    fn output_encoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
# pyre-ignore
SCRIPT_ENCODER_MAP: typing.Dict[typing.Type[ScriptCall], typing.Callable[[ScriptCall], Script]] = {{"#
        )?;
        self.out.indent();
        for abi in abis {
            writeln!(
                self.out,
                "ScriptCall__{}: encode_{}_script,",
                abi.name().to_camel_case(),
                abi.name()
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_decoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            "\nSCRIPT_DECODER_MAP: typing.Dict[bytes, typing.Callable[[Script], ScriptCall]] = {{"
        )?;
        self.out.indent();
        for abi in abis {
            writeln!(
                self.out,
                "{}_CODE: decode_{}_script,",
                abi.name().to_shouty_snake_case(),
                abi.name()
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_decoding_helpers(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let required_types = common::get_required_decoding_helper_types(abis);
        for required_type in required_types {
            self.output_decoding_helper(required_type)?;
        }
        Ok(())
    }

    fn output_decoding_helper(&mut self, type_tag: &TypeTag) -> Result<()> {
        use TypeTag::*;
        let (constructor, expr) = match type_tag {
            Bool => ("Bool", "arg.value".into()),
            U8 => ("U8", "arg.value".into()),
            U64 => ("U64", "arg.value".into()),
            U128 => ("U128", "arg.value".into()),
            Address => ("Address", "arg.value".into()),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => ("U8Vector", "arg.value".into()),
                inner_type_tag => (
                    "Vector",
                    format!(
                        "[decode_{}_argument(x) for x in arg.value]",
                        common::mangle_type(inner_type_tag)
                    ),
                ),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        };
        writeln!(
            self.out,
            r#"
def decode_{}_argument(arg: TransactionArgument) -> {}:
    if not isinstance(arg, TransactionArgument__{}):
        raise ValueError("Was expecting a {} argument")
    return {}
"#,
            common::mangle_type(type_tag),
            Self::quote_type(type_tag),
            constructor,
            constructor,
            expr,
        )
    }

    fn prepare_doc_string(doc: &str) -> String {
        let doc = crate::common::prepare_doc_string(doc);
        let s: Vec<_> = doc.splitn(2, |c| c == '.').collect();
        if s.len() <= 1 || s[1].is_empty() {
            format!("{}.", s[0])
        } else {
            format!("{}.\n\n{}", s[0], s[1].trim())
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
            Bool => "st.bool".into(),
            U8 => "st.uint8".into(),
            U64 => "st.uint64".into(),
            U128 => "st.uint128".into(),
            Address => "AccountAddress".into(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => "bytes".into(),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => format!("TransactionArgument__Bool(value={})", name),
            U8 => format!("TransactionArgument__U8(value={})", name),
            U64 => format!("TransactionArgument__U64(value={})", name),
            U128 => format!("TransactionArgument__U128(value={})", name),
            Address => format!("TransactionArgument__Address(value={})", name),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => format!("TransactionArgument__U8Vector(value={})", name),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }
}

pub struct Installer {
    install_dir: PathBuf,
    serde_package_name: Option<String>,
    diem_package_name: Option<String>,
}

impl Installer {
    pub fn new(
        install_dir: PathBuf,
        serde_package_name: Option<String>,
        diem_package_name: Option<String>,
    ) -> Self {
        Installer {
            install_dir,
            serde_package_name,
            diem_package_name,
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
        output(
            &mut file,
            self.serde_package_name.clone(),
            self.diem_package_name.clone(),
            abis,
        )?;
        Ok(())
    }
}
