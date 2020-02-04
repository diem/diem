// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    context::{Context, MaterializedPools},
    errors::*,
};

use anyhow::{bail, format_err, Result};
use bytecode_source_map::source_map::{ModuleSourceMap, SourceMap};
use libra_types::{account_address::AccountAddress, identifier::Identifier};
use move_ir_types::ast::{self, *};
use std::{
    clone::Clone,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, VecDeque,
    },
};
use vm::{
    access::ModuleAccess,
    file_format::{
        self, Bytecode, CodeOffset, CodeUnit, CompiledModule, CompiledModuleMut, CompiledProgram,
        CompiledScript, CompiledScriptMut, FieldDefinition, FieldDefinitionIndex,
        FunctionDefinition, FunctionSignature, Kind, LocalsSignature, MemberCount, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandleIndex, TableIndex,
    },
};

macro_rules! record_src_loc {
    (local: $context:expr, $var:expr) => {{
        let source_name = (Identifier::from($var.name()), $var.span);
        $context
            .source_map
            .add_local_mapping($context.current_function_definition_index(), source_name)?;
    }};
    (field: $context:expr, $field:expr) => {{
        $context
            .source_map
            .add_struct_field_mapping($context.current_struct_definition_index(), $field.span)?;
    }};
    (function_type_formals: $context:expr, $var:expr) => {
        for (ty_var, _) in $var.iter() {
            let source_name = (Identifier::from(ty_var.name()), ty_var.span);
            $context.source_map.add_function_type_parameter_mapping(
                $context.current_function_definition_index(),
                source_name,
            )?;
        }
    };
    (function_decl: $context:expr, $location:expr, $function_index:expr) => {{
        $context.set_function_index($function_index as TableIndex);
        $context.source_map.add_top_level_function_mapping(
            $context.current_function_definition_index(),
            $location,
        )?;
    }};
    (struct_type_formals: $context:expr, $var:expr) => {
        for (ty_var, _) in $var.iter() {
            let source_name = (Identifier::from(ty_var.name()), ty_var.span);
            $context.source_map.add_struct_type_parameter_mapping(
                $context.current_struct_definition_index(),
                source_name,
            )?;
        }
    };
    (struct_decl: $context:expr, $location:expr) => {
        $context
            .source_map
            .add_top_level_struct_mapping($context.current_struct_definition_index(), $location)?;
    };
}

macro_rules! make_push_instr {
    ($context:ident, $code:ident) => {
        macro_rules! push_instr {
            ($loc:expr, $instr:expr) => {{
                let code_offset = $code.len() as CodeOffset;
                $context.source_map.add_code_mapping(
                    $context.current_function_definition_index(),
                    code_offset,
                    $loc,
                )?;
                $code.push($instr);
            }};
        }
    };
}

#[derive(Debug, Default)]
struct LoopInfo {
    start_loc: usize,
    breaks: Vec<usize>,
}

// Ideally, we should capture all of this info into a CFG, but as we only have structured control
// flow currently, it would be a bit overkill. It will be a necessity if we add arbitrary branches
// in the IR, as is expressible in the bytecode
struct ControlFlowInfo {
    // A `break` is reachable iff it was used before a terminal node
    reachable_break: bool,
    // A terminal node is an infinite loop or a path that always returns
    terminal_node: bool,
}

impl ControlFlowInfo {
    fn join(f1: ControlFlowInfo, f2: ControlFlowInfo) -> ControlFlowInfo {
        ControlFlowInfo {
            reachable_break: f1.reachable_break || f2.reachable_break,
            terminal_node: f1.terminal_node && f2.terminal_node,
        }
    }
    fn successor(prev: ControlFlowInfo, next: ControlFlowInfo) -> ControlFlowInfo {
        if prev.terminal_node {
            prev
        } else {
            ControlFlowInfo {
                reachable_break: prev.reachable_break || next.reachable_break,
                terminal_node: next.terminal_node,
            }
        }
    }
}

// Inferred representation of SignatureToken's
// In essence, it's a signature token with a "bottom" type added
enum InferredType {
    // Result of the compiler failing to infer the type of an expression
    // Not translatable to a signature token
    Anything,

    // Signature tokens
    Bool,
    U8,
    U64,
    U128,
    ByteArray,
    Address,
    Struct(StructHandleIndex),
    Reference(Box<InferredType>),
    MutableReference(Box<InferredType>),
    TypeParameter(String),
}

impl InferredType {
    fn from_signature_token(sig_token: &SignatureToken) -> Self {
        use InferredType as I;
        use SignatureToken as S;
        match sig_token {
            S::Bool => I::Bool,
            S::U8 => I::U8,
            S::U64 => I::U64,
            S::U128 => I::U128,
            S::ByteArray => I::ByteArray,
            S::Address => I::Address,
            S::Struct(si, _) => I::Struct(*si),
            S::Reference(s_inner) => {
                let i_inner = Self::from_signature_token(&*s_inner);
                I::Reference(Box::new(i_inner))
            }
            S::MutableReference(s_inner) => {
                let i_inner = Self::from_signature_token(&*s_inner);
                I::MutableReference(Box::new(i_inner))
            }
            S::TypeParameter(s_inner) => I::TypeParameter(s_inner.to_string()),
        }
    }

    fn get_struct_handle(&self) -> Result<StructHandleIndex> {
        match self {
            InferredType::Anything => bail!("could not infer struct type"),
            InferredType::Bool => bail!("no struct type for Bool"),
            InferredType::U8 => bail!("no struct type for U8"),
            InferredType::U64 => bail!("no struct type for U64"),
            InferredType::U128 => bail!("no struct type for U128"),
            InferredType::ByteArray => bail!("no struct type for ByteArray"),
            InferredType::Address => bail!("no struct type for Address"),
            InferredType::Reference(inner) | InferredType::MutableReference(inner) => {
                inner.get_struct_handle()
            }
            InferredType::Struct(idx) => Ok(*idx),
            InferredType::TypeParameter(_) => bail!("no struct type for type parameter"),
        }
    }
}

// Holds information about a function being compiled.
#[derive(Debug, Default)]
struct FunctionFrame {
    local_count: u8,
    locals: HashMap<Var_, u8>,
    local_types: LocalsSignature,
    // i64 to allow the bytecode verifier to catch errors of
    // - negative stack sizes
    // - excessivley large stack sizes
    // The max stack depth of the file_format is set as u16.
    // Theoretically, we could use a BigInt here, but that is probably overkill for any testing
    max_stack_depth: i64,
    cur_stack_depth: i64,
    loops: Vec<LoopInfo>,
}

impl FunctionFrame {
    fn new() -> FunctionFrame {
        FunctionFrame::default()
    }

    // Manage the stack info for the function
    fn push(&mut self) -> Result<()> {
        if self.cur_stack_depth == i64::max_value() {
            bail!("ICE Stack depth accounting overflow. The compiler can only support a maximum stack depth of up to i64::max_value")
        }
        self.cur_stack_depth += 1;
        self.max_stack_depth = std::cmp::max(self.max_stack_depth, self.cur_stack_depth);
        Ok(())
    }

    fn pop(&mut self) -> Result<()> {
        if self.cur_stack_depth == i64::min_value() {
            bail!("ICE Stack depth accounting underflow. The compiler can only support a minimum stack depth of up to i64::min_value")
        }
        self.cur_stack_depth -= 1;
        Ok(())
    }

    fn get_local(&self, var: &Var_) -> Result<u8> {
        match self.locals.get(var) {
            None => bail!("variable {} undefined", var),
            Some(idx) => Ok(*idx),
        }
    }

    fn get_local_type(&self, idx: u8) -> Result<&SignatureToken> {
        self.local_types
            .0
            .get(idx as usize)
            .ok_or_else(|| format_err!("variable {} undefined", idx))
    }

    fn define_local(&mut self, var: &Var_, type_: SignatureToken) -> Result<u8> {
        if self.local_count >= u8::max_value() {
            bail!("Max number of locals reached");
        }

        let cur_loc_idx = self.local_count;
        let loc = var.clone();
        let entry = self.locals.entry(loc);
        match entry {
            Occupied(_) => bail!("variable redefinition {}", var),
            Vacant(e) => {
                e.insert(cur_loc_idx);
                self.local_types.0.push(type_);
                self.local_count += 1;
            }
        }
        Ok(cur_loc_idx)
    }

    fn push_loop(&mut self, start_loc: usize) -> Result<()> {
        self.loops.push(LoopInfo {
            start_loc,
            breaks: Vec::new(),
        });
        Ok(())
    }

    fn pop_loop(&mut self) -> Result<()> {
        match self.loops.pop() {
            Some(_) => Ok(()),
            None => bail!("Impossible: failed to pop loop!"),
        }
    }

    fn get_loop_start(&self) -> Result<usize> {
        match self.loops.last() {
            Some(loop_) => Ok(loop_.start_loc),
            None => bail!("continue outside loop"),
        }
    }

    fn push_loop_break(&mut self, loc: usize) -> Result<()> {
        match self.loops.last_mut() {
            Some(loop_) => {
                loop_.breaks.push(loc);
                Ok(())
            }
            None => bail!("break outside loop"),
        }
    }

    fn get_loop_breaks(&self) -> Result<&Vec<usize>> {
        match self.loops.last() {
            Some(loop_) => Ok(&loop_.breaks),
            None => bail!("Impossible: failed to get loop breaks (no loops in stack)"),
        }
    }
}

/// Compile a transaction program.
pub fn compile_program<'a, T: 'a + ModuleAccess>(
    address: AccountAddress,
    program: Program,
    deps: impl IntoIterator<Item = &'a T>,
) -> Result<(CompiledProgram, SourceMap<Loc>)> {
    let deps = deps
        .into_iter()
        .map(|dep| dep.as_module())
        .collect::<Vec<_>>();
    // This is separate to avoid unnecessary code gen due to monomorphization.
    let mut modules = vec![];
    let mut source_maps = vec![];
    for m in program.modules {
        let (module, source_map) = {
            let deps = deps.iter().copied().chain(&modules);
            compile_module(address, m, deps)?
        };
        modules.push(module);
        source_maps.push(source_map);
    }

    let deps = deps.into_iter().chain(modules.iter());
    let (script, source_map) = compile_script(address, program.script, deps)?;
    source_maps.push(source_map);
    Ok((CompiledProgram { modules, script }, source_maps))
}

/// Compile a transaction script.
pub fn compile_script<'a, T: 'a + ModuleAccess>(
    address: AccountAddress,
    script: Script,
    dependencies: impl IntoIterator<Item = &'a T>,
) -> Result<(CompiledScript, ModuleSourceMap<Loc>)> {
    let current_module = QualifiedModuleIdent {
        address,
        name: ModuleName::new(file_format::self_module_name().to_owned()),
    };
    let mut context = Context::new(dependencies, current_module)?;
    let self_name = ModuleName::new(ModuleName::self_name().into());

    compile_imports(&mut context, address, script.imports)?;
    let main_name = FunctionName::new(Identifier::new("main").unwrap());
    let function = script.main;

    let sig = function_signature(&mut context, &function.signature)?;
    context.declare_function(self_name.clone(), main_name.clone(), sig)?;
    let main = compile_function(&mut context, &self_name, main_name, function, 0)?;

    let (
        MaterializedPools {
            module_handles,
            struct_handles,
            function_handles,
            type_signatures,
            function_signatures,
            locals_signatures,
            identifiers,
            byte_array_pool,
            address_pool,
        },
        source_map,
    ) = context.materialize_pools();
    let compiled_script = CompiledScriptMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
        main,
    };
    compiled_script
        .freeze()
        .map_err(|errs| InternalCompilerError::BoundsCheckErrors(errs).into())
        .map(|frozen_script| (frozen_script, source_map))
}

/// Compile a module.
pub fn compile_module<'a, T: 'a + ModuleAccess>(
    address: AccountAddress,
    module: ModuleDefinition,
    dependencies: impl IntoIterator<Item = &'a T>,
) -> Result<(CompiledModule, ModuleSourceMap<Loc>)> {
    let current_module = QualifiedModuleIdent {
        address,
        name: module.name,
    };
    let mut context = Context::new(dependencies, current_module)?;
    let self_name = ModuleName::new(ModuleName::self_name().into());
    // Explicitly declare all imports as they will be included even if not used
    compile_imports(&mut context, address, module.imports)?;

    // Explicitly declare all structs as they will be included even if not used
    for s in &module.structs {
        let ident = QualifiedStructIdent {
            module: self_name.clone(),
            name: s.name.clone(),
        };
        let (_, tys) = type_formals(&s.type_formals)?;
        context.declare_struct_handle_index(ident, s.is_nominal_resource, tys)?;
    }

    for (name, function) in &module.functions {
        let sig = function_signature(&mut context, &function.signature)?;
        context.declare_function(self_name.clone(), name.clone(), sig)?;
    }

    // Current module

    let (struct_defs, field_defs) = compile_structs(&mut context, &self_name, module.structs)?;

    let function_defs = compile_functions(&mut context, &self_name, module.functions)?;

    let (
        MaterializedPools {
            module_handles,
            struct_handles,
            function_handles,
            type_signatures,
            function_signatures,
            locals_signatures,
            identifiers,
            byte_array_pool,
            address_pool,
        },
        source_map,
    ) = context.materialize_pools();
    let compiled_module = CompiledModuleMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        identifiers,
        byte_array_pool,
        address_pool,
        struct_defs,
        field_defs,
        function_defs,
    };
    compiled_module
        .freeze()
        .map_err(|errs| InternalCompilerError::BoundsCheckErrors(errs).into())
        .map(|frozen_module| (frozen_module, source_map))
}

fn compile_imports(
    context: &mut Context,
    address: AccountAddress,
    imports: Vec<ImportDefinition>,
) -> Result<()> {
    for import in imports {
        let ident = match import.ident {
            ModuleIdent::Transaction(name) => QualifiedModuleIdent { address, name },
            ModuleIdent::Qualified(id) => id,
        };
        context.declare_import(ident, import.alias)?;
    }
    Ok(())
}

fn type_formals(ast_tys: &[(TypeVar, ast::Kind)]) -> Result<(HashMap<TypeVar_, usize>, Vec<Kind>)> {
    let mut m = HashMap::new();
    let mut tys = vec![];
    for (idx, (ty_var, k)) in ast_tys.iter().enumerate() {
        let old = m.insert(ty_var.value.clone(), idx);
        if old.is_some() {
            bail!("Type formal '{}'' already bound", ty_var)
        }
        tys.push(kind(k));
    }
    Ok((m, tys))
}

fn kind(ast_k: &ast::Kind) -> Kind {
    match ast_k {
        ast::Kind::All => Kind::All,
        ast::Kind::Resource => Kind::Resource,
        ast::Kind::Unrestricted => Kind::Unrestricted,
    }
}

fn compile_types(context: &mut Context, tys: &[Type]) -> Result<Vec<SignatureToken>> {
    tys.iter()
        .map(|ty| compile_type(context, ty))
        .collect::<Result<_>>()
}

fn compile_type(context: &mut Context, ty: &Type) -> Result<SignatureToken> {
    Ok(match ty {
        Type::Address => SignatureToken::Address,
        Type::U8 => SignatureToken::U8,
        Type::U64 => SignatureToken::U64,
        Type::U128 => SignatureToken::U128,
        Type::Bool => SignatureToken::Bool,
        Type::ByteArray => SignatureToken::ByteArray,
        Type::Reference(is_mutable, inner_type) => {
            let inner_token = Box::new(compile_type(context, inner_type)?);
            if *is_mutable {
                SignatureToken::MutableReference(inner_token)
            } else {
                SignatureToken::Reference(inner_token)
            }
        }
        Type::Struct(ident, tys) => {
            let sh_idx = context.struct_handle_index(ident.clone())?;
            let tokens = compile_types(context, tys)?;
            SignatureToken::Struct(sh_idx, tokens)
        }
        Type::TypeParameter(ty_var) => {
            SignatureToken::TypeParameter(context.type_formal_index(ty_var)?)
        }
    })
}

fn function_signature(
    context: &mut Context,
    f: &ast::FunctionSignature,
) -> Result<FunctionSignature> {
    let (map, _) = type_formals(&f.type_formals)?;
    context.bind_type_formals(map)?;
    let return_types = compile_types(context, &f.return_type)?;
    let arg_types = f
        .formals
        .iter()
        .map(|(_, ty)| compile_type(context, ty))
        .collect::<Result<_>>()?;
    let type_formals = f.type_formals.iter().map(|(_, k)| kind(k)).collect();
    Ok(FunctionSignature {
        return_types,
        arg_types,
        type_formals,
    })
}

fn compile_structs(
    context: &mut Context,
    self_name: &ModuleName,
    structs: Vec<ast::StructDefinition>,
) -> Result<(Vec<StructDefinition>, Vec<FieldDefinition>)> {
    let mut struct_defs = vec![];
    let mut field_defs = vec![];
    for s in structs {
        let sident = QualifiedStructIdent {
            module: self_name.clone(),
            name: s.name.clone(),
        };
        let sh_idx = context.struct_handle_index(sident.clone())?;
        record_src_loc!(struct_decl: context, s.span);
        record_src_loc!(struct_type_formals: context, &s.type_formals);
        let (map, _) = type_formals(&s.type_formals)?;
        context.bind_type_formals(map)?;
        let field_information = compile_fields(context, &mut field_defs, sh_idx, s.value.fields)?;
        context.declare_struct_definition_index(s.value.name)?;
        struct_defs.push(StructDefinition {
            struct_handle: sh_idx,
            field_information,
        });
    }
    Ok((struct_defs, field_defs))
}

fn compile_fields(
    context: &mut Context,
    field_pool: &mut Vec<FieldDefinition>,
    sh_idx: StructHandleIndex,
    sfields: StructDefinitionFields,
) -> Result<StructFieldInformation> {
    Ok(match sfields {
        StructDefinitionFields::Native => StructFieldInformation::Native,
        StructDefinitionFields::Move { fields } => {
            let pool_len = field_pool.len();
            let field_count = fields.len();

            let field_information = StructFieldInformation::Declared {
                field_count: (field_count as MemberCount),
                fields: FieldDefinitionIndex(pool_len as TableIndex),
            };

            for (decl_order, (f, ty)) in fields.into_iter().enumerate() {
                let name = context.identifier_index(f.name())?;
                record_src_loc!(field: context, f);
                let sig_token = compile_type(context, &ty)?;
                let signature = context.type_signature_index(sig_token.clone())?;
                context.declare_field(sh_idx, f.value, sig_token, decl_order)?;
                field_pool.push(FieldDefinition {
                    struct_: sh_idx,
                    name,
                    signature,
                });
            }
            field_information
        }
    })
}

fn compile_functions(
    context: &mut Context,
    self_name: &ModuleName,
    functions: Vec<(FunctionName, Function)>,
) -> Result<Vec<FunctionDefinition>> {
    functions
        .into_iter()
        .enumerate()
        .map(|(func_index, (name, ast_function))| {
            compile_function(context, self_name, name, ast_function, func_index)
        })
        .collect()
}

fn compile_function(
    context: &mut Context,
    self_name: &ModuleName,
    name: FunctionName,
    ast_function: Function,
    function_index: usize,
) -> Result<FunctionDefinition> {
    record_src_loc!(function_decl: context, ast_function.span, function_index);
    record_src_loc!(
        function_type_formals: context,
        &ast_function.signature.type_formals
    );
    let fh_idx = context.function_handle(self_name.clone(), name)?.1;

    let ast_function = ast_function.value;

    let flags = match ast_function.visibility {
        FunctionVisibility::Internal => 0,
        FunctionVisibility::Public => CodeUnit::PUBLIC,
    } | match &ast_function.body {
        FunctionBody::Move { .. } => 0,
        FunctionBody::Native => CodeUnit::NATIVE,
    };
    let acquires_global_resources = ast_function
        .acquires
        .iter()
        .map(|name| context.struct_definition_index(name))
        .collect::<Result<_>>()?;

    let code = match ast_function.body {
        FunctionBody::Move { locals, code } => {
            let (m, _) = type_formals(&ast_function.signature.type_formals)?;
            context.bind_type_formals(m)?;
            compile_function_body(context, ast_function.signature.formals, locals, code)?
        }
        FunctionBody::Native => {
            for (var, _) in ast_function.signature.formals.into_iter() {
                record_src_loc!(local: context, var)
            }
            CodeUnit::default()
        }
    };
    Ok(FunctionDefinition {
        function: fh_idx,
        flags,
        acquires_global_resources,
        code,
    })
}

fn compile_function_body(
    context: &mut Context,
    formals: Vec<(Var, Type)>,
    locals: Vec<(Var, Type)>,
    block: Block_,
) -> Result<CodeUnit> {
    let mut function_frame = FunctionFrame::new();
    let mut locals_signature = LocalsSignature(vec![]);
    for (var, t) in formals {
        let sig = compile_type(context, &t)?;
        function_frame.define_local(&var, sig.clone())?;
        locals_signature.0.push(sig);
        record_src_loc!(local: context, var);
    }
    for (var_, t) in locals {
        let sig = compile_type(context, &t)?;
        function_frame.define_local(&var_.value, sig.clone())?;
        locals_signature.0.push(sig);
        record_src_loc!(local: context, var_);
    }
    let sig_idx = context.locals_signature_index(locals_signature)?;

    let mut code = vec![];
    compile_block(context, &mut function_frame, &mut code, block)?;
    let max_stack_size = if function_frame.max_stack_depth < 0 {
        0
    } else if function_frame.max_stack_depth > i64::from(u16::max_value()) {
        u16::max_value()
    } else {
        function_frame.max_stack_depth as u16
    };
    Ok(CodeUnit {
        locals: sig_idx,
        max_stack_size,
        code,
    })
}

fn compile_block(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    block: Block_,
) -> Result<ControlFlowInfo> {
    let mut cf_info = ControlFlowInfo {
        reachable_break: false,
        terminal_node: false,
    };
    for stmt in block.stmts {
        let stmt_info = match stmt {
            Statement::CommandStatement(command) => {
                compile_command(context, function_frame, code, command)?
            }
            Statement::WhileStatement(while_) => {
                // always assume the loop might not be taken
                compile_while(context, function_frame, code, while_)?
            }
            Statement::LoopStatement(loop_) => compile_loop(context, function_frame, code, loop_)?,
            Statement::IfElseStatement(if_else) => {
                compile_if_else(context, function_frame, code, if_else)?
            }
            Statement::EmptyStatement => continue,
        };
        cf_info = ControlFlowInfo::successor(cf_info, stmt_info);
    }
    Ok(cf_info)
}

fn compile_if_else(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    if_else: IfElse,
) -> Result<ControlFlowInfo> {
    make_push_instr!(context, code);
    let cond_span = if_else.cond.span;
    compile_expression(context, function_frame, code, if_else.cond)?;

    let brfalse_ins_loc = code.len();
    // placeholder, final branch target replaced later
    push_instr!(cond_span, Bytecode::BrFalse(0));
    function_frame.pop()?;
    let if_cf_info = compile_block(context, function_frame, code, if_else.if_block.value)?;

    let mut else_block_location = code.len();

    let else_cf_info = match if_else.else_block {
        None => ControlFlowInfo {
            reachable_break: false,
            terminal_node: false,
        },
        Some(else_block) => {
            let branch_ins_loc = code.len();
            if !if_cf_info.terminal_node {
                // placeholder, final branch target replaced later
                push_instr!(else_block.span, Bytecode::Branch(0));
                else_block_location += 1;
            }
            let else_cf_info = compile_block(context, function_frame, code, else_block.value)?;
            if !if_cf_info.terminal_node {
                code[branch_ins_loc] = Bytecode::Branch(code.len() as u16);
            }
            else_cf_info
        }
    };

    code[brfalse_ins_loc] = Bytecode::BrFalse(else_block_location as u16);

    let cf_info = ControlFlowInfo::join(if_cf_info, else_cf_info);
    Ok(cf_info)
}

fn compile_while(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    while_: While,
) -> Result<ControlFlowInfo> {
    make_push_instr!(context, code);
    let cond_span = while_.cond.span;
    let loop_start_loc = code.len();
    function_frame.push_loop(loop_start_loc)?;
    compile_expression(context, function_frame, code, while_.cond)?;

    let brfalse_loc = code.len();

    // placeholder, final branch target replaced later
    push_instr!(cond_span, Bytecode::BrFalse(0));
    function_frame.pop()?;

    compile_block(context, function_frame, code, while_.block.value)?;
    push_instr!(while_.block.span, Bytecode::Branch(loop_start_loc as u16));

    let loop_end_loc = code.len() as u16;
    code[brfalse_loc] = Bytecode::BrFalse(loop_end_loc);
    let breaks = function_frame.get_loop_breaks()?;
    for i in breaks {
        code[*i] = Bytecode::Branch(loop_end_loc);
    }

    function_frame.pop_loop()?;
    Ok(ControlFlowInfo {
        // this `reachable_break` break is for any outer loop
        // not the loop that was just compiled
        reachable_break: false,
        // While always has the ability to break.
        // Conceptually we treat
        //   `while (cond) { body }`
        // as `
        //   `loop { if (cond) { body; continue; } else { break; } }`
        // So a `break` is always reachable
        terminal_node: false,
    })
}

fn compile_loop(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    loop_: Loop,
) -> Result<ControlFlowInfo> {
    make_push_instr!(context, code);
    let loop_start_loc = code.len();
    function_frame.push_loop(loop_start_loc)?;

    let body_cf_info = compile_block(context, function_frame, code, loop_.block.value)?;
    push_instr!(loop_.block.span, Bytecode::Branch(loop_start_loc as u16));

    let loop_end_loc = code.len() as u16;
    let breaks = function_frame.get_loop_breaks()?;
    for i in breaks {
        code[*i] = Bytecode::Branch(loop_end_loc);
    }

    function_frame.pop_loop()?;
    // this `reachable_break` break is for any outer loop
    // not the loop that was just compiled
    let reachable_break = false;
    // If the body of the loop does not have a break, it will loop forever
    // and thus is a terminal node
    let terminal_node = !body_cf_info.reachable_break;
    Ok(ControlFlowInfo {
        reachable_break,
        terminal_node,
    })
}

fn compile_command(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    cmd: Cmd,
) -> Result<ControlFlowInfo> {
    make_push_instr!(context, code);
    let (reachable_break, terminal_node) = match &cmd.value {
            // If we are in a loop, `continue` makes a terminal node
            // Conceptually we treat
            //   `while (cond) { body }`
            // as `
            //   `loop { if (cond) { body; continue; } else { break; } }`
            Cmd_::Continue |
            // `return` and `abort` alway makes a terminal node
            Cmd_::Abort(_) |
            Cmd_::Return(_) => (false, true),
            Cmd_::Break => (true, false),
            _ => (false, false),
        };
    match cmd.value {
        Cmd_::Return(exps) => {
            compile_expression(context, function_frame, code, *exps)?;
            push_instr!(cmd.span, Bytecode::Ret);
        }
        Cmd_::Abort(exp_opt) => {
            if let Some(exp) = exp_opt {
                compile_expression(context, function_frame, code, *exp)?;
            }
            push_instr!(cmd.span, Bytecode::Abort);
            function_frame.pop()?;
        }
        Cmd_::Assign(lvalues, rhs_expressions) => {
            compile_expression(context, function_frame, code, rhs_expressions)?;
            compile_lvalues(context, function_frame, code, lvalues)?;
        }
        Cmd_::Unpack(name, tys, bindings, e) => {
            let tokens = LocalsSignature(compile_types(context, &tys)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;

            compile_expression(context, function_frame, code, *e)?;

            let def_idx = context.struct_definition_index(&name)?;
            push_instr!(cmd.span, Bytecode::Unpack(def_idx, type_actuals_id));
            function_frame.pop()?;

            for (field_, lhs_variable) in bindings.iter().rev() {
                let loc_idx = function_frame.get_local(&lhs_variable.value)?;
                let st_loc = Bytecode::StLoc(loc_idx);
                push_instr!(field_.span, st_loc);
            }
        }
        Cmd_::Continue => {
            let loc = function_frame.get_loop_start()?;
            push_instr!(cmd.span, Bytecode::Branch(loc as u16));
        }
        Cmd_::Break => {
            function_frame.push_loop_break(code.len())?;
            // placeholder, to be replaced when the enclosing while is compiled
            push_instr!(cmd.span, Bytecode::Branch(0));
        }
        Cmd_::Exp(e) => {
            compile_expression(context, function_frame, code, *e)?;
        }
    }
    Ok(ControlFlowInfo {
        reachable_break,
        terminal_node,
    })
}

fn compile_lvalues(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    lvalues: Vec<LValue>,
) -> Result<()> {
    make_push_instr!(context, code);
    for lvalue_ in lvalues.into_iter().rev() {
        match lvalue_.value {
            LValue_::Var(v) => {
                let loc_idx = function_frame.get_local(&v.value)?;
                push_instr!(lvalue_.span, Bytecode::StLoc(loc_idx));
                function_frame.pop()?;
            }
            LValue_::Mutate(e) => {
                compile_expression(context, function_frame, code, e)?;
                push_instr!(lvalue_.span, Bytecode::WriteRef);
                function_frame.pop()?;
                function_frame.pop()?;
            }
            LValue_::Pop => {
                push_instr!(lvalue_.span, Bytecode::Pop);
                function_frame.pop()?;
            }
        }
    }
    Ok(())
}

macro_rules! vec_deque {
    ($($x:expr),*) => {
        VecDeque::from(vec![$($x),*])
    }
}

fn infer_int_bin_op_result_ty(
    tys1: &VecDeque<InferredType>,
    tys2: &VecDeque<InferredType>,
) -> InferredType {
    use InferredType as I;
    if tys1.len() != 1 || tys2.len() != 1 {
        return I::Anything;
    }
    match (&tys1[0], &tys2[0]) {
        (I::U8, I::U8) => I::U8,
        (I::U64, I::U64) => I::U64,
        (I::U128, I::U128) => I::U128,
        _ => I::Anything,
    }
}

fn compile_expression(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    exp: Exp,
) -> Result<VecDeque<InferredType>> {
    make_push_instr!(context, code);
    Ok(match exp.value {
        Exp_::Move(v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let load_loc = Bytecode::MoveLoc(loc_idx);
            push_instr!(exp.span, load_loc);
            function_frame.push()?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            vec_deque![InferredType::from_signature_token(loc_type)]
        }
        Exp_::Copy(v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let load_loc = Bytecode::CopyLoc(loc_idx);
            push_instr!(exp.span, load_loc);
            function_frame.push()?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            vec_deque![InferredType::from_signature_token(loc_type)]
        }
        Exp_::BorrowLocal(is_mutable, v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            let inner_token = Box::new(InferredType::from_signature_token(loc_type));
            if is_mutable {
                push_instr!(exp.span, Bytecode::MutBorrowLoc(loc_idx));
                function_frame.push()?;
                vec_deque![InferredType::MutableReference(inner_token)]
            } else {
                push_instr!(exp.span, Bytecode::ImmBorrowLoc(loc_idx));
                function_frame.push()?;
                vec_deque![InferredType::Reference(inner_token)]
            }
        }
        Exp_::Value(cv) => match cv.value {
            CopyableVal_::Address(address) => {
                let addr_idx = context.address_index(address)?;
                push_instr!(exp.span, Bytecode::LdAddr(addr_idx));
                function_frame.push()?;
                vec_deque![InferredType::Address]
            }
            CopyableVal_::U8(i) => {
                push_instr!(exp.span, Bytecode::LdU8(i));
                function_frame.push()?;
                vec_deque![InferredType::U8]
            }
            CopyableVal_::U64(i) => {
                push_instr!(exp.span, Bytecode::LdU64(i));
                function_frame.push()?;
                vec_deque![InferredType::U64]
            }
            CopyableVal_::U128(i) => {
                push_instr!(exp.span, Bytecode::LdU128(i));
                function_frame.push()?;
                vec_deque![InferredType::U128]
            }
            CopyableVal_::ByteArray(buf) => {
                let buf_idx = context.byte_array_index(&buf)?;
                push_instr!(exp.span, Bytecode::LdByteArray(buf_idx));
                function_frame.push()?;
                vec_deque![InferredType::ByteArray]
            }
            CopyableVal_::Bool(b) => {
                push_instr! {exp.span,
                    if b {
                        Bytecode::LdTrue
                    } else {
                        Bytecode::LdFalse
                    }
                };
                function_frame.push()?;
                vec_deque![InferredType::Bool]
            }
        },
        Exp_::Pack(name, tys, fields) => {
            let tokens = LocalsSignature(compile_types(context, &tys)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;
            let def_idx = context.struct_definition_index(&name)?;

            let self_name = ModuleName::new(ModuleName::self_name().into());
            let ident = QualifiedStructIdent {
                module: self_name,
                name: name.clone(),
            };
            let sh_idx = context.struct_handle_index(ident)?;

            let num_fields = fields.len();
            for (field_order, (field, e)) in fields.into_iter().enumerate() {
                // Check that the fields are specified in order matching the definition.
                let (_, _, decl_order) = context.field(sh_idx, field.value.clone())?;
                if field_order != decl_order {
                    bail!("Field {} defined out of order for struct {}", field, name);
                }

                compile_expression(context, function_frame, code, e)?;
            }
            push_instr!(exp.span, Bytecode::Pack(def_idx, type_actuals_id));
            for _ in 0..num_fields {
                function_frame.pop()?;
            }
            function_frame.push()?;

            vec_deque![InferredType::Struct(sh_idx)]
        }
        Exp_::UnaryExp(op, e) => {
            compile_expression(context, function_frame, code, *e)?;
            match op {
                UnaryOp::Not => {
                    push_instr!(exp.span, Bytecode::Not);
                    vec_deque![InferredType::Bool]
                }
            }
        }
        Exp_::BinopExp(e1, op, e2) => {
            let tys1 = compile_expression(context, function_frame, code, *e1)?;
            let tys2 = compile_expression(context, function_frame, code, *e2)?;

            function_frame.pop()?;
            match op {
                BinOp::Add => {
                    push_instr!(exp.span, Bytecode::Add);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Sub => {
                    push_instr!(exp.span, Bytecode::Sub);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Mul => {
                    push_instr!(exp.span, Bytecode::Mul);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Mod => {
                    push_instr!(exp.span, Bytecode::Mod);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Div => {
                    push_instr!(exp.span, Bytecode::Div);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::BitOr => {
                    push_instr!(exp.span, Bytecode::BitOr);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::BitAnd => {
                    push_instr!(exp.span, Bytecode::BitAnd);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Xor => {
                    push_instr!(exp.span, Bytecode::Xor);
                    vec_deque![infer_int_bin_op_result_ty(&tys1, &tys2)]
                }
                BinOp::Shl => {
                    push_instr!(exp.span, Bytecode::Shl);
                    tys1
                }
                BinOp::Shr => {
                    push_instr!(exp.span, Bytecode::Shr);
                    tys1
                }
                BinOp::Or => {
                    push_instr!(exp.span, Bytecode::Or);
                    vec_deque![InferredType::Bool]
                }
                BinOp::And => {
                    push_instr!(exp.span, Bytecode::And);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Eq => {
                    push_instr!(exp.span, Bytecode::Eq);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Neq => {
                    push_instr!(exp.span, Bytecode::Neq);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Lt => {
                    push_instr!(exp.span, Bytecode::Lt);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Gt => {
                    push_instr!(exp.span, Bytecode::Gt);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Le => {
                    push_instr!(exp.span, Bytecode::Le);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Ge => {
                    push_instr!(exp.span, Bytecode::Ge);
                    vec_deque![InferredType::Bool]
                }
            }
        }
        Exp_::Dereference(e) => {
            let loc_type = compile_expression(context, function_frame, code, *e)?.pop_front();
            push_instr!(exp.span, Bytecode::ReadRef);
            match loc_type {
                Some(InferredType::MutableReference(sig_ref_token)) => vec_deque![*sig_ref_token],
                Some(InferredType::Reference(sig_ref_token)) => vec_deque![*sig_ref_token],
                _ => vec_deque![InferredType::Anything],
            }
        }
        Exp_::Borrow {
            is_mutable,
            exp: inner_exp,
            field,
        } => {
            let loc_type_opt =
                compile_expression(context, function_frame, code, *inner_exp)?.pop_front();
            let loc_type =
                loc_type_opt.ok_or_else(|| format_err!("Impossible no expression to borrow"))?;
            let sh_idx = loc_type.get_struct_handle()?;
            let (fd_idx, field_type, _) = context.field(sh_idx, field)?;
            function_frame.pop()?;
            let inner_token = Box::new(InferredType::from_signature_token(&field_type));
            if is_mutable {
                push_instr!(exp.span, Bytecode::MutBorrowField(fd_idx));
                function_frame.push()?;
                vec_deque![InferredType::MutableReference(inner_token)]
            } else {
                push_instr!(exp.span, Bytecode::ImmBorrowField(fd_idx));
                function_frame.push()?;
                vec_deque![InferredType::Reference(inner_token)]
            }
        }
        Exp_::FunctionCall(f, exps) => {
            let mut actuals_tys = vec_deque![];
            for types in compile_expression(context, function_frame, code, *exps)? {
                actuals_tys.push_back(types);
            }
            compile_call(context, function_frame, code, f, actuals_tys)?
        }
        Exp_::ExprList(exps) => {
            let mut result = vec_deque![];
            for e in exps {
                result.append(&mut compile_expression(context, function_frame, code, e)?);
            }
            result
        }
    })
}

fn compile_call(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    call: FunctionCall,
    mut argument_types: VecDeque<InferredType>,
) -> Result<VecDeque<InferredType>> {
    make_push_instr!(context, code);
    Ok(match call.value {
        FunctionCall_::Builtin(function) => {
            match function {
                Builtin::GetTxnSender => {
                    push_instr!(call.span, Bytecode::GetTxnSenderAddress);
                    function_frame.push()?;
                    vec_deque![InferredType::Address]
                }
                Builtin::Exists(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    push_instr!(call.span, Bytecode::Exists(def_idx, type_actuals_id));
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![InferredType::Bool]
                }
                Builtin::BorrowGlobal(mut_, name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    push_instr! {call.span,
                        if mut_ {
                            Bytecode::MutBorrowGlobal(def_idx, type_actuals_id)
                        } else {
                            Bytecode::ImmBorrowGlobal(def_idx, type_actuals_id)
                        }
                    };
                    function_frame.pop()?;
                    function_frame.push()?;

                    let self_name = ModuleName::new(ModuleName::self_name().into());
                    let ident = QualifiedStructIdent {
                        module: self_name,
                        name,
                    };
                    let sh_idx = context.struct_handle_index(ident)?;
                    let inner = Box::new(InferredType::Struct(sh_idx));
                    vec_deque![if mut_ {
                        InferredType::MutableReference(inner)
                    } else {
                        InferredType::Reference(inner)
                    }]
                }
                Builtin::MoveFrom(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    push_instr!(call.span, Bytecode::MoveFrom(def_idx, type_actuals_id));
                    function_frame.pop()?; // pop the address
                    function_frame.push()?; // push the return value

                    let self_name = ModuleName::new(ModuleName::self_name().into());
                    let ident = QualifiedStructIdent {
                        module: self_name,
                        name,
                    };
                    let sh_idx = context.struct_handle_index(ident)?;
                    vec_deque![InferredType::Struct(sh_idx)]
                }
                Builtin::MoveToSender(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;

                    push_instr!(call.span, Bytecode::MoveToSender(def_idx, type_actuals_id));
                    function_frame.push()?;
                    vec_deque![]
                }
                Builtin::Freeze => {
                    push_instr!(call.span, Bytecode::FreezeRef);
                    function_frame.pop()?; // pop mut ref
                    function_frame.push()?; // push imm ref
                    let inner_token = match argument_types.pop_front() {
                        Some(InferredType::Reference(inner_token))
                        | Some(InferredType::MutableReference(inner_token)) => inner_token,
                        // Incorrect call
                        _ => Box::new(InferredType::Anything),
                    };
                    vec_deque![InferredType::Reference(inner_token)]
                }
                Builtin::ToU8 => {
                    push_instr!(call.span, Bytecode::CastU8);
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![InferredType::U8]
                }
                Builtin::ToU64 => {
                    push_instr!(call.span, Bytecode::CastU64);
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![InferredType::U64]
                }
                Builtin::ToU128 => {
                    push_instr!(call.span, Bytecode::CastU128);
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![InferredType::U128]
                }
            }
        }
        FunctionCall_::ModuleFunctionCall {
            module,
            name,
            type_actuals,
        } => {
            let tokens = LocalsSignature(compile_types(context, &type_actuals)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;
            let fh_idx = context.function_handle(module.clone(), name.clone())?.1;
            let fcall = Bytecode::Call(fh_idx, type_actuals_id);
            push_instr!(call.span, fcall);
            for _ in 0..argument_types.len() {
                function_frame.pop()?;
            }
            // Return value of current function is pushed onto the stack.
            function_frame.push()?;
            let signature = &context.function_signature(module, name)?.0;
            signature
                .return_types
                .iter()
                .map(InferredType::from_signature_token)
                .collect()
        }
    })
}
