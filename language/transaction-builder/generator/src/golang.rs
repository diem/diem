// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    golang,
    indent::{IndentConfig, IndentedWriter},
    CodeGeneratorConfig,
};

use heck::CamelCase;
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders and decoders in Go for the given ABIs.
pub fn output(
    out: &mut dyn Write,
    serde_module_path: Option<String>,
    diem_module_path: Option<String>,
    package_name: String,
    abis: &[ScriptABI],
) -> Result<()> {
    let mut emitter = GoEmitter {
        out: IndentedWriter::new(out, IndentConfig::Tab),
        serde_module_path,
        diem_module_path,
        package_name,
    };
    emitter.output_script_call_enum_with_imports(abis)?;

    emitter.output_encode_method(abis)?;
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
    emitter.output_decoder_map(abis)?;

    emitter.output_decoding_helpers(abis)?;

    Ok(())
}

/// Shared state for the Go code generator.
struct GoEmitter<T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Go module path for Serde runtime packages
    /// `None` to use the default path.
    serde_module_path: Option<String>,
    /// Go module path for Diem types.
    /// `None` to use an empty path.
    diem_module_path: Option<String>,
    /// Name of the package owning the generated definitions (e.g. "my_package")
    package_name: String,
}

impl<T> GoEmitter<T>
where
    T: Write,
{
    fn output_script_call_enum_with_imports(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let diem_types_package = match &self.diem_module_path {
            Some(path) => format!("{}/diemtypes", path),
            None => "diemtypes".into(),
        };
        let mut external_definitions = crate::common::get_external_definitions(&diem_types_package);
        // Add standard imports
        external_definitions.insert("fmt".to_string(), Vec::new());

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
                        self.package_name.to_string(),
                        "ScriptCall".to_string(),
                        abi.name().to_camel_case(),
                    ],
                    crate::common::prepare_doc_string(abi.doc()),
                )
            })
            .collect();
        comments.insert(
            vec![self.package_name.to_string(), "ScriptCall".to_string()],
            "Structured representation of a call into a known Move script.".into(),
        );
        let config = CodeGeneratorConfig::new(self.package_name.to_string())
            .with_comments(comments)
            .with_external_definitions(external_definitions)
            .with_serialization(false);
        let mut generator = golang::CodeGenerator::new(&config);
        if let Some(path) = &self.serde_module_path {
            generator = generator.with_serde_module_path(path.clone());
        }
        generator
            .output(&mut self.out, &script_registry)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
        Ok(())
    }

    fn output_encode_method(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
// Build a Diem `Script` from a structured object `ScriptCall`.
func EncodeScript(call ScriptCall) diemtypes.Script {{"#
        )?;
        self.out.indent();
        writeln!(self.out, "switch call := call.(type) {{")?;
        for abi in abis {
            let params = std::iter::empty()
                .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
                .chain(abi.args().iter().map(ArgumentABI::name))
                .map(|name| format!("call.{}", name.to_camel_case()))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                self.out,
                r#"case *ScriptCall__{0}:
	return Encode{0}Script({1})"#,
                abi.name().to_camel_case(),
                params,
            )?;
        }
        writeln!(self.out, "}}")?;
        writeln!(self.out, "panic(\"unreachable\")")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
// Try to recognize a Diem `Script` and convert it into a structured object `ScriptCall`.
func DecodeScript(script *diemtypes.Script) (ScriptCall, error) {{
	if helper := script_decoder_map[string(script.Code)]; helper != nil {{
		val, err := helper(script)
                return val, err
	}} else {{
		return nil, fmt.Errorf("Unknown script bytecode: %s", string(script.Code))
	}}
}}"#
        )
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\n{}func Encode{}Script({}) diemtypes.Script {{",
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
            r#"return diemtypes.Script {{
	Code: append([]byte(nil), {}_code...),
	TyArgs: []diemtypes.TypeTag{{{}}},
	Args: []diemtypes.TransactionArgument{{{}}},
}}"#,
            abi.name(),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_decoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nfunc decode_{}_script(script *diemtypes.Script) (ScriptCall, error) {{",
            abi.name(),
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "if len(script.TyArgs) < {0} {{ return nil, fmt.Errorf(\"Was expecting {0} type arguments\") }}",
            abi.ty_args().len(),
        )?;
        writeln!(
            self.out,
            "if len(script.Args) < {0} {{ return nil, fmt.Errorf(\"Was expecting {0} regular arguments\") }}",
            abi.args().len(),
        )?;
        writeln!(
            self.out,
            "var call ScriptCall__{0}",
            abi.name().to_camel_case(),
        )?;
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "call.{} = script.TyArgs[{}]",
                ty_arg.name().to_camel_case(),
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                r#"if val, err := decode_{}_argument(script.Args[{}]); err == nil {{
	call.{} = val
}} else {{
	return nil, err
}}
"#,
                common::mangle_type(arg.type_tag()),
                index,
                arg.name().to_camel_case(),
            )?;
        }
        writeln!(self.out, "return &call, nil")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_decoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
var script_decoder_map = map[string]func(*diemtypes.Script) (ScriptCall, error) {{"#
        )?;
        self.out.indent();
        for abi in abis {
            writeln!(
                self.out,
                "string({}_code): decode_{}_script,",
                abi.name(),
                abi.name()
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")
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
        let default_stmt = format!("value = {}(*arg)", Self::quote_type(type_tag));
        let (constructor, stmt) = match type_tag {
            Bool => ("Bool", default_stmt),
            U8 => ("U8", default_stmt),
            U64 => ("U64", default_stmt),
            U128 => ("U128", default_stmt),
            Address => ("Address", "value = arg.Value".into()),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => ("U8Vector", default_stmt),
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        };
        writeln!(
            self.out,
            r#"
func decode_{0}_argument(arg diemtypes.TransactionArgument) (value {1}, err error) {{
	if arg, ok := arg.(*diemtypes.TransactionArgument__{2}); ok {{
		{3}
	}} else {{
		err = fmt.Errorf("Was expecting a {2} argument")
	}}
	return
}}
"#,
            common::mangle_type(type_tag),
            Self::quote_type(type_tag),
            constructor,
            stmt,
        )
    }

    fn output_code_constant(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nvar {}_code = []byte {{{}}};",
            abi.name(),
            abi.code()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }

    fn quote_doc(doc: &str) -> String {
        let doc = crate::common::prepare_doc_string(doc);
        textwrap::indent(&doc, "// ").replace("\n\n", "\n//\n")
    }

    fn quote_type_parameters(ty_args: &[TypeArgumentABI]) -> Vec<String> {
        ty_args
            .iter()
            .map(|ty_arg| format!("{} diemtypes.TypeTag", ty_arg.name()))
            .collect()
    }

    fn quote_parameters(args: &[ArgumentABI]) -> Vec<String> {
        args.iter()
            .map(|arg| format!("{} {}", arg.name(), Self::quote_type(arg.type_tag())))
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
            Bool => "bool".into(),
            U8 => "uint8".into(),
            U64 => "uint64".into(),
            U128 => "serde.Uint128".into(),
            Address => "diemtypes.AccountAddress".into(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => "[]byte".into(),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => format!("(*diemtypes.TransactionArgument__Bool)(&{})", name),
            U8 => format!("(*diemtypes.TransactionArgument__U8)(&{})", name),
            U64 => format!("(*diemtypes.TransactionArgument__U64)(&{})", name),
            U128 => format!("(*diemtypes.TransactionArgument__U128)(&{})", name),
            Address => format!("&diemtypes.TransactionArgument__Address{{{}}}", name),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => format!("(*diemtypes.TransactionArgument__U8Vector)(&{})", name),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }
}

pub struct Installer {
    install_dir: PathBuf,
    serde_module_path: Option<String>,
    diem_module_path: Option<String>,
}

impl Installer {
    pub fn new(
        install_dir: PathBuf,
        serde_module_path: Option<String>,
        diem_module_path: Option<String>,
    ) -> Self {
        Installer {
            install_dir,
            serde_module_path,
            diem_module_path,
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
        let mut file = std::fs::File::create(dir_path.join("lib.go"))?;
        output(
            &mut file,
            self.serde_module_path.clone(),
            self.diem_module_path.clone(),
            name.to_string(),
            abis,
        )?;
        Ok(())
    }
}
