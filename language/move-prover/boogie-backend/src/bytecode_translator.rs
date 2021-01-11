// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates the bytecode of a module to Boogie code.

use std::collections::BTreeMap;

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, log, warn, Level};

use bytecode::{
    function_target::FunctionTarget,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::{BorrowNode, Bytecode, Constant, Operation},
    usage_analysis,
};
use move_model::{
    code_writer::CodeWriter,
    emit, emitln,
    model::{GlobalEnv, ModuleEnv, StructEnv, TypeParameter},
    pragmas::{ADDITION_OVERFLOW_UNCHECKED_PRAGMA, SEED_PRAGMA, TIMEOUT_PRAGMA},
    ty::{PrimitiveType, Type},
};
use vm::file_format::CodeOffset;

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_caller_resource_memory_domain_name,
        boogie_debug_track_abort_via_attrib, boogie_debug_track_local_via_attrib,
        boogie_field_name, boogie_function_name, boogie_local_type, boogie_resource_memory_name,
        boogie_self_resource_memory_domain_name, boogie_struct_name, boogie_type_value,
        boogie_type_value_array, boogie_type_value_array_from_strings, boogie_type_values,
        boogie_well_formed_check,
    },
    options::BoogieOptions,
    spec_translator::SpecTranslator,
};
use bytecode::{
    function_target_pipeline::FunctionVariant,
    stackless_bytecode::{PropKind, TempIndex},
};
use move_model::{model::Loc, symbol::Symbol};

pub struct BoogieTranslator<'env> {
    env: &'env GlobalEnv,
    options: &'env BoogieOptions,
    writer: &'env CodeWriter,
    targets: &'env FunctionTargetsHolder,
}

pub struct ModuleTranslator<'env> {
    writer: &'env CodeWriter,
    module_env: ModuleEnv<'env>,
    spec_translator: SpecTranslator<'env>,
    options: &'env BoogieOptions,
    targets: &'env FunctionTargetsHolder,
}

impl<'env> BoogieTranslator<'env> {
    pub fn new(
        env: &'env GlobalEnv,
        options: &'env BoogieOptions,
        targets: &'env FunctionTargetsHolder,
        writer: &'env CodeWriter,
    ) -> Self {
        Self {
            env,
            options,
            targets,
            writer,
        }
    }

    pub fn translate(&mut self) {
        // generate definitions for all modules.
        for module_env in self.env.get_modules() {
            ModuleTranslator::new(self, module_env).translate();
        }
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Creates a new module translator.
    fn new(parent: &'env BoogieTranslator, module: ModuleEnv<'env>) -> Self {
        Self {
            writer: parent.writer,
            options: parent.options,
            module_env: module,
            spec_translator: SpecTranslator::new(parent.writer, &parent.env, parent.options),
            targets: &parent.targets,
        }
    }

    /// Translates this module.
    fn translate(&mut self) {
        log!(
            if self.module_env.is_dependency() {
                Level::Debug
            } else {
                Level::Info
            },
            "translating module {}",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        self.writer
            .set_location(&self.module_env.env.internal_loc());
        self.spec_translator.translate_spec_vars(&self.module_env);
        self.spec_translator.translate_spec_funs(&self.module_env);
        self.translate_structs();
        // Don't translate functions if the module doesn't need to be translated
        if !self.module_env.should_translate() {
            return;
        }
        self.translate_functions();
    }

    /// Translates all structs in the module.
    fn translate_structs(&self) {
        emitln!(
            self.writer,
            "\n\n// ** structs of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        for struct_env in self.module_env.get_structs() {
            // Set the location to internal so we don't see locations of pack/unpack
            // in execution traces.
            self.writer
                .set_location(&self.module_env.env.internal_loc());
            self.translate_struct_type(&struct_env);
        }
    }

    /// Translates the given struct.
    fn translate_struct_type(&self, struct_env: &StructEnv<'_>) {
        // Emit TypeName
        let struct_name = boogie_struct_name(&struct_env);
        emitln!(self.writer, "const unique {}: $TypeName;", struct_name);

        // Emit FieldNames
        for (i, field_env) in struct_env.get_fields().enumerate() {
            let field_name = boogie_field_name(&field_env);
            emitln!(
                self.writer,
                "const {}: $FieldName;\naxiom {} == {};",
                field_name,
                field_name,
                i
            );
        }

        // Emit TypeValue constructor function.
        let type_params = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("$tv{}: $TypeValue", i))
            .join(", ");
        let type_args = struct_env
            .get_type_parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| Type::TypeParameter(i as u16))
            .collect_vec();
        let type_args_array = boogie_type_value_array(struct_env.module_env.env, &type_args);
        let type_value = format!("$StructType({}, {})", struct_name, type_args_array);
        emitln!(
            self.writer,
            "function {}_type_value({}): $TypeValue {{\n    {}\n}}",
            struct_name,
            type_params,
            type_value
        );

        // Emit memory variable.
        let memory_name = boogie_resource_memory_name(
            struct_env.module_env.env,
            struct_env.get_qualified_id(),
            &None,
        );
        emitln!(self.writer, "var {}: $Memory;", memory_name);

        // Emit type assumption function.
        self.spec_translator
            .translate_assume_well_formed(&struct_env);
    }

    /// Translates all functions in the module.
    fn translate_functions(&self) {
        emitln!(
            self.writer,
            "\n\n// ** functions of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        for func_env in self.module_env.get_functions() {
            if func_env.is_native() {
                continue;
            }
            for variant in self.targets.get_target_variants(&func_env) {
                self.translate_function(variant, &self.targets.get_target(&func_env, variant));
            }
        }
    }
}

impl<'env> ModuleTranslator<'env> {
    /// Translates the given function.
    fn translate_function(&self, variant: FunctionVariant, func_target: &FunctionTarget<'_>) {
        self.generate_function_sig(variant, &func_target);
        self.generate_function_body(variant, &func_target);
        emitln!(self.writer);
    }

    /// Return a string for a boogie procedure header. Use inline attribute and name
    /// suffix as indicated by `entry_point`.
    fn generate_function_sig(&self, variant: FunctionVariant, func_target: &FunctionTarget<'_>) {
        let (args, rets) = self.generate_function_args_and_returns(func_target);

        let (suffix, attribs) = match variant {
            FunctionVariant::Baseline => ("", "{:inline 1} ".to_string()),
            FunctionVariant::Verification => {
                let timeout = func_target
                    .func_env
                    .get_num_pragma(TIMEOUT_PRAGMA, || self.options.vc_timeout);
                let attribs = if func_target.func_env.is_num_pragma_set(SEED_PRAGMA) {
                    let seed = func_target
                        .func_env
                        .get_num_pragma(SEED_PRAGMA, || self.options.random_seed);
                    format!("{{:timeLimit {}}} {{:random_seed {}}} ", timeout, seed)
                } else {
                    format!("{{:timeLimit {}}} ", timeout)
                };
                ("$verify", attribs)
            }
        };
        self.writer.set_location(&func_target.get_loc());
        emitln!(
            self.writer,
            "procedure {}{}{}({}) returns ({})",
            attribs,
            boogie_function_name(func_target.func_env),
            suffix,
            args,
            rets,
        )
    }

    /// Generate boogie representation of function args and return args.
    fn generate_function_args_and_returns(
        &self,
        func_target: &FunctionTarget<'_>,
    ) -> (String, String) {
        let args = func_target
            .get_type_parameters()
            .iter()
            .map(|TypeParameter(s, _)| {
                format!("{}: $TypeValue", s.display(func_target.symbol_pool()))
            })
            .chain((0..func_target.get_parameter_count()).map(|i| {
                let s = func_target.get_local_name(i);
                let ty = func_target.get_local_type(i);
                let orig_ty = func_target.func_env.get_local_type(i);
                // We must escape names which where originally representing mutable references,
                // since in Boogie we can't write to them. So we generate name `_n` and then
                // in the function body `var n: ...; n := _n`.
                let prefix = if orig_ty.is_mutable_reference() {
                    "$_"
                } else {
                    ""
                };
                format!(
                    "{}{}: {}",
                    prefix,
                    s.display(func_target.symbol_pool()),
                    boogie_local_type(ty)
                )
            }))
            .chain(func_target.get_modify_targets().keys().map(|ty| {
                format!(
                    "{}: {}",
                    boogie_caller_resource_memory_domain_name(func_target.global_env(), *ty),
                    "[$TypeValueArray, int]bool"
                )
            }))
            .join(", ");
        let rets = func_target
            .get_return_types()
            .iter()
            .enumerate()
            .map(|(i, ref s)| format!("$ret{}: {}", i, boogie_local_type(s)))
            .join(", ");
        (args, rets)
    }

    /// Generates boogie implementation body.
    fn generate_function_body(&self, variant: FunctionVariant, func_target: &FunctionTarget<'_>) {
        // Be sure to set back location to the whole function definition as a default.
        self.writer.set_location(&func_target.get_loc());

        emitln!(self.writer, "{");
        self.writer.indent();

        // Generate local variable declarations. They need to appear first in boogie.
        emitln!(self.writer, "// declare local variables");
        let num_args = func_target.get_parameter_count();
        for i in num_args..func_target.get_local_count() {
            let local_name = func_target.get_local_name(i);
            let local_type = func_target.get_local_type(i);
            emitln!(
                self.writer,
                "var {}: {}; // {}",
                local_name.display(func_target.symbol_pool()),
                boogie_local_type(local_type),
                boogie_type_value(self.module_env.env, local_type)
            );
        }
        // Generate declarations for renamed parameters.
        let renamed_params = self.get_renamed_parameters(func_target);
        for (name, ty) in &renamed_params {
            emitln!(
                self.writer,
                "var {}: {};",
                name.display(func_target.symbol_pool()),
                boogie_local_type(ty)
            );
        }
        // Generate declarations for modifies condition.
        func_target.get_modify_targets().keys().for_each(|ty| {
            emitln!(
                self.writer,
                "var {}: {}",
                boogie_self_resource_memory_domain_name(func_target.global_env(), *ty),
                "[$TypeValueArray, int]bool;"
            );
        });

        // Declare temporaries for debug tracing.
        emitln!(self.writer, "var $trace_abort_temp: int;");

        // Generate memory snapshot variable declarations.
        let code = func_target.get_bytecode();
        let labels = code
            .iter()
            .filter_map(|bc| {
                use Bytecode::*;
                match bc {
                    SaveMem(_, lab, mem) => Some((lab, mem)),
                    SaveSpecVar(..) => panic!("spec var memory snapshots NYI"),
                    _ => None,
                }
            })
            .collect::<BTreeMap<_, _>>();
        for (lab, mem) in labels {
            let name = boogie_resource_memory_name(self.module_env.env, *mem, &Some(*lab));
            emitln!(self.writer, "var {}: $Memory;", name);
        }

        // Initialize renamed parameters.
        for (name, _) in renamed_params {
            let d = name.display(func_target.symbol_pool());
            emitln!(self.writer, "{} := $_{};", d, d);
        }

        // Initial assumptions
        if variant == FunctionVariant::Verification {
            self.translate_verify_entry_assumptions(func_target);
        }

        // Generate bytecode
        emitln!(self.writer, "\n// bytecode translation starts here");
        for (offset, bytecode) in code.iter().enumerate() {
            self.translate_bytecode(func_target, offset as CodeOffset, bytecode);
        }

        self.writer.unindent();
        emitln!(self.writer, "}");
    }

    fn get_renamed_parameters(&self, func_target: &FunctionTarget<'_>) -> Vec<(Symbol, Type)> {
        (0..func_target.get_parameter_count())
            .filter_map(|i| {
                if func_target
                    .func_env
                    .get_local_type(i)
                    .is_mutable_reference()
                {
                    Some((
                        func_target.get_local_name(i),
                        func_target.get_local_type(i).clone(),
                    ))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn translate_verify_entry_assumptions(&self, func_target: &FunctionTarget<'_>) {
        emitln!(self.writer, "\n// verification entrypoint assumptions");

        // Prelude initialization
        emitln!(self.writer, "call $InitVerification();");

        // Assume type wellformedness of parameters.
        self.assume_wellformedness(func_target, 0..func_target.get_parameter_count());

        // Assume reference parameters to be based on the Param(i) Location, ensuring
        // they are disjoint from all other references. This prevents aliasing and is justified as
        // follows:
        // - for mutual references, by their exclusive access in Move.
        // - for immutable references, by that mutation is not possible, and they are equivalent
        //   to some given but arbitrary value.
        for i in 0..func_target.get_parameter_count() {
            let ty = func_target.get_local_type(i);
            if ty.is_reference() {
                let name = func_target
                    .symbol_pool()
                    .string(func_target.get_local_name(i));
                emitln!(
                    self.writer,
                    "assume l#$Mutation({}) == $Param({});",
                    name,
                    i
                );
                emitln!(self.writer, "assume size#Path(p#$Mutation({})) == 0;", name);
            }
        }

        // Assume used memory to be wellformed.
        let used_mem = usage_analysis::get_used_memory(&func_target);
        let env = self.module_env.env;
        for mem in used_mem {
            let struct_env = env.get_module(mem.module_id).into_struct(mem.id);
            emit!(self.writer, "assume ");
            let memory_name = boogie_resource_memory_name(func_target.global_env(), *mem, &None);
            emit!(self.writer, "(forall $inv_addr: int");
            let mut type_args = vec![];
            for i in 0..struct_env.get_type_parameters().len() {
                emit!(self.writer, ", $inv_tv{}: $TypeValue", i);
                type_args.push(format!("$inv_tv{}", i));
            }
            let get_resource = format!(
                "contents#$Memory({})[{}, $inv_addr]",
                memory_name,
                boogie_type_value_array_from_strings(&type_args)
            );
            emitln!(self.writer, " :: {{{}}}", get_resource);
            self.writer.indent();
            emitln!(
                self.writer,
                "{}_$is_well_formed({})",
                boogie_struct_name(&struct_env),
                get_resource,
            );
            self.writer.unindent();
            emitln!(self.writer, ");");
        }
    }

    /// Translates one bytecode instruction.
    fn translate_bytecode(
        &'env self,
        func_target: &FunctionTarget<'_>,
        _offset: u16,
        bytecode: &Bytecode,
    ) {
        use Bytecode::*;
        // Set location of this code in the CodeWriter.
        let loc = func_target.get_bytecode_loc(bytecode.get_attr_id());
        self.writer.set_location(&loc);
        emitln!(self.writer, "// {}", bytecode.display(func_target));

        // Helper function to get an Rc<String> for a local.
        let str_local = |idx: usize| {
            func_target
                .symbol_pool()
                .string(func_target.get_local_name(idx))
        };

        // Translate the bytecode instruction.
        match bytecode {
            SpecBlock(..) => panic!("deprecated"),
            SaveMem(_, label, mem) => {
                let snapshot =
                    boogie_resource_memory_name(self.module_env.env, *mem, &Some(*label));
                let current = boogie_resource_memory_name(self.module_env.env, *mem, &None);
                emitln!(self.writer, "{} := {};", snapshot, current);
            }
            SaveSpecVar(_, _label, _var) => {
                panic!("spec var snapshot NYI")
            }
            Prop(_, kind, exp) => match kind {
                PropKind::Assert => {
                    emit!(self.writer, "assert b#$Boolean(");
                    self.spec_translator.translate_exp(exp);
                    emitln!(self.writer, ");");
                }
                PropKind::Assume => {
                    emit!(self.writer, "assume b#$Boolean(");
                    self.spec_translator.translate_exp(exp);
                    emitln!(self.writer, ");");
                }
                PropKind::Modifies => {
                    unimplemented!()
                }
            },
            Label(_, label) => {
                self.writer.unindent();
                emitln!(self.writer, "L{}:", label.as_usize());
                self.writer.indent();
            }
            Jump(_, target) => emitln!(self.writer, "goto L{};", target.as_usize()),
            Branch(_, then_target, else_target, idx) => emitln!(
                self.writer,
                "if (b#$Boolean({})) {{ goto L{}; }} else {{ goto L{}; }}",
                str_local(*idx),
                then_target.as_usize(),
                else_target.as_usize(),
            ),
            OnAbort(_, target, code) => {
                emitln!(self.writer, "if ($abort_flag) {");
                self.writer.indent();
                let code_str = str_local(*code);
                emitln!(self.writer, "{} := $Integer($abort_code);", code_str);
                self.track_abort(&loc, &code_str);
                emitln!(self.writer, "goto L{};", target.as_usize());
                self.writer.unindent();
                emitln!(self.writer, "}");
            }
            Assign(_, dest, src, _) => {
                if func_target.get_local_type(*dest).is_reference() {
                    emitln!(
                        self.writer,
                        "call {} := $CopyOrMoveRef({});",
                        str_local(*dest),
                        str_local(*src)
                    );
                } else {
                    emitln!(
                        self.writer,
                        "call {} := $CopyOrMoveValue({});",
                        str_local(*dest),
                        str_local(*src)
                    );
                }
            }
            Ret(_, rets) => {
                for (i, r) in rets.iter().enumerate() {
                    emitln!(self.writer, "$ret{} := {};", i, str_local(*r));
                }
                emitln!(self.writer, "return;");
            }
            Load(_, idx, c) => {
                let value = match c {
                    Constant::Bool(true) => "$Boolean(true)".to_string(),
                    Constant::Bool(false) => "$Boolean(false)".to_string(),
                    Constant::U8(num) => format!("$Integer({})", num),
                    Constant::U64(num) => format!("$Integer({})", num),
                    Constant::U128(num) => format!("$Integer({})", num),
                    Constant::Address(val) => format!("$Address({})", val),
                    Constant::ByteArray(val) => boogie_byte_blob(self.options, val),
                };
                emitln!(self.writer, "{} := {};", str_local(*idx), value);
            }
            Call(_, dests, oper, srcs) => {
                use Operation::*;
                match oper {
                    FreezeRef => unreachable!(),
                    UnpackRef | UnpackRefDeep | PackRef | PackRefDeep => {
                        // No effect
                    }
                    WriteBack(dest) => {
                        use BorrowNode::*;
                        let src = srcs[0];
                        match dest {
                            GlobalRoot(struct_decl) => {
                                let memory = struct_decl.module_id.qualified(struct_decl.id);
                                let memory_name = boogie_resource_memory_name(
                                    func_target.global_env(),
                                    memory,
                                    &None,
                                );
                                emitln!(
                                    self.writer,
                                    "call {} := $WritebackToGlobal({}, {});",
                                    memory_name,
                                    memory_name,
                                    str_local(src),
                                );
                            }
                            LocalRoot(idx) => {
                                emitln!(
                                    self.writer,
                                    "call {} := $WritebackToValue({}, {}, {});",
                                    str_local(*idx),
                                    str_local(src),
                                    idx,
                                    str_local(*idx)
                                );
                            }
                            Reference(idx) => {
                                emitln!(
                                    self.writer,
                                    "call {} := $WritebackToReference({}, {});",
                                    str_local(*idx),
                                    str_local(src),
                                    str_local(*idx)
                                );
                            }
                        }
                    }
                    Splice(map) => {
                        let src = srcs[0];
                        assert!(!map.is_empty());
                        emitln!(
                            self.writer,
                            "call {} := $Splice{}({}, {});",
                            str_local(src),
                            map.len(),
                            map.iter()
                                .map(|(pos, idx)| format!("{}, {}", pos, str_local(*idx)))
                                .join(", "),
                            str_local(src)
                        );
                    }
                    BorrowLoc => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $BorrowLoc({}, {});",
                            str_local(dest),
                            src,
                            str_local(src)
                        );
                    }
                    ReadRef => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $ReadRef({});",
                            str_local(dest),
                            str_local(src)
                        );
                        emit!(
                            self.writer,
                            &boogie_well_formed_check(
                                self.module_env.env,
                                str_local(dest).as_str(),
                                &func_target.get_local_type(dest),
                            )
                        );
                    }
                    WriteRef => {
                        let reference = srcs[0];
                        let value = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $WriteRef({}, {});",
                            str_local(reference),
                            str_local(reference),
                            str_local(value),
                        );
                    }
                    Function(mid, fid, type_actuals) => {
                        let callee_env = self.module_env.env.get_module(*mid).into_function(*fid);

                        let args_str = std::iter::once(boogie_type_values(
                            func_target.func_env.module_env.env,
                            type_actuals,
                        ))
                        .chain(
                            srcs.iter()
                                .map(|arg_idx| format!("{}", str_local(*arg_idx))),
                        )
                        .filter(|s| !s.is_empty())
                        .join(", ");

                        let dest_str = dests
                            .iter()
                            .map(|dest_idx| str_local(*dest_idx).to_string())
                            .join(", ");

                        if dest_str.is_empty() {
                            emitln!(
                                self.writer,
                                "call {}({});",
                                boogie_function_name(&callee_env),
                                args_str
                            );
                        } else {
                            emitln!(
                                self.writer,
                                "call {} := {}({});",
                                dest_str,
                                boogie_function_name(&callee_env),
                                args_str
                            );
                        }
                    }
                    Pack(mid, sid, _type_actuals) => {
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let mut ctor_expr = "$MapConstValue($DefaultValue())".to_owned();
                        for (i, field_env) in struct_env.get_fields().enumerate() {
                            ctor_expr = format!(
                                "{}[{} := {}]",
                                ctor_expr,
                                boogie_field_name(&field_env),
                                str_local(srcs[i])
                            );
                        }
                        emitln!(
                            self.writer,
                            "{} := $Vector($ValueArray({}, {}));",
                            str_local(dests[0]),
                            ctor_expr,
                            struct_env.get_field_count()
                        );
                    }
                    Unpack(mid, sid, _type_actuals) => {
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        for (i, field_env) in struct_env.get_fields().enumerate() {
                            emitln!(
                                self.writer,
                                "{} := $SelectField({}, {});",
                                str_local(dests[i]),
                                str_local(srcs[0]),
                                boogie_field_name(&field_env)
                            );
                            let type_check = boogie_well_formed_check(
                                self.module_env.env,
                                &format!("{}", str_local(dests[i])),
                                &field_env.get_type(),
                            );
                            emit!(self.writer, &type_check);
                        }
                    }
                    BorrowField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowField({}, {});",
                            str_local(dest),
                            str_local(src),
                            boogie_field_name(field_env)
                        );
                    }
                    GetField(mid, sid, _, field_offset) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let struct_env = func_target
                            .func_env
                            .module_env
                            .env
                            .get_module(*mid)
                            .into_struct(*sid);
                        let field_env = &struct_env.get_field_by_offset(*field_offset);
                        let is_ref = func_target.get_local_type(src).is_reference();
                        emitln!(
                            self.writer,
                            "call {} := {}({}, {});",
                            str_local(dest),
                            if is_ref {
                                "$GetFieldFromReference"
                            } else {
                                "$GetFieldFromValue"
                            },
                            str_local(src),
                            boogie_field_name(field_env)
                        );
                    }
                    Exists(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let memory = boogie_resource_memory_name(
                            self.module_env.env,
                            mid.qualified(*sid),
                            &None,
                        );
                        emitln!(
                            self.writer,
                            "{} := $ResourceExists({}, {}, {});",
                            str_local(dest),
                            memory,
                            type_args,
                            str_local(addr),
                        );
                    }
                    BorrowGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let addr_name = str_local(addr);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $BorrowGlobal({}, {}, {});",
                            str_local(dest),
                            memory_name,
                            addr_name,
                            type_args,
                        );
                    }
                    GetGlobal(mid, sid, type_actuals) => {
                        let addr = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $GetGlobal({}, {}, {});",
                            str_local(dest),
                            memory_name,
                            str_local(addr),
                            type_args,
                        );
                    }
                    MoveTo(mid, sid, type_actuals) => {
                        let value = srcs[0];
                        let signer = srcs[1];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let signer_name = str_local(signer);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {} := $MoveTo({}, {}, {}, {});",
                            memory_name,
                            memory_name,
                            type_args,
                            str_local(value),
                            signer_name,
                        );
                    }
                    MoveFrom(mid, sid, type_actuals) => {
                        let src = srcs[0];
                        let dest = dests[0];
                        let type_args = boogie_type_value_array(self.module_env.env, type_actuals);
                        let src_name = str_local(src);
                        let memory = mid.qualified(*sid);
                        let memory_name =
                            boogie_resource_memory_name(self.module_env.env, memory, &None);
                        emitln!(
                            self.writer,
                            "call {}, {} := $MoveFrom({}, {}, {});",
                            memory_name,
                            str_local(dest),
                            memory_name,
                            src_name,
                            type_args,
                        );
                    }
                    CastU8 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU8({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    CastU64 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU64({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    CastU128 => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $CastU128({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    Not => {
                        let src = srcs[0];
                        let dest = dests[0];
                        emitln!(
                            self.writer,
                            "call {} := $Not({});",
                            str_local(dest),
                            str_local(src)
                        );
                    }
                    Add => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let unchecked = if func_target
                            .is_pragma_true(ADDITION_OVERFLOW_UNCHECKED_PRAGMA, || false)
                        {
                            "_unchecked"
                        } else {
                            ""
                        };
                        let add_type = match func_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8".to_string(),
                            Type::Primitive(PrimitiveType::U64) => format!("U64{}", unchecked),
                            Type::Primitive(PrimitiveType::U128) => format!("U128{}", unchecked),
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call {} := $Add{}({}, {});",
                            str_local(dest),
                            add_type,
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Sub => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Sub({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Mul => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        let mul_type = match func_target.get_local_type(dest) {
                            Type::Primitive(PrimitiveType::U8) => "U8",
                            Type::Primitive(PrimitiveType::U64) => "U64",
                            Type::Primitive(PrimitiveType::U128) => "U128",
                            _ => unreachable!(),
                        };
                        emitln!(
                            self.writer,
                            "call {} := $Mul{}({}, {});",
                            str_local(dest),
                            mul_type,
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Div => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Div({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Mod => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Mod({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Shl => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Shl({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Shr => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Shr({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Lt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Lt({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Gt => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Gt({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Le => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Le({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Ge => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Ge({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Or => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $Or({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    And => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "call {} := $And({}, {});",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Eq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "{} := $Boolean($IsEqual({}, {}));",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    Neq => {
                        let dest = dests[0];
                        let op1 = srcs[0];
                        let op2 = srcs[1];
                        emitln!(
                            self.writer,
                            "{} := $Boolean(!$IsEqual({}, {}));",
                            str_local(dest),
                            str_local(op1),
                            str_local(op2)
                        );
                    }
                    BitOr | BitAnd | Xor => {
                        emitln!(
                            self.writer,
                            "// bit operation not supported: {:?}\nassert false;",
                            bytecode
                        );
                    }
                    Destroy => {}
                    TraceLocal(idx) => {
                        self.track_local(func_target, &loc, *idx, &str_local(srcs[0]));
                    }
                    TraceReturn(i) => {
                        self.track_local(
                            func_target,
                            &loc,
                            func_target.get_local_count() + i,
                            &str_local(srcs[0]),
                        );
                    }
                    TraceAbort => self.track_abort(&loc, &str_local(srcs[0])),
                }
            }
            Abort(_, src) => {
                emitln!(
                    self.writer,
                    "$abort_code := i#$Integer({});",
                    str_local(*src)
                );
                emitln!(self.writer, "$abort_flag := true;");
                for (i, ty) in func_target.get_return_types().iter().enumerate() {
                    let ret_str = format!("$ret{}", i);
                    if ty.is_reference() {
                        emitln!(self.writer, "{} := $DefaultMutation;", &ret_str);
                    } else {
                        emitln!(self.writer, "{} := $DefaultValue();", &ret_str);
                    }
                }
                emitln!(self.writer, "return;")
            }
            Nop(..) => {}
        }
        emitln!(self.writer);
    }

    fn track_abort(&self, loc: &Loc, code_var: &str) {
        let file_idx = self
            .module_env
            .env
            .file_id_to_idx(loc.file_id())
            .to_string();
        let pos = loc.span().start().to_string();
        let track = boogie_debug_track_abort_via_attrib(&file_idx, &pos, code_var);
        if !track.is_empty() {
            emitln!(self.writer, &track);
        }
    }

    /// Generates an update of the model debug variable at given location.
    fn track_local(&self, func_target: &FunctionTarget<'_>, loc: &Loc, idx: usize, value: &str) {
        // Check whether this is a temporary, which we do not want to track. Indices >=
        // local_count are return values which we do track.
        if idx >= func_target.get_user_local_count() && idx < func_target.get_local_count() {
            return;
        }
        let ty = if idx < func_target.get_local_count() {
            func_target.get_local_type(idx)
        } else {
            func_target.get_return_type(idx - func_target.get_local_count())
        };
        let value = if ty.is_reference() {
            format!("$Dereference({})", value)
        } else {
            value.to_string()
        };
        let file_idx = func_target
            .func_env
            .module_env
            .env
            .file_id_to_idx(loc.file_id())
            .to_string();
        let pos = loc.span().start().to_string();
        let local_idx = idx.to_string();
        let track = boogie_debug_track_local_via_attrib(&file_idx, &pos, &local_idx, &value);
        if !track.is_empty() {
            emitln!(self.writer, &track);
        }
    }

    fn assume_wellformedness(
        &self,
        func_target: &FunctionTarget<'_>,
        locals: impl Iterator<Item = TempIndex>,
    ) {
        for local in locals {
            let ty = func_target.get_local_type(local);
            let name = func_target
                .symbol_pool()
                .string(func_target.get_local_name(local));
            let check = boogie_well_formed_check(self.module_env.env, &name, ty);
            if !check.is_empty() {
                emitln!(self.writer, &check);
            }
        }
    }
}
