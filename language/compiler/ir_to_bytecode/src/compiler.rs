// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    parser::ast::{
        BinOp, Block, Builtin, Cmd, CopyableVal, Exp, Field, Fields, Function, FunctionBody,
        FunctionCall, FunctionSignature as AstFunctionSignature, FunctionVisibility, IfElse, Kind,
        Loop, ModuleDefinition, ModuleIdent, ModuleName, Program, Statement,
        StructDefinition as MoveStruct, Tag, Type, UnaryOp, Var, Var_, While,
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
use types::{account_address::AccountAddress, byte_array::ByteArray};
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeUnit, CompiledModule,
        CompiledModuleMut, CompiledProgram, CompiledScriptMut, FieldDefinition,
        FieldDefinitionIndex, FunctionDefinition, FunctionDefinitionIndex, FunctionHandle,
        FunctionHandleIndex, FunctionSignature, FunctionSignatureIndex, LocalsSignature,
        LocalsSignatureIndex, MemberCount, ModuleHandle, ModuleHandleIndex, SignatureToken,
        StringPoolIndex, StructDefinition, StructDefinitionIndex, StructHandle, StructHandleIndex,
        TableIndex, TypeSignature, TypeSignatureIndex, SELF_MODULE_NAME,
    },
    printers::TableAccess,
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
            S::Struct(si) => I::Struct(*si),
            S::Reference(s_inner) => {
                let i_inner = Self::from_signature_token(&*s_inner);
                I::Reference(Box::new(i_inner))
            }
            S::MutableReference(s_inner) => {
                let i_inner = Self::from_signature_token(&*s_inner);
                I::MutableReference(Box::new(i_inner))
            }
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
    // The max stack depth of the file_format is set as u16
    // Theoritically, we could use a BigInt here, but that is probably overkill for any testing
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

type ModuleIndex = u8; // 2^8 max number of modules per compilation
type FieldMap = HashMap<String, FieldDefinitionIndex>;

/// Global scope where all modules are imported.
/// This is a read only scope and holds the compilation references.
/// The handles are in the scope of the compilation unit, the def in the scope of the imported unit.
/// Those maps also help in resolution of fields and functions, which is name based in the IR
/// (as opposed to signature based in the VM - field type, function signature)
#[derive(Debug)]
struct CompilationScope<'a> {
    // maps from handles in the compilation unit (reference) to the definitions
    // in the imported unit (definitions).
    imported_modules: HashMap<String, (ModuleIndex, ModuleHandleIndex)>,
    function_definitions:
        HashMap<ModuleHandleIndex, HashMap<String, (ModuleIndex, FunctionDefinitionIndex)>>,
    // imported modules (external contracts you compile
    pub modules: Vec<&'a CompiledModule>,
}

impl<'a> CompilationScope<'a> {
    fn new(modules: impl IntoIterator<Item = &'a CompiledModule>) -> Self {
        CompilationScope {
            imported_modules: HashMap::new(),
            function_definitions: HashMap::new(),
            modules: modules.into_iter().collect(),
        }
    }

    // TODO: Change `module_name` to be something like a ModuleIdent when we have better data
    // structure for dependency modules input.
    fn link_module(
        &mut self,
        import_name: &str,
        module_name: &str,
        mh_idx: ModuleHandleIndex,
    ) -> Result<()> {
        for idx in 0..self.modules.len() {
            if self.modules[idx].name() == module_name {
                self.imported_modules
                    .insert(import_name.to_string(), (idx as u8, mh_idx));
                return Ok(());
            }
        }
        bail!("can't find module {} in dependency list", import_name);
    }

    fn link_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        fd_idx: FunctionDefinitionIndex,
    ) -> Result<()> {
        let (module_index, mh_idx) = self.imported_modules[module_name];
        let func_map = self
            .function_definitions
            .entry(mh_idx)
            .or_insert_with(HashMap::new);
        func_map.insert(function_name.to_string(), (module_index, fd_idx));
        Ok(())
    }

    fn get_imported_module_impl(&self, name: &str) -> Result<(&CompiledModule, ModuleHandleIndex)> {
        match self.imported_modules.get(name) {
            None => bail!("no module named {}", name),
            Some((module_index, mh_idx)) => Ok((self.modules[*module_index as usize], *mh_idx)),
        }
    }

    fn get_imported_module(&self, name: &str) -> Result<&CompiledModule> {
        let (module, _) = self.get_imported_module_impl(name)?;
        Ok(module)
    }

    fn get_imported_module_handle(&self, name: &str) -> Result<ModuleHandleIndex> {
        let (_, mh_idx) = self.get_imported_module_impl(name)?;
        Ok(mh_idx)
    }

    fn get_function_signature(
        &self,
        mh_idx: ModuleHandleIndex,
        name: &str,
    ) -> Result<&FunctionSignature> {
        let func_map = match self.function_definitions.get(&mh_idx) {
            None => bail!(
                "no module handle index {} in function definition table",
                mh_idx
            ),
            Some(func_map) => func_map,
        };

        let (module_index, fd_idx) = match func_map.get(name) {
            None => bail!("no function {} in module {}", name, mh_idx),
            Some(res) => res,
        };

        let module = &self.modules[*module_index as usize];

        let fh_idx = module.function_def_at(*fd_idx).function;
        let fh = module.function_handle_at(fh_idx);
        Ok(module.function_signature_at(fh.signature))
    }
}

#[derive(Debug)]
struct ModuleScope<'a> {
    // parent scope, the global module scope
    pub compilation_scope: CompilationScope<'a>,
    // builds a struct map based on handles and signatures
    struct_definitions: HashMap<String, (bool, StructDefinitionIndex)>,
    field_definitions: HashMap<StructHandleIndex, FieldMap>,
    function_definitions: HashMap<String, FunctionDefinitionIndex>,
    // the module being compiled
    pub module: CompiledModuleMut,
}

impl<'a> ModuleScope<'a> {
    fn new(
        module: CompiledModuleMut,
        modules: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> Self {
        ModuleScope {
            compilation_scope: CompilationScope::new(modules),
            struct_definitions: HashMap::new(),
            field_definitions: HashMap::new(),
            function_definitions: HashMap::new(),
            module,
        }
    }
}

#[derive(Debug)]
struct ScriptScope<'a> {
    // parent scope, the global module scope
    compilation_scope: CompilationScope<'a>,
    pub script: CompiledScriptMut,
}

impl<'a> ScriptScope<'a> {
    fn new(
        script: CompiledScriptMut,
        modules: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> Self {
        ScriptScope {
            compilation_scope: CompilationScope::new(modules),
            script,
        }
    }
}

fn add_item<U>(item: U, table: &mut Vec<U>) -> Result<TableIndex> {
    let size = table.len();
    if size >= TABLE_MAX_SIZE {
        bail!("Max table size reached!")
    }
    table.push(item);
    Ok(size as TableIndex)
}

trait Scope {
    fn make_string(&mut self, s: String) -> Result<StringPoolIndex>;
    fn make_byte_array(&mut self, buf: ByteArray) -> Result<ByteArrayPoolIndex>;
    fn make_address(&mut self, s: AccountAddress) -> Result<AddressPoolIndex>;
    fn make_type_signature(&mut self, s: TypeSignature) -> Result<TypeSignatureIndex>;
    fn make_function_signature(&mut self, s: FunctionSignature) -> Result<FunctionSignatureIndex>;
    fn make_locals_signature(&mut self, s: LocalsSignature) -> Result<LocalsSignatureIndex>;

    fn make_module_handle(
        &mut self,
        addr_idx: AddressPoolIndex,
        name_idx: StringPoolIndex,
    ) -> Result<ModuleHandleIndex>;
    fn make_struct_handle(
        &mut self,
        module_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        is_resource: bool,
    ) -> Result<StructHandleIndex>;
    fn make_function_handle(
        &mut self,
        mh_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        sig_idx: FunctionSignatureIndex,
    ) -> Result<FunctionHandleIndex>;

    fn publish_struct_def(
        &mut self,
        name: &str,
        is_resource: bool,
        struct_def: StructDefinition,
    ) -> Result<StructDefinitionIndex>;
    fn publish_function_def(
        &mut self,
        function_def: FunctionDefinition,
    ) -> Result<FunctionDefinitionIndex>;
    fn publish_field_def(&mut self, field_def: FieldDefinition) -> Result<FieldDefinitionIndex>;
    fn publish_code(&mut self, name: &str, code: CodeUnit) -> Result<()>;

    fn link_module(
        &mut self,
        import_name: &str,
        module_name: &str,
        mh_idx: ModuleHandleIndex,
    ) -> Result<()>;
    fn link_field(
        &mut self,
        sh_idx: StructHandleIndex,
        name: &str,
        fd_idx: FieldDefinitionIndex,
    ) -> Result<()>;
    fn link_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        fd_idx: FunctionDefinitionIndex,
    ) -> Result<()>;

    fn get_imported_module(&self, name: &str) -> Result<&CompiledModule>;
    fn get_imported_module_handle(&self, name: &str) -> Result<ModuleHandleIndex>;
    fn get_next_field_definition_index(&mut self) -> Result<FieldDefinitionIndex>;
    fn get_struct_def(&self, name: &str) -> Result<(bool, StructDefinitionIndex)>;
    fn get_field_def(&self, sh_idx: StructHandleIndex, name: &str) -> Result<FieldDefinitionIndex>;
    fn get_field_type(&self, sh_idx: StructHandleIndex, name: &str) -> Result<&TypeSignature>;
    fn get_function_signature(
        &self,
        mh_idx: ModuleHandleIndex,
        name: &str,
    ) -> Result<&FunctionSignature>;

    fn get_name(&self) -> Result<String>;
}

impl<'a> Scope for ModuleScope<'a> {
    fn make_string(&mut self, s: String) -> Result<StringPoolIndex> {
        add_item(s, &mut self.module.string_pool).map(StringPoolIndex::new)
    }

    fn make_byte_array(&mut self, buf: ByteArray) -> Result<ByteArrayPoolIndex> {
        add_item(buf, &mut self.module.byte_array_pool).map(ByteArrayPoolIndex::new)
    }

    fn make_address(&mut self, addr: AccountAddress) -> Result<AddressPoolIndex> {
        add_item(addr, &mut self.module.address_pool).map(AddressPoolIndex::new)
    }

    fn make_type_signature(&mut self, sig: TypeSignature) -> Result<TypeSignatureIndex> {
        add_item(sig, &mut self.module.type_signatures).map(TypeSignatureIndex::new)
    }

    fn make_function_signature(
        &mut self,
        sig: FunctionSignature,
    ) -> Result<FunctionSignatureIndex> {
        add_item(sig, &mut self.module.function_signatures).map(FunctionSignatureIndex::new)
    }

    fn make_locals_signature(&mut self, sig: LocalsSignature) -> Result<LocalsSignatureIndex> {
        add_item(sig, &mut self.module.locals_signatures).map(LocalsSignatureIndex::new)
    }

    fn make_module_handle(
        &mut self,
        addr_idx: AddressPoolIndex,
        name_idx: StringPoolIndex,
    ) -> Result<ModuleHandleIndex> {
        let mh = ModuleHandle {
            address: addr_idx,
            name: name_idx,
        };
        let size = self.module.module_handles.len();
        if size >= STRUCTS_MAX_SIZE {
            bail!("Max table size reached!")
        }
        self.module.module_handles.push(mh);
        Ok(ModuleHandleIndex::new(size as u16))
    }

    fn make_struct_handle(
        &mut self,
        module_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        is_resource: bool,
    ) -> Result<StructHandleIndex> {
        let sh = StructHandle {
            module: module_idx,
            name: name_idx,
            is_resource,
        };
        let size = self.module.struct_handles.len();
        if size >= TABLE_MAX_SIZE {
            bail!("Max table size reached!")
        }
        self.module.struct_handles.push(sh);
        Ok(StructHandleIndex::new(size as u16))
    }

    fn make_function_handle(
        &mut self,
        mh_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        sig_idx: FunctionSignatureIndex,
    ) -> Result<FunctionHandleIndex> {
        let fh = FunctionHandle {
            module: mh_idx,
            name: name_idx,
            signature: sig_idx,
        };
        add_item(fh, &mut self.module.function_handles).map(FunctionHandleIndex::new)
    }

    fn publish_struct_def(
        &mut self,
        name: &str,
        is_resource: bool,
        struct_def: StructDefinition,
    ) -> Result<StructDefinitionIndex> {
        let idx = self.module.struct_defs.len();
        if idx >= STRUCTS_MAX_SIZE {
            bail!("Max number of structs reached")
        }
        let sd_idx = StructDefinitionIndex::new(idx as TableIndex);
        self.module.struct_defs.push(struct_def);
        self.struct_definitions
            .insert(name.to_string(), (is_resource, sd_idx));
        Ok(sd_idx)
    }

    fn publish_function_def(
        &mut self,
        function_def: FunctionDefinition,
    ) -> Result<FunctionDefinitionIndex> {
        let idx = self.module.function_defs.len();
        if idx >= FUNCTIONS_MAX_SIZE {
            bail!("Max number of functions reached")
        }
        let fd_idx = FunctionDefinitionIndex::new(idx as TableIndex);
        self.module.function_defs.push(function_def);
        Ok(fd_idx)
    }

    /// Compute the index of the next field definition
    fn get_next_field_definition_index(&mut self) -> Result<FieldDefinitionIndex> {
        let idx = self.module.field_defs.len();
        if idx >= FIELDS_MAX_SIZE {
            bail!("Max number of fields reached")
        }
        let fd_idx = FieldDefinitionIndex::new(idx as TableIndex);
        Ok(fd_idx)
    }

    fn publish_field_def(&mut self, field_def: FieldDefinition) -> Result<FieldDefinitionIndex> {
        let fd_idx = self.get_next_field_definition_index()?;
        self.module.field_defs.push(field_def);
        Ok(fd_idx)
    }

    fn publish_code(&mut self, name: &str, code: CodeUnit) -> Result<()> {
        let fd_idx = match self.function_definitions.get(name) {
            None => bail!("Cannot find function {}", name),
            Some(def_idx) => def_idx,
        };
        let func_def = match self.module.function_defs.get_mut(fd_idx.0 as usize) {
            None => bail!("Cannot find function def for {}", name),
            Some(func_def) => func_def,
        };
        func_def.code = code;
        Ok(())
    }

    fn link_module(
        &mut self,
        import_name: &str,
        module_name: &str,
        mh_idx: ModuleHandleIndex,
    ) -> Result<()> {
        self.compilation_scope
            .link_module(import_name, module_name, mh_idx)
    }

    fn link_field(
        &mut self,
        sh_idx: StructHandleIndex,
        name: &str,
        fd_idx: FieldDefinitionIndex,
    ) -> Result<()> {
        let field_map = self
            .field_definitions
            .entry(sh_idx)
            .or_insert_with(HashMap::new);
        field_map.insert(name.to_string(), fd_idx);
        Ok(())
    }

    fn link_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        fd_idx: FunctionDefinitionIndex,
    ) -> Result<()> {
        if module_name.is_empty() {
            self.function_definitions
                .insert(function_name.to_string(), fd_idx);
            Ok(())
        } else {
            self.compilation_scope
                .link_function(module_name, function_name, fd_idx)
        }
    }

    fn get_imported_module(&self, name: &str) -> Result<&CompiledModule> {
        self.compilation_scope.get_imported_module(name)
    }

    fn get_imported_module_handle(&self, name: &str) -> Result<ModuleHandleIndex> {
        self.compilation_scope.get_imported_module_handle(name)
    }

    fn get_struct_def(&self, name: &str) -> Result<(bool, StructDefinitionIndex)> {
        match self.struct_definitions.get(name) {
            None => bail!("No struct definition for name {}", name),
            Some((is_resource, def_idx)) => Ok((*is_resource, *def_idx)),
        }
    }

    fn get_field_def(&self, sh_idx: StructHandleIndex, name: &str) -> Result<FieldDefinitionIndex> {
        let field_map = match self.field_definitions.get(&sh_idx) {
            None => bail!("no struct handle index {}", sh_idx),
            Some(map) => map,
        };
        match field_map.get(name) {
            None => bail!("no field {} in struct handle index {}", name, sh_idx),
            Some(def_idx) => Ok(*def_idx),
        }
    }

    fn get_field_type(&self, sh_idx: StructHandleIndex, name: &str) -> Result<&TypeSignature> {
        let fd_idx = self.get_field_def(sh_idx, name)?;
        let sig_idx = match self.module.field_defs.get(fd_idx.0 as usize) {
            None => bail!(
                "No field definition index {} in field definition table",
                fd_idx
            ),
            Some(field_def) => field_def.signature,
        };
        match self.module.type_signatures.get(sig_idx.0 as usize) {
            None => bail!("missing type signature index {}", sig_idx),
            Some(sig) => Ok(sig),
        }
    }

    fn get_function_signature(
        &self,
        mh_idx: ModuleHandleIndex,
        name: &str,
    ) -> Result<&FunctionSignature> {
        // Call into an external module.
        if mh_idx.0 != 0 {
            return self.compilation_scope.get_function_signature(mh_idx, name);
        }

        let fd_idx = match self.function_definitions.get(name) {
            None => bail!("no function {} in module {}", name, mh_idx),
            Some(def_idx) => def_idx,
        };

        let fh_idx = match self.module.function_defs.get(fd_idx.0 as usize) {
            None => bail!(
                "No function definition index {} in function definition table",
                fd_idx
            ),
            Some(function_def) => function_def.function,
        };

        let fh = self.module.get_function_at(fh_idx)?;
        self.module.get_function_signature_at(fh.signature)
    }

    fn get_name(&self) -> Result<String> {
        let mh = self.module.get_module_at(ModuleHandleIndex::new(0))?;
        let name_ref = self.module.get_string_at(mh.name)?;
        Ok(name_ref.clone())
    }
}

impl<'a> Scope for ScriptScope<'a> {
    fn make_string(&mut self, s: String) -> Result<StringPoolIndex> {
        add_item(s, &mut self.script.string_pool).map(StringPoolIndex::new)
    }

    fn make_byte_array(&mut self, buf: ByteArray) -> Result<ByteArrayPoolIndex> {
        add_item(buf, &mut self.script.byte_array_pool).map(ByteArrayPoolIndex::new)
    }

    fn make_address(&mut self, addr: AccountAddress) -> Result<AddressPoolIndex> {
        add_item(addr, &mut self.script.address_pool).map(AddressPoolIndex::new)
    }

    fn make_type_signature(&mut self, sig: TypeSignature) -> Result<TypeSignatureIndex> {
        add_item(sig, &mut self.script.type_signatures).map(TypeSignatureIndex::new)
    }

    fn make_function_signature(
        &mut self,
        sig: FunctionSignature,
    ) -> Result<FunctionSignatureIndex> {
        add_item(sig, &mut self.script.function_signatures).map(FunctionSignatureIndex::new)
    }

    fn make_locals_signature(&mut self, sig: LocalsSignature) -> Result<LocalsSignatureIndex> {
        add_item(sig, &mut self.script.locals_signatures).map(LocalsSignatureIndex::new)
    }

    fn make_module_handle(
        &mut self,
        addr_idx: AddressPoolIndex,
        name_idx: StringPoolIndex,
    ) -> Result<ModuleHandleIndex> {
        let mh = ModuleHandle {
            address: addr_idx,
            name: name_idx,
        };
        let size = self.script.module_handles.len();
        if size >= STRUCTS_MAX_SIZE {
            bail!("Max table size reached!")
        }
        self.script.module_handles.push(mh);
        Ok(ModuleHandleIndex::new(size as u16))
    }

    fn make_struct_handle(
        &mut self,
        module_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        is_resource: bool,
    ) -> Result<StructHandleIndex> {
        let sh = StructHandle {
            module: module_idx,
            name: name_idx,
            is_resource,
        };
        let size = self.script.struct_handles.len();
        if size >= TABLE_MAX_SIZE {
            bail!("Max table size reached!")
        }
        self.script.struct_handles.push(sh);
        Ok(StructHandleIndex::new(size as u16))
    }

    fn make_function_handle(
        &mut self,
        mh_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        sig_idx: FunctionSignatureIndex,
    ) -> Result<FunctionHandleIndex> {
        let fh = FunctionHandle {
            module: mh_idx,
            name: name_idx,
            signature: sig_idx,
        };
        add_item(fh, &mut self.script.function_handles).map(FunctionHandleIndex::new)
    }

    fn publish_struct_def(
        &mut self,
        _name: &str,
        _is_resource: bool,
        _struct_def: StructDefinition,
    ) -> Result<StructDefinitionIndex> {
        bail!("Cannot publish structs in scripts")
    }

    fn publish_function_def(
        &mut self,
        _function_def: FunctionDefinition,
    ) -> Result<FunctionDefinitionIndex> {
        bail!("Cannot publish functions in scripts")
    }

    fn publish_field_def(&mut self, _field_def: FieldDefinition) -> Result<FieldDefinitionIndex> {
        bail!("Cannot publish fields in scripts")
    }

    fn publish_code(&mut self, _name: &str, _code: CodeUnit) -> Result<()> {
        bail!("No function definitions in scripts")
    }

    fn link_module(
        &mut self,
        import_name: &str,
        module_name: &str,
        mh_idx: ModuleHandleIndex,
    ) -> Result<()> {
        self.compilation_scope
            .link_module(import_name, module_name, mh_idx)
    }

    fn link_field(
        &mut self,
        _sh_idx: StructHandleIndex,
        _name: &str,
        _fd_idx: FieldDefinitionIndex,
    ) -> Result<()> {
        bail!("no field linking in scripts");
    }

    fn link_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        fd_idx: FunctionDefinitionIndex,
    ) -> Result<()> {
        self.compilation_scope
            .link_function(module_name, function_name, fd_idx)
    }

    fn get_imported_module(&self, name: &str) -> Result<&CompiledModule> {
        self.compilation_scope.get_imported_module(name)
    }

    fn get_imported_module_handle(&self, name: &str) -> Result<ModuleHandleIndex> {
        self.compilation_scope.get_imported_module_handle(name)
    }

    fn get_next_field_definition_index(&mut self) -> Result<FieldDefinitionIndex> {
        bail!("no field definition referencing in scripts")
    }

    fn get_struct_def(&self, _name: &str) -> Result<(bool, StructDefinitionIndex)> {
        bail!("no struct definition referencing in scripts")
    }

    fn get_field_def(
        &self,
        _sh_idx: StructHandleIndex,
        _name: &str,
    ) -> Result<FieldDefinitionIndex> {
        bail!("no field definition referencing in scripts")
    }

    fn get_field_type(&self, _sh_idx: StructHandleIndex, _name: &str) -> Result<&TypeSignature> {
        bail!("no field type referencing in scripts")
    }

    fn get_function_signature(
        &self,
        mh_idx: ModuleHandleIndex,
        name: &str,
    ) -> Result<&FunctionSignature> {
        self.compilation_scope.get_function_signature(mh_idx, name)
    }

    fn get_name(&self) -> Result<String> {
        bail!("no name for scripts")
    }
}

struct Compiler<S: Scope + Sized> {
    // identity maps
    // Map a handle to its position in its table
    // TODO: those could be expressed as references and it would make for better code.
    // For now this is easier to do and those are intended to be "primitive" values so we'll get
    // back to this...
    modules: HashMap<ModuleHandle, ModuleHandleIndex>,
    structs: HashMap<StructHandle, StructHandleIndex>,
    functions: HashMap<FunctionHandle, FunctionHandleIndex>,
    strings: HashMap<String, StringPoolIndex>,
    byte_arrays: HashMap<ByteArray, ByteArrayPoolIndex>,
    addresses: HashMap<AccountAddress, AddressPoolIndex>,
    type_signatures: HashMap<TypeSignature, TypeSignatureIndex>,
    function_signatures: HashMap<FunctionSignature, FunctionSignatureIndex>,
    locals_signatures: HashMap<LocalsSignature, LocalsSignatureIndex>,
    // resolution scope
    scope: S,
}

const STRUCTS_MAX_SIZE: usize = TABLE_MAX_SIZE;
const FIELDS_MAX_SIZE: usize = TABLE_MAX_SIZE;
const FUNCTIONS_MAX_SIZE: usize = TABLE_MAX_SIZE;
const TABLE_MAX_SIZE: usize = u16::max_value() as usize;

//
// Module/Contract compilation
//

/// Compile a module
pub fn compile_module<'a, T: 'a + ModuleAccess>(
    address: &AccountAddress,
    module: &ModuleDefinition,
    modules: impl IntoIterator<Item = &'a T>,
) -> Result<CompiledModule> {
    // Convert to &CompiledModule as that's what's used throughout internally.
    let modules = modules.into_iter().map(|module| module.as_module());

    let compiled_module = CompiledModuleMut::default();
    let scope = ModuleScope::new(compiled_module, modules);
    // This is separate to avoid unnecessary code gen due to monomorphization.
    compile_module_impl(address, module, scope)
}

fn compile_module_impl<'a>(
    address: &AccountAddress,
    module: &ModuleDefinition,
    scope: ModuleScope<'a>,
) -> Result<CompiledModule> {
    let mut compiler = Compiler::new(scope);
    let addr_idx = compiler.make_address(&address)?;
    let name_idx = compiler.make_string(module.name.name_ref())?;
    let mh_idx = compiler.make_module_handle(addr_idx, name_idx)?;
    assert!(mh_idx.0 == 0);
    for import in &module.imports {
        compiler.import_module(
            match &import.ident {
                ModuleIdent::Transaction(_) => address,
                ModuleIdent::Qualified(id) => &id.address,
            },
            &import.ident.get_name().name_ref(),
            &import.alias,
        )?;
    }
    for struct_ in &module.structs {
        compiler.define_struct(mh_idx, &struct_)?;
    }
    for (name, function) in &module.functions {
        compiler.define_function(name.name_ref(), &function)?;
    }
    for (name, function) in &module.functions {
        match &function.body {
            FunctionBody::Move { locals, code } => {
                debug!("compile move function: {} {}", name, &function.signature);
                let compiled_code =
                    compiler.compile_function(&function.signature.formals, locals, code)?;
                compiler
                    .scope
                    .publish_code(name.name_ref(), compiled_code)?;
            }
            FunctionBody::Native => (),
        }
    }
    compiler
        .scope
        .module
        .freeze()
        .map_err(|errs| InternalCompilerError::BoundsCheckErrors(errs).into())
}

//
// Transaction/Script compilation
//

/// Compile a transaction program
pub fn compile_program<'a, T: 'a + ModuleAccess>(
    address: &AccountAddress,
    program: &Program,
    deps: impl IntoIterator<Item = &'a T>,
) -> Result<CompiledProgram> {
    // Normalize into a Vec<&CompiledModule>.
    let deps: Vec<&CompiledModule> = deps.into_iter().map(|dep| dep.as_module()).collect();

    // This is separate to avoid unnecessary code gen due to monomorphization.
    compile_program_impl(address, program, deps)
}

fn compile_program_impl(
    address: &AccountAddress,
    program: &Program,
    deps: Vec<&CompiledModule>,
) -> Result<CompiledProgram> {
    // Compile modules in the program
    let mut modules = vec![];
    for m in &program.modules {
        let module = {
            let deps = deps.iter().copied().chain(&modules);
            compile_module(address, &m, deps)?
        };
        modules.push(module);
    }

    // Compile transaction script
    let func_def: FunctionDefinition;
    let compiled_script = CompiledScriptMut::default();

    let scope = {
        let deps = deps.iter().copied().chain(&modules);
        ScriptScope::new(compiled_script, deps)
    };
    let mut compiler = Compiler::new(scope);
    let addr_idx = compiler.make_address(&address)?;
    let name_idx = compiler.make_string(SELF_MODULE_NAME)?;
    let mh_idx = compiler.make_module_handle(addr_idx, name_idx)?;
    assert!(mh_idx.0 == 0);

    for import in &program.script.imports {
        compiler.import_module(
            match &import.ident {
                ModuleIdent::Transaction(_) => address,
                ModuleIdent::Qualified(id) => &id.address,
            },
            &import.ident.get_name().name_ref(),
            &import.alias,
        )?;
    }

    func_def = compiler.compile_main(&program.script.main)?;

    let mut script = compiler.scope.script;
    script.main = func_def;
    let script = match script.freeze() {
        Ok(script) => script,
        Err(errs) => bail_err!(InternalCompilerError::BoundsCheckErrors(errs)),
    };

    Ok(CompiledProgram::new(modules, script))
}

impl<S: Scope + Sized> Compiler<S> {
    fn new(scope: S) -> Self {
        Compiler {
            modules: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            strings: HashMap::new(),
            byte_arrays: HashMap::new(),
            addresses: HashMap::new(),
            type_signatures: HashMap::new(),
            function_signatures: HashMap::new(),
            locals_signatures: HashMap::new(),
            // resolution scope
            scope,
        }
    }

    fn import_module(
        &mut self,
        address: &AccountAddress,
        name: &str,
        module_alias: &ModuleName,
    ) -> Result<()> {
        let addr_idx = self.make_address(address)?;
        let name_idx = self.make_string(&name)?;
        let mh_idx = self.make_module_handle(addr_idx, name_idx)?;
        self.scope
            .link_module(module_alias.name_ref(), name, mh_idx)
    }

    fn import_signature_token(
        &mut self,
        module_name: &str,
        sig_token: SignatureToken,
    ) -> Result<SignatureToken> {
        match sig_token {
            SignatureToken::Bool
            | SignatureToken::U64
            | SignatureToken::String
            | SignatureToken::ByteArray
            | SignatureToken::Address => Ok(sig_token),
            SignatureToken::Struct(sh_idx) => {
                let (defining_module_name, name, is_resource) = {
                    let module = self.scope.get_imported_module(module_name)?;
                    let struct_handle = module.struct_handle_at(sh_idx);
                    let defining_module_handle = module.module_handle_at(struct_handle.module);
                    (
                        module.string_at(defining_module_handle.name),
                        module.string_at(struct_handle.name).to_string(),
                        struct_handle.is_resource,
                    )
                };
                let mh_idx = self
                    .scope
                    .get_imported_module_handle(defining_module_name)?;
                let name_idx = self.make_string(&name)?;
                let local_sh_idx = self.make_struct_handle(mh_idx, name_idx, is_resource)?;
                Ok(SignatureToken::Struct(local_sh_idx))
            }
            SignatureToken::Reference(sub_sig_token) => Ok(SignatureToken::Reference(Box::new(
                self.import_signature_token(module_name, *sub_sig_token)?,
            ))),
            SignatureToken::MutableReference(sub_sig_token) => {
                Ok(SignatureToken::MutableReference(Box::new(
                    self.import_signature_token(module_name, *sub_sig_token)?,
                )))
            }
        }
    }

    fn import_function_signature(
        &mut self,
        module_name: &str,
        func_sig: FunctionSignature,
    ) -> Result<FunctionSignature> {
        if module_name == ModuleName::SELF {
            Ok(func_sig)
        } else {
            let mut return_types = Vec::<SignatureToken>::new();
            let mut arg_types = Vec::<SignatureToken>::new();
            for e in func_sig.return_types {
                return_types.push(self.import_signature_token(module_name, e)?);
            }
            for e in func_sig.arg_types {
                arg_types.push(self.import_signature_token(module_name, e)?);
            }
            Ok(FunctionSignature {
                return_types,
                arg_types,
            })
        }
    }

    fn define_struct(&mut self, module_idx: ModuleHandleIndex, struct_: &MoveStruct) -> Result<()> {
        let name = struct_.name.name_ref();
        let name_idx = self.make_string(name)?;
        let sh_idx = self.make_struct_handle(module_idx, name_idx, struct_.resource_kind)?;
        let struct_def = self.define_fields(sh_idx, &struct_.fields)?;
        self.scope
            .publish_struct_def(name, struct_.resource_kind, struct_def)?;
        Ok(())
    }

    fn define_fields(
        &mut self,
        sh_idx: StructHandleIndex,
        fields: &Fields<Type>,
    ) -> Result<StructDefinition> {
        let field_count = fields.len();
        let struct_def = StructDefinition {
            struct_handle: sh_idx,
            field_count: (field_count as MemberCount),
            fields: self.scope.get_next_field_definition_index()?,
        };

        if field_count > FIELDS_MAX_SIZE {
            bail!("too many fields {}", struct_def.struct_handle)
        }

        for field in fields {
            let field_name = field.0.name();
            let field_type = field.1;
            self.publish_field(struct_def.struct_handle, field_name, field_type)?;
        }
        Ok(struct_def)
    }

    // Compile a main function in a Script.
    fn compile_main(&mut self, main: &Function) -> Result<FunctionDefinition> {
        // make main entry point
        let main_name = "main".to_string();
        let main_name_idx = self.make_string(&main_name)?;
        let signature = self.build_function_signature(&main.signature)?;
        let sig_idx = self.make_function_signature(&signature)?;
        let fh_idx =
            self.make_function_handle(ModuleHandleIndex::new(0), main_name_idx, sig_idx)?;
        // compile script
        let code = match &main.body {
            FunctionBody::Move { code, locals } => {
                self.compile_function(&main.signature.formals, locals, code)?
            }
            FunctionBody::Native => bail!("main() cannot be a native function"),
        };
        Ok(FunctionDefinition {
            function: fh_idx,
            flags: CodeUnit::PUBLIC,
            code,
        })
    }

    //
    // Reference tables filling: string, byte_array, address, signature, *handles
    //

    fn make_string(&mut self, s: &str) -> Result<StringPoolIndex> {
        let mut empty = false;
        let idx;
        {
            let str_idx = self.strings.get(s);
            idx = match str_idx {
                None => {
                    empty = true;
                    self.scope.make_string(s.to_string())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.strings.insert(s.to_string(), idx);
        }
        Ok(idx)
    }

    fn make_byte_array(&mut self, buf: &ByteArray) -> Result<ByteArrayPoolIndex> {
        let mut empty = false;
        let idx;
        {
            let byte_array_idx = self.byte_arrays.get(buf);
            idx = match byte_array_idx {
                None => {
                    empty = true;
                    self.scope.make_byte_array(buf.clone())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.byte_arrays.insert(buf.clone(), idx);
        }
        Ok(idx)
    }

    fn make_address(&mut self, addr: &AccountAddress) -> Result<AddressPoolIndex> {
        let mut empty = false;
        let idx;
        {
            let addr_idx = self.addresses.get(addr);
            idx = match addr_idx {
                None => {
                    empty = true;
                    self.scope.make_address(addr.clone())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.addresses.insert(addr.clone(), idx);
        }
        Ok(idx)
    }

    fn make_type_signature(&mut self, _type: &Type) -> Result<TypeSignatureIndex> {
        let signature = self.build_type_signature(_type)?;
        let mut empty = false;
        let idx;
        {
            let sig_idx = self.type_signatures.get(&signature);
            idx = match sig_idx {
                None => {
                    empty = true;
                    self.scope.make_type_signature(signature.clone())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.type_signatures.insert(signature.clone(), idx);
        }
        Ok(idx)
    }

    fn make_function_signature(
        &mut self,
        signature: &FunctionSignature,
    ) -> Result<FunctionSignatureIndex> {
        let mut empty = false;
        let idx;
        {
            let sig_idx = self.function_signatures.get(&signature);
            idx = match sig_idx {
                None => {
                    empty = true;
                    self.scope.make_function_signature(signature.clone())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.function_signatures.insert(signature.clone(), idx);
        }
        Ok(idx)
    }

    fn make_locals_signature(
        &mut self,
        signature: &LocalsSignature,
    ) -> Result<LocalsSignatureIndex> {
        let mut empty = false;
        let idx;
        {
            let sig_idx = self.locals_signatures.get(signature);
            idx = match sig_idx {
                None => {
                    empty = true;
                    self.scope.make_locals_signature(signature.clone())?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.locals_signatures.insert(signature.clone(), idx);
        }
        Ok(idx)
    }

    fn make_module_handle(
        &mut self,
        addr_idx: AddressPoolIndex,
        name_idx: StringPoolIndex,
    ) -> Result<ModuleHandleIndex> {
        let mh = ModuleHandle {
            address: addr_idx,
            name: name_idx,
        };
        let mut empty = false;
        let idx;
        {
            let mh_idx = self.modules.get(&mh);
            idx = match mh_idx {
                None => {
                    empty = true;
                    self.scope.make_module_handle(addr_idx, name_idx)?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.modules.insert(mh, idx);
        }
        Ok(idx)
    }

    fn make_struct_handle(
        &mut self,
        module_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        is_resource: bool,
    ) -> Result<StructHandleIndex> {
        let sh = StructHandle {
            module: module_idx,
            name: name_idx,
            is_resource,
        };
        Ok(match self.structs.get(&sh) {
            None => {
                let idx = self
                    .scope
                    .make_struct_handle(module_idx, name_idx, is_resource)?;
                self.structs.insert(sh, idx);
                idx
            }
            Some(idx) => *idx,
        })
    }

    fn make_function_handle(
        &mut self,
        mh_idx: ModuleHandleIndex,
        name_idx: StringPoolIndex,
        sig_idx: FunctionSignatureIndex,
    ) -> Result<FunctionHandleIndex> {
        let fh = FunctionHandle {
            module: mh_idx,
            name: name_idx,
            signature: sig_idx,
        };
        let mut empty = false;
        let idx;
        {
            let fh_idx = self.functions.get(&fh);
            idx = match fh_idx {
                None => {
                    empty = true;
                    self.scope.make_function_handle(mh_idx, name_idx, sig_idx)?
                }
                Some(idx) => *idx,
            };
        }
        if empty {
            self.functions.insert(fh, idx);
        }
        Ok(idx)
    }

    //
    // Create definitions, this is effectively only used when compiling modules
    //

    fn publish_field(
        &mut self,
        sh_idx: StructHandleIndex,
        name: &str,
        sig: &Type,
    ) -> Result<FieldDefinitionIndex> {
        let name_idx = self.make_string(name)?;
        let sig_idx = self.make_type_signature(sig)?;
        let field_def = FieldDefinition {
            struct_: sh_idx,
            name: name_idx,
            signature: sig_idx,
        };
        let fd_idx = self.scope.publish_field_def(field_def)?;
        self.scope.link_field(sh_idx, name, fd_idx)?;
        Ok(fd_idx)
    }

    fn define_function(
        &mut self,
        name: &str,
        function: &Function,
    ) -> Result<FunctionDefinitionIndex> {
        // Use ModuleHandleIndex::new(0) here because module 0 refers to the module being currently
        // compiled.
        let mh = ModuleHandleIndex::new(0);

        let name_idx = self.make_string(name)?;
        let sig = self.build_function_signature(&function.signature)?;
        let sig_idx = self.make_function_signature(&sig)?;
        let fh_idx = self.make_function_handle(mh, name_idx, sig_idx)?;

        let flags = match function.visibility {
            FunctionVisibility::Internal => 0,
            FunctionVisibility::Public => CodeUnit::PUBLIC,
        } | match function.body {
            FunctionBody::Move { .. } => 0,
            FunctionBody::Native => CodeUnit::NATIVE,
        };

        let func_def = FunctionDefinition {
            function: fh_idx,
            flags,
            code: CodeUnit::default(), // TODO: eliminate usage of default
        };

        let fd_idx = self.scope.publish_function_def(func_def)?;
        self.scope.link_function("", name, fd_idx)?;
        Ok(fd_idx)
    }

    //
    // Signature building methods
    //

    fn build_type_signature(&mut self, type_: &Type) -> Result<TypeSignature> {
        let signature_token = self.build_signature_token(type_)?;
        Ok(TypeSignature(signature_token))
    }

    fn build_function_signature(
        &mut self,
        signature: &AstFunctionSignature,
    ) -> Result<FunctionSignature> {
        let mut ret_sig: Vec<SignatureToken> = Vec::new();
        for t in &signature.return_type {
            ret_sig.push(self.build_signature_token(&t)?);
        }
        let mut arg_sig: Vec<SignatureToken> = Vec::new();
        for formal in &signature.formals {
            arg_sig.push(self.build_signature_token(&formal.1)?)
        }
        Ok(FunctionSignature {
            return_types: ret_sig,
            arg_types: arg_sig,
        })
    }

    fn build_signature_token(&mut self, type_: &Type) -> Result<SignatureToken> {
        match type_ {
            Type::Normal(kind, tag) => self.build_normal_signature_token(&kind, &tag),
            Type::Reference {
                is_mutable,
                kind,
                tag,
            } => {
                let inner_token = Box::new(self.build_normal_signature_token(kind, tag)?);
                if *is_mutable {
                    Ok(SignatureToken::MutableReference(inner_token))
                } else {
                    Ok(SignatureToken::Reference(inner_token))
                }
            }
        }
    }

    fn build_normal_signature_token(&mut self, kind: &Kind, tag: &Tag) -> Result<SignatureToken> {
        match (kind, tag) {
            (Kind::Value, Tag::Address) => Ok(SignatureToken::Address),
            (Kind::Value, Tag::U64) => Ok(SignatureToken::U64),
            (Kind::Value, Tag::Bool) => Ok(SignatureToken::Bool),
            (Kind::Value, Tag::ByteArray) => Ok(SignatureToken::ByteArray),
            (kind, Tag::Struct(ctype)) => {
                let module_name = &ctype.module().name();
                let module_idx = if self.scope.get_name().is_ok() && module_name == ModuleName::SELF
                {
                    ModuleHandleIndex::new(0)
                } else {
                    self.scope.get_imported_module_handle(module_name)?
                };
                let name_idx = self.make_string(&ctype.name().name_ref())?;
                let is_resource = match kind {
                    Kind::Value => false,
                    Kind::Resource => true,
                };
                let sh_idx = self.make_struct_handle(module_idx, name_idx, is_resource)?;
                Ok(SignatureToken::Struct(sh_idx))
            }
            (Kind::Value, _) => bail!("unknown value type {:?}", tag),
            (Kind::Resource, _) => bail!("unknown resource type {:?}", tag),
        }
    }

    //
    // Code compilation functions, walk the IR and generate bytecodes
    //
    fn compile_function(
        &mut self,
        formals: &[(Var, Type)],
        locals: &[(Var_, Type)],
        body: &Block,
    ) -> Result<CodeUnit> {
        let mut code = CodeUnit::default();
        let mut function_frame = FunctionFrame::new();
        for (var, t) in formals {
            let type_sig = self.build_signature_token(t)?;
            function_frame.define_local(var, type_sig)?;
        }
        for (var_, t) in locals {
            let type_sig = self.build_signature_token(t)?;
            function_frame.define_local(&var_.value, type_sig)?;
        }
        self.compile_block(body, &mut code, &mut function_frame)?;
        let sig_idx = self.make_locals_signature(&function_frame.local_types)?;
        code.locals = sig_idx;
        code.max_stack_size = if function_frame.max_stack_depth < 0 {
            0
        } else if function_frame.max_stack_depth > i64::from(u16::max_value()) {
            u16::max_value()
        } else {
            function_frame.max_stack_depth as u16
        };
        Ok(code)
    }

    fn compile_block(
        &mut self,
        body: &Block,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<ControlFlowInfo> {
        let mut cf_info = ControlFlowInfo {
            reachable_break: false,
            terminal_node: false,
        };
        for stmt in &body.stmts {
            debug!("{}", stmt);
            let stmt_info;
            match stmt {
                Statement::CommandStatement(command) => {
                    stmt_info = self.compile_command(&command, code, function_frame)?;
                    debug!("{:?}", code);
                }
                Statement::WhileStatement(while_) => {
                    // always assume the loop might not be taken
                    stmt_info = self.compile_while(&while_, code, function_frame)?;
                    debug!("{:?}", code);
                }
                Statement::LoopStatement(loop_) => {
                    stmt_info = self.compile_loop(&loop_, code, function_frame)?;
                    debug!("{:?}", code);
                }
                Statement::IfElseStatement(if_else) => {
                    stmt_info = self.compile_if_else(&if_else, code, function_frame)?;
                    debug!("{:?}", code);
                }
                Statement::VerifyStatement(_) | Statement::AssumeStatement(_) => continue,
                Statement::EmptyStatement => continue,
            };
            cf_info = ControlFlowInfo::successor(cf_info, stmt_info);
        }
        Ok(cf_info)
    }

    fn compile_if_else(
        &mut self,
        if_else: &IfElse,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<ControlFlowInfo> {
        self.compile_expression(&if_else.cond, code, function_frame)?;

        let brfalse_ins_loc = code.code.len();
        code.code.push(Bytecode::BrFalse(0)); // placeholder, final branch target replaced later
        function_frame.pop()?;
        let if_cf_info = self.compile_block(&if_else.if_block, code, function_frame)?;

        let mut else_block_location = code.code.len();

        let else_cf_info = match if_else.else_block {
            None => ControlFlowInfo {
                reachable_break: false,
                terminal_node: false,
            },
            Some(ref else_block) => {
                let branch_ins_loc = code.code.len();
                if !if_cf_info.terminal_node {
                    code.code.push(Bytecode::Branch(0)); // placeholder, final branch target replaced later
                    else_block_location += 1;
                }
                let else_cf_info = self.compile_block(else_block, code, function_frame)?;
                if !if_cf_info.terminal_node {
                    code.code[branch_ins_loc] = Bytecode::Branch(code.code.len() as u16);
                }
                else_cf_info
            }
        };

        code.code[brfalse_ins_loc] = Bytecode::BrFalse(else_block_location as u16);

        let cf_info = ControlFlowInfo::join(if_cf_info, else_cf_info);
        Ok(cf_info)
    }

    fn compile_while(
        &mut self,
        while_: &While,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<ControlFlowInfo> {
        let loop_start_loc = code.code.len();
        function_frame.push_loop(loop_start_loc)?;
        self.compile_expression(&while_.cond, code, function_frame)?;

        let brfalse_loc = code.code.len();
        code.code.push(Bytecode::BrFalse(0)); // placeholder, final branch target replaced later
        function_frame.pop()?;

        self.compile_block(&while_.block, code, function_frame)?;
        code.code.push(Bytecode::Branch(loop_start_loc as u16));

        let loop_end_loc = code.code.len() as u16;
        code.code[brfalse_loc] = Bytecode::BrFalse(loop_end_loc);
        let breaks = function_frame.get_loop_breaks()?;
        for i in breaks {
            code.code[*i] = Bytecode::Branch(loop_end_loc);
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
        &mut self,
        loop_: &Loop,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<ControlFlowInfo> {
        let loop_start_loc = code.code.len();
        function_frame.push_loop(loop_start_loc)?;

        let body_cf_info = self.compile_block(&loop_.block, code, function_frame)?;
        code.code.push(Bytecode::Branch(loop_start_loc as u16));

        let loop_end_loc = code.code.len() as u16;
        let breaks = function_frame.get_loop_breaks()?;
        for i in breaks {
            code.code[*i] = Bytecode::Branch(loop_end_loc);
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
        &mut self,
        cmd: &Cmd,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<ControlFlowInfo> {
        debug!("compile command {}", cmd);
        match cmd {
            Cmd::Return(exps) => {
                for exp in exps {
                    self.compile_expression(exp, code, function_frame)?;
                }
                code.code.push(Bytecode::Ret);
            }
            Cmd::Assign(lhs_variable, rhs_expression) => {
                let _expr_type = self.compile_expression(rhs_expression, code, function_frame)?;
                let loc_idx = function_frame.get_local(&lhs_variable.value)?;
                let st_loc = Bytecode::StLoc(loc_idx);
                code.code.push(st_loc);
                function_frame.pop()?;
            }
            Cmd::Unpack(name, bindings, e) => {
                self.compile_expression(e, code, function_frame)?;

                let (_is_resource, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                code.code.push(Bytecode::Unpack(def_idx));
                function_frame.pop()?;

                for lhs_variable in bindings.values().rev() {
                    let loc_idx = function_frame.get_local(&lhs_variable.value)?;
                    let st_loc = Bytecode::StLoc(loc_idx);
                    code.code.push(st_loc);
                }
            }
            Cmd::Call {
                ref return_bindings,
                ref call,
                ref actuals,
            } => {
                let mut actuals_tys = VecDeque::new();
                for exp in actuals.iter() {
                    actuals_tys.push_back(self.compile_expression(exp, code, function_frame)?);
                }
                let _ret_types =
                    self.compile_call(&call.value, code, function_frame, actuals_tys)?;
                for return_var in return_bindings.iter().rev() {
                    let loc_idx = function_frame.get_local(&return_var.value)?;
                    let st_loc = Bytecode::StLoc(loc_idx);
                    code.code.push(st_loc);
                }
            }
            Cmd::Mutate(exp, op) => {
                self.compile_expression(op, code, function_frame)?;
                self.compile_expression(exp, code, function_frame)?;
                code.code.push(Bytecode::WriteRef);
                function_frame.pop()?;
                function_frame.pop()?;
            }
            Cmd::Assert(ref condition, ref error_code) => {
                self.compile_expression(error_code, code, function_frame)?;
                self.compile_expression(condition, code, function_frame)?;
                code.code.push(Bytecode::Assert);
                function_frame.pop()?;
                function_frame.pop()?;
            }
            Cmd::Continue => {
                let loc = function_frame.get_loop_start()?;
                code.code.push(Bytecode::Branch(loc as u16));
            }
            Cmd::Break => {
                function_frame.push_loop_break(code.code.len())?;
                // placeholder, to be replaced when the enclosing while is compiled
                code.code.push(Bytecode::Branch(0));
            }
        }
        let (reachable_break, terminal_node) = match cmd {
            // If we are in a loop, `continue` makes a terminal node
            // Conceptually we treat
            //   `while (cond) { body }`
            // as `
            //   `loop { if (cond) { body; continue; } else { break; } }`
            Cmd::Continue |
            // `return` always makes a terminal node
            Cmd::Return(_) => (false, true),
            Cmd::Break => (true, false),
            _ => (false, false),
        };
        Ok(ControlFlowInfo {
            reachable_break,
            terminal_node,
        })
    }

    fn compile_expression(
        &mut self,
        exp: &Exp,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<InferredType> {
        debug!("compile expression {}", exp);
        match exp {
            Exp::Move(ref x) => self.compile_move_local(&x.value, code, function_frame),
            Exp::Copy(ref x) => self.compile_copy_local(&x.value, code, function_frame),
            Exp::BorrowLocal(ref is_mutable, ref x) => {
                self.compile_borrow_local(&x.value, *is_mutable, code, function_frame)
            }
            Exp::Value(cv) => match cv.as_ref() {
                CopyableVal::Address(address) => {
                    let addr_idx = self.make_address(&address)?;
                    code.code.push(Bytecode::LdAddr(addr_idx));
                    function_frame.push()?;
                    Ok(InferredType::Address)
                }
                CopyableVal::U64(i) => {
                    code.code.push(Bytecode::LdConst(*i));
                    function_frame.push()?;
                    Ok(InferredType::U64)
                }
                CopyableVal::ByteArray(buf) => {
                    let buf_idx = self.make_byte_array(buf)?;
                    code.code.push(Bytecode::LdByteArray(buf_idx));
                    function_frame.push()?;
                    Ok(InferredType::ByteArray)
                }
                CopyableVal::Bool(b) => {
                    if *b {
                        code.code.push(Bytecode::LdTrue);
                    } else {
                        code.code.push(Bytecode::LdFalse);
                    }
                    function_frame.push()?;
                    Ok(InferredType::Bool)
                }
                CopyableVal::String(_) => bail!("nice try! come back later {:?}", cv),
            },
            Exp::Pack(name, fields) => {
                let module_idx = ModuleHandleIndex::new(0);
                let name_idx = self.make_string(name.name_ref())?;
                let (is_resource, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                let sh = self.make_struct_handle(module_idx, name_idx, is_resource)?;
                for (_, exp) in fields.iter() {
                    self.compile_expression(exp, code, function_frame)?;
                }

                code.code.push(Bytecode::Pack(def_idx));
                for _ in fields.iter() {
                    function_frame.pop()?;
                }
                function_frame.push()?;
                Ok(InferredType::Struct(sh))
            }
            Exp::UnaryExp(op, e) => {
                self.compile_expression(e, code, function_frame)?;
                match op {
                    UnaryOp::Not => {
                        code.code.push(Bytecode::Not);
                        Ok(InferredType::Bool)
                    }
                }
            }
            Exp::BinopExp(e1, op, e2) => {
                self.compile_expression(e1, code, function_frame)?;
                self.compile_expression(e2, code, function_frame)?;
                function_frame.pop()?;
                match op {
                    BinOp::Add => {
                        code.code.push(Bytecode::Add);
                        Ok(InferredType::U64)
                    }
                    BinOp::Sub => {
                        code.code.push(Bytecode::Sub);
                        Ok(InferredType::U64)
                    }
                    BinOp::Mul => {
                        code.code.push(Bytecode::Mul);
                        Ok(InferredType::U64)
                    }
                    BinOp::Mod => {
                        code.code.push(Bytecode::Mod);
                        Ok(InferredType::U64)
                    }
                    BinOp::Div => {
                        code.code.push(Bytecode::Div);
                        Ok(InferredType::U64)
                    }
                    BinOp::BitOr => {
                        code.code.push(Bytecode::BitOr);
                        Ok(InferredType::U64)
                    }
                    BinOp::BitAnd => {
                        code.code.push(Bytecode::BitAnd);
                        Ok(InferredType::U64)
                    }
                    BinOp::Xor => {
                        code.code.push(Bytecode::Xor);
                        Ok(InferredType::U64)
                    }
                    BinOp::Or => {
                        code.code.push(Bytecode::Or);
                        Ok(InferredType::Bool)
                    }
                    BinOp::And => {
                        code.code.push(Bytecode::And);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Eq => {
                        code.code.push(Bytecode::Eq);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Neq => {
                        code.code.push(Bytecode::Neq);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Lt => {
                        code.code.push(Bytecode::Lt);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Gt => {
                        code.code.push(Bytecode::Gt);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Le => {
                        code.code.push(Bytecode::Le);
                        Ok(InferredType::Bool)
                    }
                    BinOp::Ge => {
                        code.code.push(Bytecode::Ge);
                        Ok(InferredType::Bool)
                    }
                }
            }
            Exp::Dereference(e) => {
                let loc_type = self.compile_expression(e, code, function_frame)?;
                code.code.push(Bytecode::ReadRef);
                match loc_type {
                    InferredType::MutableReference(sig_ref_token) => Ok(*sig_ref_token),
                    InferredType::Reference(sig_ref_token) => Ok(*sig_ref_token),
                    _ => Ok(InferredType::Anything),
                }
            }
            Exp::Borrow {
                ref is_mutable,
                ref exp,
                ref field,
            } => {
                let this_type = self.compile_expression(exp, code, function_frame)?;
                self.compile_load_field_reference(
                    this_type,
                    field,
                    *is_mutable,
                    code,
                    function_frame,
                )
            }
        }
    }

    fn compile_load_field_reference(
        &mut self,
        struct_type: InferredType,
        struct_field: &Field,
        is_mutable: bool,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<InferredType> {
        let sh_idx = struct_type.get_struct_handle()?;
        // TODO: the clone is to avoid the problem with mut/immut references, review...
        let field_type = self.scope.get_field_type(sh_idx, struct_field.name())?;
        let fd_idx = self.scope.get_field_def(sh_idx, struct_field.name())?;
        function_frame.pop()?;
        code.code.push(Bytecode::BorrowField(fd_idx));
        function_frame.push()?;
        let input_is_mutable = match struct_type {
            InferredType::Reference(_) => false,
            _ => true,
        };
        let inner_token = Box::new(InferredType::from_signature_token(&field_type.0));
        Ok(if is_mutable {
            if !input_is_mutable {
                bail!("Unsupported Syntax: Cannot take a mutable field reference in an immutable reference. It is not expressible in the bytecode");
            }
            InferredType::MutableReference(inner_token)
        } else {
            if input_is_mutable {
                code.code.push(Bytecode::FreezeRef);
            }
            InferredType::Reference(inner_token)
        })
    }

    fn compile_copy_local(
        &self,
        v: &Var,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<InferredType> {
        let loc_idx = function_frame.get_local(&v)?;
        let load_loc = Bytecode::CopyLoc(loc_idx);
        code.code.push(load_loc);
        function_frame.push()?;
        let loc_type = function_frame.get_local_type(loc_idx)?;
        Ok(InferredType::from_signature_token(loc_type))
    }

    fn compile_move_local(
        &self,
        v: &Var,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<InferredType> {
        let loc_idx = function_frame.get_local(&v)?;
        let load_loc = Bytecode::MoveLoc(loc_idx);
        code.code.push(load_loc);
        function_frame.push()?;
        let loc_type = function_frame.get_local_type(loc_idx)?;
        Ok(InferredType::from_signature_token(loc_type))
    }

    fn compile_borrow_local(
        &self,
        v: &Var,
        is_mutable: bool,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
    ) -> Result<InferredType> {
        let loc_idx = function_frame.get_local(&v)?;
        code.code.push(Bytecode::BorrowLoc(loc_idx));
        function_frame.push()?;
        let loc_type = function_frame.get_local_type(loc_idx)?;
        let inner_token = Box::new(InferredType::from_signature_token(loc_type));
        Ok(if is_mutable {
            InferredType::MutableReference(inner_token)
        } else {
            code.code.push(Bytecode::FreezeRef);
            InferredType::Reference(inner_token)
        })
    }

    fn compile_call(
        &mut self,
        call: &FunctionCall,
        code: &mut CodeUnit,
        function_frame: &mut FunctionFrame,
        mut argument_types: VecDeque<InferredType>,
    ) -> Result<Vec<InferredType>> {
        match call {
            FunctionCall::Builtin(function) => {
                match function {
                    Builtin::GetTxnGasUnitPrice => {
                        code.code.push(Bytecode::GetTxnGasUnitPrice);
                        function_frame.push()?;
                        Ok(vec![InferredType::U64])
                    }
                    Builtin::GetTxnMaxGasUnits => {
                        code.code.push(Bytecode::GetTxnMaxGasUnits);
                        function_frame.push()?;
                        Ok(vec![InferredType::U64])
                    }
                    Builtin::GetGasRemaining => {
                        code.code.push(Bytecode::GetGasRemaining);
                        function_frame.push()?;
                        Ok(vec![InferredType::U64])
                    }
                    Builtin::GetTxnSender => {
                        code.code.push(Bytecode::GetTxnSenderAddress);
                        function_frame.push()?;
                        Ok(vec![InferredType::Address])
                    }
                    Builtin::Exists(name) => {
                        let (_, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                        code.code.push(Bytecode::Exists(def_idx));
                        function_frame.pop()?;
                        function_frame.push()?;
                        Ok(vec![InferredType::Bool])
                    }
                    Builtin::BorrowGlobal(name) => {
                        let (is_resource, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                        code.code.push(Bytecode::BorrowGlobal(def_idx));
                        function_frame.pop()?;
                        function_frame.push()?;

                        let module_idx = ModuleHandleIndex::new(0);
                        let name_idx = self.make_string(name.name_ref())?;
                        let sh = self.make_struct_handle(module_idx, name_idx, is_resource)?;
                        Ok(vec![InferredType::MutableReference(Box::new(
                            InferredType::Struct(sh),
                        ))])
                    }
                    Builtin::Release => {
                        code.code.push(Bytecode::ReleaseRef);
                        function_frame.pop()?;
                        function_frame.push()?;
                        Ok(vec![])
                    }
                    Builtin::CreateAccount => {
                        code.code.push(Bytecode::CreateAccount);
                        function_frame.pop()?;
                        function_frame.push()?;
                        Ok(vec![])
                    }
                    Builtin::EmitEvent => {
                        code.code.push(Bytecode::EmitEvent);
                        function_frame.pop()?;
                        function_frame.pop()?;
                        function_frame.pop()?;
                        Ok(vec![])
                    }
                    Builtin::MoveFrom(name) => {
                        let (is_resource, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                        code.code.push(Bytecode::MoveFrom(def_idx));
                        function_frame.pop()?; // pop the address
                        function_frame.push()?; // push the return value

                        let module_idx = ModuleHandleIndex::new(0);
                        let name_idx = self.make_string(name.name_ref())?;
                        let sh = self.make_struct_handle(module_idx, name_idx, is_resource)?;
                        Ok(vec![InferredType::Struct(sh)])
                    }
                    Builtin::MoveToSender(name) => {
                        let (_, def_idx) = self.scope.get_struct_def(name.name_ref())?;
                        code.code.push(Bytecode::MoveToSender(def_idx));
                        function_frame.push()?;
                        Ok(vec![])
                    }
                    Builtin::GetTxnSequenceNumber => {
                        code.code.push(Bytecode::GetTxnSequenceNumber);
                        function_frame.push()?;
                        Ok(vec![InferredType::U64])
                    }
                    Builtin::GetTxnPublicKey => {
                        code.code.push(Bytecode::GetTxnPublicKey);
                        function_frame.push()?;
                        Ok(vec![InferredType::ByteArray])
                    }
                    Builtin::Freeze => {
                        code.code.push(Bytecode::FreezeRef);
                        function_frame.pop()?; // pop mut ref
                        function_frame.push()?; // push imm ref
                        let inner_token = match argument_types.pop_front() {
                            Some(InferredType::Reference(inner_token))
                            | Some(InferredType::MutableReference(inner_token)) => inner_token,
                            // Incorrect call
                            _ => Box::new(InferredType::Anything),
                        };
                        Ok(vec![InferredType::Reference(inner_token)])
                    }
                    _ => bail!("unsupported builtin function: {}", function),
                }
            }
            FunctionCall::ModuleFunctionCall { module, name } => {
                let scope_name = self.scope.get_name();

                let mh = if scope_name.is_ok() && module.name() == ModuleName::SELF {
                    ModuleHandleIndex::new(0)
                } else {
                    let target_module = self.scope.get_imported_module(module.name_ref())?;

                    let mut idx = 0;
                    while idx < target_module.function_defs().len() {
                        let fh_idx = target_module
                            .function_def_at(FunctionDefinitionIndex::new(idx as TableIndex))
                            .function;
                        let fh = target_module.function_handle_at(fh_idx);
                        let func_name = target_module.string_at(fh.name);
                        if func_name == name.name_ref() {
                            break;
                        }
                        idx += 1;
                    }
                    if idx == target_module.function_defs().len() {
                        bail!(
                            "Cannot find function `{}' in module `{}'",
                            name.name_ref(),
                            module.name_ref()
                        );
                    }

                    self.scope.link_function(
                        module.name_ref(),
                        name.name_ref(),
                        FunctionDefinitionIndex::new(idx as u16),
                    )?;
                    self.scope.get_imported_module_handle(module.name_ref())?
                };

                let name_ref = name.name_ref();
                let name_idx = self.make_string(name_ref)?;
                let mut func_sig = self.scope.get_function_signature(mh, name_ref)?.clone();
                func_sig = self.import_function_signature(module.name_ref(), func_sig)?;
                let return_types = func_sig
                    .return_types
                    .iter()
                    .map(InferredType::from_signature_token)
                    .collect();
                let args_count = func_sig.arg_types.len();
                let sig_idx = self.make_function_signature(&func_sig)?;
                let fh_idx = self.make_function_handle(mh, name_idx, sig_idx)?;
                let call = Bytecode::Call(fh_idx);
                code.code.push(call);
                for _ in 0..args_count {
                    function_frame.pop()?;
                }
                // Return value of current function is pushed onto the stack.
                function_frame.push()?;
                Ok(return_types)
            }
        }
    }
}
