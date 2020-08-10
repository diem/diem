// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::{make_abi_enum_container, mangle_type, type_not_allowed};
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde_generate::{
    indent::{IndentConfig, IndentedWriter},
    java, CodeGeneratorConfig,
};

use heck::{CamelCase, ShoutySnakeCase};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

/// Output transaction builders and decoders in Java for the given ABIs.
/// Source files will be located in a subdirectory of `install_dir` corresponding to the given
/// package name (if any, otherwise `install_dir` it self).
pub fn write_source_files(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    write_script_call_files(install_dir.clone(), package_name, abis)?;
    write_helper_file(install_dir, package_name, abis)
}

/// Output transaction helper functions for the given ABIs.
fn write_helper_file(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    let mut dir_path = install_dir;
    let parts = package_name.split('.').collect::<Vec<_>>();
    for part in &parts {
        dir_path = dir_path.join(part);
    }
    std::fs::create_dir_all(&dir_path)?;

    let mut file = std::fs::File::create(dir_path.join("Helpers.java"))?;
    let mut emitter = JavaEmitter {
        out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
        package_name,
    };
    emitter.output_preamble()?;
    writeln!(emitter.out, "\npublic final class Helpers {{")?;
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

    emitter.output_decoding_helpers()?;

    emitter.out.unindent();
    writeln!(emitter.out, "\n}}\n")
}

fn write_script_call_files(
    install_dir: std::path::PathBuf,
    package_name: &str,
    abis: &[ScriptABI],
) -> Result<()> {
    let external_definitions = crate::common::get_external_definitions("org.libra.types");
    let script_registry: BTreeMap<_, _> =
        vec![("ScriptCall".to_string(), make_abi_enum_container(abis))]
            .into_iter()
            .collect();
    let mut comments: BTreeMap<_, _> = abis
        .iter()
        .map(|abi| {
            let mut paths = package_name
                .split('.')
                .map(String::from)
                .collect::<Vec<_>>();
            paths.push("ScriptCall".to_string());
            paths.push(abi.name().to_camel_case());
            (paths, crate::common::prepare_doc_string(abi.doc()))
        })
        .collect();
    comments.insert(
        vec!["ScriptCall".to_string()],
        "Structured representation of a call into a known Move script.".into(),
    );

    let config = CodeGeneratorConfig::new(package_name.to_string())
        .with_comments(comments)
        .with_external_definitions(external_definitions)
        .with_serialization(false);
    java::CodeGenerator::new(&config)
        .write_source_files(install_dir, &script_registry)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
    Ok(())
}

/// Shared state for the Java code generator.
struct JavaEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Name of the package owning the generated definitions (e.g. "com.facebook.my_package")
    package_name: &'a str,
}

impl<'a, T> JavaEmitter<'a, T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(self.out, "package {};\n", self.package_name)?;
        writeln!(
            self.out,
            r#"
import java.math.BigInteger;
import java.lang.IllegalArgumentException;
import java.lang.IndexOutOfBoundsException;
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

    fn output_encode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/**
 * Build a Libra `Script` from a structured object `ScriptCall`.
 */
public static Script encode_script(ScriptCall call) {{
    EncodingHelper helper = SCRIPT_ENCODER_MAP.get(call.getClass());
    return helper.encode(call);
}}"#
        )
    }

    fn output_decode_method(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
/**
 * Try to recognize a Libra `Script` and convert it into a structured object `ScriptCall`.
 */
public static ScriptCall decode_script(Script script) throws IllegalArgumentException, IndexOutOfBoundsException {{
    DecodingHelper helper = SCRIPT_DECODER_MAP.get(script.code);
    if (helper == null) {{
        throw new IllegalArgumentException("Unknown script bytecode");
    }}
    return helper.decode(script);
}}
"#
        )
    }

    fn output_script_encoder_function(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\n{}public static Script encode_{}_script({}) {{",
            Self::quote_doc(abi.doc()),
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
            r#"Script.Builder builder = new Script.Builder();
builder.code = new Bytes({}_CODE);
builder.ty_args = java.util.Arrays.asList({});
builder.args = java.util.Arrays.asList({});
return builder.build();"#,
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
            "\nprivate static ScriptCall decode_{}_script(Script {}script) throws IllegalArgumentException, IndexOutOfBoundsException {{",
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
            "ScriptCall.{0}.Builder builder = new ScriptCall.{0}.Builder();",
            abi.name().to_camel_case(),
        )?;
        for (index, ty_arg) in abi.ty_args().iter().enumerate() {
            writeln!(
                self.out,
                "builder.{} = script.ty_args.get({});",
                ty_arg.name(),
                index,
            )?;
        }
        for (index, arg) in abi.args().iter().enumerate() {
            writeln!(
                self.out,
                "builder.{} = Helpers.decode_{}_argument(script.args.get({}));",
                arg.name(),
                mangle_type(arg.type_tag()),
                index,
            )?;
        }
        writeln!(self.out, "return builder.build();")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_encoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
interface EncodingHelper {{
    public Script encode(ScriptCall call);
}}

private static final java.util.Map<Class<?>, EncodingHelper> SCRIPT_ENCODER_MAP = initEncoderMap();

private static java.util.Map<Class<?>, EncodingHelper> initEncoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "java.util.HashMap<Class<?>, EncodingHelper> map = new java.util.HashMap<>();"
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
                "map.put(ScriptCall.{0}.class, (EncodingHelper)((call) -> {{
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

    fn output_decoder_map(&mut self, abis: &[ScriptABI]) -> Result<()> {
        writeln!(
            self.out,
            r#"
interface DecodingHelper {{
    public ScriptCall decode(Script script);
}}

private static final java.util.Map<Bytes, DecodingHelper> SCRIPT_DECODER_MAP = initDecoderMap();

private static java.util.Map<Bytes, DecodingHelper> initDecoderMap() {{"#
        )?;
        self.out.indent();
        writeln!(
            self.out,
            "java.util.HashMap<Bytes, DecodingHelper> map = new java.util.HashMap<>();"
        )?;
        for abi in abis {
            writeln!(
                self.out,
                "map.put(new Bytes({}_CODE), (DecodingHelper)((script) -> Helpers.decode_{}_script(script)));",
                abi.name().to_shouty_snake_case(),
                abi.name()
            )?;
        }
        writeln!(self.out, "return map;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_decoding_helpers(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"
private static Boolean decode_bool_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.Bool)) {{
        throw new IllegalArgumentException("Was expecting a Bool argument");
    }}
    return ((TransactionArgument.Bool) arg).value;
}}

private static @com.facebook.serde.Unsigned Byte decode_u8_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.U8)) {{
        throw new IllegalArgumentException("Was expecting a U8 argument");
    }}
    return ((TransactionArgument.U8) arg).value;
}}

private static @com.facebook.serde.Unsigned Long decode_u64_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.U64)) {{
        throw new IllegalArgumentException("Was expecting a U64 argument");
    }}
    return ((TransactionArgument.U64) arg).value;
}}

private static @com.facebook.serde.Unsigned @com.facebook.serde.Int128 BigInteger decode_u128_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.U128)) {{
        throw new IllegalArgumentException("Was expecting a U128 argument");
    }}
    return ((TransactionArgument.U128) arg).value;
}}

private static AccountAddress decode_address_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.Address)) {{
        throw new IllegalArgumentException("Was expecting an Address argument");
    }}
    return ((TransactionArgument.Address) arg).value;
}}

private static Bytes decode_u8vector_argument(TransactionArgument arg) {{
    if (!(arg instanceof TransactionArgument.U8Vector)) {{
        throw new IllegalArgumentException("Was expecting a U8Vector argument");
    }}
    return ((TransactionArgument.U8Vector) arg).value;
}}
"#
        )
    }

    fn output_code_constant(&mut self, abi: &ScriptABI) -> Result<()> {
        writeln!(
            self.out,
            "\npublic static byte[] {}_CODE = {{{}}};",
            abi.name().to_shouty_snake_case(),
            abi.code()
                .iter()
                .map(|x| format!("{}", *x as i8))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        Ok(())
    }

    fn quote_doc(doc: &str) -> String {
        let doc = crate::common::prepare_doc_string(doc);
        let text = textwrap::indent(&doc, " * ").replace("\n\n", "\n *\n");
        format!("/**\n{} */\n", text)
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
                _ => type_not_allowed(type_tag),
            },

            Struct(_) | Signer => type_not_allowed(type_tag),
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
        package_name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error> {
        write_source_files(self.install_dir.clone(), package_name, abis)?;
        Ok(())
    }
}
