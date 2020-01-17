// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use bytecode_verifier::VerifiedModule;
use ir_to_bytecode_syntax::ast::{BinOp, CopyableVal, Field, QualifiedStructIdent, Type};
use ir_to_bytecode_syntax::spec_language_ast::{Condition, SpecExp, StorageLocation};
use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use num::{BigInt, Num};

use crate::cli::abort_with_error;
use crate::translator::{format_type_value, FunctionInfo};
use vm::access::ModuleAccess;
use vm::file_format::{ModuleHandleIndex, SignatureToken, StructHandle, StructHandleIndex};
use vm::views::{
    FieldDefinitionView, FunctionDefinitionView, FunctionHandleView, FunctionSignatureView,
    ModuleHandleView, StructDefinitionView, StructHandleView,
};

pub struct SpecTranslator<'a> {
    all_modules: &'a [VerifiedModule],
    module: &'a VerifiedModule,
    function_info: &'a FunctionInfo,
}

/// Represents a boogie expression as a string and its type. The type is used to access
/// necessary context information for generating boogie expressions, as well as for type
/// checking.
struct BoogieExpr(String, SignatureToken);

impl BoogieExpr {
    fn result(self) -> String {
        self.0
    }
}

/// A dummy type to represent an error. We use a type parameter for this, with an index
/// which no one will/can ever realistically use.
const ERROR_TYPE: SignatureToken = SignatureToken::TypeParameter(std::u16::MAX);

/// A dummy type to represent an unknown type. We use a type parameter for this, with an index
/// which no one will/can ever realistically use. We currently need this to support a hack
/// for specification helper functions (SpecExp::Call) whose type we do not know, and which
/// are defined in Boogie.
const UNKNOWN_TYPE: SignatureToken = SignatureToken::TypeParameter(std::u16::MAX - 1);

impl<'a> SpecTranslator<'a> {
    pub fn new(
        all_modules: &'a [VerifiedModule],
        module: &'a VerifiedModule,
        function_info: &'a FunctionInfo,
    ) -> SpecTranslator<'a> {
        SpecTranslator {
            all_modules,
            module,
            function_info,
        }
    }

    /// Reports an error. At the moment, we simply abort.
    // TODO: extend this to collect errors and add source info. Code needs to expect this
    //   returns with the given `pass_through` value.
    pub fn error<T>(&mut self, msg: &str, _pass_through: T) -> T {
        abort_with_error(&format!(
            "[in function `{}`] {}",
            self.get_current_function_name(),
            msg
        ))
    }

    /// Generates a boogie string containing pre/post conditions.
    pub fn translate(&mut self) -> String {
        let mut res = String::new();

        // Generate pre-conditions
        // For this transaction to be executed, it MUST have had
        // a valid signature for the sender's account. Therefore,
        // the senders account resource (which contains the pubkey)
        // must have existed! So we can assume txn_sender account
        // exists in pre-condition.
        for cond in &self.function_info.specification {
            if let Condition::Requires(expr) = cond {
                res.push_str(&format!(
                    "requires b#Boolean({});\n",
                    self.translate_expr(expr).result()
                ))
            }
        }
        res.push_str("requires ExistsTxnSenderAccount(m, txn);\n");

        // Generate succeeds_if and aborts_if setup (for post conditions)

        // When all succeeds_if conditions hold, function must not abort.
        let mut succeeds_if_string = self
            .function_info
            .specification
            .iter()
            .filter_map(|c| match c {
                Condition::SucceedsIf(expr) => {
                    Some(format!("b#Boolean({})", self.translate_expr(expr).result()))
                }
                _ => None,
            })
            .join(" && ");

        // abort_if P means function MUST abort if P holds.
        // multiple abort_if conditions are "or"ed.
        let aborts_if_string = self
            .function_info
            .specification
            .iter()
            .filter_map(|c| match c {
                Condition::AbortsIf(expr) => {
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
        for cond in &self.function_info.specification {
            if let Condition::Ensures(expr) = cond {
                // FIXME: Do we really need to check whether succeeds_if & aborts_if are
                // empty, below?
                res.push_str(&format!(
                    "ensures {}b#Boolean({});\n",
                    if !succeeds_if_string.is_empty() || !aborts_if_string.is_empty() {
                        // Guard the condition to be only effective if not aborted.
                        "!abort_flag ==> "
                    } else {
                        ""
                    },
                    self.translate_expr(expr).result()
                ));
            }
        }

        // emit the Boogie ensures condition for succeeds_if
        if !succeeds_if_string.is_empty() {
            // If succeeds_if condition is met, function must NOT abort.
            res.push_str(&format!(
                "ensures old({}) ==> !abort_flag;\n",
                succeeds_if_string
            ));
        }

        // emit Boogie ensures condition for aborts_if
        if !aborts_if_string.is_empty() {
            res.push_str(&format!(
                "ensures old({}) ==> abort_flag;\n",
                aborts_if_string
            ));
        }

        res
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
                let _ = self.require_type(at, &SignatureToken::Address);
                BoogieExpr(
                    format!(
                        "ExistsResource(m, {}, a#Address({}))",
                        self.translate_resource_type(type_, type_actuals).0,
                        a
                    ),
                    SignatureToken::Bool,
                )
            }
            SpecExp::Dereference(loc) => self.translate_dref(loc),
            SpecExp::Reference(loc) => self.translate_dref(loc),
            SpecExp::Not(expr) => {
                let BoogieExpr(s, t) = self.translate_expr(expr);
                BoogieExpr(
                    format!("Boolean(!(b#Boolean({})))", s),
                    self.require_type(t, &SignatureToken::Bool),
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
        if let SignatureToken::Reference(sig) | SignatureToken::MutableReference(sig) = t {
            BoogieExpr(format!("Dereference(m, {})", s), *sig)
        } else {
            self.error(
                &format!(
                    "expected reference type, found `{}`",
                    format_type_value(self.module, &t)
                ),
                BoogieExpr("<deref>".to_string(), ERROR_TYPE),
            )
        }
    }

    /// Translates a binary operator.
    fn translate_binop(&mut self, op: &BinOp, left: BoogieExpr, right: BoogieExpr) -> BoogieExpr {
        match op {
            // u64
            BinOp::Add => self.translate_op_helper(
                "+",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Sub => self.translate_op_helper(
                "-",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Mul => self.translate_op_helper(
                "*",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Mod => self.translate_op_helper(
                "mod",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Div => self.translate_op_helper(
                "div",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::BitAnd => self.translate_op_helper(
                "&",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::BitOr => self.translate_op_helper(
                "|",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Xor => self.translate_op_helper(
                "^",
                &SignatureToken::U64,
                SignatureToken::U64,
                left,
                right,
            ),
            BinOp::Shl => unimplemented!(),
            BinOp::Shr => unimplemented!(),

            // bool
            BinOp::And => self.translate_op_helper(
                "&&",
                &SignatureToken::Bool,
                SignatureToken::Bool,
                left,
                right,
            ),
            BinOp::Or => self.translate_op_helper(
                "||",
                &SignatureToken::Bool,
                SignatureToken::Bool,
                left,
                right,
            ),

            // generic equality
            BinOp::Eq => BoogieExpr(
                format!("Boolean(({}) == ({}))", left.0, right.0),
                SignatureToken::Bool,
            ),
            BinOp::Neq => BoogieExpr(
                format!("Boolean(({}) != ({}))", left.0, right.0),
                SignatureToken::Bool,
            ),

            // Ordering
            // TODO: is this defined also for non-integer types?
            BinOp::Lt => self.translate_op_helper(
                "<",
                &SignatureToken::U64,
                SignatureToken::Bool,
                left,
                right,
            ),
            BinOp::Gt => self.translate_op_helper(
                ">",
                &SignatureToken::U64,
                SignatureToken::Bool,
                left,
                right,
            ),
            BinOp::Le => self.translate_op_helper(
                "<=",
                &SignatureToken::U64,
                SignatureToken::Bool,
                left,
                right,
            ),
            BinOp::Ge => self.translate_op_helper(
                ">=",
                &SignatureToken::U64,
                SignatureToken::Bool,
                left,
                right,
            ),
        }
    }

    /// Helper for translating a binary op with type check of arguments and unwrapping/wrapping
    /// Boogie values.
    fn translate_op_helper(
        &mut self,
        op: &str,
        expected_operand_type: &SignatureToken,
        result_type: SignatureToken,
        BoogieExpr(l, lt): BoogieExpr,
        BoogieExpr(r, rt): BoogieExpr,
    ) -> BoogieExpr {
        let _ = self.require_type(rt, expected_operand_type);
        let _ = self.require_type(lt, expected_operand_type);
        let expr = match expected_operand_type {
            SignatureToken::U64 => format!("i#Integer({}) {} i#Integer({})", l, op, r),
            SignatureToken::Bool => format!("b#Boolean({}) {} b#Boolean({})", l, op, r),
            _ => panic!("unexpected type"),
        };
        match result_type {
            SignatureToken::U64 => BoogieExpr(format!("Integer({})", expr), result_type),
            SignatureToken::Bool => BoogieExpr(format!("Boolean({})", expr), result_type),
            _ => panic!("unexpected type"),
        }
    }

    /// Translates a constant.
    fn translate_constant(&mut self, val: &CopyableVal) -> BoogieExpr {
        match val {
            CopyableVal::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                SignatureToken::Address,
            ),
            CopyableVal::U8(_) => unimplemented!(),
            CopyableVal::U64(val) => BoogieExpr(format!("Integer({})", val), SignatureToken::U64),
            CopyableVal::U128(_) => unimplemented!(),
            CopyableVal::Bool(val) => BoogieExpr(format!("Boolean({})", val), SignatureToken::Bool),
            // TODO: byte arrays
            CopyableVal::ByteArray(_arr) => BoogieExpr(
                self.error("ByteArray not implemented", "<bytearray>".to_string()),
                SignatureToken::ByteArray,
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
                "Address(TxnSenderAddress(txn))".to_string(),
                SignatureToken::Address,
            ),
            StorageLocation::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                SignatureToken::Address,
            ),
            StorageLocation::AccessPath { base, fields } => {
                let BoogieExpr(mut res, mut t) = self.translate_location_as_value(base);
                // If the type of the location is a reference, dref it now.
                if let SignatureToken::Reference(vt) | SignatureToken::MutableReference(vt) = t {
                    res = format!("Dereference(m, {})", res);
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
                if let SignatureToken::Reference(_) | SignatureToken::MutableReference(_) = t {
                    BoogieExpr(s, SignatureToken::Reference(Box::new(t)))
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
                let _ = self.require_type(at, &SignatureToken::Address);
                BoogieExpr(
                    format!("GetResourceReference({}, a#Address({}))", s, a),
                    SignatureToken::Reference(Box::new(t)),
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
    fn require_type(&mut self, t: SignatureToken, expected: &SignatureToken) -> SignatureToken {
        if t != ERROR_TYPE && t != UNKNOWN_TYPE && t != *expected {
            self.error(
                &format!(
                    "incompatible types: expected `{}`, found `{}`",
                    format_type_value(self.module, expected),
                    format_type_value(self.module, &t)
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
    ) -> (String, SignatureToken) {
        // TODO: incorporate type actuals. For that, we need a translator from Type AST into
        //   SignatureToken, which requires name resolution.
        let struct_name = id.name.as_inner();
        let mut module_name = id.module.as_inner().to_string();
        if module_name == "Self" {
            let module_view = ModuleHandleView::new(
                self.module,
                self.module.module_handle_at(ModuleHandleIndex(0 as u16)),
            );
            module_name = module_view.module_id().name().to_string();
        }
        for (index, handle) in self.module.struct_handles().iter().enumerate() {
            let view = StructHandleView::new(self.module, handle);
            if view.name() == struct_name && view.module_id().name().as_str() == module_name {
                let resource_type =
                    SignatureToken::Struct(StructHandleIndex::new(index as u16), vec![]);
                return (
                    format_type_value(self.module, &resource_type),
                    resource_type,
                );
            }
        }
        self.error(
            &format!("cannot resolve struct `{}`", id),
            (format!("<struct {}>", id), SignatureToken::U64),
        )
    }

    /// Translate a function parameter.
    fn translate_param(&mut self, name: &str) -> BoogieExpr {
        // Look up parameter index.
        let arg_index = {
            self.function_info
                .arg_names
                .iter()
                .position(|s| s == name)
                .unwrap_or_else(|| {
                    self.error(
                        &format!("cannot determine position of parameter `{}`", name),
                        0,
                    )
                })
        };
        // Get the parameter type.
        let arg_sig = {
            if let Some(ty) = self.get_param_types().get(arg_index) {
                ty.clone()
            } else {
                self.error(
                    &format!("cannot determine type of parameter {}", name),
                    ERROR_TYPE,
                )
            }
        };
        BoogieExpr(format!("arg{}", arg_index), arg_sig)
    }

    /// Translate a function return value.
    fn translate_return(&mut self, index: usize) -> BoogieExpr {
        let sig = self.get_function_signature();
        let return_count = sig.return_count();
        if return_count < 1 {
            self.error(
                "function does not return a value",
                BoogieExpr("<ret>".to_string(), SignatureToken::U64),
            )
        } else if index >= return_count {
            self.error(
                &format!(
                    "RET index {} out of bounds; function declaration has {} return values",
                    index, return_count
                ),
                BoogieExpr("<ret>".to_string(), SignatureToken::U64),
            )
        } else {
            BoogieExpr(
                format!("ret{}", index),
                sig.return_tokens()
                    .nth(index as usize)
                    .expect("non-empty return")
                    .signature_token()
                    .clone(),
            )
        }
    }

    /// Translates a field name, where `sig` is the type from which the field is selected.
    /// Returns boogie field name and type.
    fn translate_field(
        &mut self,
        mut sig: &SignatureToken,
        field: &Field,
    ) -> (String, SignatureToken) {
        // If this is a reference, use the underlying type. This function works with both
        // references and non-references.
        let is_ref = if let SignatureToken::Reference(s) = sig {
            sig = &*s;
            true
        } else {
            false
        };
        if let SignatureToken::Struct(handle_idx, type_args) = sig {
            let struct_handle = self.module.struct_handle_at(*handle_idx);
            // We cannot lookup type information for fields of imported modules via the handle view.
            // Instead we need to retrieve the struct definition, which might be in another module.
            // TODO: validate that I've not missed some feature in file_format.rs. It might be
            //   considered an issue in the byte code representation that we can't do this; it
            //   might be better that bytecode fully specifies all metadata about dependencies.
            if let Some(struct_def) = self.get_struct_definition(struct_handle) {
                let struct_name = struct_def.name().to_string(); // because of borrowing
                if let Some(field_def) = self.get_field_definition(&struct_def, field) {
                    let field_type = field_def.signature_token().substitute(type_args);
                    (
                        format!(
                            "{}_{}",
                            self.get_struct_name_for_boogie(struct_handle),
                            field_def.name()
                        ),
                        if is_ref {
                            SignatureToken::Reference(Box::new(field_type))
                        } else {
                            field_type
                        },
                    )
                } else {
                    self.error(
                        &format!("field `{}` undefined in struct `{}`", field, struct_name),
                        ("<field>".to_string(), ERROR_TYPE),
                    )
                }
            } else {
                self.error(
                    &format!("cannot determine struct and module for `{}`", field),
                    ("<field>".to_string(), ERROR_TYPE),
                )
            }
        } else if *sig == UNKNOWN_TYPE {
            self.error(
                "unknown result type of helper function; cannot select field",
                ("field".to_string(), ERROR_TYPE),
            )
        } else {
            self.error(
                &format!(
                    "expected Struct but found `{}`",
                    format_type_value(self.module, sig)
                ),
                ("<field>".to_string(), ERROR_TYPE),
            )
        }
    }

    /// Gets the current functions signature.
    fn get_function_signature(&self) -> FunctionSignatureView<VerifiedModule> {
        let function_def = self.module.function_def_at(self.function_info.index);
        let function_view = FunctionHandleView::new(
            self.module,
            self.module.function_handle_at(function_def.function),
        );
        function_view.signature()
    }

    /// Returns parameter types of the current function.
    fn get_param_types(&self) -> Vec<SignatureToken> {
        self.get_function_signature()
            .arg_tokens()
            .map(|t| t.signature_token().clone())
            .collect_vec()
    }

    /// Get a struct definition from a StructHandle.
    fn get_struct_definition(
        &self,
        handle: &StructHandle,
    ) -> Option<StructDefinitionView<VerifiedModule>> {
        let struct_view = StructHandleView::new(self.module, handle);
        let module_view = ModuleHandleView::new(self.module, struct_view.module_handle());
        let module_id = module_view.module_id();

        // The definition of this struct might be in a different module as for which are translating
        // right now. We search for it by module id (pair of address and module name).
        for module in self.all_modules {
            let this_module_handle = module.module_handle_at(ModuleHandleIndex(0));
            if module.module_id_for_handle(this_module_handle) != module_id {
                continue;
            }
            for struct_def in module.struct_defs() {
                let struct_def_view = StructDefinitionView::new(module, struct_def);
                if struct_def_view.name() == struct_view.name() {
                    return Some(struct_def_view);
                }
            }
        }
        None
    }

    /// Get a field definition from a StructDefinition and field name.
    fn get_field_definition(
        &self,
        struct_def: &StructDefinitionView<'a, VerifiedModule>,
        field: &Field,
    ) -> Option<FieldDefinitionView<'a, VerifiedModule>> {
        if let Some(mut fields) = struct_def.fields() {
            fields.find(|d| d.name() == field.name())
        } else {
            None
        }
    }

    /// Get struct name as expected by boogie.
    fn get_struct_name_for_boogie(&self, handle: &StructHandle) -> String {
        let struct_view = StructHandleView::new(self.module, handle);
        let module_view = ModuleHandleView::new(self.module, struct_view.module_handle());
        format!("{}_{}", module_view.module_id().name(), struct_view.name())
    }

    /// Get function name.
    fn get_current_function_name(&self) -> String {
        let function_view = FunctionDefinitionView::new(
            self.module,
            self.module.function_def_at(self.function_info.index),
        );
        function_view.name().to_string()
    }
}
