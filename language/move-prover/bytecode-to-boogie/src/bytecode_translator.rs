// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::BTreeSet;

use itertools::Itertools;
use log::info;

use stackless_bytecode_generator::{
    stackless_bytecode::StacklessBytecode::{self, *},
    stackless_bytecode_generator::{StacklessFunction, StacklessModuleGenerator},
};

use crate::boogie_helpers::{
    boogie_field_name, boogie_function_name, boogie_local_type, boogie_struct_name,
    boogie_struct_type_value, boogie_type_check, boogie_type_value, boogie_type_values,
};
use crate::env::{
    FunctionEnv, GlobalEnv, GlobalType, ModuleEnv, Parameter, StructEnv, TypeParameter,
};
use crate::spec_translator::SpecTranslator;
use libra_types::account_address::AccountAddress;
use libra_types::language_storage::ModuleId;

pub struct BoogieTranslator<'env> {
    pub env: &'env GlobalEnv,
}

pub struct ModuleTranslator<'env> {
    pub parent: &'env BoogieTranslator<'env>,
    pub module_env: ModuleEnv<'env>,
    pub stackless_bytecode: Vec<StacklessFunction>,
}

/// Returns true if for the module no code should be produced because its already defined
/// in the prelude.
pub fn is_module_provided_by_prelude(id: &ModuleId) -> bool {
    id.name().as_str() == "Vector"
        && *id.address() == AccountAddress::from_hex_literal("0x0").unwrap()
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(env: &'env GlobalEnv) -> Self {
        Self { env }
    }

    pub fn translate(&mut self) -> String {
        let mut res = String::new();

        // generate definitions for all modules.
        for module_env in self.env.get_modules() {
            let mut mt = ModuleTranslator::new(self, module_env);
            res.push_str(&mt.translate());
        }
        res
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Creates a new module translator. Calls the stackless bytecode generator and wraps
    /// result into the translator.
    fn new(parent: &'env BoogieTranslator, module: ModuleEnv<'env>) -> Self {
        let stackless_bytecode =
            StacklessModuleGenerator::new(module.get_verified_module().as_inner())
                .generate_module();
        Self {
            parent,
            module_env: module,
            stackless_bytecode,
        }
    }

    /// Translates this module.
    fn translate(&mut self) -> String {
        let mut res = String::new();
        if !is_module_provided_by_prelude(self.module_env.get_id()) {
            info!("translating module {}", self.module_env.get_id().name());
            res.push_str(&self.translate_structs());
            res.push_str(&self.translate_functions());
        }
        res
    }

    /// Translates all structs in the module.
    fn translate_structs(&self) -> String {
        let mut res = String::new();
        res.push_str(&format!(
            "\n\n// ** structs of module {}\n\n",
            self.module_env.get_id().name()
        ));
        for struct_env in self.module_env.get_structs() {
            res.push_str(&self.translate_struct_type(&struct_env));
            if !struct_env.is_native() {
                res.push_str(&self.translate_struct_accessors(&struct_env));
            }
        }
        res
    }

    /// Translates the given struct.
    fn translate_struct_type(&self, struct_env: &StructEnv<'_>) -> String {
        let mut res = String::new();
        // Emit TypeName
        let struct_name = boogie_struct_name(&struct_env);
        res.push_str(&format!("const unique {}: TypeName;\n", struct_name));

        // Emit FieldNames
        for (i, field_env) in struct_env.get_fields().enumerate() {
            let field_name = boogie_field_name(&field_env);
            res.push_str(&format!(
                "const {}: FieldName;\naxiom {} == {};\n",
                field_name, field_name, i
            ));
        }

        // Emit TypeValue constructor function.
        let type_args = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("tv{}: TypeValue", i))
            .join(", ");
        let mut field_types = String::from("EmptyTypeValueArray");
        for field_env in struct_env.get_fields() {
            field_types = format!(
                "ExtendTypeValueArray({}, {})",
                field_types,
                boogie_type_value(self.module_env.env, &field_env.get_type())
            );
        }
        let type_value = format!("StructType({}, {})", struct_name, field_types);
        if struct_name == "LibraAccount_T" {
            // Special treatment of well-known resource LibraAccount_T. The type_value
            // function is forward-declared in the prelude, here we only add an axiom for
            // it.
            res.push_str(&format!(
                "axiom {}_type_value() == {};\n",
                struct_name, type_value
            ));
        } else {
            res.push_str(&format!(
                "function {}_type_value({}): TypeValue {{\n    {}\n}}\n",
                struct_name, type_args, type_value
            ));
        }
        res
    }

    /// Translates struct accessors (pack/unpack).
    fn translate_struct_accessors(&self, struct_env: &StructEnv<'_>) -> String {
        let mut res = String::new();

        // Pack function
        let type_args_str = struct_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(ref i, _)| format!("{}: TypeValue", i))
            .join(", ");
        let args_str = struct_env
            .get_fields()
            .map(|field_env| format!("{}: Value", field_env.get_name()))
            .join(", ");
        res.push_str(&format!(
            "procedure {{:inline 1}} Pack_{}({}) returns (_struct: Value)\n{{\n",
            boogie_struct_name(struct_env),
            if !args_str.is_empty() && !type_args_str.is_empty() {
                format!("{}, {}", type_args_str, args_str)
            } else if args_str.is_empty() {
                type_args_str
            } else {
                args_str.clone()
            }
        ));
        let mut fields_str = String::from("EmptyValueArray");
        for field_env in struct_env.get_fields() {
            let type_check = boogie_type_check(
                self.module_env.env,
                field_env.get_name().as_str(),
                &field_env.get_type(),
            );
            if !type_check.is_empty() {
                res.push_str(&format!("    {}", type_check));
            }
            fields_str = format!("ExtendValueArray({}, {})", fields_str, field_env.get_name());
        }
        res.push_str(&format!("    _struct := Vector({});\n", fields_str));
        res.push_str("\n}\n\n");

        // Unpack function
        res.push_str(&format!(
            "procedure {{:inline 1}} Unpack_{}(_struct: Value) returns ({})\n{{\n",
            boogie_struct_name(struct_env),
            args_str
        ));
        res.push_str("    assume is#Vector(_struct);\n");
        for field_env in struct_env.get_fields() {
            res.push_str(&format!(
                "    {} := SelectField(_struct, {});\n",
                field_env.get_name(),
                boogie_field_name(&field_env)
            ));
            let type_check = boogie_type_check(
                self.module_env.env,
                field_env.get_name().as_str(),
                &field_env.get_type(),
            );
            if !type_check.is_empty() {
                res.push_str(&format!("    {}", type_check));
            }
        }
        res.push_str("}\n\n");

        res
    }

    /// Translates all functions in the module.
    fn translate_functions(&self) -> String {
        let mut res = String::new();
        res.push_str(&format!(
            "\n\n// ** functions of module {}\n\n",
            self.module_env.get_id().name()
        ));
        for func_env in self.module_env.get_functions() {
            res.push_str(&self.translate_function(&func_env));
        }
        res
    }

    /// Translates the given function.
    fn translate_function(&self, func_env: &FunctionEnv<'_>) -> String {
        let mut res = String::new();
        if func_env.is_native() {
            if self.module_env.env.options.native_stubs {
                res.push_str(&self.generate_function_sig(func_env, true));
                res.push_str(";");
                res.push_str(&self.generate_function_spec(func_env));
                res.push_str("\n");
            }
            return res;
        }

        // generate inline function with function body
        res.push_str(&self.generate_function_sig(func_env, true)); // inlined version of function
        res.push_str(&self.generate_function_spec(func_env));
        res.push_str(&self.generate_inline_function_body(func_env)); // generate function body
        res.push_str("\n");

        // generate the _verify version of the function which calls inline version for standalone
        // verification.
        res.push_str(&self.generate_function_sig(func_env, false)); // no inline
        res.push_str(&self.generate_verify_function_body(func_env)); // function body just calls inlined version
        res
    }

    /// Translates one bytecode instruction.
    fn translate_bytecode(
        &self,
        func_env: &FunctionEnv<'_>,
        _offset: usize,
        bytecode: &StacklessBytecode,
    ) -> (String, String) {
        let var_decls = String::new();
        let mut res = String::new();
        let propagate_abort = "if (abort_flag) { goto Label_Abort; }".to_string();
        let stmts = match bytecode {
            Branch(target) => vec![format!("goto Label_{};", target)],
            BrTrue(target, idx) => {
                vec![format!(
                    "tmp := GetLocal(m, old_size + {});\n    if (b#Boolean(tmp)) {{ goto Label_{}; }}",
                    idx, target,
                )]
            }
            BrFalse(target, idx) => {
                vec![format!(
                    "tmp := GetLocal(m, old_size + {});\n    if (!b#Boolean(tmp)) {{ goto Label_{}; }}",
                    idx, target,
                )]
            }
            MoveLoc(dest, src) => {
                if self.get_local_type(func_env, *dest).is_reference() {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        func_env.get_local_name(*src)
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
                if self.get_local_type(func_env, *dest).is_reference() {
                    vec![format!(
                        "call t{} := CopyOrMoveRef({});",
                        dest,
                        func_env.get_local_name(*src)
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
                if self.get_local_type(func_env, *dest as usize).is_reference() {
                    vec![format!(
                        "call {} := CopyOrMoveRef(t{});",
                        func_env.get_local_name(*dest),
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
                boogie_type_check(self.module_env.env, "tmp", &self.get_local_type(func_env, *dest)),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            WriteRef(dest, src) => vec![format!(
                "call WriteRef(t{}, GetLocal(m, old_size + {}));",
                dest, src
            )],
            FreezeRef(dest, src) => vec![format!("call t{} := FreezeRef(t{});", dest, src)],
            Call(dests, callee_index, type_actuals, args) => {
                let (callee_module_env, callee_def_idx) = self.module_env.get_callee_info(callee_index);
                let callee_env = callee_module_env.get_function(&callee_def_idx);
                let mut dest_str = String::new();
                let mut args_str = String::new();
                let mut dest_type_assumptions = vec![];
                let mut tmp_assignments = vec![];

                args_str.push_str(&boogie_type_values(func_env.module_env.env, &func_env.module_env.get_type_actuals(*type_actuals)));
                if !args_str.is_empty() && !args.is_empty() {
                    args_str.push_str(", ");
                }
                args_str.push_str(
                    &args.iter().map(|arg_idx| {
                        if self.get_local_type(func_env, *arg_idx).is_reference() {
                            format!("t{}", arg_idx)
                        } else {
                            format!("GetLocal(m, old_size + {})", arg_idx)
                        }
                    }).join(", "));
                dest_str.push_str(&dests.iter().map(|dest_idx| {
                    let dest = format!("t{}", dest_idx);
                    let dest_type = &self.get_local_type(func_env, *dest_idx);
                    dest_type_assumptions.push(boogie_type_check(self.module_env.env,
                                                                 &dest,
                                                                 dest_type));
                    if !dest_type.is_reference() {
                        tmp_assignments.push(format!(
                            "m := UpdateLocal(m, old_size + {}, t{});",
                            dest_idx, dest_idx
                        ));
                    }
                    dest
                }).join(", "));
                let mut res_vec = vec![];
                if dest_str == "" {
                    res_vec.push(format!("call {}({});", boogie_function_name(&callee_env), args_str))
                } else {
                    res_vec.push(format!(
                        "call {} := {}({});",
                        dest_str, boogie_function_name(&callee_env), args_str
                    ));
                }
                res_vec.push(propagate_abort);
                res_vec.extend(dest_type_assumptions);
                res_vec.extend(tmp_assignments);
                res_vec
            }
            Pack(dest, struct_def_index, type_actuals, fields) => {
                let struct_env = func_env.module_env.get_struct(struct_def_index);
                let args_str =
                    func_env.module_env.get_type_actuals(*type_actuals)
                        .iter()
                        .map(|s| boogie_type_value(self.module_env.env, s))
                        .chain(
                            fields.iter()
                                .map(|i| format!("GetLocal(m, old_size + {})", i))
                        )
                        .join(", ");
                let mut res_vec = vec![];
                res_vec.push(format!("call tmp := Pack_{}({});", boogie_struct_name(&struct_env), args_str));
                res_vec.push(format!("m := UpdateLocal(m, old_size + {}, tmp);", dest));
                res_vec
            }
            Unpack(dests, struct_def_index, _, src) => {
                let struct_env = &func_env.module_env.get_struct(struct_def_index);
                let mut dests_str = String::new();
                let mut tmp_assignments = vec![];
                for dest in dests.iter() {
                    if !dests_str.is_empty() {
                        dests_str.push_str(", ");
                    }
                    let dest_str = &format!("t{}", dest);
                    let dest_type = &self.get_local_type(func_env, *dest);
                    dests_str.push_str(dest_str);
                    if !dest_type.is_reference() {
                        tmp_assignments.push(format!(
                            "m := UpdateLocal(m, old_size + {}, t{});",
                            dest, dest
                        ));
                    }
                }
                let mut res_vec = vec![format!(
                    "call {} := Unpack_{}(GetLocal(m, old_size + {}));",
                    dests_str, boogie_struct_name(struct_env), src
                )];
                res_vec.extend(tmp_assignments);
                res_vec
            }
            BorrowField(dest, src, field_def_index) => {
                let struct_env = self.module_env.get_struct_of_field(field_def_index);
                let field_env = &struct_env.get_field(field_def_index);
                vec![format!(
                    "call t{} := BorrowField(t{}, {});",
                    dest, src, boogie_field_name(field_env)
                )]
            }
            Exists(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(self.module_env.env, self.module_env.get_module_idx(),
                                                             struct_def_index, &self.module_env.get_type_actuals(*type_actuals));
                vec![
                    format!(
                        "call tmp := Exists(GetLocal(m, old_size + {}), {});",
                        addr, resource_type
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ]
            }
            BorrowGlobal(dest, addr, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(self.module_env.env, self.module_env.get_module_idx(),
                                                             struct_def_index, &self.module_env.get_type_actuals(*type_actuals));
                vec![format!(
                    "call t{} := BorrowGlobal(GetLocal(m, old_size + {}), {});",
                    dest, addr, resource_type,
                ), propagate_abort]
            }
            MoveToSender(src, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(self.module_env.env, self.module_env.get_module_idx(),
                                                             struct_def_index, &self.module_env.get_type_actuals(*type_actuals));
                vec![format!(
                    "call MoveToSender({}, GetLocal(m, old_size + {}));",
                    resource_type, src,
                ), propagate_abort]
            }
            MoveFrom(dest, src, struct_def_index, type_actuals) => {
                let resource_type = boogie_struct_type_value(self.module_env.env, self.module_env.get_module_idx(),
                                                             struct_def_index, &self.module_env.get_type_actuals(*type_actuals));
                vec![
                    format!(
                        "call tmp := MoveFrom(GetLocal(m, old_size + {}), {});",
                        src, resource_type,
                    ),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                    boogie_type_check(self.module_env.env, &format!("t{}", dest),
                                      &self.get_local_type(func_env, *dest)),
                    propagate_abort
                ]
            }
            Ret(rets) => {
                let mut ret_assignments = vec![];
                for (i, r) in rets.iter().enumerate() {
                    if self.get_local_type(func_env, *r).is_reference() {
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
            CastU8(dest, src) =>
                vec![
                    format!("call tmp := CastU8(GetLocal(m, old_size + {}));", src),
                    propagate_abort,
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ],
            CastU64(dest, src) =>
                vec![
                    format!("call tmp := CastU64(GetLocal(m, old_size + {}));", src),
                    propagate_abort,
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ],
            CastU128(dest, src) =>
                vec![
                    format!("call tmp := CastU128(GetLocal(m, old_size + {}));", src),
                    propagate_abort,
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ],
            LdAddr(idx, addr_idx) => {
                let addr_int = self.module_env.get_address(addr_idx);
                vec![
                    format!("call tmp := LdAddr({});", addr_int),
                    format!("m := UpdateLocal(m, old_size + {}, tmp);", idx),
                ]
            }
            Not(dest, operand) => vec![
                format!("call tmp := Not(GetLocal(m, old_size + {}));", operand),
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Add(dest, op1, op2) => {
                let add_type = match self.get_local_type(func_env, *dest) {
                    GlobalType::U8 => "U8",
                    GlobalType::U64 => "U64",
                    GlobalType::U128 => "U128",
                    _ => unreachable!()
                };
                vec![
                format!(
                    "call tmp := Add{}(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    add_type, op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
                ]
            }
            Sub(dest, op1, op2) => vec![
                format!(
                    "call tmp := Sub(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ],
            Mul(dest, op1, op2) => {
                let mul_type = match self.get_local_type(func_env, *dest) {
                    GlobalType::U8 => "U8",
                    GlobalType::U64 => "U64",
                    GlobalType::U128 => "U128",
                    _ => unreachable!()
                };
                vec![
                format!(
                    "call tmp := Mul{}(GetLocal(m, old_size + {}), GetLocal(m, old_size + {}));",
                    mul_type,
                    op1, op2
                ),
                propagate_abort,
                format!("m := UpdateLocal(m, old_size + {}, tmp);", dest),
            ]
            }
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

    /// Return a string for a boogie procedure header.
    /// if inline = true, add the inline attribute and use the plain function name
    /// for the procedure name. Also inject pre/post conditions if defined.
    /// Else, generate the function signature without the ":inline" attribute, and
    /// append _verify to the function name.
    fn generate_function_sig(&self, func_env: &FunctionEnv<'_>, inline: bool) -> String {
        let (args, rets) = self.generate_function_args_and_returns(func_env);
        if inline {
            format!(
                "procedure {{:inline 1}} {} ({}) returns ({})",
                boogie_function_name(func_env),
                args,
                rets,
            )
        } else {
            format!(
                "procedure {}_verify ({}) returns ({})",
                boogie_function_name(func_env),
                args,
                rets
            )
        }
    }

    /// Generate boogie representation of function args and return args.
    fn generate_function_args_and_returns(&self, func_env: &FunctionEnv<'_>) -> (String, String) {
        let args = func_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(ref i, _)| format!("{}: TypeValue", i))
            .chain(
                func_env
                    .get_parameters()
                    .iter()
                    .map(|Parameter(ref i, ref s)| format!("{}: {}", i, boogie_local_type(s))),
            )
            .join(", ");
        let rets = func_env
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("ret{}: {}", i, boogie_local_type(s)))
            .join(", ");
        (args, rets)
    }

    /// Return string for the function specification.
    fn generate_function_spec(&self, func_env: &FunctionEnv<'_>) -> String {
        format!("\n{}", &SpecTranslator::new(func_env).translate())
    }

    /// Return string for body of verify function, which is just a call to the
    /// inline version of the function.
    fn generate_verify_function_body(&self, func_env: &FunctionEnv<'_>) -> String {
        let args = func_env
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(i, _)| i.as_str().to_string())
            .chain(
                func_env
                    .get_parameters()
                    .iter()
                    .map(|Parameter(i, _)| i.as_str().to_string()),
            )
            .join(", ");
        let rets = (0..func_env.get_return_types().len())
            .map(|i| format!("ret{}", i))
            .join(", ");
        let assumptions = "    assume ExistsTxnSenderAccount(m, txn);\n";
        if rets.is_empty() {
            format!(
                "\n{{\n{}    call {}({});\n}}\n\n",
                assumptions,
                boogie_function_name(func_env),
                args
            )
        } else {
            format!(
                "\n{{\n{}    call {} := {}({});\n}}\n\n",
                assumptions,
                rets,
                boogie_function_name(func_env),
                args
            )
        }
    }

    /// This generates boogie code for everything after the function signature
    /// The function body is only generated for the "inline" version of the function.
    fn generate_inline_function_body(&self, func_env: &FunctionEnv<'_>) -> String {
        let mut var_decls = String::new();
        let mut res = String::new();
        let code = &self.stackless_bytecode[func_env.get_def_idx().0 as usize];

        var_decls.push_str("{\n");
        var_decls.push_str("    // declare local variables\n");

        let num_args = func_env.get_parameters().len();
        let mut arg_assignment_str = String::new();
        let mut arg_value_assumption_str = String::new();
        for (i, local_type) in code.local_types.iter().enumerate() {
            let local_name = func_env.get_local_name(i as u8);
            let local_type = &self.module_env.globalize_signature(local_type);
            if i < num_args {
                if !local_type.is_reference() {
                    arg_assignment_str.push_str(&format!(
                        "    m := UpdateLocal(m, old_size + {}, {});\n",
                        i, local_name
                    ));
                }
                arg_value_assumption_str.push_str(&format!(
                    "    {}",
                    boogie_type_check(self.module_env.env, &local_name, local_type)
                ));
            } else {
                var_decls.push_str(&format!(
                    "    var {}: {}; // {}\n",
                    local_name,
                    boogie_local_type(local_type),
                    boogie_type_value(self.module_env.env, local_type)
                ));
            }
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
            let (new_var_decls, new_res) = self.translate_bytecode(func_env, offset, bytecode);
            var_decls.push_str(&new_var_decls);
            res.push_str(&new_res);
        }
        res.push_str("Label_Abort:\n");
        res.push_str("    abort_flag := true;\n");
        res.push_str("    m := saved_m;\n");
        for (i, ref sig) in func_env.get_return_types().iter().enumerate() {
            if sig.is_reference() {
                res.push_str(&format!("    ret{} := DefaultReference;\n", i));
            } else {
                res.push_str(&format!("    ret{} := DefaultValue;\n", i));
            }
        }
        res.push_str("}\n");
        var_decls.push_str(&res);
        var_decls
    }

    /// Looks up the type of a local in the stackless bytecode representation.
    fn get_local_type(&self, func_env: &FunctionEnv<'_>, local_idx: usize) -> GlobalType {
        self.module_env.globalize_signature(
            &self.stackless_bytecode[func_env.get_def_idx().0 as usize].local_types[local_idx],
        )
    }
}
