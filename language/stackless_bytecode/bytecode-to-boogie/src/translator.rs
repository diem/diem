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

        // calculate maximum number of locals and generate this many local names
        res.push_str(&self.generate_local_names());

        // generate IsPrefix and UpdateValue to the max depth
        res.push_str(&self.emit_stratified_functions());

        for module in self.modules.iter() {
            let mut mt = ModuleTranslator::new(&module);
            res.push_str(&mt.translate());
        }
        res
    }

    pub fn generate_local_names(&self) -> String {
        let mut res = String::new();
        let mut max_local_num = 0;
        for module in self.modules.iter() {
            let stackless_bytecode =
                StacklessModuleGenerator::new(module.as_inner()).generate_module();
            max_local_num = std::cmp::max(
                max_local_num,
                stackless_bytecode
                    .iter()
                    .map(|c| c.local_types.len())
                    .fold(0, std::cmp::max),
            );
        }

        for i in 0..max_local_num {
            res.push_str(&format!("const unique t{}_LocalName: LocalName;\n", i,));
        }
        res.push_str("\n");
        res
    }

    pub fn emit_struct_code(&mut self) -> String {
        let mut res = String::new();
        for module in self.modules.iter() {
            for (def_idx, struct_def) in module.struct_defs().iter().enumerate() {
                let struct_name = struct_name_from_handle_index(module, struct_def.struct_handle);
                res.push_str(&format!("const unique {}: TypeName;\n", struct_name));
                res.push_str(&format!("var rs_{}: ResourceStore;\n", struct_name));
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
        res.push_str(&self.generate_verify_function_body(idx, &None));
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
            BrTrue(target, idx) => vec![format!(
                "if (b#Boolean(t{})) {{ goto Label_{}; }}",
                idx, target
            )],
            BrFalse(target, idx) => vec![format!(
                "if (!b#Boolean(t{})) {{ goto Label_{}; }}",
                idx, target
            )],
            MoveLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
                } else {
                    vec![format!(
                        "call t{} := CopyOrMoveValue({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
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
                    vec![format!(
                        "call t{} := CopyOrMoveValue({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
                }
            }
            StLoc(dest, src) => {
                if self.is_local_ref(*dest as usize, func_idx) {
                    // TODO: release the scoop
                    vec![format!(
                        "call {} := CopyOrMoveRef(t{});",
                        self.get_local_name(*dest as usize, arg_names),
                        src
                    )]
                } else {
                    vec![format!(
                        "call {} := CopyOrMoveValue(t{});",
                        self.get_local_name(*dest as usize, arg_names),
                        src
                    )]
                }
            }
            BorrowLoc(dest, src) => vec![format!(
                "call t{} := BorrowLoc(c, t{}_LocalName, {});",
                dest,
                src,
                self.get_local_name(*src as usize, arg_names)
            )],
            ReadRef(dest, src) => vec![format!("call t{} := ReadRef(t{});", dest, src)],
            WriteRef(dest, src) => {
                vec![format!("call t{} := WriteRef(t{}, t{});", dest, dest, src)]
            }
            FreezeRef(dest, src) => vec![format!("call t{} := FreezeRef(t{});", dest, src)],
            Call(dests, callee_index, _, args) => {
                let callee_name = self.function_name_from_handle_index(*callee_index);
                let callee_function_handle = self.module.function_handle_at(*callee_index);
                let callee_function_signature = self
                    .module
                    .function_signature_at(callee_function_handle.signature);
                let mut dest_str = String::new();
                let mut args_str = String::new();
                let mut dest_type_assumptions = vec![];
                for (i, arg) in args.iter().enumerate() {
                    args_str.push_str(&format!(", t{}", arg));
                    if callee_function_signature.arg_types[i].is_mutable_reference() {
                        dest_str.push_str(&format!(", t{}", arg));
                        dest_type_assumptions.push(self.format_type_checking(
                            format!("t{}", arg),
                            &self.get_local_type(*arg, func_idx),
                        ));
                    }
                }
                for dest in dests.iter() {
                    dest_str.push_str(&format!(", t{}", dest));
                    dest_type_assumptions.push(self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                }
                let mut res_vec = vec![format!(
                    "call addr_exists'{} := {}(c', addr_exists'{});",
                    dest_str, callee_name, args_str
                )];
                res_vec.extend(dest_type_assumptions);
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
                    fields_str.push_str(&format!("t{}", field_temp));
                    res_vec.push(self.format_type_checking(
                        format!("t{}", field_temp),
                        &self.get_local_type(*field_temp, func_idx),
                    ));
                }
                res_vec.push(format!(
                    "call t{} := Pack_{}({});",
                    dest, struct_str, fields_str
                ));
                res_vec
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut dests_str = String::new();
                let mut dest_type_assumptions = vec![];
                for (idx, dest) in dests.iter().enumerate() {
                    if idx > 0 {
                        dests_str.push_str(", ");
                    }
                    dests_str.push_str(&format!("t{}", dest));
                    dest_type_assumptions.push(self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                }
                let mut res_vec = vec![format!(
                    "call {} := Unpack_{}(t{});",
                    dests_str, struct_str, src
                )];
                res_vec.extend(dest_type_assumptions);
                res_vec
            }
            BorrowField(dest, src, field_def_index) => {
                let field_name = self.field_name_from_index(*field_def_index);
                let field_sig = self.get_local_type(*dest, func_idx);
                vec![
                    format!("call t{} := BorrowField(t{}, {});", dest, src, field_name),
                    self.format_type_checking(format!("t{}", dest), &field_sig),
                ]
            }
            Exists(dest, addr, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call t{} := Exists(t{}, rs_{});",
                    dest, addr, struct_str
                )]
            }
            BorrowGlobal(dest, addr, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![
                    format!(
                        "call t{} := BorrowGlobal(t{}, {}, rs_{});",
                        dest, addr, struct_str, struct_str,
                    ),
                    format!("assume is#Global(rt#Reference(t{}));", dest),
                    format!("assume is#Map(v#Reference(t{}));", dest),
                ]
            }
            MoveToSender(src, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![format!(
                    "call rs_{} := MoveToSender(rs_{}, t{});",
                    struct_str, struct_str, src,
                )]
            }
            MoveFrom(dest, src, struct_def_index, _) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                vec![
                    format!(
                        "call t{}, rs_{} := MoveFrom(t{}, rs_{});",
                        dest, struct_str, src, struct_str,
                    ),
                    self.format_type_checking(
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ),
                ]
            }
            Ret(rets) => {
                let mut ret_assignments = vec![];
                for (i, r) in rets.iter().enumerate() {
                    ret_assignments.push(format!("ret{} := t{};", i, r));
                }
                ret_assignments.push("return;".to_string());
                ret_assignments
            }
            LdTrue(idx) => vec![format!("call t{} := LdTrue();", idx)],
            LdFalse(idx) => vec![format!("call t{} := LdFalse();", idx)],
            LdConst(idx, num) => vec![format!("call t{} := LdConst({});", idx, num)],
            LdAddr(idx, addr_idx) => {
                let addr = self.module.address_pool()[(*addr_idx).into_index()];
                let addr_int = BigInt::from_str_radix(&addr.to_string(), 16).unwrap();
                vec![format!("call t{} := LdAddr({});", idx, addr_int)]
            }
            Not(dest, operand) => vec![format!("call t{} := Not(t{});", dest, operand)],
            Add(dest, op1, op2) => vec![format!("call t{} := Add(t{}, t{});", dest, op1, op2)],
            Sub(dest, op1, op2) => vec![format!("call t{} := Sub(t{}, t{});", dest, op1, op2)],
            Mul(dest, op1, op2) => vec![format!("call t{} := Mul(t{}, t{});", dest, op1, op2)],
            Div(dest, op1, op2) => vec![format!("call t{} := Div(t{}, t{});", dest, op1, op2)],
            Mod(dest, op1, op2) => vec![format!("call t{} := Mod(t{}, t{});", dest, op1, op2)],
            Lt(dest, op1, op2) => vec![format!("call t{} := Lt(t{}, t{});", dest, op1, op2)],
            Gt(dest, op1, op2) => vec![format!("call t{} := Gt(t{}, t{});", dest, op1, op2)],
            Le(dest, op1, op2) => vec![format!("call t{} := Le(t{}, t{});", dest, op1, op2)],
            Ge(dest, op1, op2) => vec![format!("call t{} := Ge(t{}, t{});", dest, op1, op2)],
            Or(dest, op1, op2) => vec![format!("call t{} := Or(t{}, t{});", dest, op1, op2)],
            And(dest, op1, op2) => vec![format!("call t{} := And(t{}, t{});", dest, op1, op2)],
            Eq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![format!(
                    "call t{} := Eq_{}(t{}, t{});",
                    dest,
                    format_type(self.module, &operand_type),
                    op1,
                    op2
                )]
            }
            Neq(dest, op1, op2) => {
                let operand_type = self.get_local_type(*op1, func_idx);
                vec![format!(
                    "call t{} := Neq_{}(t{}, t{});",
                    dest,
                    format_type(self.module, &operand_type),
                    op1,
                    op2
                )]
            }
            BitOr(_, _, _) | BitAnd(_, _, _) | Xor(_, _, _) => {
                vec!["// bit operation not supported".into()]
            }
            Abort(_) => vec!["abort_flag := true;".into()],
            GetGasRemaining(idx) => vec![format!("call t{} := GetGasRemaining();", idx)],
            GetTxnSequenceNumber(idx) => vec![format!("call t{} := GetTxnSequenceNumber();", idx)],
            GetTxnPublicKey(idx) => vec![format!("call t{} := GetTxnPublicKey();", idx)],
            GetTxnSenderAddress(idx) => vec![format!("call t{} := GetTxnSenderAddress();", idx)],
            GetTxnMaxGasUnits(idx) => vec![format!("call t{} := GetTxnMaxGasUnits();", idx)],
            GetTxnGasUnitPrice(idx) => vec![format!("call t{} := GetTxnGasUnitPrice();", idx)],
            CreateAccount(idx) => vec![format!(
                "call addr_exists' := CreateAccount(t{}, addr_exists');",
                idx
            )],
            _ => vec!["// unimplemented instruction".into()],
        };
        for code in stmts {
            res.push_str(&format!("    {}\n", code));
        }
        res.push('\n');
        res
    }

    // return a string for a boogie procedure header.
    // if inline = true, add the inline attribute and use the plain function name
    // for the procedure name.
    // else, generate the function signature without the ":inlne" attribute, and
    // append _verify to the function name.
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
        let mut arg_decls = String::new(); // vector of ", argname : type"
        let mut rets = String::new();
        for (i, arg_type) in function_signature.arg_types.iter().enumerate() {
            // "arg_decls" is substituted later in an arglist that already has a few entries,
            // which is why there is a comma before first arg.
            arg_decls.push_str(&format!(
                ", {}: {}",
                self.get_arg_name(i, arg_names),
                self.format_value_or_ref(&arg_type)
            ));
            // *** Some derivative of this code needs to go into the verify function call to inline function
            if arg_type.is_mutable_reference() {
                rets.push_str(&format!(
                    ", {}: {}",
                    self.get_local_name(i, arg_names),
                    self.format_value_or_ref(&arg_type)
                ));
            }
        }
        for (i, return_type) in function_signature.return_types.iter().enumerate() {
            rets.push_str(&format!(
                ", ret{}: {}",
                i,
                self.format_value_or_ref(&return_type)
            ));
        }
        if inline {
            // generates the "inlined" version of the procedure, which has the unadorned
            // procedure name.
            // FIXME: Can we make this more readable using backslash to linebreak the string?
            format!(
                "procedure {{:inline 1}} {} (c: CreationTime, addr_exists: [Address]bool{}) returns (addr_exists': [Address]bool{})",
                fun_name, arg_decls, rets
            )
        } else {
            // generates the "verify" version of the procedure, which just calls the "inlined"
            // version.  "Verify" version will have specs integrated later.
            // FIXME: function name is misleading.  Maybe just generate the real body in the "inline" case?
            // FIXME: This is not going to work, because main.rs integrates the specs after the signature and before body.
            // So, minimal change is to generate body separately.
            format!(
                "procedure {}_verify (c: CreationTime, addr_exists: [Address]bool{}) returns (addr_exists': [Address]bool{})",
                fun_name, arg_decls, rets
            )
        }
    }

    // return string for body of verify function, which is just a call to the
    // inline version of the function.
    pub fn generate_verify_function_body(
        &self,
        idx: usize,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        let fun_name = self.function_name_from_definition_index(idx);
        let function_def = &self.module.function_defs()[idx];
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let mut args = String::new(); // vector of ", argname"
        let mut rets = String::new(); // vector of ", argname"
                                      // return values are: addr_exists' (always), <mutable references>, <actual returns>
        for (i, arg_type) in function_signature.arg_types.iter().enumerate() {
            args.push_str(&format!(", {}", self.get_arg_name(i, arg_names)));
            // collect mutable reference return values
            if arg_type.is_mutable_reference() {
                rets.push_str(&format!(", {}", self.get_local_name(i, arg_names),));
            }
        }
        // Next loop collects actual return values from Move function
        for i in 0..function_signature.return_types.len() {
            rets.push_str(&format!(", ret{}", i));
        }
        format!(
            "\n{{\n    call addr_exists' {} := {}(c, addr_exists {});\n}}\n\n",
            rets, fun_name, args
        )
    }

    // This generates boogie code for everything after the function signature
    // The function body is only generated for the "inline" version of the function.
    pub fn generate_inline_function_body(
        &self,
        idx: usize,
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
                arg_assignment_str.push_str(&format!(
                    "    {} := {};\n",
                    self.get_local_name(i, arg_names),
                    self.get_arg_name(i, arg_names)
                ));
                arg_value_assumption_str.push_str(&format!(
                    "    {}",
                    self.format_type_checking(self.get_arg_name(i, arg_names), local_type)
                ));

                if self.is_local_ref(i, idx) {
                    arg_value_assumption_str.push_str(&format!(
                        "    if (is#Local(rt#Reference({}))) {{\n",
                        self.get_arg_name(i, arg_names)
                    ));
                    arg_value_assumption_str.push_str(&format!(
                        "        assume c#Local(rt#Reference({})) < c;\n    }}\n",
                        self.get_arg_name(i, arg_names)
                    ));
                }
            }
            if SignatureTokenView::new(self.module, local_type).is_reference() {
                ref_vars.insert(i);
            } else {
                val_vars.insert(i);
            }
            if i < num_args && local_type.is_mutable_reference() {
                continue;
            }
            res.push_str(&format!(
                "    var {}: {}; // {}\n",
                self.get_local_name(i, arg_names),
                self.format_value_or_ref(&local_type),
                format_type(self.module, &local_type)
            ));
        }

        res.push_str("\n    // declare a new creation time for calls inside this function\n");
        res.push_str("    var c': CreationTime;\n    assume c' > c;\n");
        // DD 10/8/2019: I think it's ok to assume !abort_flag even if function is inlined.
        // if !inline {
        res.push_str("    assume !abort_flag;\n");
        // }
        res.push_str("\n    // assume arguments are of correct types\n");
        res.push_str(&arg_value_assumption_str);
        res.push_str("\n    // assign arguments to locals so they can be modified\n");
        res.push_str(&arg_assignment_str);
        res.push_str("\n    // assign ResourceStores to locals so they can be modified\n");
        res.push_str("    addr_exists' := addr_exists;\n");
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
            if let WriteRef(dest, src) = bytecode {
                // update everything that might be related to the updated reference
                for s in &ref_vars {
                    if s == dest {
                        continue;
                    }
                    res.push_str(&format!(
                        "    call {} := DeepUpdateReference({}, {});\n",
                        self.get_local_name(*s, arg_names),
                        self.get_local_name(*dest, arg_names),
                        self.get_local_name(*s, arg_names)
                    ));
                }
                for t in &self.all_type_strs {
                    res.push_str(&format!(
                        "    call rs_{} := DeepUpdateGlobal({}, {}, rs_{});\n",
                        t,
                        t,
                        self.get_local_name(*dest, arg_names),
                        t
                    ));
                }
                for s in &val_vars {
                    if s == src {
                        continue;
                    }
                    res.push_str(&format!(
                        "    call {} := DeepUpdateLocal(c, t{}_LocalName, {}, {});\n",
                        self.get_local_name(*s, arg_names),
                        s,
                        self.get_local_name(*dest, arg_names),
                        self.get_local_name(*s, arg_names)
                    ));
                }
                res.push_str("\n");
            }
            if let Call(_, _, _, args) = bytecode {
                // update everything that might be related to the updated reference
                for dest in args {
                    if !self.is_local_mutable_ref(*dest, idx) {
                        continue;
                    }
                    for s in &ref_vars {
                        if s == dest {
                            continue;
                        }
                        res.push_str(&format!(
                            "    call {} := DeepUpdateReference({}, {});\n",
                            self.get_local_name(*s, arg_names),
                            self.get_local_name(*dest, arg_names),
                            self.get_local_name(*s, arg_names)
                        ));
                    }
                    for t in &self.all_type_strs {
                        res.push_str(&format!(
                            "    call rs_{} := DeepUpdateGlobal({}, {}, rs_{});\n",
                            t,
                            t,
                            self.get_local_name(*dest, arg_names),
                            t
                        ));
                    }
                    for s in &val_vars {
                        res.push_str(&format!(
                            "    call {} := DeepUpdateLocal(c, t{}_LocalName, {}, {});\n",
                            self.get_local_name(*s, arg_names),
                            s,
                            self.get_local_name(*dest, arg_names),
                            self.get_local_name(*s, arg_names)
                        ));
                    }
                    res.push_str("\n");
                }
            }
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
            SignatureToken::Reference(s) | SignatureToken::MutableReference(s) => format!(
                "assume is#{}(v#Reference({}));\n",
                format_value_cons(s),
                name,
            ),
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
