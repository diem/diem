// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    context::{Context, MaterializedPools},
    errors::*,
    parser::ast::{
        self, BinOp, Block, Builtin, Cmd, Cmd_, CopyableVal, Exp, Exp_, Function, FunctionBody,
        FunctionCall, FunctionName, FunctionSignature as AstFunctionSignature, FunctionVisibility,
        IfElse, ImportDefinition, LValue, LValue_, Loop, ModuleDefinition, ModuleIdent, ModuleName,
        Program, QualifiedModuleIdent, QualifiedStructIdent, Script, Statement,
        StructDefinition as MoveStruct, StructDefinitionFields, Type, TypeVar, UnaryOp, Var, Var_,
        While,
    },
};

use failure::*;
use std::{
    clone::Clone,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, VecDeque,
    },
};
use types::account_address::AccountAddress;
use vm::{
    access::ModuleAccess,
    file_format::{
        self, Bytecode, CodeUnit, CompiledModule, CompiledModuleMut, CompiledProgram,
        CompiledScript, CompiledScriptMut, FieldDefinition, FieldDefinitionIndex,
        FunctionDefinition, FunctionSignature, Kind, LocalsSignature, MemberCount, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandleIndex, TableIndex,
    },
};

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
    U64,
    String,
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
            S::U64 => I::U64,
            S::String => I::String,
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
            InferredType::U64 => bail!("no struct type for U64"),
            InferredType::String => bail!("no struct type for String"),
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
    locals: HashMap<Var, u8>,
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

    fn get_local(&self, var: &Var) -> Result<u8> {
        match self.locals.get(var) {
            None => bail!("variable {} undefined", var),
            Some(idx) => Ok(*idx),
        }
    }

    fn get_local_type(&self, idx: u8) -> Result<&SignatureToken> {
        match self.local_types.0.get(idx as usize) {
            None => bail!("variable {} undefined", idx),
            Some(sig_token) => Ok(sig_token),
        }
    }

    fn define_local(&mut self, var: &Var, type_: SignatureToken) -> Result<u8> {
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
) -> Result<CompiledProgram> {
    let deps = deps
        .into_iter()
        .map(|dep| dep.as_module())
        .collect::<Vec<_>>();
    // This is separate to avoid unnecessary code gen due to monomorphization.
    let mut modules = vec![];
    for m in program.modules {
        let module = {
            let deps = deps.iter().copied().chain(&modules);
            compile_module(address, m, deps)?
        };
        modules.push(module);
    }

    let deps = deps.into_iter().chain(modules.iter());
    let script = compile_script(address, program.script, deps)?;
    Ok(CompiledProgram { modules, script })
}

/// Compile a transaction script.
pub fn compile_script<'a, T: 'a + ModuleAccess>(
    address: AccountAddress,
    script: Script,
    dependencies: impl IntoIterator<Item = &'a T>,
) -> Result<CompiledScript> {
    let current_module = QualifiedModuleIdent {
        address,
        name: ModuleName::new(file_format::SELF_MODULE_NAME.to_string()),
    };
    let mut context = Context::new(dependencies, current_module)?;
    let self_name = ModuleName::new(ModuleName::SELF.to_string());

    compile_imports(&mut context, address, script.imports)?;
    let main_name = FunctionName::new("main".to_string());
    let function = script.main;

    let sig = function_signature(&mut context, &function.signature)?;
    context.declare_function(self_name.clone(), main_name.clone(), sig)?;
    let main = compile_function(&mut context, &self_name, main_name, function)?;

    let MaterializedPools {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        string_pool,
        user_strings,
        byte_array_pool,
        address_pool,
    } = context.materialize_pools();
    let compiled_script = CompiledScriptMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        string_pool,
        user_strings,
        byte_array_pool,
        address_pool,
        main,
    };
    compiled_script
        .freeze()
        .map_err(|errs| InternalCompilerError::BoundsCheckErrors(errs).into())
}

/// Compile a module.
pub fn compile_module<'a, T: 'a + ModuleAccess>(
    address: AccountAddress,
    module: ModuleDefinition,
    dependencies: impl IntoIterator<Item = &'a T>,
) -> Result<CompiledModule> {
    let current_module = QualifiedModuleIdent {
        address,
        name: module.name,
    };
    let mut context = Context::new(dependencies, current_module)?;
    let self_name = ModuleName::new(ModuleName::SELF.to_string());
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

    let MaterializedPools {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        string_pool,
        user_strings,
        byte_array_pool,
        address_pool,
    } = context.materialize_pools();
    let compiled_module = CompiledModuleMut {
        module_handles,
        struct_handles,
        function_handles,
        type_signatures,
        function_signatures,
        locals_signatures,
        string_pool,
        user_strings,
        byte_array_pool,
        address_pool,
        struct_defs,
        field_defs,
        function_defs,
    };
    compiled_module
        .freeze()
        .map_err(|errs| InternalCompilerError::BoundsCheckErrors(errs).into())
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

fn type_formals(ast_tys: &[(TypeVar, ast::Kind)]) -> Result<(HashMap<TypeVar, usize>, Vec<Kind>)> {
    let mut m = HashMap::new();
    let mut tys = vec![];
    for (idx, (ty_var, k)) in ast_tys.iter().enumerate() {
        let old = m.insert(ty_var.clone(), idx);
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
        Type::U64 => SignatureToken::U64,
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
        Type::String => bail!("`string` type is currently unused"),
    })
}

fn function_signature(
    context: &mut Context,
    f: &AstFunctionSignature,
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
    structs: Vec<MoveStruct>,
) -> Result<(Vec<StructDefinition>, Vec<FieldDefinition>)> {
    let mut struct_defs = vec![];
    let mut field_defs = vec![];
    for s in structs {
        let sident = QualifiedStructIdent {
            module: self_name.clone(),
            name: s.name.clone(),
        };
        let sh_idx = context.struct_handle_index(sident.clone())?;
        let (map, _) = type_formals(&s.type_formals)?;
        context.bind_type_formals(map)?;
        let field_information = compile_fields(context, &mut field_defs, sh_idx, s.fields)?;
        context.declare_struct_definition_index(s.name)?;
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

            for (f, ty) in fields {
                let name = context.string_index(f.name())?;
                let sig_token = compile_type(context, &ty)?;
                let signature = context.type_signature_index(sig_token.clone())?;
                context.declare_field(sh_idx, f, sig_token)?;
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
        .map(|(name, ast_function)| compile_function(context, self_name, name, ast_function))
        .collect()
}

fn compile_function(
    context: &mut Context,
    self_name: &ModuleName,
    name: FunctionName,
    ast_function: Function,
) -> Result<FunctionDefinition> {
    let fh_idx = context.function_handle(self_name.clone(), name)?.1;

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
        FunctionBody::Native => CodeUnit::default(),
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
    locals: Vec<(Var_, Type)>,
    block: Block,
) -> Result<CodeUnit> {
    let mut function_frame = FunctionFrame::new();
    let mut locals_signature = LocalsSignature(vec![]);
    for (var, t) in formals {
        let sig = compile_type(context, &t)?;
        function_frame.define_local(&var, sig.clone())?;
        locals_signature.0.push(sig);
    }
    for (var_, t) in locals {
        let sig = compile_type(context, &t)?;
        function_frame.define_local(&var_.value, sig.clone())?;
        locals_signature.0.push(sig);
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
    block: Block,
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
    compile_expression(context, function_frame, code, if_else.cond)?;

    let brfalse_ins_loc = code.len();
    code.push(Bytecode::BrFalse(0)); // placeholder, final branch target replaced later
    function_frame.pop()?;
    let if_cf_info = compile_block(context, function_frame, code, if_else.if_block)?;

    let mut else_block_location = code.len();

    let else_cf_info = match if_else.else_block {
        None => ControlFlowInfo {
            reachable_break: false,
            terminal_node: false,
        },
        Some(else_block) => {
            let branch_ins_loc = code.len();
            if !if_cf_info.terminal_node {
                code.push(Bytecode::Branch(0)); // placeholder, final branch target replaced later
                else_block_location += 1;
            }
            let else_cf_info = compile_block(context, function_frame, code, else_block)?;
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
    let loop_start_loc = code.len();
    function_frame.push_loop(loop_start_loc)?;
    compile_expression(context, function_frame, code, while_.cond)?;

    let brfalse_loc = code.len();
    code.push(Bytecode::BrFalse(0)); // placeholder, final branch target replaced later
    function_frame.pop()?;

    compile_block(context, function_frame, code, while_.block)?;
    code.push(Bytecode::Branch(loop_start_loc as u16));

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
    let loop_start_loc = code.len();
    function_frame.push_loop(loop_start_loc)?;

    let body_cf_info = compile_block(context, function_frame, code, loop_.block)?;
    code.push(Bytecode::Branch(loop_start_loc as u16));

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
    cmd: Cmd_,
) -> Result<ControlFlowInfo> {
    let (reachable_break, terminal_node) = match &cmd.value {
            // If we are in a loop, `continue` makes a terminal node
            // Conceptually we treat
            //   `while (cond) { body }`
            // as `
            //   `loop { if (cond) { body; continue; } else { break; } }`
            Cmd::Continue |
            // `return` and `abort` alway makes a terminal node
            Cmd::Abort(_) |
            Cmd::Return(_) => (false, true),
            Cmd::Break => (true, false),
            _ => (false, false),
        };
    match cmd.value {
        Cmd::Return(exps) => {
            compile_expression(context, function_frame, code, *exps)?;
            code.push(Bytecode::Ret);
        }
        Cmd::Abort(exp_opt) => {
            if let Some(exp) = exp_opt {
                compile_expression(context, function_frame, code, *exp)?;
            }
            code.push(Bytecode::Abort);
            function_frame.pop()?;
        }
        Cmd::Assign(lvalues, rhs_expressions) => {
            compile_expression(context, function_frame, code, rhs_expressions)?;
            compile_lvalues(context, function_frame, code, lvalues)?;
        }
        Cmd::Unpack(name, tys, bindings, e) => {
            let tokens = LocalsSignature(compile_types(context, &tys)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;

            compile_expression(context, function_frame, code, *e)?;

            let def_idx = context.struct_definition_index(&name)?;
            code.push(Bytecode::Unpack(def_idx, type_actuals_id));
            function_frame.pop()?;

            for lhs_variable in bindings.values().rev() {
                let loc_idx = function_frame.get_local(&lhs_variable.value)?;
                let st_loc = Bytecode::StLoc(loc_idx);
                code.push(st_loc);
            }
        }
        Cmd::Continue => {
            let loc = function_frame.get_loop_start()?;
            code.push(Bytecode::Branch(loc as u16));
        }
        Cmd::Break => {
            function_frame.push_loop_break(code.len())?;
            // placeholder, to be replaced when the enclosing while is compiled
            code.push(Bytecode::Branch(0));
        }
        Cmd::Exp(e) => {
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
    lvalues: Vec<LValue_>,
) -> Result<()> {
    for lvalue_ in lvalues.into_iter().rev() {
        match lvalue_.value {
            LValue::Var(v) => {
                let loc_idx = function_frame.get_local(&v.value)?;
                code.push(Bytecode::StLoc(loc_idx));
                function_frame.pop()?;
            }
            LValue::Mutate(e) => {
                compile_expression(context, function_frame, code, e)?;
                code.push(Bytecode::WriteRef);
                function_frame.pop()?;
                function_frame.pop()?;
            }
            LValue::Pop => {
                code.push(Bytecode::Pop);

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

fn compile_expression(
    context: &mut Context,
    function_frame: &mut FunctionFrame,
    code: &mut Vec<Bytecode>,
    exp: Exp_,
) -> Result<VecDeque<InferredType>> {
    Ok(match exp.value {
        Exp::Move(v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let load_loc = Bytecode::MoveLoc(loc_idx);
            code.push(load_loc);
            function_frame.push()?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            vec_deque![InferredType::from_signature_token(loc_type)]
        }
        Exp::Copy(v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let load_loc = Bytecode::CopyLoc(loc_idx);
            code.push(load_loc);
            function_frame.push()?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            vec_deque![InferredType::from_signature_token(loc_type)]
        }
        Exp::BorrowLocal(is_mutable, v) => {
            let loc_idx = function_frame.get_local(&v.value)?;
            let loc_type = function_frame.get_local_type(loc_idx)?;
            let inner_token = Box::new(InferredType::from_signature_token(loc_type));
            if is_mutable {
                code.push(Bytecode::MutBorrowLoc(loc_idx));
                function_frame.push()?;
                vec_deque![InferredType::MutableReference(inner_token)]
            } else {
                code.push(Bytecode::ImmBorrowLoc(loc_idx));
                function_frame.push()?;
                vec_deque![InferredType::Reference(inner_token)]
            }
        }
        Exp::Value(cv) => match cv.value {
            CopyableVal::Address(address) => {
                let addr_idx = context.address_index(address)?;
                code.push(Bytecode::LdAddr(addr_idx));
                function_frame.push()?;
                vec_deque![InferredType::Address]
            }
            CopyableVal::U64(i) => {
                code.push(Bytecode::LdConst(i));
                function_frame.push()?;
                vec_deque![InferredType::U64]
            }
            CopyableVal::ByteArray(buf) => {
                let buf_idx = context.byte_array_index(&buf)?;
                code.push(Bytecode::LdByteArray(buf_idx));
                function_frame.push()?;
                vec_deque![InferredType::ByteArray]
            }
            CopyableVal::Bool(b) => {
                code.push(if b {
                    Bytecode::LdTrue
                } else {
                    Bytecode::LdFalse
                });
                function_frame.push()?;
                vec_deque![InferredType::Bool]
            }
            CopyableVal::String(_) => bail!("nice try! come back later {:?}", cv),
        },
        Exp::Pack(name, tys, fields) => {
            let tokens = LocalsSignature(compile_types(context, &tys)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;
            let def_idx = context.struct_definition_index(&name)?;

            let num_fields = fields.len();
            for (_, e) in fields {
                compile_expression(context, function_frame, code, e)?;
            }
            code.push(Bytecode::Pack(def_idx, type_actuals_id));
            for _ in 0..num_fields {
                function_frame.pop()?;
            }
            function_frame.push()?;

            let self_name = ModuleName::new(ModuleName::SELF.to_string());
            let ident = QualifiedStructIdent {
                module: self_name,
                name,
            };
            let sh_idx = context.struct_handle_index(ident)?;
            vec_deque![InferredType::Struct(sh_idx)]
        }
        Exp::UnaryExp(op, e) => {
            compile_expression(context, function_frame, code, *e)?;
            match op {
                UnaryOp::Not => {
                    code.push(Bytecode::Not);
                    vec_deque![InferredType::Bool]
                }
            }
        }
        Exp::BinopExp(e1, op, e2) => {
            compile_expression(context, function_frame, code, *e1)?;
            compile_expression(context, function_frame, code, *e2)?;
            function_frame.pop()?;
            match op {
                BinOp::Add => {
                    code.push(Bytecode::Add);
                    vec_deque![InferredType::U64]
                }
                BinOp::Sub => {
                    code.push(Bytecode::Sub);
                    vec_deque![InferredType::U64]
                }
                BinOp::Mul => {
                    code.push(Bytecode::Mul);
                    vec_deque![InferredType::U64]
                }
                BinOp::Mod => {
                    code.push(Bytecode::Mod);
                    vec_deque![InferredType::U64]
                }
                BinOp::Div => {
                    code.push(Bytecode::Div);
                    vec_deque![InferredType::U64]
                }
                BinOp::BitOr => {
                    code.push(Bytecode::BitOr);
                    vec_deque![InferredType::U64]
                }
                BinOp::BitAnd => {
                    code.push(Bytecode::BitAnd);
                    vec_deque![InferredType::U64]
                }
                BinOp::Xor => {
                    code.push(Bytecode::Xor);
                    vec_deque![InferredType::U64]
                }
                BinOp::Or => {
                    code.push(Bytecode::Or);
                    vec_deque![InferredType::Bool]
                }
                BinOp::And => {
                    code.push(Bytecode::And);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Eq => {
                    code.push(Bytecode::Eq);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Neq => {
                    code.push(Bytecode::Neq);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Lt => {
                    code.push(Bytecode::Lt);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Gt => {
                    code.push(Bytecode::Gt);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Le => {
                    code.push(Bytecode::Le);
                    vec_deque![InferredType::Bool]
                }
                BinOp::Ge => {
                    code.push(Bytecode::Ge);
                    vec_deque![InferredType::Bool]
                }
            }
        }
        Exp::Dereference(e) => {
            let loc_type = compile_expression(context, function_frame, code, *e)?.pop_front();

            code.push(Bytecode::ReadRef);
            match loc_type {
                Some(InferredType::MutableReference(sig_ref_token)) => vec_deque![*sig_ref_token],
                Some(InferredType::Reference(sig_ref_token)) => vec_deque![*sig_ref_token],
                _ => vec_deque![InferredType::Anything],
            }
        }
        Exp::Borrow {
            is_mutable,
            exp,
            field,
        } => {
            let loc_type_opt = compile_expression(context, function_frame, code, *exp)?.pop_front();
            let loc_type = match loc_type_opt {
                Some(t) => t,
                None => bail!("Impossible no expression to borrow"),
            };
            let sh_idx = loc_type.get_struct_handle()?;
            let (fd_idx, field_type) = context.field(sh_idx, field)?;
            function_frame.pop()?;
            let inner_token = Box::new(InferredType::from_signature_token(&field_type));
            if is_mutable {
                code.push(Bytecode::MutBorrowField(fd_idx));
                function_frame.push()?;
                vec_deque![InferredType::MutableReference(inner_token)]
            } else {
                code.push(Bytecode::ImmBorrowField(fd_idx));
                function_frame.push()?;
                vec_deque![InferredType::Reference(inner_token)]
            }
        }
        Exp::FunctionCall(f, exps) => {
            let mut actuals_tys = vec_deque![];
            for types in compile_expression(context, function_frame, code, *exps)? {
                actuals_tys.push_back(types);
            }
            compile_call(context, function_frame, code, f, actuals_tys)?
        }
        Exp::ExprList(exps) => {
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
    Ok(match call {
        FunctionCall::Builtin(function) => {
            match function {
                Builtin::GetTxnGasUnitPrice => {
                    code.push(Bytecode::GetTxnGasUnitPrice);
                    function_frame.push()?;
                    vec_deque![InferredType::U64]
                }
                Builtin::GetTxnMaxGasUnits => {
                    code.push(Bytecode::GetTxnMaxGasUnits);
                    function_frame.push()?;
                    vec_deque![InferredType::U64]
                }
                Builtin::GetGasRemaining => {
                    code.push(Bytecode::GetGasRemaining);
                    function_frame.push()?;
                    vec_deque![InferredType::U64]
                }
                Builtin::GetTxnSender => {
                    code.push(Bytecode::GetTxnSenderAddress);
                    function_frame.push()?;
                    vec_deque![InferredType::Address]
                }
                Builtin::Exists(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    code.push(Bytecode::Exists(def_idx, type_actuals_id));
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![InferredType::Bool]
                }
                Builtin::BorrowGlobal(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    code.push(Bytecode::BorrowGlobal(def_idx, type_actuals_id));
                    function_frame.pop()?;
                    function_frame.push()?;

                    let self_name = ModuleName::new(ModuleName::SELF.to_string());
                    let ident = QualifiedStructIdent {
                        module: self_name,
                        name,
                    };
                    let sh_idx = context.struct_handle_index(ident)?;
                    vec_deque![InferredType::MutableReference(Box::new(
                        InferredType::Struct(sh_idx)
                    ),)]
                }
                Builtin::CreateAccount => {
                    code.push(Bytecode::CreateAccount);
                    function_frame.pop()?;
                    function_frame.push()?;
                    vec_deque![]
                }
                Builtin::MoveFrom(name, tys) => {
                    let tokens = LocalsSignature(compile_types(context, &tys)?);
                    let type_actuals_id = context.locals_signature_index(tokens)?;
                    let def_idx = context.struct_definition_index(&name)?;
                    code.push(Bytecode::MoveFrom(def_idx, type_actuals_id));
                    function_frame.pop()?; // pop the address
                    function_frame.push()?; // push the return value

                    let self_name = ModuleName::new(ModuleName::SELF.to_string());
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

                    code.push(Bytecode::MoveToSender(def_idx, type_actuals_id));
                    function_frame.push()?;
                    vec_deque![]
                }
                Builtin::GetTxnSequenceNumber => {
                    code.push(Bytecode::GetTxnSequenceNumber);
                    function_frame.push()?;
                    vec_deque![InferredType::U64]
                }
                Builtin::GetTxnPublicKey => {
                    code.push(Bytecode::GetTxnPublicKey);
                    function_frame.push()?;
                    vec_deque![InferredType::ByteArray]
                }
                Builtin::Freeze => {
                    code.push(Bytecode::FreezeRef);
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
            }
        }
        FunctionCall::ModuleFunctionCall {
            module,
            name,
            type_actuals,
        } => {
            let tokens = LocalsSignature(compile_types(context, &type_actuals)?);
            let type_actuals_id = context.locals_signature_index(tokens)?;
            let fh_idx = context.function_handle(module.clone(), name.clone())?.1;
            let call = Bytecode::Call(fh_idx, type_actuals_id);
            code.push(call);
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
