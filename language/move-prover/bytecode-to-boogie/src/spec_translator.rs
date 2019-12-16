// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use bytecode_verifier::VerifiedModule;
use ir_to_bytecode_syntax::ast::{BinOp, CopyableVal, Field, StructName};
use ir_to_bytecode_syntax::spec_language_ast::{Condition, SpecExp, StorageLocation};
use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use num::{BigInt, Num};

use crate::cli::abort_with_error;
use crate::translator::FunctionInfo;
use vm::access::ModuleAccess;
use vm::views::StructHandleView;

pub struct SpecTranslator<'a> {
    #[allow(dead_code)]
    module: &'a VerifiedModule,
    function_info: &'a FunctionInfo,
}

impl<'a> SpecTranslator<'a> {
    pub fn new(module: &'a VerifiedModule, function_info: &'a FunctionInfo) -> SpecTranslator<'a> {
        SpecTranslator {
            module,
            function_info,
        }
    }

    /// Reports an error. At the moment, we simply abort.
    // TODO: extend this to collect errors and add source info. Code needs to expect this
    //   returns with the given `pass_through` value.
    pub fn error<T>(&mut self, msg: &str, _pass_through: T) -> T {
        abort_with_error(msg)
    }

    /// Generates a boogie string containing pre/post conditions.
    pub fn translate(&mut self) -> String {
        let mut res = String::new();

        // Generate pre-conditions
        for cond in &self.function_info.specification {
            if let Condition::Requires(expr) = cond {
                res.push_str(&format!(
                    "requires b#Boolean({});\n",
                    self.translate_expr(expr)
                ))
            }
        }
        res.push_str("requires ExistsTxnSenderAccount();\n");

        // Generate succeeds_if and aborts_if setup (for post conditions)
        let succeeds_if_string = self
            .function_info
            .specification
            .iter()
            .filter_map(|c| match c {
                Condition::SucceedsIf(expr) => {
                    Some(format!("b#Boolean({})", self.translate_expr(expr)))
                }
                _ => None,
            })
            .join(" && ");
        if !succeeds_if_string.is_empty() {
            // Ensure that violation of condition at entry leads to abort.
            res.push_str(&format!(
                "ensures !old({}) ==> abort_flag;\n",
                succeeds_if_string
            ));
        }
        let aborts_if_string = self
            .function_info
            .specification
            .iter()
            .filter_map(|c| match c {
                Condition::AbortsIf(expr) => {
                    Some(format!("b#Boolean({})", self.translate_expr(expr)))
                }
                _ => None,
            })
            .join(" || ");

        // Generate post conditions.
        for cond in &self.function_info.specification {
            if let Condition::Ensures(expr) = cond {
                res.push_str(&format!(
                    "ensures {}b#Boolean({});\n",
                    if !succeeds_if_string.is_empty() || !aborts_if_string.is_empty() {
                        // Guard the condition to be only effective if not aborted.
                        "!abort_flag ==> "
                    } else {
                        ""
                    },
                    self.translate_expr(expr)
                ));
            }
        }

        // Emit aborts_if/succeeds_if.
        let mut no_abort_cond = vec![];
        if !succeeds_if_string.is_empty() {
            no_abort_cond.push(format!("({})", succeeds_if_string));
        }
        if !aborts_if_string.is_empty() {
            no_abort_cond.push(format!("!({})", aborts_if_string));
        }
        if !no_abort_cond.is_empty() {
            res.push_str(&format!(
                "ensures {} ==> !abort_flag;\n",
                no_abort_cond.iter().join(" && ")
            ));
        }

        res
    }

    /// Translates a specification expression into boogie.
    ///
    /// This returns a boogie expression of type Value or Reference.
    fn translate_expr(&mut self, expr: &SpecExp) -> String {
        match expr {
            SpecExp::Constant(val) => self.translate_constant(val),
            SpecExp::StorageLocation(loc) => self.translate_location_as_value(loc),
            SpecExp::GlobalExists { type_, address } => format!(
                "ExistsResource(gs, {}, {})",
                self.translate_location_as_address(address),
                self.translate_resource_name(type_)
            ),
            SpecExp::Dereference(loc) => format!(
                "Dereference(m, {})",
                self.translate_location_as_reference(loc)
            ),
            SpecExp::Reference(loc) => self.translate_location_as_reference(loc),
            SpecExp::Not(expr) => format!("Boolean(!(b#Boolean({}))", self.translate_expr(expr)),
            SpecExp::Binop(left, op, right) => {
                let left = self.translate_expr(left);
                let right = self.translate_expr(right);
                self.translate_binop(op, &left, &right)
            }
        }
    }

    /// Translates a binary operator.
    fn translate_binop(&mut self, op: &BinOp, left: &str, right: &str) -> String {
        match op {
            // u64
            BinOp::Add => format!("Integer(i#Integer({}) + i#Integer({}))", left, right),
            BinOp::Sub => format!("Integer(i#Integer({}) - i#Integer({}))", left, right),
            BinOp::Mul => format!("Integer(i#Integer({}) * i#Integer({}))", left, right),
            BinOp::Mod => format!("Integer(i#Integer({}) mod i#Integer({}))", left, right),
            BinOp::Div => format!("Integer(i#Integer({}) div i#Integer({}))", left, right),
            BinOp::BitOr => format!("Integer(i#Integer({}) | i#Integer({}))", left, right),
            BinOp::BitAnd => format!("Integer(i#Integer({}) & i#Integer({}))", left, right),
            BinOp::Xor => format!("Integer(i#Integer({}) ^ i#Integer({}))", left, right),

            // bool
            BinOp::And => format!("Boolean(b#Boolean({}) && b#Boolean({}))", left, right),
            BinOp::Or => format!("Boolean(b#Boolean({}) || b#Boolean({}))", left, right),

            // generic equality
            BinOp::Eq => format!("Boolean(({}) == ({}))", left, right),
            BinOp::Neq => format!("Boolean(({}) != ({}))", left, right),

            // Ordering
            // TODO: is this defined also for non-integer types?
            BinOp::Lt => format!("Boolean(i#Integer({}) < i#Integer({}))", left, right),
            BinOp::Gt => format!("Boolean(i#Integer({}) > i#Integer({}))", left, right),
            BinOp::Le => format!("Boolean(i#Integer({}) <= i#Integer({}))", left, right),
            BinOp::Ge => format!("Boolean(i#Integer({}) >= i#Integer({}))", left, right),
        }
    }

    /// Translates a constant.
    fn translate_constant(&mut self, val: &CopyableVal) -> String {
        match val {
            CopyableVal::Address(addr) => {
                format!("Address({})", self.translate_account_address(addr))
            }
            CopyableVal::U64(val) => format!("Integer({})", val),
            CopyableVal::Bool(val) => format!("Boolean({})", val),
            // TODO: byte arrays
            CopyableVal::ByteArray(_arr) => {
                self.error("ByteArray not implemented", "<bytearray>".to_string())
            }
        }
    }

    /// Translates a location into a boogie expression of type Value.
    ///
    /// The StorageLocation AST can represent different types in the Boogie model,
    /// therefore we need to interpret in context. See also `translate_location_as_reference`
    /// and `translate_location_as_address`.
    fn translate_location_as_value(&mut self, loc: &StorageLocation) -> String {
        match loc {
            StorageLocation::Formal(name) => {
                // TODO: we like to preserve original argument names in the boogie code,
                //   which can be achieved with a little more effort.
                return format!("arg{}", self.get_param_index(name));
            }
            StorageLocation::Old(loc) => format!("old({})", self.translate_location_as_value(loc)),
            StorageLocation::Ret => {
                // TODO: the parser currently only handles `return` for single value. Need to
                //   generalize for multiple ret0, ret1, ...
                "ret0".to_string()
            }
            other => {
                if self.is_address_location(other) {
                    // Interpret as address value.
                    format!("Address({})", self.translate_location_as_address(other))
                } else {
                    // Interpret as reference which we dref.
                    // TODO: we may want to optimize some cases here to avoid constructing
                    //   an intermediate Reference with subsequent dereference.
                    format!(
                        "Dereference(m, {})",
                        self.translate_location_as_reference(other)
                    )
                }
            }
        }
    }

    /// Translate a location as Reference.
    fn translate_location_as_reference(&mut self, loc: &StorageLocation) -> String {
        match loc {
            StorageLocation::Formal(name) => {
                format!("GetLocalReference(m_size, {})", self.get_param_index(name))
            }
            StorageLocation::GlobalResource { type_, address } => format!(
                "GetResourceReference(gs, {}, {})",
                self.translate_location_as_address(address),
                self.translate_resource_name(type_)
            ),
            StorageLocation::AccessPath { base, fields } => {
                let mut res = self.translate_location_as_reference(base);
                for f in fields {
                    res.push_str(&format!(
                        "SelectField({}, {})",
                        res,
                        self.translate_field(f)
                    ));
                }
                res
            }
            _ => self.error(
                &format!("cannot translate as reference: {:?}", loc),
                "<ref>".to_string(),
            ),
        }
    }

    /// Translate a location as Address.
    fn translate_location_as_address(&mut self, addr: &StorageLocation) -> String {
        match addr {
            StorageLocation::TxnSenderAddress => "TxnSenderAddress()".into(),
            StorageLocation::Address(addr) => self.translate_account_address(addr),
            _ => self.error(
                &format!("cannot translate as address: {:?}", addr),
                "<addr>".to_string(),
            ),
        }
    }

    /// Checks whether this is a translatable address.
    fn is_address_location(&self, addr: &StorageLocation) -> bool {
        match addr {
            StorageLocation::TxnSenderAddress => true,
            StorageLocation::Address(_) => true,
            _ => false,
        }
    }

    /// Translate an account address literal.
    fn translate_account_address(&self, addr: &AccountAddress) -> String {
        format!("{}", BigInt::from_str_radix(&addr.to_string(), 16).unwrap())
    }

    /// Translates a resource name.
    fn translate_resource_name(&mut self, name: &StructName) -> String {
        // TODO: right now StructName is simple (not qualified) so this isn't working if
        //   different modules have same resource names. We need to extend the parser to
        //   allow qualified/dotted names here.
        for handle in self.module.struct_handles() {
            let view = StructHandleView::new(self.module, handle);
            if view.name() == name.as_inner() {
                let module_name = self.module.identifier_at(view.module_handle().name);
                return format!("{}_{}", module_name, view.name());
            }
        }
        self.error(
            &format!("cannot resolve struct `{}`", name),
            format!("<struct {}>", name),
        )
    }

    /// Translate a parameter name.
    fn get_param_index(&mut self, name: &str) -> usize {
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
    }

    /// Translates a field name.
    fn translate_field(&mut self, _field: &Field) -> String {
        // TODO: We need to generate here `<module_name>_<type_name>_<field_name>`, but there
        //   is not enough context to do this. Holding back with a solution until we are clear
        //   about spec language context/type check approach.
        self.error("currently cannot translate fields", "<field>".to_string())
    }
}
