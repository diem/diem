// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use itertools::Itertools;
use num::{BigInt, Num};

use ir_to_bytecode_syntax::ast::{BinOp, CopyableVal, Field, Loc, QualifiedStructIdent, Type};
use ir_to_bytecode_syntax::spec_language_ast::{Condition, SpecExp, StorageLocation};
use libra_types::account_address::AccountAddress;

use crate::boogie_helpers::{boogie_field_name, boogie_type_value};
use crate::code_writer::CodeWriter;
use crate::env::{FunctionEnv, GlobalType, Parameter};
use codespan_reporting::{Diagnostic, Label};

pub struct SpecTranslator<'env> {
    func_env: &'env FunctionEnv<'env>,
    writer: &'env CodeWriter,
    current_loc: Loc, // Location used for type checking errors
}

/// Represents a boogie expression as a string and its type. The type is used to access
/// necessary context information for generating boogie expressions, as well as for type
/// checking.
struct BoogieExpr(String, GlobalType);

impl BoogieExpr {
    fn result(self) -> String {
        self.0
    }
}

/// A dummy type to represent an error. We use a type parameter for this, with an index
/// which no one will/can ever realistically use.
const ERROR_TYPE: GlobalType = GlobalType::TypeParameter(std::u16::MAX);

/// A dummy type to represent an unknown type. We use a type parameter for this, with an index
/// which no one will/can ever realistically use. We currently need this to support a hack
/// for specification helper functions (SpecExp::Call) whose type we do not know, and which
/// are defined in Boogie.
const UNKNOWN_TYPE: GlobalType = GlobalType::TypeParameter(std::u16::MAX - 1);

impl<'env> SpecTranslator<'env> {
    pub fn new(
        func_env: &'env FunctionEnv<'env>,
        writer: &'env CodeWriter,
    ) -> SpecTranslator<'env> {
        SpecTranslator {
            func_env,
            writer,
            current_loc: Loc::default(),
        }
    }

    /// Reports a type checking error.
    pub fn error<T>(&self, msg: &str, pass_through: T) -> T {
        let diag = Diagnostic::new_error(msg).with_label(Label::new_primary(self.current_loc));
        self.func_env.module_env.add_diag(diag);
        pass_through
    }

    /// Helper to update the current location. This is both used for informing the `error` function
    /// above for type checking errors and the CodeWriter for being able to map boogie errors
    /// back to the source.
    fn update_location(&mut self, loc: Loc) {
        self.current_loc = loc;
        self.writer
            .set_location(self.func_env.module_env.get_module_idx(), loc);
    }

    /// Generates a boogie containing pre/post conditions.
    pub fn translate(&mut self) {
        // Generate pre-conditions
        // For this transaction to be executed, it MUST have had
        // a valid signature for the sender's account. Therefore,
        // the senders account resource (which contains the pubkey)
        // must have existed! So we can assume txn_sender account
        // exists in pre-condition.
        for cond in self.func_env.get_specification() {
            if let Condition::Requires(expr) = &cond.value {
                self.update_location(cond.span);
                emitln!(
                    self.writer,
                    "requires b#Boolean({});",
                    self.translate_expr(expr).result()
                );
            }
        }
        emitln!(self.writer, "requires ExistsTxnSenderAccount(__m, __txn);");

        // Generate succeeds_if and aborts_if setup (for post conditions)

        // When all succeeds_if conditions hold, function must not abort.
        let mut succeeds_if_string = self
            .func_env
            .get_specification()
            .iter()
            .filter_map(|c| match &c.value {
                Condition::SucceedsIf(expr) => {
                    self.update_location(c.span);
                    Some(format!("b#Boolean({})", self.translate_expr(expr).result()))
                }
                _ => None,
            })
            .join(" && ");

        // abort_if P means function MUST abort if P holds.
        // multiple abort_if conditions are "or"ed.
        let aborts_if_string = self
            .func_env
            .get_specification()
            .iter()
            .filter_map(|c| match &c.value {
                Condition::AbortsIf(expr) => {
                    self.update_location(c.span);
                    Some(format!("b#Boolean({})", self.translate_expr(expr).result()))
                }
                _ => None,
            })
            .join(" || ");

        // negation of abort_if condition is an implicit succeeds_if condition.
        // So, if no explicit succeeds_if specifications exist, the function must
        // succeed whenever the aborts_if condition does not hold.
        // If there are explicit succeeds_if conditions, there may be cases where
        // no aborts_if holds and not all succeeds_if conditions hold.  In that case,
        // the function may or may not abort. If it does not abort, it must meet all
        // explicit ensures conditions.
        // NOTE: It's not yet clear that succeeds_if is useful, or if this is the most
        // useful interpretation.
        if !aborts_if_string.is_empty() {
            if !succeeds_if_string.is_empty() {
                succeeds_if_string = format!("!({}) && ({})", aborts_if_string, succeeds_if_string);
            } else {
                succeeds_if_string = format!("!({})", aborts_if_string);
            }
        }

        // Generate explicit ensures conditions
        for cond in self.func_env.get_specification() {
            if let Condition::Ensures(expr) = &cond.value {
                // FIXME: Do we really need to check whether succeeds_if & aborts_if are
                // empty, below?
                self.update_location(cond.span);
                emitln!(
                    self.writer,
                    "ensures {}b#Boolean({});",
                    if !succeeds_if_string.is_empty() || !aborts_if_string.is_empty() {
                        // Guard the condition to be only effective if not aborted.
                        "!__abort_flag ==> "
                    } else {
                        ""
                    },
                    self.translate_expr(expr).result()
                );
            }
        }

        // emit the Boogie ensures condition for succeeds_if
        if !succeeds_if_string.is_empty() {
            // If succeeds_if condition is met, function must NOT abort.
            emitln!(
                self.writer,
                "ensures old({}) ==> !__abort_flag;",
                succeeds_if_string
            );
        }

        // emit Boogie ensures condition for aborts_if
        if !aborts_if_string.is_empty() {
            emitln!(
                self.writer,
                "ensures old({}) ==> __abort_flag;\n",
                aborts_if_string
            );
        }
    }

    /// Translates a specification expression into boogie.
    ///
    /// This returns a boogie expression of type Value or Reference.
    fn translate_expr(&mut self, expr: &SpecExp) -> BoogieExpr {
        match expr {
            SpecExp::Constant(val) => self.translate_constant(val),
            SpecExp::StorageLocation(loc) => self.translate_location_as_value(loc),
            SpecExp::GlobalExists {
                type_,
                type_actuals,
                address,
            } => {
                let BoogieExpr(a, at) = self.translate_location_as_value(address);
                let _ = self.require_type(at, &GlobalType::Address);
                BoogieExpr(
                    format!(
                        "ExistsResource(__m, {}, a#Address({}))",
                        self.translate_resource_type(type_, type_actuals).0,
                        a
                    ),
                    GlobalType::Bool,
                )
            }
            SpecExp::Dereference(loc) => self.translate_dref(loc),
            SpecExp::Reference(loc) => self.translate_location_as_reference(loc),
            SpecExp::Not(expr) => {
                let BoogieExpr(s, t) = self.translate_expr(expr);
                BoogieExpr(
                    format!("Boolean(!(b#Boolean({})))", s),
                    self.require_type(t, &GlobalType::Bool),
                )
            }
            SpecExp::Binop(left, op, right) => {
                let left = self.translate_expr(left);
                let right = self.translate_expr(right);
                self.translate_binop(op, left, right)
            }
            SpecExp::Old(expr) => {
                let BoogieExpr(s, t) = self.translate_expr(expr);
                BoogieExpr(format!("old({})", s), t)
            }
            SpecExp::Call(name, exprs) => BoogieExpr(
                format!(
                    "{}({})",
                    name,
                    exprs.iter().map(|e| self.translate_expr(e).0).join(", ")
                ),
                // We currently do not have a way to know the return type (and expected argument
                // types) of a helper function.
                UNKNOWN_TYPE,
            ),
        }
    }

    /// Translate a dereference.
    fn translate_dref(&mut self, loc: &StorageLocation) -> BoogieExpr {
        let BoogieExpr(s, t) = self.translate_location_as_reference(loc);
        if let GlobalType::Reference(sig) | GlobalType::MutableReference(sig) = t {
            BoogieExpr(format!("Dereference(__m, {})", s), *sig)
        } else {
            self.error(
                &format!(
                    "expected reference type, found `{}`",
                    boogie_type_value(self.func_env.module_env.env, &t)
                ),
                BoogieExpr("<deref>".to_string(), ERROR_TYPE),
            )
        }
    }

    /// Translates a binary operator.
    fn translate_binop(&mut self, op: &BinOp, left: BoogieExpr, right: BoogieExpr) -> BoogieExpr {
        let operand_type = left.1.clone();
        match op {
            // u64
            BinOp::Add => {
                self.translate_op_helper("+", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Sub => {
                self.translate_op_helper("-", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Mul => {
                self.translate_op_helper("*", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Mod => {
                self.translate_op_helper("mod", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Div => {
                self.translate_op_helper("div", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::BitAnd => {
                self.translate_op_helper("&", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::BitOr => {
                self.translate_op_helper("|", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Xor => {
                self.translate_op_helper("^", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Shl => unimplemented!(),
            BinOp::Shr => unimplemented!(),

            // bool
            BinOp::And => {
                self.translate_op_helper("&&", &GlobalType::Bool, GlobalType::Bool, left, right)
            }
            BinOp::Or => {
                self.translate_op_helper("||", &GlobalType::Bool, GlobalType::Bool, left, right)
            }

            // generic equality
            BinOp::Eq => BoogieExpr(
                if left.1.is_reference() || right.1.is_reference() {
                    format!("Boolean(({}) == ({}))", left.0, right.0)
                } else {
                    format!("Boolean(IsEqual({}, {}))", left.0, right.0)
                },
                GlobalType::Bool,
            ),
            BinOp::Neq => BoogieExpr(
                if left.1.is_reference() || right.1.is_reference() {
                    format!("Boolean(({}) != ({}))", left.0, right.0)
                } else {
                    format!("Boolean(!IsEqual({}, {}))", left.0, right.0)
                },
                GlobalType::Bool,
            ),

            // Ordering
            // TODO: is this defined also for non-integer types?
            BinOp::Lt => {
                self.translate_op_helper("<", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Gt => {
                self.translate_op_helper(">", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Le => {
                self.translate_op_helper("<=", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Ge => {
                self.translate_op_helper(">=", &operand_type, GlobalType::Bool, left, right)
            }
        }
    }

    /// Helper for translating a binary op with type check of arguments and unwrapping/wrapping
    /// Boogie values.
    fn translate_op_helper(
        &mut self,
        op: &str,
        expected_operand_type: &GlobalType,
        result_type: GlobalType,
        BoogieExpr(l, lt): BoogieExpr,
        BoogieExpr(r, rt): BoogieExpr,
    ) -> BoogieExpr {
        let _ = self.require_type(rt, expected_operand_type);
        let _ = self.require_type(lt, expected_operand_type);
        let expr = match expected_operand_type {
            GlobalType::U8 | GlobalType::U64 | GlobalType::U128 => {
                format!("i#Integer({}) {} i#Integer({})", l, op, r)
            }
            GlobalType::Bool => format!("b#Boolean({}) {} b#Boolean({})", l, op, r),
            _ => panic!("unexpected type"),
        };
        match result_type {
            GlobalType::U8 | GlobalType::U64 | GlobalType::U128 => {
                BoogieExpr(format!("Integer({})", expr), result_type)
            }
            GlobalType::Bool => BoogieExpr(format!("Boolean({})", expr), result_type),
            _ => panic!("unexpected type"),
        }
    }

    /// Translates a constant.
    fn translate_constant(&mut self, val: &CopyableVal) -> BoogieExpr {
        match val {
            CopyableVal::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                GlobalType::Address,
            ),
            CopyableVal::U8(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U8),
            CopyableVal::U64(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U64),
            CopyableVal::U128(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U128),
            CopyableVal::Bool(val) => BoogieExpr(format!("Boolean({})", val), GlobalType::Bool),
            // TODO: byte arrays
            CopyableVal::ByteArray(_arr) => BoogieExpr(
                self.error("ByteArray not implemented", "<bytearray>".to_string()),
                GlobalType::ByteArray,
            ),
        }
    }

    /// Translates a location into a boogie expression of type Value.
    ///
    /// The StorageLocation AST can represent different types in the Boogie model,
    /// therefore we need to interpret in context. See also `translate_location_as_reference`.
    fn translate_location_as_value(&mut self, loc: &StorageLocation) -> BoogieExpr {
        match loc {
            StorageLocation::Formal(name) => self.translate_param(name),

            StorageLocation::Ret(index) => self.translate_return(*index as usize),
            StorageLocation::TxnSenderAddress => BoogieExpr(
                "Address(TxnSenderAddress(__txn))".to_string(),
                GlobalType::Address,
            ),
            StorageLocation::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                GlobalType::Address,
            ),
            StorageLocation::AccessPath { base, fields } => {
                let BoogieExpr(mut res, mut t) = self.translate_location_as_value(base);
                // If the type of the location is a reference, dref it now.
                if let GlobalType::Reference(vt) | GlobalType::MutableReference(vt) = t {
                    res = format!("Dereference(__m, {})", res);
                    t = *vt;
                }
                for f in fields {
                    let (s, tf) = self.translate_field(&t, f);
                    res = format!("SelectField({}, {})", res, s);
                    t = tf;
                }
                BoogieExpr(res, t)
            }
            _ => {
                // Interpret as reference which we immediately dref.
                self.translate_dref(loc)
            }
        }
    }

    /// Translate a location as Reference.
    fn translate_location_as_reference(&mut self, loc: &StorageLocation) -> BoogieExpr {
        match loc {
            StorageLocation::Formal(name) => {
                let BoogieExpr(s, t) = self.translate_param(name);
                if let GlobalType::Reference(d) | GlobalType::MutableReference(d) = t {
                    BoogieExpr(s, GlobalType::Reference(Box::new(*d)))
                } else {
                    self.error(
                        &format!("`{}` expected to be a reference", name),
                        BoogieExpr("<ref>".to_string(), ERROR_TYPE),
                    )
                }
            }
            StorageLocation::GlobalResource {
                type_,
                type_actuals,
                address,
            } => {
                let (s, t) = self.translate_resource_type(type_, type_actuals);
                let BoogieExpr(a, at) = self.translate_location_as_value(address);
                let _ = self.require_type(at, &GlobalType::Address);
                BoogieExpr(
                    format!("GetResourceReference({}, a#Address({}))", s, a),
                    GlobalType::Reference(Box::new(t)),
                )
            }
            StorageLocation::AccessPath { base, fields } => {
                let BoogieExpr(mut res, mut t) = self.translate_location_as_reference(base);
                for f in fields {
                    let (s, tf) = self.translate_field(&t, f);
                    res = format!("SelectFieldFromRef({}, {})", res, s);
                    t = tf;
                }
                BoogieExpr(res, t)
            }
            _ => self.error(
                &format!("cannot translate as reference: {:?}", loc),
                BoogieExpr("<ref>".to_string(), ERROR_TYPE),
            ),
        }
    }

    /// Checks for an expected type.
    fn require_type(&mut self, t: GlobalType, expected: &GlobalType) -> GlobalType {
        if t != ERROR_TYPE && t != UNKNOWN_TYPE && t != *expected {
            println!("t: {:?} expected {:?}", t, expected);
            self.error(
                &format!(
                    "incompatible types: expected `{}`, found `{}`",
                    boogie_type_value(self.func_env.module_env.env, expected),
                    boogie_type_value(self.func_env.module_env.env, &t)
                ),
                expected.clone(),
            )
        } else {
            t
        }
    }

    /// Translate an account address literal.
    fn translate_account_address(&self, addr: &AccountAddress) -> String {
        format!("{}", BigInt::from_str_radix(&addr.to_string(), 16).unwrap())
    }

    /// Translates a resource name with type actuals
    fn translate_resource_type(
        &mut self,
        id: &QualifiedStructIdent,
        _type_actuals: &[Type],
    ) -> (String, GlobalType) {
        // TODO: incorporate type actuals. For that, we need a translator from Type AST into
        //   GlobalSignatureToken, which requires name resolution.
        // TODO: do we need to incorporate address in the resolution?
        let struct_name = id.name.as_inner();
        let mut module_name = id.module.as_inner().to_string();
        if module_name == "Self" {
            module_name = self.func_env.module_env.get_id().name().to_string();
        }
        for module_env in self.func_env.module_env.env.get_modules() {
            if module_env.get_id().name().as_str() != module_name {
                continue;
            }
            if let Some(struct_env) = module_env.find_struct(struct_name) {
                let resource_type = GlobalType::Struct(
                    module_env.get_module_idx(),
                    struct_env.get_def_idx(),
                    vec![],
                );
                return (
                    boogie_type_value(self.func_env.module_env.env, &resource_type),
                    resource_type,
                );
            }
        }
        self.error(
            &format!("Cannot resolve type `{}`", id),
            (format!("<struct {}>", id), GlobalType::U64),
        )
    }

    /// Translate a function parameter.
    fn translate_param(&mut self, name: &str) -> BoogieExpr {
        // Look up parameter.
        if let Some(Parameter(name, sig)) = self
            .func_env
            .get_parameters()
            .iter()
            .find(|Parameter(n, _)| n.as_str() == name)
        {
            BoogieExpr(name.as_str().to_string(), sig.clone())
        } else {
            self.error(
                &format!("parameter `{}` undefined", name),
                BoogieExpr(name.to_string(), ERROR_TYPE),
            )
        }
    }

    /// Translate a function return value.
    fn translate_return(&mut self, index: usize) -> BoogieExpr {
        let return_types = self.func_env.get_return_types();
        if return_types.is_empty() {
            self.error(
                "function does not return a value",
                BoogieExpr("<ret>".to_string(), GlobalType::U64),
            )
        } else if index >= return_types.len() {
            self.error(
                &format!(
                    "RET index {} out of bounds; function declaration has {} return values",
                    index,
                    return_types.len()
                ),
                BoogieExpr("<ret>".to_string(), GlobalType::U64),
            )
        } else {
            BoogieExpr(format!("ret{}", index), return_types[index].clone())
        }
    }

    /// Translates a field name, where `sig` is the type from which the field is selected.
    /// Returns boogie field name and type.
    fn translate_field(&mut self, mut sig: &GlobalType, field: &Field) -> (String, GlobalType) {
        // If this is a reference, use the underlying type. This function works with both
        // references and non-references.
        let is_ref = if let GlobalType::Reference(s) = sig {
            sig = &*s;
            true
        } else {
            false
        };
        if *sig == ERROR_TYPE {
            return ("<field>".to_string(), ERROR_TYPE);
        }
        if *sig == UNKNOWN_TYPE {
            return self.error(
                "unknown result type of helper function; cannot select field",
                ("<field>".to_string(), ERROR_TYPE),
            );
        }
        if let GlobalType::Struct(module_index, struct_index, _actuals) = sig {
            let module_env = self.func_env.module_env.env.get_module(*module_index);
            let struct_env = module_env.get_struct(&struct_index);
            if let Some(field_env) = struct_env.find_field(field.name()) {
                (
                    boogie_field_name(&field_env),
                    if is_ref {
                        GlobalType::Reference(Box::new(field_env.get_type()))
                    } else {
                        field_env.get_type()
                    },
                )
            } else {
                self.error(
                    &format!(
                        "struct `{}` does not have field `{}`",
                        struct_env.get_name(),
                        field
                    ),
                    ("<field>".to_string(), ERROR_TYPE),
                )
            }
        } else {
            self.error(
                &format!(
                    "expected Struct but found `{}`",
                    boogie_type_value(self.func_env.module_env.env, sig)
                ),
                ("<field>".to_string(), ERROR_TYPE),
            )
        }
    }
}
