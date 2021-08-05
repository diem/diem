// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::AddressBytes;
use anyhow::{anyhow, Result};
use move_binary_format::{
    access::ModuleAccess,
    file_format::{
        Ability, AbilitySet, CompiledModule, FunctionDefinition, ModuleHandle, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandleIndex, StructTypeParameter,
        TypeParameterIndex, Visibility,
    },
};
use move_core_types::language_storage::ModuleId;
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
pub fn write_file_to_string(
    named_address_mapping: &BTreeMap<ModuleId, String>,
    compiled_module_file_input_path: &str,
) -> Result<(ModuleId, String)> {
    let file_contents = fs::read(compiled_module_file_input_path)?;
    let module = CompiledModule::deserialize(&file_contents).map_err(|e| {
        anyhow!(
            "Unable to deserialize module at '{}': {}",
            compiled_module_file_input_path,
            e
        )
    })?;
    write_module_to_string(named_address_mapping, &module)
}

pub fn write_module_to_string(
    named_address_mapping: &BTreeMap<ModuleId, String>,
    module: &CompiledModule,
) -> Result<(ModuleId, String)> {
    let mut out = String::new();

    let id = module.self_id();
    push_line!(
        out,
        format!("module {} {{", write_module_id(named_address_mapping, &id))
    );
    push_line!(out, "");

    let mut context = Context::new(module);
    let mut members = vec![];

    for fdecl in module.friend_decls() {
        members.push(write_friend_decl(&mut context, fdecl));
    }
    if !module.friend_decls().is_empty() {
        members.push("".to_string());
    }

    for sdef in module.struct_defs() {
        members.push(write_struct_def(&mut context, sdef));
    }
    if !module.struct_defs().is_empty() {
        members.push("".to_string());
    }

    let mut externally_visible_funs = module
        .function_defs()
        .iter()
        .filter(|fdef| match fdef.visibility {
            Visibility::Public | Visibility::Script | Visibility::Friend => true,
            Visibility::Private => false,
        })
        .peekable();
    let has_externally_visible_funs = externally_visible_funs.peek().is_some();
    if has_externally_visible_funs {
        members.push(format!("    {}", DISCLAIMER));
    }
    for fdef in externally_visible_funs {
        members.push(write_function_def(&mut context, fdef));
    }
    if has_externally_visible_funs {
        members.push("".to_string());
    }

    let has_uses = !context.uses.is_empty();
    for (module_id, alias) in context.uses {
        let use_ = if module_id.name().as_str() == alias {
            format!(
                "    use {};",
                write_module_id(named_address_mapping, &module_id),
            )
        } else {
            format!(
                "    use {} as {};",
                write_module_id(named_address_mapping, &module_id),
                alias
            )
        };
        push_line!(out, use_);
    }
    if has_uses {
        push_line!(out, "");
    }

    if !members.is_empty() {
        push_line!(out, members.join("\n"));
    }
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

    fn module_alias(&mut self, module_id: ModuleId) -> &String {
        let module_name = module_id.name().to_owned().into_string();
        let counts = &mut self.counts;
        self.uses.entry(module_id).or_insert_with(|| {
            let count = *counts
                .entry(module_name.clone())
                .and_modify(|c| *c += 1)
                .or_insert(0);
            if count == 0 {
                module_name
            } else {
                format!("{}_{}", module_name, count)
            }
        })
    }
}

const DISCLAIMER: &str =
    "// NOTE: Functions are 'native' for simplicity. They may or may not be native in actuality.";

fn write_module_id(named_address_mapping: &BTreeMap<ModuleId, String>, id: &ModuleId) -> String {
    format!(
        "{}::{}",
        named_address_mapping
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("{}", AddressBytes::new(id.address().to_u8()))),
        id.name(),
    )
}

fn write_friend_decl(ctx: &mut Context, fdecl: &ModuleHandle) -> String {
    format!(
        "    friend {};",
        ctx.module_alias(ctx.module.module_id_for_handle(fdecl))
    )
}

fn write_struct_def(ctx: &mut Context, sdef: &StructDefinition) -> String {
    let mut out = String::new();

    let shandle = ctx.module.struct_handle_at(sdef.struct_handle);

    push_line!(
        out,
        format!(
            "    struct {}{}{} {{",
            ctx.module.identifier_at(shandle.name),
            write_struct_type_parameters(&shandle.type_parameters),
            write_ability_modifiers(shandle.abilities),
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
        "    native {}fun {}{}({}){};",
        write_visibility(fdef.visibility),
        ctx.module.identifier_at(fhandle.name),
        write_fun_type_parameters(&fhandle.type_parameters),
        write_parameters(ctx, parameters),
        write_return_type(ctx, return_)
    )
}

fn write_visibility(visibility: Visibility) -> String {
    match visibility {
        Visibility::Public => "public ",
        Visibility::Script => "public(script) ",
        Visibility::Friend => "public(friend) ",
        Visibility::Private => "",
    }
    .to_string()
}

fn write_ability_modifiers(abs: AbilitySet) -> String {
    if abs == AbilitySet::EMPTY {
        return "".to_string();
    }
    format!(
        " has {}",
        abs.into_iter()
            .map(write_ability)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn write_ability_constraint(abs: AbilitySet) -> String {
    if abs == AbilitySet::EMPTY {
        return "".to_string();
    }
    format!(
        ": {}",
        abs.into_iter()
            .map(write_ability)
            .collect::<Vec<_>>()
            .join("+ ")
    )
}

fn write_ability(ab: Ability) -> String {
    use crate::parser::ast::Ability_ as A_;
    match ab {
        Ability::Copy => A_::COPY,
        Ability::Drop => A_::DROP,
        Ability::Store => A_::STORE,
        Ability::Key => A_::KEY,
    }
    .to_string()
}

fn write_struct_type_parameters(tps: &[StructTypeParameter]) -> String {
    if tps.is_empty() {
        return "".to_string();
    }

    let tp_and_constraints = tps
        .iter()
        .enumerate()
        .map(|(idx, ty_param)| {
            format!(
                "{}{}{}",
                if ty_param.is_phantom { "phantom " } else { "" },
                write_type_parameter(idx as TypeParameterIndex),
                write_ability_constraint(ty_param.constraints),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", tp_and_constraints)
}

fn write_fun_type_parameters(tps: &[AbilitySet]) -> String {
    if tps.is_empty() {
        return "".to_string();
    }

    let tp_and_constraints = tps
        .iter()
        .enumerate()
        .map(|(idx, abs)| {
            format!(
                "{}{}",
                write_type_parameter(idx as TypeParameterIndex),
                write_ability_constraint(*abs),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", tp_and_constraints)
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
    let module_alias = ctx.module_alias(struct_module_id).clone();

    format!(
        "{}::{}",
        module_alias,
        ctx.module.identifier_at(struct_handle.name)
    )
}

fn write_type_parameter(idx: TypeParameterIndex) -> String {
    format!("T{}", idx)
}
