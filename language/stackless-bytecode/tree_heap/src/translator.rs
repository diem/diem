// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use bytecode_verifier::VerifiedModule;
use libra_types::identifier::Identifier;
use num::{BigInt, Num};
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    file_format::{
        FieldDefinitionIndex, FunctionHandleIndex, ModuleHandleIndex, SignatureToken,
        StructDefinitionIndex, StructHandleIndex,
    },
    internals::ModuleIndex,
    views::{
        FieldDefinitionView, FunctionHandleView, SignatureTokenView, StructDefinitionView,
        StructHandleView, ViewInternals,
    },
};

pub struct BoogieTranslator {
    pub modules: Vec<VerifiedModule>,
    pub struct_defs: BTreeMap<String, usize>,
    pub max_struct_depth: usize,
    pub module_name_to_idx: BTreeMap<Identifier, usize>,
}

pub struct ModuleTranslator<'a> {
    pub module: &'a VerifiedModule,
    pub stackless_bytecode: Vec<StacklessFunction>,
    pub all_type_strs: BTreeSet<String>,
}

impl BoogieTranslator {
    pub fn new(modules: &[VerifiedModule]) -> Self {
        let mut struct_defs: BTreeMap<String, usize> = BTreeMap::new();
        let mut module_name_to_idx: BTreeMap<Identifier, usize> = BTreeMap::new();
        for (module_idx, module) in modules.iter().enumerate() {
            let module_name =
                module.identifier_at(module.module_handle_at(ModuleHandleIndex::new(0)).name);
            module_name_to_idx.insert(module_name.into(), module_idx);
            for (idx, struct_def) in module.struct_defs().iter().enumerate() {
                let struct_name = format!(
                    "{}_{}",
                    module_name,
                    module
                        .identifier_at(module.struct_handle_at(struct_def.struct_handle).name)
                        .to_string()
                );
                struct_defs.insert(struct_name, idx);
            }
        }
        Self {
            modules: modules.to_vec(),
            struct_defs,
            max_struct_depth: 0,
            module_name_to_idx,
        }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::from("\n\n// everything below is auto generated\n\n");
        // generate names and struct specific functions for all structs
        res.push_str(&self.emit_struct_code());

        // generate IsPrefix and UpdateValue to the max depth
        res.push_str(&self.emit_stratified_functions());

        for module in self.modules.iter() {
            let mut mt = ModuleTranslator::new(&module);
            res.push_str(&mt.translate());
        }
        res
    }

    pub fn emit_struct_code(&mut self) -> String {
        let mut res = String::new();
        for module in self.modules.iter() {
            for (def_idx, struct_def) in module.struct_defs().iter().enumerate() {
                let struct_name = struct_name_from_handle_index(module, struct_def.struct_handle);
                res.push_str(&format!("const unique {}: TypeName;\n", struct_name));
                let struct_definition_view = StructDefinitionView::new(module, struct_def);
                if struct_definition_view.is_native() {
                    continue;
                }
                let field_info = get_field_info_from_def_index(module, def_idx);
                for (field_name, _) in field_info {
                    res.push_str(&format!(
                        "const unique {}_{}: FieldName;\n",
                        struct_name, field_name
                    ));
                }
                res.push_str(&self.emit_struct_specific_functions(module, def_idx));
                let struct_handle_index = struct_def.struct_handle;
                // calculate the max depth of a struct
                self.max_struct_depth = std::cmp::max(
                    self.max_struct_depth,
                    self.get_struct_depth(
                        module,
                        &SignatureToken::Struct(struct_handle_index, vec![]),
                    ),
                );
            }
        }
        res
    }

    fn get_struct_depth(&self, module: &VerifiedModule, sig: &SignatureToken) -> usize {
        if let SignatureToken::Struct(idx, _) = sig {
            let mut max_field_depth = 0;
            let struct_handle = module.struct_handle_at(*idx);
            let struct_handle_view = StructHandleView::new(module, struct_handle);
            let module_name = module.identifier_at(struct_handle_view.module_handle().name);
            let def_module_idx = self
                .module_name_to_idx
                .get(module_name)
                .unwrap_or_else(|| panic!("no module named {}", module_name));
            let def_module = &self.modules[*def_module_idx];
            let struct_name = struct_name_from_handle_index(module, *idx);
            let def_idx = *self
                .struct_defs
                .get(&struct_name)
                .expect("can't find struct def");
            let struct_definition = &def_module.struct_defs()[def_idx];
            let struct_definition_view = StructDefinitionView::new(def_module, struct_definition);
            if struct_definition_view.is_native() {
                return 0;
            }
            for field_definition_view in struct_definition_view.fields().unwrap() {
                let field_depth = self.get_struct_depth(
                    def_module,
                    field_definition_view.type_signature().token().as_inner(),
                );
                max_field_depth = std::cmp::max(max_field_depth, field_depth);
            }
            max_field_depth + 1
        } else {
            0
        }
    }
}

impl<'a> ModuleTranslator<'a> {
    pub fn new(module: &'a VerifiedModule) -> Self {
        let stackless_bytecode = StacklessModuleGenerator::new(module.as_inner()).generate_module();
        let mut all_type_strs = BTreeSet::new();
        for struct_def in module.struct_defs().iter() {
            let struct_name = struct_name_from_handle_index(module, struct_def.struct_handle);
            all_type_strs.insert(struct_name);
        }
        Self {
            module,
            stackless_bytecode,
            all_type_strs,
        }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::new();
        // translation of stackless bytecode
        for (idx, function_def) in self.module.function_defs().iter().enumerate() {
            if function_def.is_native() {
                res.push_str(&self.generate_function_sig(idx, false, &None));
                res.push_str(";\n");
                continue;
            }
            res.push_str(&self.translate_function(idx));
        }
        res
    }

    pub fn translate_function(&self, idx: usize) -> String {
        let mut res = String::new();
        // generate function signature
        res.push_str(&self.generate_function_sig(idx, false, &None)); // no inline
                                                                      // generate function body
        res.push_str(&self.generate_function_body(idx, false, &None));
        res
    }

    pub fn translate_bytecode(
        &self,
        bytecode: &StacklessBytecode,
        func_idx: usize,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        let mut res = String::new();
        let stmts = match bytecode {
            Branch(target) => vec![format!("goto Label_{};", target)],
            BrTrue(target, idx) => vec![
                format!("tmp := ls[old_size + {}];", idx),
                format!("if (b#Boolean(tmp)) {{ goto Label_{}; }}", target),
            ],
            BrFalse(target, idx) => vec![
                format!("tmp := ls[old_size + {}];", idx),
                format!("if (!b#Boolean(tmp)) {{ goto Label_{}; }}", target),
            ],
            MoveLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
                } else {
                    vec![
                        format!("call tmp := CopyOrMoveValue(ls[old_size+{}]);", src),
                        format!("ls[old_size+{}] := tmp;", dest),
                    ]
                }
            }
            CopyLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
                } else {
                    vec![
                        format!("call tmp := CopyOrMoveValue(ls[old_size+{}]);", src),
                        format!("ls[old_size+{}] := tmp;", dest),
                    ]
                }
            }
            StLoc(dest, src) => {
                if self.is_local_ref(*dest as usize, func_idx) {
                    vec![format!(
                        "call {} := CopyOrMoveRef(t{});",
                        self.get_local_name(*dest as usize, arg_names),
                        src
                    )]
                } else {
                    vec![
                        format!("call tmp := CopyOrMoveValue(ls[old_size+{}]);", src),
                        format!("ls[old_size+{}] := tmp;", dest),
                    ]
                }
            }
            BorrowLoc(dest, src) => vec![format!("call t{} := BorrowLoc(old_size+{});", dest, src)],
            ReadRef(dest, src) => vec![
                format!("call tmp := ReadRef(t{});", src),
                self.format_type_checking("tmp".to_string(), &self.get_local_type(*dest, func_idx)),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            WriteRef(dest, src) => vec![format!("call WriteRef(t{}, ls[old_size+{}]);", dest, src)],
            FreezeRef(dest, src) => vec![format!("call t{} := FreezeRef(t{});", dest, src)],
            Call(dests, callee_index, _, args) => {
                let callee_name = self.function_name_from_handle_index(*callee_index);
                let mut dest_str = String::new();
                let mut args_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        args_str.push_str(", ");
                    }
                    if self.is_local_ref(*arg, func_idx) {
                        args_str.push_str(&format!("t{}", arg));
                    } else {
                        args_str.push_str(&format!("ls[old_size+{}]", arg));
                    }
                }
                for (i, dest) in dests.iter().enumerate() {
                    if i > 0 {
                        dest_str.push_str(", ");
                    }
                    dest_str.push_str(&format!("t{}", dest));
                    dest_type_assumptions.push(self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                    if !self.is_local_ref(*dest, func_idx) {
                        tmp_assignments.push(format!("ls[old_size+{}] := t{};", dest, dest));
                    }
                }
                let mut res_vec = vec![];
                if dest_str == "" {
                    res_vec.push(format!("call {}({});", callee_name, args_str))
                } else {
                    res_vec.push(format!(
                        "call {} := {}({});",
                        dest_str, callee_name, args_str
                    ));
                }
                res_vec.extend(dest_type_assumptions);
                res_vec.extend(tmp_assignments);
                res_vec
            }
            Pack(dest, struct_def_index, _, fields) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut fields_str = String::new();
                let mut res_vec = vec![];
                for (idx, field_temp) in fields.iter().enumerate() {
                    if idx > 0 {
                        fields_str.push_str(", ");
                    }
                    fields_str.push_str(&format!("ls[old_size+{}]", field_temp));
                    res_vec.push(self.format_type_checking(
                        format!("ls[old_size+{}]", field_temp),
                        &self.get_local_type(*field_temp, func_idx),
                    ));
                }
                res_vec.push(format!("call tmp := Pack_{}({});", struct_str, fields_str));
                res_vec.push(format!("ls[old_size+{}] := tmp;", dest));
                res_vec
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut dests_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];
                for (idx, dest) in dests.iter().enumerate() {
                    if idx > 0 {
                        dests_str.push_str(", ");
                    }
                    dests_str.push_str(&format!("t{}", dest));
                    dest_type_assumptions.push(self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                    if !self.is_local_ref(*dest, func_idx) {
                        tmp_assignments.push(format!("ls[old_size+{}] := t{};", dest, dest));
                    }
                }
                let mut res_vec = vec![format!(
                    "call {} := Unpack_{}(ls[old_size+{}]);",
                    dests_str, struct_str, src
                )];
                res_vec.extend(dest_type_assumptions);
                res_vec.extend(tmp_assignments);
                res_vec
            }
            BorrowField(dest, src, field_def_index) => {
                let field_name = self.field_name_from_index(*field_def_index);
                vec![format!(
                    "call t{} := BorrowField(t{}, {});",
                    dest, src, field_name
                )]
            }
            Exists(dest, addr, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![
                    format!("call tmp := Exists(ls[old_size+{}], {});", addr, struct_str),
                    format!("ls[old_size+{}] := tmp;", dest),
                ]
            }
            BorrowGlobal(dest, addr, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call t{} := BorrowGlobal(ls[old_size+{}], {});",
                    dest, addr, struct_str,
                )]
            }
            MoveToSender(src, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call MoveToSender({}, ls[old_size+{}]);",
                    struct_str, src,
                )]
            }
            MoveFrom(dest, src, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![
                    format!(
                        "call tmp := MoveFrom(ls[old_size+{}], {});",
                        src, struct_str,
                    ),
                    format!("ls[old_size+{}] := tmp;", dest),
                    self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ),
                ]
            }
            Ret(rets) => {
                let mut ret_assignments = vec![];
                for (i, r) in rets.iter().enumerate() {
                    if self.is_local_ref(*r, func_idx) {
                        ret_assignments.push(format!("ret{} := t{};", i, r));
                    } else {
                        ret_assignments.push(format!("ret{} := ls[old_size+{}];", i, r));
                    }
                }
                ret_assignments.push("return;".to_string());
                ret_assignments
            }
            LdTrue(idx) => vec![
                "call tmp := LdTrue();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            LdFalse(idx) => vec![
                "call tmp := LdFalse();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            LdConst(idx, num) => vec![
                format!("call tmp := LdConst({});", num),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            LdAddr(idx, addr_idx) => {
                let addr = self.module.address_pool()[(*addr_idx).into_index()];
                let addr_int = BigInt::from_str_radix(&addr.to_string(), 16).unwrap();
                vec![
                    format!("call tmp := LdAddr({});", addr_int),
                    format!("ls[old_size+{}] := tmp;", idx),
                ]
            }
            Not(dest, operand) => vec![
                format!("call tmp := Not(ls[old_size+{}]);", operand),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Add(dest, op1, op2) => vec![
                format!(
                    "call tmp := Add(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Sub(dest, op1, op2) => vec![
                format!(
                    "call tmp := Sub(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Mul(dest, op1, op2) => vec![
                format!(
                    "call tmp := Mul(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Div(dest, op1, op2) => vec![
                format!(
                    "call tmp := Div(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Mod(dest, op1, op2) => vec![
                format!(
                    "call tmp := Mod(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Lt(dest, op1, op2) => vec![
                format!(
                    "call tmp := Lt(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Gt(dest, op1, op2) => vec![
                format!(
                    "call tmp := Gt(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Le(dest, op1, op2) => vec![
                format!(
                    "call tmp := Le(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Ge(dest, op1, op2) => vec![
                format!(
                    "call tmp := Ge(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Or(dest, op1, op2) => vec![
                format!(
                    "call tmp := Or(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            And(dest, op1, op2) => vec![
                format!(
                    "call tmp := And(ls[old_size+{}], ls[old_size+{}]);",
                    op1, op2
                ),
                format!("ls[old_size+{}] := tmp;", dest),
            ],
            Eq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![
                    format!(
                        "call tmp := Eq_{}(ls[old_size+{}], ls[old_size+{}]);",
                        format_type(self.module, &operand_type),
                        op1,
                        op2
                    ),
                    format!("ls[old_size+{}] := tmp;", dest),
                ]
            }
            Neq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![
                    format!(
                        "call tmp := Neq_{}(ls[old_size+{}], ls[old_size+{}]);",
                        format_type(self.module, &operand_type),
                        op1,
                        op2
                    ),
                    format!("ls[old_size+{}] := tmp;", dest),
                ]
            }
            BitOr(_, _, _) | BitAnd(_, _, _) | Xor(_, _, _) => {
                vec!["// bit operation not supported".into()]
            }
            Abort(_) => vec!["assert false;".into()],
            GetGasRemaining(idx) => vec![
                "call tmp := GetGasRemaining();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            GetTxnSequenceNumber(idx) => vec![
                "call tmp := GetTxnSequenceNumber();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            GetTxnPublicKey(idx) => vec![
                "call tmp := GetTxnPublicKey();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            GetTxnSenderAddress(idx) => vec![
                "call tmp := GetTxnSenderAddress();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            GetTxnMaxGasUnits(idx) => vec![
                "call tmp := GetTxnMaxGasUnits();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            GetTxnGasUnitPrice(idx) => vec![
                "call tmp := GetTxnGasUnitPrice();".to_string(),
                format!("ls[old_size+{}] := tmp;", idx),
            ],
            CreateAccount(idx) => vec![format!("call CreateAccount(t{});", idx)],
            _ => vec!["// unimplemented instruction".into()],
        };
        for code in stmts {
            res.push_str(&format!("    {}\n", code));
        }
        res.push('\n');
        res
    }

    pub fn generate_function_sig(
        &self,
        idx: usize,
        inline: bool,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        let function_def = &self.module.function_defs()[idx];
        let fun_name = self.function_name_from_definition_index(idx);
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let mut args = String::new();
        let mut rets = String::new();
        for (i, arg_type) in function_signature.arg_types.iter().enumerate() {
            if i > 0 {
                args.push_str(", ");
            }
            args.push_str(&format!(
                "{}: {}",
                self.get_arg_name(i, arg_names),
                self.format_value_or_ref(&arg_type)
            ));
        }
        for (i, return_type) in function_signature.return_types.iter().enumerate() {
            if i > 0 {
                rets.push_str(", ");
            }
            rets.push_str(&format!(
                "ret{}: {}",
                i,
                self.format_value_or_ref(&return_type)
            ));
        }
        if inline {
            format!(
                "procedure {{:inline 1}} {}_inline ({}) returns ({})",
                fun_name, args, rets
            )
        } else {
            format!("procedure {} ({}) returns ({})", fun_name, args, rets)
        }
    }

    pub fn generate_function_body(
        &self,
        idx: usize,
        inline: bool,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        let mut res = String::new();
        let function_def = &self.module.function_defs()[idx];
        let code = &self.stackless_bytecode[idx];

        res.push_str("\n{\n");
        res.push_str("    // declare local variables\n");

        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let num_args = function_signature.arg_types.len();
        let mut ref_vars = BTreeSet::new(); // set of locals that are references
        let mut val_vars = BTreeSet::new(); // set of locals that are not
        let mut arg_assignment_str = String::new();
        let mut arg_value_assumption_str = String::new();
        for (i, local_type) in code.local_types.iter().enumerate() {
            if i < num_args {
                if !self.is_local_ref(i, idx) {
                    arg_assignment_str.push_str(&format!(
                        "    ls[old_size+{}] := {};\n",
                        i,
                        self.get_arg_name(i, arg_names)
                    ));
                } else {
                    arg_assignment_str.push_str(&format!(
                        "    {} := {};\n",
                        self.get_local_name(i, arg_names),
                        self.get_arg_name(i, arg_names)
                    ));
                }

                arg_value_assumption_str.push_str(&format!(
                    "    {}",
                    self.format_type_checking(self.get_arg_name(i, arg_names), local_type)
                ));
            }
            if SignatureTokenView::new(self.module, local_type).is_reference() {
                ref_vars.insert(i);
            } else {
                val_vars.insert(i);
            }

            res.push_str(&format!(
                "    var {}: {}; // {}\n",
                self.get_local_name(i, arg_names),
                self.format_value_or_ref(&local_type),
                format_type(self.module, &local_type)
            ));
        }
        res.push_str("\n    var tmp: Value;\n");
        res.push_str("    var old_size: int;\n");
        if !inline {
            res.push_str("    assume !abort_flag;\n");
        }
        res.push_str("\n    // assume arguments are of correct types\n");
        res.push_str(&arg_value_assumption_str);
        res.push_str("\n    old_size := ls_size;\n");
        res.push_str(&format!(
            "    ls_size := ls_size + {};\n",
            code.local_types.len()
        ));
        res.push_str(&arg_assignment_str);
        res.push_str("\n    // bytecode translation starts here\n");

        // identify all the branching targets so we can insert labels in front of them
        let mut branching_targets: BTreeSet<usize> = BTreeSet::new();
        for bytecode in code.code.iter() {
            match bytecode {
                Branch(target) | BrTrue(target, _) | BrFalse(target, _) => {
                    branching_targets.insert(*target as usize);
                }
                _ => {}
            }
        }

        for (offset, bytecode) in code.code.iter().enumerate() {
            // uncomment to print out bytecode for debugging purpose
            // println!("{:?}", bytecode);

            // insert labels for branching targets
            if branching_targets.contains(&offset) {
                res.push_str(&format!("Label_{}:\n", offset));
            }
            res.push_str(&self.translate_bytecode(bytecode, idx, arg_names));
        }
        res.push_str("}\n");
        res
    }

    pub fn get_local_name(&self, idx: usize, arg_names: &Option<Vec<String>>) -> String {
        if let Some(names) = arg_names {
            if idx < names.len() {
                return format!("new_{}", names[idx]);
            }
        }
        format!("t{}", idx)
    }

    pub fn get_arg_name(&self, idx: usize, arg_names: &Option<Vec<String>>) -> String {
        if let Some(names) = arg_names {
            format!("old_{}", names[idx])
        } else {
            format!("arg{}", idx)
        }
    }

    /*
        utility functions below
    */
    pub fn struct_name_from_definition_index(&self, idx: StructDefinitionIndex) -> String {
        let struct_handle = self.module.struct_def_at(idx).struct_handle;
        struct_name_from_handle_index(self.module, struct_handle)
    }

    pub fn field_name_from_index(&self, idx: FieldDefinitionIndex) -> String {
        let field_definition = self.module.field_def_at(idx);
        let struct_handle_index = field_definition.struct_;
        let struct_name = struct_name_from_handle_index(self.module, struct_handle_index);
        let field_name = FieldDefinitionView::new(self.module, field_definition).name();
        format!("{}_{}", struct_name, field_name)
    }

    fn function_name_from_definition_index(&self, idx: usize) -> String {
        let function_handle_index = self.module.function_defs()[idx].function;
        self.function_name_from_handle_index(function_handle_index)
    }

    fn function_name_from_handle_index(&self, idx: FunctionHandleIndex) -> String {
        let function_handle = self.module.function_handle_at(idx);
        let module_handle_index = function_handle.module;
        let mut module_name = self
            .module
            .identifier_at(self.module.module_handle_at(module_handle_index).name)
            .as_str();
        if module_name == "<SELF>" {
            module_name = "self";
        } // boogie doesn't allow '<' or '>'
        let function_handle_view = FunctionHandleView::new(self.module, function_handle);
        let function_name = function_handle_view.name();
        format!("{}_{}", module_name, function_name)
    }

    pub fn get_local_type(&self, local_idx: usize, func_idx: usize) -> SignatureToken {
        self.stackless_bytecode[func_idx].local_types[local_idx].clone()
    }

    pub fn is_local_ref(&self, local_idx: usize, func_idx: usize) -> bool {
        let sig = &self.stackless_bytecode[func_idx].local_types[local_idx];
        match sig {
            SignatureToken::MutableReference(_) | SignatureToken::Reference(_) => true,
            _ => false,
        }
    }

    pub fn is_local_mutable_ref(&self, local_idx: usize, func_idx: usize) -> bool {
        let sig = &self.stackless_bytecode[func_idx].local_types[local_idx];
        match sig {
            SignatureToken::MutableReference(_) => true,
            _ => false,
        }
    }

    pub fn format_value_or_ref(&self, sig: &SignatureToken) -> String {
        match sig {
            SignatureToken::Reference(_) | SignatureToken::MutableReference(_) => "Reference",
            _ => "Value",
        }
        .into()
    }

    pub fn format_type_checking(&self, name: String, sig: &SignatureToken) -> String {
        match sig {
            SignatureToken::Reference(_) | SignatureToken::MutableReference(_) => "".to_string(),
            _ => format!("assume is#{}({});\n", format_value_cons(sig), name,),
        }
    }
}

pub fn struct_name_from_handle_index(module: &VerifiedModule, idx: StructHandleIndex) -> String {
    let struct_handle = module.struct_handle_at(idx);
    let struct_handle_view = StructHandleView::new(module, struct_handle);
    let module_name = module.identifier_at(struct_handle_view.module_handle().name);
    let struct_name = struct_handle_view.name();
    format!("{}_{}", module_name, struct_name)
}

pub fn format_type(module: &VerifiedModule, sig: &SignatureToken) -> String {
    match sig {
        SignatureToken::Bool => "bool".into(),
        SignatureToken::U64 => "int".into(),
        SignatureToken::String => "string".into(),
        SignatureToken::ByteArray => "bytearray".into(),
        SignatureToken::Address => "address".into(),
        SignatureToken::Struct(idx, _) => struct_name_from_handle_index(module, *idx),
        SignatureToken::Reference(t) | SignatureToken::MutableReference(t) => {
            format!("{}_ref", format_type(module, &*t))
        }
        SignatureToken::TypeParameter(_) => "unsupported".into(),
    }
}

pub fn format_value_cons(sig: &SignatureToken) -> String {
    match sig {
        SignatureToken::Bool => "Boolean",
        SignatureToken::U64 => "Integer",
        SignatureToken::String => "Str",
        SignatureToken::ByteArray => "ByteArray",
        SignatureToken::Address => "Address",
        SignatureToken::Struct(_, _) => "Map",
        _ => "unsupported",
    }
    .into()
}

pub fn get_field_info_from_def_index(
    module: &VerifiedModule,
    def_idx: usize,
) -> BTreeMap<String, (String, String)> {
    let mut name_to_type = BTreeMap::new();
    let struct_definition = &module.struct_defs()[def_idx];
    let struct_definition_view = StructDefinitionView::new(module, struct_definition);
    for field_definition_view in struct_definition_view.fields().unwrap() {
        let field_name = field_definition_view.name().to_string();
        let sig = field_definition_view.type_signature().token().as_inner();
        name_to_type.insert(
            field_name,
            (format_type(module, sig), format_value_cons(sig)),
        );
    }
    name_to_type
}
