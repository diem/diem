// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.
//!
//! TODO: there is several cleanup to do in this module.
//!
//! 1. We should generalize the state and various helper functions into an 'environment'/'context'
//!    abstraction which is shared between phases like bytecode and spec translation. This
//!    environment should. among other things, allow to collect diagnostics related to the
//!    translation.
//! 2. We should factor out basic functions which represent interacting with the boogie model,
//!    for example, how to construct a TypeValue from a SignatureToken, how to access memory,
//!    and so on.

use crate::cli::Options;
use crate::spec_translator::SpecTranslator;
use bytecode_source_map::source_map::ModuleSourceMap;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::parser::ast::Loc;
use ir_to_bytecode_syntax::spec_language_ast::Condition;
use itertools::Itertools;
use libra_types::{account_address::AccountAddress, identifier::Identifier};
use log::info;
use num::{BigInt, Num};
use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    file_format::{
        FieldDefinitionIndex, FunctionDefinitionIndex, FunctionHandleIndex, LocalsSignatureIndex,
        ModuleHandleIndex, SignatureToken, StructDefinition, StructDefinitionIndex,
        StructHandleIndex,
    },
    internals::ModuleIndex,
    views::{
        FieldDefinitionView, FunctionHandleView, SignatureTokenView, StructDefinitionView,
        StructHandleView, ViewInternals,
    },
};

/// Represents information about a module.
#[derive(Debug)]
pub struct ModuleInfo {
    /// The name of this module.
    pub name: String,
    /// List of function specifications, in function declaration order.
    pub function_infos: Vec<FunctionInfo>,
}

/// Represents information about a function.
#[derive(Debug)]
pub struct FunctionInfo {
    /// Index of this function in the function definition table.
    pub index: FunctionDefinitionIndex,
    /// List of function argument names. Those aren't present in the bytecode but we obtain them
    /// from the AST.
    pub arg_names: Vec<String>,
    /// List of type argument names.
    pub type_arg_names: Vec<String>,
    /// List of specification conditions.
    pub specification: Vec<Condition>,
}

pub struct BoogieTranslator<'a> {
    pub options: &'a Options,
    pub modules: &'a [VerifiedModule],
    pub module_infos: &'a [ModuleInfo],
    pub source_maps: &'a [ModuleSourceMap<Loc>],
    pub struct_defs: BTreeMap<String, usize>,
    pub max_struct_depth: usize,
    pub module_name_to_idx: BTreeMap<Identifier, usize>,
    /// If set, this narrows down output for module code on the given modules.
    pub target_modules: Option<Vec<String>>,
}

pub struct ModuleTranslator<'a> {
    pub parent: &'a BoogieTranslator<'a>,
    pub module: &'a VerifiedModule,
    pub module_info: &'a ModuleInfo,
    pub source_map: &'a ModuleSourceMap<Loc>,
    pub stackless_bytecode: Vec<StacklessFunction>,
    pub all_type_strs: BTreeSet<String>,
    pub ignore: bool,
}

impl<'a> BoogieTranslator<'a> {
    pub fn new(
        options: &'a Options,
        modules: &'a [VerifiedModule],
        module_infos: &'a [ModuleInfo],
        source_maps: &'a [ModuleSourceMap<Loc>],
    ) -> Self {
        let mut struct_defs: BTreeMap<String, usize> = BTreeMap::new();
        let mut module_name_to_idx: BTreeMap<Identifier, usize> = BTreeMap::new();
        for (module_idx, module) in modules.iter().enumerate() {
            let module_name = module_name(module);
            let module_str = module_name.to_string();
            module_name_to_idx.insert(module_name, module_idx);
            for (idx, struct_def) in module.struct_defs().iter().enumerate() {
                let struct_name = format!(
                    "{}_{}",
                    module_str,
                    module
                        .identifier_at(module.struct_handle_at(struct_def.struct_handle).name)
                        .to_string()
                );
                struct_defs.insert(struct_name, idx);
            }
        }
        Self {
            options,
            modules,
            module_infos,
            source_maps,
            struct_defs,
            max_struct_depth: 0,
            module_name_to_idx,
            target_modules: None,
        }
    }

    /// Sets the target modules for this translator. If this is set, output will be pruned to
    /// those target modules (where the compilation scheme allows). This is currently used for
    /// testing only; the produced output will not be accepted by Boogie.
    pub fn set_target_modules(mut self, modules: &[&str]) -> Self {
        self.target_modules = Some(modules.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Returns true of this module should be ignored for output. The module will still be
    /// processed, but all code output for it will be omitted. This is used e.g. to suppress
    /// output for builtin modules which have special semantics in boogie, as 0x0.Vector.
    fn shall_ignore_module(&self, module: &VerifiedModule) -> bool {
        module.name().to_string() == "Vector"
            && *module.address() == AccountAddress::from_hex_literal("0x0").unwrap()
            || match &self.target_modules {
                Some(modules) => !modules.contains(&module.name().to_string()),
                _ => false,
            }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::new();
        // generate names and struct specific functions for all structs
        res.push_str(&self.emit_struct_code());

        for (module_idx, module) in self.modules.iter().enumerate() {
            let mut mt = ModuleTranslator::new(
                self,
                &module,
                &self.module_infos[module_idx],
                &self.source_maps[module_idx],
            );
            res.push_str(&mt.translate());
        }
        res
    }

    pub fn emit_struct_code(&mut self) -> String {
        let mut res = String::new();
        for module in self.modules.iter() {
            let shall_ignore = self.shall_ignore_module(module);
            let mut emit_str = |s: &String| {
                if !shall_ignore {
                    res.push_str(s);
                }
            };
            emit_str(&format!(
                "\n\n// ** structs of module {}\n\n",
                module_name(module)
            ));
            for (def_idx, struct_def) in module.struct_defs().iter().enumerate() {
                // Emit TypeName
                let struct_name = struct_name_from_handle_index(module, struct_def.struct_handle);
                emit_str(&format!("const unique {}: TypeName;\n", struct_name));

                // Emit FieldNames
                let field_infos = get_field_infos(module, struct_def);
                field_infos
                    .iter()
                    .enumerate()
                    .for_each(|(i, (field_name, _))| {
                        emit_str(&format!(
                            "const {}_{}: FieldName;\naxiom {}_{} == {};\n",
                            struct_name, field_name, struct_name, field_name, i
                        ));
                    });

                // Emit TypeValue constructor function.
                let struct_def_view = StructDefinitionView::new(module, struct_def);
                let type_args = struct_def_view
                    .type_formals()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("tv{0}: TypeValue", i))
                    .join(", ");
                let mut field_types = String::from("EmptyTypeValueArray");
                for (_, field_type) in field_infos {
                    field_types = format!(
                        "ExtendTypeValueArray({}, {})",
                        field_types,
                        format_type_value(module, &field_type)
                    );
                }

                let type_value = format!("StructType({}, {})", struct_name, field_types);
                if struct_name == "LibraAccount_T" {
                    // Special treatment of well-known resource LibraAccount_T. The type_value
                    // function is forward-declared in the prelude, here we only add an axiom for
                    // it.
                    emit_str(&format!(
                        "axiom {}_type_value() == {};\n",
                        struct_name, type_value
                    ));
                } else {
                    emit_str(&format!(
                        "function {}_type_value({}): TypeValue {{\n    {}\n}}\n",
                        struct_name, type_args, type_value
                    ));
                }

                // Emit other struct specific boilerplate.
                if !struct_def_view.is_native() {
                    emit_str(&self.emit_struct_specific_functions(module, def_idx));
                    let struct_handle_index = struct_def.struct_handle;

                    // Calculate the max depth of a struct
                    self.max_struct_depth = std::cmp::max(
                        self.max_struct_depth,
                        self.get_struct_depth(
                            module,
                            &SignatureToken::Struct(struct_handle_index, vec![]),
                        ),
                    );
                }
            }
        }
        self.max_struct_depth += 1;
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
    pub fn new(
        parent: &'a BoogieTranslator,
        module: &'a VerifiedModule,
        module_info: &'a ModuleInfo,
        source_map: &'a ModuleSourceMap<Loc>,
    ) -> Self {
        let stackless_bytecode = StacklessModuleGenerator::new(module.as_inner()).generate_module();
        let mut all_type_strs = BTreeSet::new();
        for struct_def in module.struct_defs().iter() {
            let struct_name = struct_name_from_handle_index(module, struct_def.struct_handle);
            all_type_strs.insert(struct_name);
        }
        Self {
            parent,
            module,
            module_info,
            source_map,
            stackless_bytecode,
            all_type_strs,
            ignore: parent.shall_ignore_module(module),
        }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::new();
        if self.ignore {
            return res;
        }
        info!("translating module {}", module_name(self.module));
        res.push_str(&format!(
            "\n\n// ** functions of module {}\n\n",
            module_name(self.module)
        ));
        // translation of stackless bytecode
        for (idx, function_def) in self.module.function_defs().iter().enumerate() {
            if function_def.is_native() {
                if self.parent.options.native_stubs {
                    res.push_str(&self.generate_function_sig(idx, true, &None));
                    res.push_str(";");
                    res.push_str(&self.generate_function_spec(idx, &None));
                    res.push_str("\n");
                }
                continue;
            }
            res.push_str(&self.translate_function(idx));
        }
        res
    }

    pub fn translate_function(&self, idx: usize) -> String {
        let mut res = String::new();
        // generate inline function with function body
        res.push_str(&self.generate_function_sig(idx, true, &None)); // inlined version of function
        res.push_str(&self.generate_function_spec(idx, &None));
        res.push_str(&self.generate_inline_function_body(idx, &None)); // generate function body
        res.push_str("\n");

        // generate non-line function which calls inline version for verification
        res.push_str(&self.generate_function_sig(idx, false, &None)); // no inline
        res.push_str(&self.generate_verify_function_body(idx, &None)); // function body just calls inlined version
        res
    }

    pub fn translate_bytecode(
        &self,
        offset: usize,
        bytecode: &StacklessBytecode,
        func_idx: usize,
        arg_names: &Option<Vec<String>>,
    ) -> (String, String) {
        let fun_name = self.function_name_from_definition_index(func_idx);
        let mut var_decls = String::new();
        let mut res = String::new();
        let propagate_abort = "if (abort_flag) { goto Label_Abort; }".to_string();
        let stmts = match bytecode {
            Branch(target) => vec![format!("goto Label_{};", target)],
            BrTrue(target, idx) => {
                let (dbg_branch_taken_str, dbg_branch_not_taken_str) =
                    if self.dbg_branches_enabled(&fun_name) {
                        let dbg_branch_var_name = format!(
                            "dbg_branch_at_line_{}",
                            self.get_line_number(func_idx, offset)
                        );
                        var_decls.push_str(&format!("    var {} : bool;\n", dbg_branch_var_name));
                        (
                            format!("assume {} == true; ", dbg_branch_var_name),
                            format!("\n    assume {} == false;", dbg_branch_var_name),
                        )
                    } else {
                        (String::new(), String::new())
                    };
                vec![format!(
                    "tmp := GetLocal(m, old_size + {});\nif (b#Boolean(tmp)) {{ {}goto Label_{}; }}{}",
                    idx, dbg_branch_taken_str, target, dbg_branch_not_taken_str
                )]
            }
            BrFalse(target, idx) => {
                let (dbg_branch_taken_str, dbg_branch_not_taken_str) =
                    if self.dbg_branches_enabled(&fun_name) {
                        let dbg_branch_var_name = format!(
                            "dbg_branch_at_line_{}",
                            self.get_line_number(func_idx, offset)
                        );
                        var_decls.push_str(&format!("    var {} : bool;\n", dbg_branch_var_name));
                        (
                            format!("assume {} == true; ", dbg_branch_var_name),
                            format!("\n    assume {} == false;", dbg_branch_var_name),
                        )
                    } else {
                        (String::new(), String::new())
                    };
                vec![format!(
                    "tmp := GetLocal(m, old_size + {});\n    if (!b#Boolean(tmp)) {{ {}goto Label_{}; }}{}",
                    idx, dbg_branch_taken_str, target, dbg_branch_not_taken_str
                )]
            }
            MoveLoc(dest, src) => {
                if self.is_local_ref(*dest, func_idx) {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        self.get_local_name(*src as usize, arg_names)
                    )]
                } else {
                    vec![
                        format!(
                            "call tmp := CopyOrMoveValue(GetLocal(m, old_size + {}));",
                            src
                        ),
                        format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
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
                        format!(
                            "call tmp := CopyOrMoveValue(GetLocal(m, old_size + {}));",
                            src
                        ),
                        format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
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
                        format!(
                            "call tmp := CopyOrMoveValue(GetLocal(m, old_size + {}));",
                            src
                        ),
                        format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                    ]
                }
            }
            BorrowLoc(dest, src) => vec![format!("call t{} := BorrowLoc(old_size+{});", dest, src)],
            ReadRef(dest, src) => vec![
                format!("call tmp := ReadRef(t{});", src),
                format_type_checking(self.module, "tmp".to_string(), &self.get_local_type(*dest, func_idx)),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            WriteRef(dest, src) => vec![format!(
                "call WriteRef(t{}, GetLocal(m, old_size + {}));",
                dest, src
            )],
            FreezeRef(dest, src) => vec![format!("call t{} := FreezeRef(t{});", dest, src)],
            Call(dests, callee_index, type_actuals, args) => {
                let callee_name = self.function_name_from_handle_index(*callee_index);
                let mut dest_str = String::new();
                let mut args_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];

                args_str.push_str(&format_type_actuals(self.module, type_actuals));
                for arg in args.iter() {
                    if !args_str.is_empty() {
                        args_str.push_str(", ");
                    }
                    if self.is_local_ref(*arg, func_idx) {
                        args_str.push_str(&format!("t{}", arg));
                    } else {
                        args_str.push_str(&format!("GetLocal(m, old_size + {})", arg));
                    }
                }
                for (i, dest) in dests.iter().enumerate() {
                    if i > 0 {
                        dest_str.push_str(", ");
                    }
                    dest_str.push_str(&format!("t{}", dest));
                    dest_type_assumptions.push(format_type_checking(self.module,
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                    if !self.is_local_ref(*dest, func_idx) {
                        tmp_assignments.push(format!(
                            "m := UpdateLocal(m, old_size + {}, t{});",
                            dest, dest
                        ));
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
                res_vec.push(propagate_abort);
                res_vec.extend(dest_type_assumptions);
                res_vec.extend(tmp_assignments);
                res_vec
            }
            Pack(dest, struct_def_index, type_actuals, fields) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut fields_str = String::new();
                let mut res_vec = vec![];
                fields_str.push_str(&format_type_actuals(self.module, type_actuals));
                for field_temp in fields.iter() {
                    if !fields_str.is_empty() {
                        fields_str.push_str(", ");
                    }
                    fields_str.push_str(&format!("GetLocal(m, old_size + {})", field_temp));
                    res_vec.push(format_type_checking(self.module,
                        format!("GetLocal(m, old_size + {})", field_temp),
                        &self.get_local_type(*field_temp, func_idx),
                    ));
                }
                res_vec.push(format!("call tmp := Pack_{}({});", struct_str, fields_str));
                res_vec.push(format!("m := UpdateLocal(m, old_size + {}, tmp);", dest));
                res_vec
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_str = self.struct_name_from_definition_index(*struct_def_index);
                let mut dests_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];
                for dest in dests.iter() {
                    if !dests_str.is_empty() {
                        dests_str.push_str(", ");
                    }
                    dests_str.push_str(&format!("t{}", dest));
                    dest_type_assumptions.push(format_type_checking(self.module,
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ));
                    if !self.is_local_ref(*dest, func_idx) {
                        tmp_assignments.push(format!(
                            "m := UpdateLocal(m, old_size + {}, t{});",
                            dest, dest
                        ));
                    }
                }
                let mut res_vec = vec![format!(
                    "call {} := Unpack_{}(GetLocal(m, old_size + {}));",
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
            Exists(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = format_struct_type_value_from_def_idx(
                    self.module,
                    struct_def_index,
                    type_actuals,
                );
                vec![
                    format!(
                        "call tmp := Exists(GetLocal(m, old_size + {}), {});",
                        addr, resource_type
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ]
            }
            BorrowGlobal(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = format_struct_type_value_from_def_idx(
                    self.module,
                    struct_def_index,
                    type_actuals,
                );
                vec![format!(
                    "call t{} := BorrowGlobal(GetLocal(m, old_size + {}), {});",
                    dest, addr, resource_type,
                ), propagate_abort]
            }
            MoveToSender(src, struct_def_index, type_actuals) => {
                let resource_type = format_struct_type_value_from_def_idx(
                    self.module,
                    struct_def_index,
                    type_actuals,
                );
                vec![format!(
                    "call MoveToSender({}, GetLocal(m, old_size + {}));",
                    resource_type, src,
                ), propagate_abort]
            }
            MoveFrom(dest, src, struct_def_index, type_actuals) => {
                let resource_type = format_struct_type_value_from_def_idx(
                    self.module,
                    struct_def_index,
                    type_actuals,
                );
                vec![
                    format!(
                        "call tmp := MoveFrom(GetLocal(m, old_size + {}), {});",
                        src, resource_type,
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                    format_type_checking(self.module,
                        format!("t{}", dest),
                        &self.get_local_type(*dest, func_idx),
                    ),
                    propagate_abort
                ]
            }
            Ret(rets) => {
                let mut ret_assignments = vec![];
                for (i, r) in rets.iter().enumerate() {
                    if self.is_local_ref(*r, func_idx) {
                        ret_assignments.push(format!("ret{} := t{};", i, r));
                    } else {
                        ret_assignments.push(format!("ret{} := GetLocal(m, old_size + {});", i, r));
                    }
                }
                ret_assignments.push("return;".to_string());
                ret_assignments
            }
            LdTrue(idx) => vec![
                "call tmp := LdTrue();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            LdFalse(idx) => vec![
                "call tmp := LdFalse();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            LdU8(idx, num) =>
                vec![
                    format!("call tmp := LdConst({});", num),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
                ],
            LdU64(idx, num) =>
                vec![
                    format!("call tmp := LdConst({});", num),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
                ],
            LdU128(idx, num) =>
                vec![
                    format!("call tmp := LdConst({});", num),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
                ],
            CastU8(dest, src) => vec![
                // TODO: u8 bounds check
                format!("m := UpdateLocal(m, old_size + {}, GetLocal(m, old_size + {}));", dest, src),
            ],
            CastU64(dest, src) =>
                vec![
                // TODO: u64 bounds check
                format!("m := UpdateLocal(m, old_size + {}, GetLocal(m, old_size + {}));", dest, src),
            ],
            CastU128(dest, src) => vec![
                format!("m := UpdateLocal(m, old_size + {}, GetLocal(m, old_size + {}));", dest, src),
            ],
            LdAddr(idx, addr_idx) => {
                let addr = self.module.address_pool()[(*addr_idx).into_index()];
                let addr_int = BigInt::from_str_radix(&addr.to_string(), 16).unwrap();
                vec![
                    format!("call tmp := LdAddr({});", addr_int),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
                ]
            }
            Not(dest, operand) => vec![
                format!("call tmp := Not(GetLocal(m, old_size + {}));", operand),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Add(dest, op1, op2) => vec![
                format!(
                    "call tmp := Add(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Sub(dest, op1, op2) => vec![
                format!(
                    "call tmp := Sub(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Mul(dest, op1, op2) => vec![
                format!(
                    "call tmp := Mul(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Div(dest, op1, op2) => vec![
                format!(
                    "call tmp := Div(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Mod(dest, op1, op2) => vec![
                format!(
                    "call tmp := Mod(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Lt(dest, op1, op2) => vec![
                format!(
                    "call tmp := Lt(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Gt(dest, op1, op2) => vec![
                format!(
                    "call tmp := Gt(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Le(dest, op1, op2) => vec![
                format!(
                    "call tmp := Le(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Ge(dest, op1, op2) => vec![
                format!(
                    "call tmp := Ge(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Or(dest, op1, op2) => vec![
                format!(
                    "call tmp := Or(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            And(dest, op1, op2) => vec![
                format!(
                    "call tmp := And(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Eq(dest, op1, op2) => {
                vec![
                    format!(
                        "tmp := Boolean(IsEqual(GetLocal(m, old_size + {}), GetLocal(m, old_size + {})));",
                        op1,
                        op2
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ]
            }
            Neq(dest, op1, op2) => {
                vec![
                    format!(
                        "tmp := Boolean(!IsEqual(GetLocal(m, old_size + {}), GetLocal(m, old_size + {})));",
                        op1,
                        op2
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ]
            }
            BitOr(_, _, _) | BitAnd(_, _, _) | Xor(_, _, _) => {
                vec!["// bit operation not supported".into()]
            }
            Abort(_) => vec!["goto Label_Abort;".into()],
            GetGasRemaining(idx) => vec![
                "call tmp := GetGasRemaining();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            GetTxnSequenceNumber(idx) => vec![
                "call tmp := GetTxnSequenceNumber();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            GetTxnPublicKey(idx) => vec![
                "call tmp := GetTxnPublicKey();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            GetTxnSenderAddress(idx) => vec![
                "call tmp := GetTxnSenderAddress();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            GetTxnMaxGasUnits(idx) => vec![
                "call tmp := GetTxnMaxGasUnits();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            GetTxnGasUnitPrice(idx) => vec![
                "call tmp := GetTxnGasUnitPrice();".to_string(),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
            ],
            _ => vec!["// unimplemented instruction".into()],
        };
        for code in stmts {
            res.push_str(&format!("    {}\n", code));
        }
        res.push('\n');
        (var_decls, res)
    }

    // return a string for a boogie procedure header.
    // if inline = true, add the inline attribute and use the plain function name
    // for the procedure name. Also inject pre/post conditions if defined.
    // Else, generate the function signature without the ":inline" attribute, and
    // append _verify to the function name.
    pub fn generate_function_sig(
        &self,
        idx: usize,
        inline: bool,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        if self.ignore {
            return "".to_string();
        }
        let function_def = &self.module.function_defs()[idx];
        let fun_name = self.function_name_from_definition_index(idx);
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let mut args = String::new();
        let mut rets = String::new();
        for i in 0..function_signature.type_formals.len() {
            if i > 0 {
                args.push_str(", ");
            }
            args.push_str(&format!("tv{0}: TypeValue", i));
        }
        for (i, arg_type) in function_signature.arg_types.iter().enumerate() {
            if i > 0 || !function_signature.type_formals.is_empty() {
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
                "procedure {{:inline 1}} {} ({}) returns ({})",
                fun_name, args, rets,
            )
        } else {
            format!(
                "procedure {}_verify ({}) returns ({})",
                fun_name, args, rets
            )
        }
    }

    // Return string for the function specification.
    pub fn generate_function_spec(&self, idx: usize, _arg_names: &Option<Vec<String>>) -> String {
        let function_info = &self.module_info.function_infos[idx];
        format!(
            "\n{}",
            &SpecTranslator::new(self.parent.modules, self.module, function_info).translate()
        )
    }

    // return string for body of verify function, which is just a call to the
    // inline version of the function.
    pub fn generate_verify_function_body(
        &self,
        idx: usize,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        if self.ignore {
            return "".to_string();
        }
        let fun_name = self.function_name_from_definition_index(idx);
        let function_def = &self.module.function_defs()[idx];
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let mut args = String::new(); // vector of ", argname"
        let mut rets = String::new(); // vector of ", argname"
                                      // return values are: <mutable references>, <actual returns>
        for i in 0..self
            .module
            .function_signature_at(function_handle.signature)
            .type_formals
            .len()
        {
            if !args.is_empty() {
                args.push_str(", ");
            }
            args.push_str(&format!("tv{}: TypeValue", i));
        }
        for (i, _arg_type) in function_signature.arg_types.iter().enumerate() {
            if !args.is_empty() {
                args.push_str(", ");
            }
            args.push_str(&self.get_arg_name(i, arg_names).to_string());
        }
        // Next loop collects actual return values from Move function
        for i in 0..function_signature.return_types.len() {
            if !rets.is_empty() {
                rets.push_str(", ");
            }
            rets.push_str(&format!("ret{}", i));
        }
        let assumptions = "    assume ExistsTxnSenderAccount(m, txn);\n";
        if function_signature.return_types.is_empty() {
            format!(
                "\n{{\n{}    call {}({});\n}}\n\n",
                assumptions, fun_name, args
            )
        } else {
            format!(
                "\n{{\n{}    call {} := {}({});\n}}\n\n",
                assumptions, rets, fun_name, args
            )
        }
    }

    // This generates boogie code for everything after the function signature
    // The function body is only generated for the "inline" version of the function.
    pub fn generate_inline_function_body(
        &self,
        idx: usize,
        arg_names: &Option<Vec<String>>,
    ) -> String {
        if self.ignore {
            return "".to_string();
        }
        let mut var_decls = String::new();
        let mut res = String::new();
        let function_def = &self.module.function_defs()[idx];
        let code = &self.stackless_bytecode[idx];

        var_decls.push_str("{\n");
        var_decls.push_str("    // declare local variables\n");

        let fun_name = self.function_name_from_definition_index(idx);
        let function_handle = self.module.function_handle_at(function_def.function);
        let function_signature = self.module.function_signature_at(function_handle.signature);
        let num_args = function_signature.arg_types.len();
        let mut ref_vars = BTreeSet::new(); // set of locals that are references
        let mut val_vars = BTreeSet::new(); // set of locals that are not
        let mut arg_assignment_str = String::new();
        let mut arg_value_assumption_str = String::new();
        let mut dbg_arg_assumption_str = String::new();
        for (i, local_type) in code.local_types.iter().enumerate() {
            if i < num_args {
                if !self.is_local_ref(i, idx) {
                    arg_assignment_str.push_str(&format!(
                        "    m := UpdateLocal(m, old_size + {}, {});\n",
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
                    format_type_checking(self.module, self.get_arg_name(i, arg_names), local_type)
                ));
                if self.dbg_args_enabled(&fun_name) {
                    var_decls.push_str(&format!(
                        "    var dbg_param_{}: {};\n",
                        self.get_orig_arg_name(i),
                        self.format_value_or_ref(&local_type)
                    ));
                    dbg_arg_assumption_str.push_str(&format!(
                        "    assume dbg_param_{} == {};\n",
                        self.get_orig_arg_name(i),
                        self.get_arg_name(i, arg_names)
                    ));
                }
            }
            if SignatureTokenView::new(self.module, local_type).is_reference() {
                ref_vars.insert(i);
            } else {
                val_vars.insert(i);
            }
            var_decls.push_str(&format!(
                "    var {}: {}; // {}\n",
                self.get_local_name(i, arg_names),
                self.format_value_or_ref(&local_type),
                format_type_value(self.module, &local_type)
            ));
        }
        var_decls.push_str("\n    var tmp: Value;\n");
        var_decls.push_str("    var old_size: int;\n");
        res.push_str("\n    var saved_m: Memory;\n");
        res.push_str("    assume !abort_flag;\n");
        res.push_str("    saved_m := m;\n");
        res.push_str("\n    // assume arguments are of correct types\n");
        res.push_str(&arg_value_assumption_str);
        res.push_str("\n    old_size := local_counter;\n");
        res.push_str(&format!(
            "    local_counter := local_counter + {};\n",
            code.local_types.len()
        ));
        res.push_str(&arg_assignment_str);
        if self.dbg_args_enabled(&fun_name) {
            res.push_str("\n    // record values of parameters\n");
            res.push_str(&dbg_arg_assumption_str);
        }
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
            let (new_var_decls, new_res) =
                self.translate_bytecode(offset, bytecode, idx, arg_names);
            var_decls.push_str(&new_var_decls);
            res.push_str(&new_res);
        }
        res.push_str("Label_Abort:\n");
        res.push_str("    abort_flag := true;\n");
        res.push_str("    m := saved_m;\n");
        for (i, sig) in get_return_types(self.module, idx).iter().enumerate() {
            if let SignatureToken::Reference(_) = sig {
                res.push_str(&format!("    ret{} := DefaultReference;\n", i));
            } else {
                res.push_str(&format!("    ret{} := DefaultValue;\n", i));
            }
        }
        res.push_str("}\n");
        var_decls.push_str(&res);
        var_decls
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

    // FIXME: Stub for now: eventually get source-level name of arg
    pub fn get_orig_arg_name(&self, idx: usize) -> String {
        format!("arg{}", idx)
    }

    // Currently gets byte offset, not line number
    pub fn get_line_number(&self, func_idx: usize, offset: usize) -> usize {
        let function_definition_index = FunctionDefinitionIndex(func_idx as u16);
        let loc = self
            .source_map
            .get_code_location(function_definition_index, offset as u16)
            .unwrap();
        loc.start().to_usize()
    }

    // Stubs for now: eventually should have a command-line or other flag to enable or disable debugging info.
    pub fn dbg_args_enabled(&self, _fun_name: &str) -> bool {
        false
    }

    pub fn dbg_branches_enabled(&self, _fun_name: &str) -> bool {
        false
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
}

pub fn format_type_checking(
    _module: &VerifiedModule,
    name: String,
    sig: &SignatureToken,
) -> String {
    let mut params = name;
    let mut ret = String::new();
    let check = match sig {
        SignatureToken::U8 | SignatureToken::U64 | SignatureToken::U128 => "IsValidInteger",
        SignatureToken::Bool => "is#Boolean",
        SignatureToken::Address => "is#Address",
        SignatureToken::ByteArray => "is#ByteArray",
        // Only need to check Struct for top-level; fields will be checked as we extract them.
        SignatureToken::Struct(_, _) => "is#Vector",
        SignatureToken::Reference(rtype) | SignatureToken::MutableReference(rtype) => {
            let n = format!("Dereference(m, {})", params);
            ret = format_type_checking(_module, n, rtype);
            ret += "    ";
            params = format!("m, local_counter, {}", params);
            "IsValidReferenceParameter"
        }
        // Otherwise it is a type parameter which is opaque
        SignatureToken::TypeParameter(_) => "",
    };
    let ret2 = if check.is_empty() {
        "".to_string()
    } else {
        format!("assume {}({});\n", check, params)
    };
    ret + &ret2
}

pub fn struct_name_from_handle_index(module: &VerifiedModule, idx: StructHandleIndex) -> String {
    let struct_handle = module.struct_handle_at(idx);
    let struct_handle_view = StructHandleView::new(module, struct_handle);
    let module_name = module.identifier_at(struct_handle_view.module_handle().name);
    let struct_name = struct_handle_view.name();
    format!("{}_{}", module_name, struct_name)
}

pub fn module_name(module: &VerifiedModule) -> Identifier {
    module
        .identifier_at(module.module_handle_at(ModuleHandleIndex::new(0)).name)
        .into()
}

pub fn struct_type_arity_from_handle_index(
    module: &VerifiedModule,
    idx: StructHandleIndex,
) -> usize {
    let struct_handle = module.struct_handle_at(idx);
    let struct_handle_view = StructHandleView::new(module, struct_handle);
    struct_handle_view.type_formals().len()
}

pub fn is_struct_vector(module: &VerifiedModule, idx: StructHandleIndex) -> bool {
    let struct_handle = module.struct_handle_at(idx);
    let struct_handle_view = StructHandleView::new(module, struct_handle);
    let module_name = module.identifier_at(struct_handle_view.module_handle().name);
    let module_address = module.address_at(struct_handle_view.module_handle().address);
    module_name.to_string() == "Vector"
        && *module_address == AccountAddress::from_hex_literal("0x0").unwrap()
}

pub fn format_type(module: &VerifiedModule, sig: &SignatureToken) -> String {
    match sig {
        SignatureToken::Bool => "bool".into(),
        SignatureToken::U8 => unimplemented!(),
        SignatureToken::U64 => "int".into(),
        SignatureToken::U128 => unimplemented!(),
        SignatureToken::ByteArray => "bytearray".into(),
        SignatureToken::Address => "address".into(),
        SignatureToken::Struct(idx, _) => struct_name_from_handle_index(module, *idx),
        SignatureToken::Reference(t) | SignatureToken::MutableReference(t) => {
            format!("{}_ref", format_type(module, &*t))
        }
        SignatureToken::TypeParameter(_) => "typeparam".into(),
    }
}

/// Creates the boogie representation of a TypeValue from a SignatureToken. For type
/// parameters, this generates a reference to `tvN` (type argument N) which must be bound
/// in the context. For structs, instead of inlining the type value (which is not possible
/// for an imported module anyway), we call the function `<struct_name>_type_value(type_args)`.
pub fn format_type_value(module: &VerifiedModule, sig: &SignatureToken) -> String {
    match sig {
        SignatureToken::Bool => "BooleanType()".to_string(),
        SignatureToken::U8 | SignatureToken::U64 | SignatureToken::U128 => {
            "IntegerType()".to_string()
        }
        SignatureToken::ByteArray => "ByteArrayType()".to_string(),
        SignatureToken::Address => "AddressType()".to_string(),
        SignatureToken::Reference(t) | SignatureToken::MutableReference(t) => {
            format!("ReferenceType({})", format_type_value(module, &*t))
        }
        SignatureToken::TypeParameter(index) => format!("tv{}", index),
        SignatureToken::Struct(handle_index, args) => {
            format_struct_type_value(module, handle_index, args)
        }
    }
}

/// Create list of type values.
pub fn format_type_values(module: &VerifiedModule, actuals: &[SignatureToken]) -> String {
    actuals
        .iter()
        .map(|sig| format_type_value(module, sig))
        .join(", ")
}

/// Create list of actual type parameters from LocalSignatureIndex.
pub fn format_type_actuals(
    module: &VerifiedModule,
    type_actuals_idx: &LocalsSignatureIndex,
) -> String {
    format_type_values(module, &module.locals_signature_at(*type_actuals_idx).0)
}

/// Create type value for a struct with given type actuals.
pub fn format_struct_type_value(
    module: &VerifiedModule,
    struct_handle_idx: &StructHandleIndex,
    args: &[SignatureToken],
) -> String {
    let struct_name = struct_name_from_handle_index(module, *struct_handle_idx);
    format!(
        "{}_type_value({})",
        struct_name,
        format_type_values(module, args)
    )
}

/// Create type value for a struct specified by a definition index and actuals
/// specified by LocalsSignatureIndex.
pub fn format_struct_type_value_from_def_idx(
    module: &VerifiedModule,
    struct_def_index: &StructDefinitionIndex,
    type_actuals_idx: &LocalsSignatureIndex,
) -> String {
    format_struct_type_value(
        module,
        &module.struct_def_at(*struct_def_index).struct_handle,
        &module.locals_signature_at(*type_actuals_idx).0,
    )
}

/// Return a vector of pairs of field name and type for a StructDefinition.
pub fn get_field_infos(
    module: &VerifiedModule,
    struct_def: &StructDefinition,
) -> Vec<(String, SignatureToken)> {
    let struct_def_view = StructDefinitionView::new(module, struct_def);
    if struct_def_view.is_native() {
        vec![]
    } else {
        struct_def_view
            .fields()
            .unwrap()
            .map(|field_def_view| {
                (
                    field_def_view.name().to_string(),
                    field_def_view.type_signature().token().as_inner().clone(),
                )
            })
            .collect()
    }
}

/// Get return types of function via definition index.
pub fn get_return_types(module: &VerifiedModule, idx: usize) -> &[SignatureToken] {
    let function_def = &module.function_defs()[idx];
    let function_handle = module.function_handle_at(function_def.function);
    let function_signature = module.function_signature_at(function_handle.signature);
    &function_signature.return_types
}
