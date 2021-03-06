// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common;
use diem_types::transaction::{ArgumentABI, ScriptABI, TransactionScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    csharp,
    indent::{IndentConfig, IndentedWriter},
    CodeGeneratorConfig,
};

use heck::{CamelCase, ShoutySnakeCase};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders and decoders in C# for the given ABIs.
/// Source files will be located in a subdirectory of `install_dir` corresponding to the given
/// namespace name (if any, otherwise `install_dir` it self).
pub fn write_source_files(
    install_dir: std::path::PathBuf,
    namespace_name: &str,
    abis: &[TransactionScriptABI],
) -> Result<()> {
    write_script_call_files(install_dir.clone(), namespace_name, abis)?;
    write_helper_file(install_dir, namespace_name, abis)
}

/// Output transaction helper functions for the given ABIs.
fn write_helper_file(
    install_dir: std::path::PathBuf,
    namespace_name: &str,
    abis: &[TransactionScriptABI],
) -> Result<()> {
    let mut dir_path = install_dir;
    let parts = namespace_name.split('.').collect::<Vec<_>>();
    for part in &parts {
        dir_path = dir_path.join(part);
    }
    std::fs::create_dir_all(&dir_path)?;

    let mut file = std::fs::File::create(dir_path.join("Helpers.cs"))?;
    let mut emitter = CsharpEmitter {
        out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
        namespace_name,
    };
    emitter.output_preamble()?;
    writeln!(emitter.out, "\npublic static class Helpers {{")?;
    emitter.out.indent();

    emitter.output_encode_method()?;
    emitter.output_decode_method()?;

    for abi in abis {
        emitter.output_script_encoder_function(abi)?;
    }
    for abi in abis {
        emitter.output_script_decoder_function(abi)?;
    }

    emitter.output_encoder_map(abis)?;
    for abi in abis {
        emitter.output_code_constant(abi)?;
    }
    // Must be defined after the constants.
    emitter.output_decoder_map(abis)?;

    emitter.output_decoding_helpers(abis)?;

    emitter.out.unindent();
    writeln!(emitter.out, "\n}}\n")?; // class
    emitter.out.unindent();
    writeln!(emitter.out, "\n}}\n") // namespace
}

fn write_script_call_files(
    install_dir: std::path::PathBuf,
    namespace_name: &str,
    abis: &[TransactionScriptABI],
) -> Result<()> {
    let external_definitions = crate::common::get_external_definitions("Diem.Types");
    let script_registry: BTreeMap<_, _> = vec![(
        "ScriptCall".to_string(),
        common::make_abi_enum_container(abis),
    )]
    .into_iter()
    .collect();
    let mut comments: BTreeMap<_, _> = abis
        .iter()
        .map(|abi| {
            let mut paths = namespace_name
                .split('.')
                .map(String::from)
                .collect::<Vec<_>>();
            paths.push("ScriptCall".to_string());
            paths.push(abi.name().to_camel_case());
            (paths, prepare_doc_string(abi.doc()))
        })
        .collect();
    comments.insert(
        vec!["ScriptCall".to_string()],
        "Structured representation of a call into a known Move script.".into(),
    );

    let config = CodeGeneratorConfig::new(namespace_name.to_string())
        .with_comments(comments)
        .with_external_definitions(external_definitions)
        .with_serialization(false);
    csharp::CodeGenerator::new(&config)
        .write_source_files(install_dir, &script_registry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
    Ok(())
}

/// Add minimal escaping for XML Comments.
fn prepare_doc_string(doc: &str) -> String {
    let doc = crate::common::prepare_doc_string(doc);
    // Escape quoted strings and special characters.
    let doc = regex::Regex::new("`([^`]*)`|(>=)|(<=)|(>)|(<)")
        .unwrap()
        .replace_all(&doc, "<code>$1$2$3$4$5</code>");
    // Replace subsection titles.
    let doc = regex::Regex::new("##* (.*)\n")
        .unwrap()
        .replace_all(&doc, "<p><b>$1</b></p>\n");
    // Simulate lists.
    let doc = regex::Regex::new("\n\\* (.*)")
        .unwrap()
        .replace_all(&doc, "\n<ul><li>$1</li></ul>");
    doc.to_string()
}

/// Shared state for the Java code generator.
struct CsharpEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Name of the namespace owning the generated definitions (e.g. "MyOrg.MyTechnology")
    namespace_name: &'a str,
}

impl<'a, T> CsharpEmitter<'a, T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
using System; // For ArgumentException and IndexOutOfRangeException
using System.Numerics; // For BigInteger
using Diem.Types; // For Script, TransactionArgument, TypeTag
using Serde; // For ValueArray (e.g., ValueArray<byte>)
using System.Collections;
using System.Collections.Generic; // For List, Dictonary
"#,
        )?;
        writeln!(self.out, "namespace {}\n{{", self.namespace_name)?;
        Ok(())
    }

    fn output_encode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/// <summary>
/// Build a Diem Script from a structured value {{@link ScriptCall}}.
/// </summary>
/// <returns>
/// Encoded Script
/// </returns>
/// <param name="call">ScriptCall value to encode.</param>
public static Script EncodeScript(ScriptCall call) {{
    EncodingHelper helper = SCRIPT_ENCODER_MAP[call.GetType()];
    return helper(call);
}}"#
        )
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/// <summary>
/// Try to recognize a Diem Script and convert it into a structured value Script.
/// </summary>
/// <returns>
/// Decoded ScriptCall value
/// </returns>
/// <param name="script">Script values to decode</param>
public static ScriptCall DecodeScript(Script script) {{
    DecodingHelper helper = SCRIPT_DECODER_MAP[script.code];
    if (helper == null) {{
        throw new ArgumentException("Unknown script bytecode");
    }}
    return helper(script);
}}
"#
        )
    }

    fn output_script_encoder_function(&mut self, abi: &TransactionScriptABI) -> Result<()> {
        let quoted_type_params: Vec<_> = abi
            .ty_args()
            .iter()
            .map(|ty_arg| format!("TypeTag {}", ty_arg.name()))
            .collect();
        let quoted_type_params_doc: Vec<_> = abi
            .ty_args()
            .iter()
            .map(|ty_arg| format!("{} <code>TypeTag</code> value", ty_arg.name()))
            .collect();
        let quoted_params: Vec<_> = abi
            .args()
            .iter()
            .map(|arg| format!("{} {}", Self::quote_type(arg.type_tag()), arg.name()))
            .collect();
        let quoted_params_doc: Vec<_> = abi
            .args()
            .iter()
            .map(|arg| {
                format!(
                    "{} <code>{}</code> value",
                    arg.name(),
                    Self::quote_type(arg.type_tag())
                )
            })
            .collect();

        writeln!(
            self.out,
            "\n{}public static Script encode_{}_script({}) {{",
            Self::quote_doc(
                abi.doc(),
                [quoted_type_params_doc, quoted_params_doc].concat(),
                "Encoded Diem.Types.Script value.",
            ),
            abi.name(),
            [quoted_type_params, quoted_params].concat().join(", ")
        )?;
        self.out.indent();
        writeln!(
            self.out,
            r#"TypeTag[] tt = new TypeTag[] {{{}}};
TransactionArgument[] ta = new TransactionArgument[] {{{}}};
return new Script(
  new ValueArray<byte>((byte[]) (Array) {}_CODE),
  new ValueArray<TypeTag>(tt),
  new ValueArray<TransactionArgument>(ta)
);"#,
            Self::quote_type_arguments(abi.ty_args()),
            Self::quote_arguments(abi.args()),
            abi.name().to_shouty_snake_case(),
        )?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_script_decoder_function(&mut self, abi: &TransactionScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nprivate static ScriptCall decode_{}_script(Script {}script) {{",
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
            "return new ScriptCall.{0}(",
            abi.name().to_camel_case(),
        )?;
        let mut params = String::from("");
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            params.push_str(&format!(
                "    // {}\n    script.ty_args[{}],\n",
                ty_arg.name(),
                index,
            ));
        }
        for (index, arg) in abi.args().iter().enumerate() {
            params.push_str(&format!(
                "    Helpers.decode_{}_argument(script.args[{}]),\n",
                common::mangle_type(arg.type_tag()),
                index,
            ));
        }
        params.pop(); // removes last newline
        params.pop(); // removes last trailing comma
        writeln!(self.out, "{}", params)?;
        writeln!(self.out, ");")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_encoder_map(&mut self, abis: &[TransactionScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
delegate Script EncodingHelper(ScriptCall call);

private static readonly System.Collections.Generic.Dictionary<Type, EncodingHelper> SCRIPT_ENCODER_MAP = initEncoderMap();

private static System.Collections.Generic.Dictionary<Type, EncodingHelper> initEncoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "System.Collections.Generic.Dictionary<Type, EncodingHelper> map = new System.Collections.Generic.Dictionary<Type, EncodingHelper>();"
        )?;
        for abi in abis {
            let params = std::iter::empty()
                .chain(abi.ty_args().iter().map(TypeArgumentABI::name))
                .chain(abi.args().iter().map(ArgumentABI::name))
                .map(|name| format!("obj.{}", name))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                self.out,
                "map.Add(typeof(ScriptCall.{0}), (EncodingHelper)((call) => {{
    ScriptCall.{0} obj = (ScriptCall.{0})call;
    return Helpers.encode_{1}_script({2});
}}));",
                abi.name().to_camel_case(),
                abi.name(),
                params,
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_decoder_map(&mut self, abis: &[TransactionScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
delegate ScriptCall DecodingHelper(Script script);

private static readonly System.Collections.Generic.Dictionary<ValueArray<byte>, DecodingHelper> SCRIPT_DECODER_MAP = initDecoderMap();

private static System.Collections.Generic.Dictionary<ValueArray<byte>, DecodingHelper> initDecoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "System.Collections.Generic.Dictionary<ValueArray<byte>, DecodingHelper> map = new System.Collections.Generic.Dictionary<ValueArray<byte>, DecodingHelper>();"
        )?;
        for abi in abis {
            writeln!(
                self.out,
                "map.Add(new ValueArray<byte>((byte[]) (Array){}_CODE), (DecodingHelper)((script) => Helpers.decode_{}_script(script)));",
                abi.name().to_shouty_snake_case(),
                abi.name()
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_decoding_helpers(&mut self, abis: &[TransactionScriptABI]) -> Result<()> {
        let required_types = common::get_required_decoding_helper_types(abis);
        for required_type in required_types {
            self.output_decoding_helper(required_type)?;
        }
        Ok(())
    }

    fn output_decoding_helper(&mut self, type_tag: &TypeTag) -> Result<()> {
        use TypeTag::*;
        let (constructor, expr) = match type_tag {
            Bool => ("Bool", String::new()),
            U8 => ("U8", String::new()),
            U64 => ("U64", String::new()),
            U128 => ("U128", String::new()),
            Address => ("Address", String::new()),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => ("U8Vector", String::new()),
                _ => common::type_not_allowed(type_tag),
            },
            Struct(_) | Signer => common::type_not_allowed(type_tag),
        };
        writeln!(
            self.out,
            r#"
private static {} decode_{}_argument(TransactionArgument arg) {{
    if (!(arg is TransactionArgument.{})) {{
        throw new ArgumentException("Was expecting a {} argument");
    }}
    return ((TransactionArgument.{}) arg).value{};
}}
"#,
            Self::quote_type(type_tag),
            common::mangle_type(type_tag),
            constructor,
            constructor,
            constructor,
            expr,
        )
    }

    fn output_code_constant(&mut self, abi: &TransactionScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\nprivate static sbyte[] {}_CODE = {{{}}};",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("{}", *x as i8))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }

    fn quote_doc<I>(doc: &str, params_doc: I, return_doc: &str) -> String
    where
        I: IntoIterator<Item = String>,
    {
        let mut doc = prepare_doc_string(doc) + "\n";
        for text in params_doc {
            doc = format!("{}\n<param>{}</param>", doc, text);
        }
        doc = format!("{}\n<return> {}", doc, return_doc);
        let text = textwrap::indent(&doc, " * ").replace("\n\n", "\n *\n");
        format!("/**\n{} */\n", text)
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
            U8 => "byte".into(),
            U64 => "ulong".into(),
            U128 => "BigInteger".into(),
            Address => "AccountAddress".into(),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => "ValueArray<byte>".into(),
                _ => common::type_not_allowed(type_tag),
            },

            Struct(_) | Signer => common::type_not_allowed(type_tag),
        }
    }

    fn quote_transaction_argument(type_tag: &TypeTag, name: &str) -> String {
        use TypeTag::*;
        match type_tag {
            Bool => format!("new TransactionArgument.Bool({})", name),
            U8 => format!("new TransactionArgument.U8({})", name),
            U64 => format!("new TransactionArgument.U64({})", name),
            U128 => format!("new TransactionArgument.U128({})", name),
            Address => format!("new TransactionArgument.Address({})", name),
            Vector(type_tag) => match type_tag.as_ref() {
                U8 => format!("new TransactionArgument.U8Vector({})", name),
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
        namespace_name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        // TODO(#7876): Update to handle script function ABIs
        let abis = abis
            .iter()
            .cloned()
            .filter_map(|abi| match abi {
                ScriptABI::TransactionScript(abi) => Some(abi),
                ScriptABI::ScriptFunction(_) => None,
            })
            .collect::<Vec<_>>();
        write_source_files(self.install_dir.clone(), namespace_name, &abis)?;
        Ok(())
    }
}
