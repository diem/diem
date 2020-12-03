// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::Address;
use anyhow::{anyhow, Result};
use move_core_types::language_storage::ModuleId;
use move_vm::{
    access::ModuleAccess,
    file_format::{
        CompiledModule, FunctionDefinition, Kind, SignatureToken, StructDefinition,
        StructFieldInformation, StructHandleIndex, TypeParameterIndex,
    },
};
use std::{collections::BTreeMap, fs};

macro_rules! push_line {
    ($s:ident, $e:expr) => {{
        $s = format!("{}{}\n", $s, $e);
    }};
}

macro_rules! push {
    ($s:ident, $e:expr) => {{
        $s = format!("{}{}", $s, $e);
    }};
}

/// Generate the text for the "interface" file of a compiled module. This "interface" is the
/// publically visible contents of the CompiledModule, represented in source language syntax
/// Additionally, it returns the module id (address+name) of the module that was deserialized
pub fn write_to_string(compiled_module_file_input_path: &str) -> Result<(ModuleId, String)> {
    let mut out = String::new();

    let file_contents = fs::read(compiled_module_file_input_path)?;
    let module = CompiledModule::deserialize(&file_contents).map_err(|e| {
        anyhow!(
            "Unable to deserialize module at '{}': {}",
            compiled_module_file_input_path,
            e
        )
    })?;

    let id = module.self_id();
    push_line!(
        out,
        format!("address {} {{", Address::new(id.address().to_u8()),)
    );
    push_line!(out, format!("module {} {{", id.name()));
    push_line!(out, "");

    let mut context = Context::new(&module);
    let mut members = vec![];
    for sdef in module.struct_defs() {
        members.push(write_struct_def(&mut context, sdef))
    }
    if !members.is_empty() {
        members.push("".to_string());
    }

    let mut public_funs = module
        .function_defs()
        .iter()
        .filter(|fdef| fdef.is_public)
        .peekable();
    if public_funs.peek().is_some() {
        members.push(format!("    {}", DISCLAIMER));
    }
    for public_fdef in public_funs {
        members.push(write_function_def(&mut context, public_fdef));
    }
    if !members.is_empty() {
        members.push("".to_string());
    }

    let has_uses = !context.uses.is_empty();
    for (module_id, alias) in context.uses {
        if module_id.name().as_str() == alias {
            push_line!(
                out,
                format!(
                    "    use {}::{};",
                    Address::new(module_id.address().to_u8()),
                    module_id.name()
                )
            );
        } else {
            push_line!(
                out,
                format!(
                    "    use {}::{} as {};",
                    Address::new(module_id.address().to_u8()),
                    module_id.name(),
                    alias
                )
            );
        }
    }
    if has_uses {
        push_line!(out, "");
    }

    if !members.is_empty() {
        push_line!(out, members.join("\n"));
    }
    push_line!(out, "}");
    push_line!(out, "}");
    Ok((id, out))
}

struct Context<'a> {
    module: &'a CompiledModule,
    uses: BTreeMap<ModuleId, String>,
    counts: BTreeMap<String, usize>,
}

impl<'a> Context<'a> {
    fn new(module: &'a CompiledModule) -> Self {
        Self {
            module,
            uses: BTreeMap::new(),
            counts: BTreeMap::new(),
        }
    }
}

const DISCLAIMER: &str =
    "// NOTE: Functions are 'native' for simplicity. They may or may not be native in actuality.";

fn write_struct_def(ctx: &mut Context, sdef: &StructDefinition) -> String {
    let mut out = String::new();

    let shandle = ctx.module.struct_handle_at(sdef.struct_handle);
    let resource_mod = if shandle.is_nominal_resource {
        "resource "
    } else {
        ""
    };

    push_line!(
        out,
        format!(
            "    {}struct {}{} {{",
            resource_mod,
            ctx.module.identifier_at(shandle.name),
            write_type_paramters(&shandle.type_parameters),
        )
    );

    let fields = match &sdef.field_information {
        StructFieldInformation::Native => {
            push!(out, "    }");
            return out;
        }
        StructFieldInformation::Declared(fields) => fields,
    };
    for field in fields {
        push_line!(
            out,
            format!(
                "        {}: {},",
                ctx.module.identifier_at(field.name),
                write_signature_token(ctx, &field.signature.0),
            )
        )
    }

    push!(out, "    }");
    out
}

fn write_function_def(ctx: &mut Context, fdef: &FunctionDefinition) -> String {
    let fhandle = ctx.module.function_handle_at(fdef.function);
    let parameters = &ctx.module.signature_at(fhandle.parameters).0;
    let return_ = &ctx.module.signature_at(fhandle.return_).0;
    format!(
        "    native public fun {}{}({}){};",
        ctx.module.identifier_at(fhandle.name),
        write_type_paramters(&fhandle.type_parameters),
        write_parameters(ctx, parameters),
        write_return_type(ctx, return_)
    )
}

fn write_type_paramters(tps: &[Kind]) -> String {
    if tps.is_empty() {
        return "".to_string();
    }

    let tp_and_constraints = tps
        .iter()
        .enumerate()
        .map(|(idx, kind)| {
            format!(
                "{}{}",
                write_type_parameter(idx as TypeParameterIndex),
                write_kind_contraint(kind)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", tp_and_constraints)
}

fn write_kind_contraint(kind: &Kind) -> String {
    match kind {
        Kind::All => "".to_string(),
        Kind::Resource => ": resource".to_string(),
        Kind::Copyable => ": copyable".to_string(),
    }
}

fn write_parameters(ctx: &mut Context, params: &[SignatureToken]) -> String {
    params
        .iter()
        .enumerate()
        .map(|(idx, ty)| format!("a{}: {}", idx, write_signature_token(ctx, ty)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_return_type(ctx: &mut Context, tys: &[SignatureToken]) -> String {
    match tys.len() {
        0 => "".to_string(),
        1 => format!(": {}", write_signature_token(ctx, &tys[0])),
        _ => format!(
            ": ({})",
            tys.iter()
                .map(|ty| write_signature_token(ctx, ty))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn write_signature_token(ctx: &mut Context, t: &SignatureToken) -> String {
    match t {
        SignatureToken::Bool => "bool".to_string(),
        SignatureToken::U8 => "u8".to_string(),
        SignatureToken::U64 => "u64".to_string(),
        SignatureToken::U128 => "u128".to_string(),
        SignatureToken::Address => "address".to_string(),
        SignatureToken::Signer => "signer".to_string(),
        SignatureToken::Vector(inner) => format!("vector<{}>", write_signature_token(ctx, inner)),
        SignatureToken::Struct(idx) => write_struct_handle_type(ctx, *idx),
        SignatureToken::StructInstantiation(idx, types) => {
            let n = write_struct_handle_type(ctx, *idx);
            let tys = types
                .iter()
                .map(|ty| write_signature_token(ctx, ty))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", n, tys)
        }
        SignatureToken::Reference(inner) => format!("&{}", write_signature_token(ctx, inner)),
        SignatureToken::MutableReference(inner) => {
            format!("&mut {}", write_signature_token(ctx, inner))
        }
        SignatureToken::TypeParameter(idx) => write_type_parameter(*idx),
    }
}

fn write_struct_handle_type(ctx: &mut Context, idx: StructHandleIndex) -> String {
    let struct_handle = ctx.module.struct_handle_at(idx);
    let struct_module_handle = ctx.module.module_handle_at(struct_handle.module);
    let struct_module_id = ctx.module.module_id_for_handle(struct_module_handle);
    let struct_module_name = struct_module_id.name().to_string();

    let counts = &mut ctx.counts;
    let module_alias = ctx.uses.entry(struct_module_id).or_insert_with(|| {
        let count = *counts
            .entry(struct_module_name.clone())
            .and_modify(|c| *c += 1)
            .or_insert(0);
        if count == 0 {
            struct_module_name
        } else {
            format!("{}_{}", struct_module_name, count)
        }
    });
    format!(
        "{}::{}",
        module_alias,
        ctx.module.identifier_at(struct_handle.name)
    )
}

fn write_type_parameter(idx: TypeParameterIndex) -> String {
    format!("T{}", idx)
}
