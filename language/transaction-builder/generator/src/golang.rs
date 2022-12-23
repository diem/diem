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
    emitter.output_transaction_script_decode_method()?;
    emitter.output_script_function_decode_method()?;

    for abi in abis {
        match abi {
            ScriptABI::TransactionScript(abi) => {
                emitter.output_transaction_script_encoder_function(abi)?
            }
            ScriptABI::ScriptFunction(abi) => {
                emitter.output_script_function_encoder_function(abi)?
            }
        };
    }
    for abi in abis {
        match abi {
            ScriptABI::TransactionScript(abi) => {
                emitter.output_transaction_script_decoder_function(abi)?
            }
            ScriptABI::ScriptFunction(abi) => {
                emitter.output_script_function_decoder_function(abi)?
            }
        };
    }

    for abi in abis {
        emitter.output_code_constant(abi)?;
    }
    emitter.output_transaction_script_decoder_map(&common::transaction_script_abis(abis))?;
    emitter.output_script_function_decoder_map(&common::script_function_abis(abis))?;

    emitter.output_encoding_helpers(abis)?;
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
        // We need BCS for argument encoding and decoding
        external_definitions.insert(
            "github.com/diem/serde-reflection-diem/serde-generate/runtime/golang/bcs".to_string(),
            Vec::new(),
        );
        // Add standard imports
        external_definitions.insert("fmt".to_string(), Vec::new());

        let (transaction_script_abis, script_fun_abis): (Vec<_>, Vec<_>) = abis
            .iter()
            .cloned()
            .partition(|abi| abi.is_transaction_script_abi());

        // Generate `ScriptCall` enums for all old-style transaction scripts
        let mut script_registry: BTreeMap<_, _> = vec![(
            "ScriptCall".to_string(),
            crate::common::make_abi_enum_container(transaction_script_abis.as_slice()),
        )]
        .into_iter()
        .collect();

        // Generate `ScriptFunctionCall` enums for all new transaction scripts
        let mut script_function_registry: BTreeMap<_, _> = vec![(
            "ScriptFunctionCall".to_string(),
            crate::common::make_abi_enum_container(script_fun_abis.as_slice()),
        )]
        .into_iter()
        .collect();

        script_registry.append(&mut script_function_registry);

        let mut comments: BTreeMap<_, _> = abis
            .iter()
            .map(|abi| {
                (
                    vec![
                        self.package_name.to_string(),
                        if abi.is_transaction_script_abi() {
                            "ScriptCall".to_string()
                        } else {
                            "ScriptFunctionCall".to_string()
                        },
                        abi.name().to_camel_case(),
                    ],
                    crate::common::prepare_doc_string(abi.doc()),
                )
            })
            .collect();
        comments.insert(
            vec![self.package_name.to_string(), "ScriptCall".to_string()],
            "Structured representation of a call into a known Move transaction script (legacy)."
                .into(),
        );
        comments.insert(
            vec![
                self.package_name.to_string(),
                "ScriptFunctionCall".to_string(),
            ],
            "Structured representation of a call into a known Move script function.".into(),
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
        let (transaction_script_abis, script_fun_abis): (Vec<_>, Vec<_>) = abis
            .iter()
            .cloned()
            .partition(|abi| abi.is_transaction_script_abi());

        if !transaction_script_abis.is_empty() {
            writeln!(
                self.out,
                r#"
// Build a Diem `Script` from a structured object `ScriptCall`.
func EncodeScript(call ScriptCall) diemtypes.Script {{"#
            )?;
            self.out.indent();
            writeln!(self.out, "switch call := call.(type) {{")?;
            for abi in transaction_script_abis {
                if let ScriptABI::TransactionScript(abi) = abi {
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
            }
            writeln!(self.out, "}}")?;
            writeln!(self.out, "panic(\"unreachable\")")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
        }

        if !script_fun_abis.is_empty() {
            writeln!(
                self.out,
                r#"
// Build a Diem `TransactionPayload` from a structured object `ScriptFunctionCall`.
func EncodeScriptFunction(call ScriptFunctionCall) diemtypes.TransactionPayload {{"#
            )?;
            self.out.indent();
            writeln!(self.out, "switch call := call.(type) {{")?;
            for abi in script_fun_abis {
                if let ScriptABI::ScriptFunction(abi) = abi {
                    let params = std::iter::empty()
                        .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
                        .chain(abi.args().iter().map(ArgumentABI::name))
                        .map(|name| format!("call.{}", name.to_camel_case()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    writeln!(
                        self.out,
                        r#"case *ScriptFunctionCall__{0}:
                return Encode{0}ScriptFunction({1})"#,
                        abi.name().to_camel_case(),
                        params,
                    )?;
                }
            }
            writeln!(self.out, "}}")?;
            writeln!(self.out, "panic(\"unreachable\")")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
        }
        Ok(())
    }

    fn output_transaction_script_decode_method(&mut self) -> Result<()> {
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

    fn output_script_function_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
// Try to recognize a Diem `TransactionPayload` and convert it into a structured object `ScriptFunctionCall`.
func DecodeScriptFunctionPayload(script diemtypes.TransactionPayload) (ScriptFunctionCall, error) {{
    switch script := script.(type) {{
        case *diemtypes.TransactionPayload__ScriptFunction:
            if helper := script_function_decoder_map[string(script.Value.Module.Name) + string(script.Value.Function)]; helper != nil {{
                    val, err := helper(script)
                    return val, err
            }} else {{
                    return nil, fmt.Errorf("Unknown script function: %s::%s", script.Value.Module.Name, script.Value.Function)
            }}
        default:
                return nil, fmt.Errorf("Unknown transaction payload encountered when decoding")
    }}
}}"#
        )
    }

    fn output_transaction_script_encoder_function(
        &mut self,
        abi: &TransactionScriptABI,
    ) -> Result<()> {
        writeln!(
            self.out,
            "\n{}\nfunc Encode{}Script({}) diemtypes.Script {{",
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
            Self::quote_arguments_for_script(abi.args()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_function_encoder_function(&mut self, abi: &ScriptFunctionABI) -> Result<()> {
        writeln!(
            self.out,
            "\n{}\nfunc Encode{}ScriptFunction({}) diemtypes.TransactionPayload {{",
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
            r#"return &diemtypes.TransactionPayload__ScriptFunction {{
            diemtypes.ScriptFunction {{
                Module: {},
                Function: {},
                TyArgs: []diemtypes.TypeTag{{{}}},
                Args: [][]byte{{{}}},
    }},
}}"#,
            Self::quote_module_id(abi.module_name()),
            Self::quote_identifier(abi.name()),
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_transaction_script_decoder_function(
        &mut self,
        abi: &TransactionScriptABI,
    ) -> Result<()> {
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

    fn output_script_function_decoder_function(&mut self, abi: &ScriptFunctionABI) -> Result<()> {
        writeln!(
            self.out,
            "\nfunc decode_{}_script_function(script diemtypes.TransactionPayload) (ScriptFunctionCall, error) {{",
            abi.name(),
        )?;
        self.out.indent();
        writeln!(self.out, "switch script := interface{{}}(script).(type) {{")?;
        self.out.indent();
        writeln!(
            self.out,
            "case *diemtypes.TransactionPayload__ScriptFunction:"
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "if len(script.Value.TyArgs) < {0} {{ return nil, fmt.Errorf(\"Was expecting {0} type arguments\") }}",
            abi.ty_args().len(),
        )?;
        writeln!(
            self.out,
            "if len(script.Value.Args) < {0} {{ return nil, fmt.Errorf(\"Was expecting {0} regular arguments\") }}",
            abi.args().len(),
        )?;
        writeln!(
            self.out,
            "var call ScriptFunctionCall__{0}",
            abi.name().to_camel_case(),
        )?;
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "call.{} = script.Value.TyArgs[{}]",
                ty_arg.name().to_camel_case(),
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            let decoding = match Self::bcs_primitive_type_name(arg.type_tag()) {
                None => {
                    let quoted_type = Self::quote_type(arg.type_tag());
                    let splits: Vec<_> = quoted_type.rsplitn(2, '.').collect();
                    format!(
                        "{}.BcsDeserialize{}(script.Value.Args[{}])",
                        splits[1], splits[0], index
                    )
                }
                Some(type_name) => format!(
                    "bcs.NewDeserializer(script.Value.Args[{}]).Deserialize{}()",
                    index, type_name
                ),
            };
            writeln!(
                self.out,
                r#"
if val, err := {}; err == nil {{
	call.{} = val
}} else {{
	return nil, err
}}
"#,
                decoding,
                arg.name().to_camel_case(),
            )?;
        }
        writeln!(self.out, "return &call, nil")?;
        self.out.unindent();
        writeln!(
            self.out,
            r#"default:
    return nil, fmt.Errorf("Unexpected TransactionPayload encountered when decoding a script function")"#
        )?;

        self.out.unindent();
        writeln!(self.out, "}}")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_transaction_script_decoder_map(
        &mut self,
        abis: &[TransactionScriptABI],
    ) -> Result<()> {
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

    fn output_script_function_decoder_map(&mut self, abis: &[ScriptFunctionABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
var script_function_decoder_map = map[string]func(diemtypes.TransactionPayload) (ScriptFunctionCall, error) {{"#
        )?;
        self.out.indent();
        for abi in abis {
            writeln!(
                self.out,
                "\"{}{}\": decode_{1}_script_function,",
                abi.module_name().name(),
                abi.name(),
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_encoding_helpers(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let required_types = common::get_required_helper_types(abis);
        for required_type in required_types {
            self.output_encoding_helper(required_type)?;
        }
        Ok(())
    }

    fn output_encoding_helper(&mut self, type_tag: &TypeTag) -> Result<()> {
        let encoding = match Self::bcs_primitive_type_name(type_tag) {
            None => r#"
    if val, err := arg.BcsSerialize(); err == nil {{
        return val;
    }}
    "#
            .into(),
            Some(type_name) => {
                format!(
                    r#"
    s := bcs.NewSerializer();
    if err := s.Serialize{}(arg); err == nil {{
        return s.GetBytes();
    }}
    "#,
                    type_name
                )
            }
        };
        writeln!(
            self.out,
            r#"
func encode_{}_argument(arg {}) []byte {{
    {}
    panic("Unable to serialize argument of type {}");
}}
"#,
            common::mangle_type(type_tag),
            Self::quote_type(type_tag),
            encoding,
            common::mangle_type(type_tag)
        )
    }

    fn output_decoding_helpers(&mut self, abis: &[ScriptABI]) -> Result<()> {
        let required_types = common::get_required_helper_types(abis);
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
        if let ScriptABI::TransactionScript(abi) = abi {
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
        }
        Ok(())
    }

    fn quote_identifier(ident: &str) -> String {
        format!("\"{}\"", ident)
    }

    fn quote_address(address: &AccountAddress) -> String {
        format!(
            "[16]uint8{{ {} }}",
            address
                .to_vec()
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn quote_module_id(module_id: &ModuleId) -> String {
        format!(
            "diemtypes.ModuleId {{ Address: {}, Name: {} }}",
            Self::quote_address(module_id.address()),
            Self::quote_identifier(module_id.name().as_str()),
        )
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
        format!(
            "encode_{}_argument({})",
            common::mangle_type(type_tag),
            name
        )
    }

    fn quote_transaction_argument_for_script(type_tag: &TypeTag, name: &str) -> String {
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

    // - if a `type_tag` is a primitive type in BCS, we can call
    //   `NewSerializer().Serialize<name>(arg)` and `NewDeserializer().Deserialize<name>(arg)`
    //   to convert into and from `[]byte`.
    // - otherwise, we can use `<arg>.BcsSerialize()`, `<arg>.BcsDeserialize()` to do the work.
    fn bcs_primitive_type_name(type_tag: &TypeTag) -> Option<&'static str> {
        use TypeTag::*;
        match type_tag {
            Bool => Some("Bool"),
            U8 => Some("U8"),
            U64 => Some("U64"),
            U128 => Some("U128"),
            Address => None,
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => Some("Bytes"),
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
