// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::{BigUint, ToPrimitive, Zero};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use diem_crypto::HashValue;

use bytecode::{
    function_target::FunctionTarget,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::{
        AbortAction, AssignKind, BorrowEdge, BorrowNode, Bytecode, Constant, HavocKind, Label,
        Operation,
    },
};
use move_binary_format::{errors::PartialVMResult, file_format::CodeOffset};
use move_core_types::{
    account_address::AccountAddress,
    vm_status::{sub_status, StatusCode},
};
use move_model::{
    ast::TempIndex,
    model::{FunId, FunctionEnv, ModuleId, StructId},
    ty as MT,
};

use crate::{
    concrete::{
        local_state::{AbortInfo, LocalState, TerminationStatus},
        ty::{
            convert_model_base_type, convert_model_local_type, convert_model_struct_type, BaseType,
            Type,
        },
        value::{GlobalState, LocalSlot, Pointer, TypedValue},
    },
    shared::variant::choose_variant,
};

//**************************************************************************************************
// Constants
//**************************************************************************************************

const DIEM_CORE_ADDR: AccountAddress =
    AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

// TODO(mengxu): these constants are defined in values_impl.rs which are currently not exposed.
const INDEX_OUT_OF_BOUNDS: u64 = sub_status::NFE_VECTOR_ERROR_BASE + 1;
const POP_EMPTY_VEC: u64 = sub_status::NFE_VECTOR_ERROR_BASE + 2;
const DESTROY_NON_EMPTY_VEC: u64 = sub_status::NFE_VECTOR_ERROR_BASE + 3;

//**************************************************************************************************
// Execution context
//**************************************************************************************************

struct FunctionContext<'env> {
    holder: &'env FunctionTargetsHolder,
    target: FunctionTarget<'env>,
    ty_args: Vec<BaseType>,
    label_offsets: BTreeMap<Label, CodeOffset>,
}

impl<'env> FunctionContext<'env> {
    pub fn new(
        holder: &'env FunctionTargetsHolder,
        target: FunctionTarget<'env>,
        ty_args: Vec<BaseType>,
    ) -> Self {
        let label_offsets = Bytecode::label_offsets(target.get_bytecode());
        Self {
            holder,
            target,
            ty_args,
            label_offsets,
        }
    }

    //
    // execution
    //

    /// Execute a user function with value arguments.
    pub fn exec_user_function(
        &self,
        typed_args: Vec<TypedValue>,
        global_state: &mut GlobalState,
    ) -> PartialVMResult<LocalState> {
        let mut local_state = self.prepare_local_state(typed_args);
        let instructions = self.target.get_bytecode();
        while !local_state.is_terminated() {
            let pc = local_state.get_pc() as usize;
            let bytecode = instructions.get(pc).unwrap();
            self.exec_bytecode(bytecode, &mut local_state, global_state)?;
        }
        Ok(local_state)
    }

    /// Execute a native function with the type arguments and value arguments.
    fn exec_native_function(
        &self,
        srcs: &[TempIndex],
        typed_args: Vec<TypedValue>,
        local_state: &mut LocalState,
        global_state: &mut GlobalState,
    ) -> Result<Vec<TypedValue>, AbortInfo> {
        let mut dummy_state = self.prepare_local_state(typed_args);
        if cfg!(debug_assertions) {
            assert_eq!(dummy_state.num_slots(), srcs.len());
        }

        // locate
        let env = self.target.global_env();
        let addr = *self.target.module_env().self_address();
        let module_name = env
            .symbol_pool()
            .string(self.target.module_env().get_name().name());
        let function_name = env.symbol_pool().string(self.target.get_name());

        // dispatch
        match (addr, module_name.as_str(), function_name.as_str()) {
            (DIEM_CORE_ADDR, "Vector", "empty") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 0);
                }
                let res = self.native_vector_empty();
                Ok(vec![res])
            }
            (DIEM_CORE_ADDR, "Vector", "length") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_vector_length(dummy_state.del_value(0));
                Ok(vec![res])
            }
            (DIEM_CORE_ADDR, "Vector", "borrow") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 2);
                }
                self.native_vector_borrow(
                    false,
                    *srcs.get(0).unwrap(),
                    dummy_state.del_value(0),
                    dummy_state.del_value(1),
                )
                .map(|res| vec![res])
            }
            (DIEM_CORE_ADDR, "Vector", "borrow_mut") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 2);
                }
                self.native_vector_borrow(
                    true,
                    *srcs.get(0).unwrap(),
                    dummy_state.del_value(0),
                    dummy_state.del_value(1),
                )
                .map(|res| vec![res])
            }
            (DIEM_CORE_ADDR, "Vector", "push_back") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 2);
                }
                let res = self
                    .native_vector_push_back(dummy_state.del_value(0), dummy_state.del_value(1));
                local_state.put_value_override(*srcs.get(0).unwrap(), res);
                Ok(vec![])
            }
            (DIEM_CORE_ADDR, "Vector", "pop_back") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_vector_pop_back(dummy_state.del_value(0));
                match res {
                    Ok((new_vec, elem_val)) => {
                        local_state.put_value_override(*srcs.get(0).unwrap(), new_vec);
                        Ok(vec![elem_val])
                    }
                    Err(e) => Err(e),
                }
            }
            (DIEM_CORE_ADDR, "Vector", "destroy_empty") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_vector_destroy_empty(dummy_state.del_value(0));
                match res {
                    Ok(_) => {
                        local_state.del_value(*srcs.get(0).unwrap());
                        Ok(vec![])
                    }
                    Err(e) => Err(e),
                }
            }
            (DIEM_CORE_ADDR, "Vector", "swap") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 3);
                }
                let res = self.native_vector_swap(
                    dummy_state.del_value(0),
                    dummy_state.del_value(1),
                    dummy_state.del_value(2),
                );
                match res {
                    Ok(new_vec) => {
                        local_state.put_value_override(*srcs.get(0).unwrap(), new_vec);
                        Ok(vec![])
                    }
                    Err(e) => Err(e),
                }
            }
            (DIEM_CORE_ADDR, "Signer", "borrow_address") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_signer_borrow_address(dummy_state.del_value(0));
                Ok(vec![res])
            }
            (DIEM_CORE_ADDR, "Hash", "sha2_256") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_hash_sha2_256(dummy_state.del_value(0));
                Ok(vec![res])
            }
            (DIEM_CORE_ADDR, "Hash", "sha3_256") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                let res = self.native_hash_sha3_256(dummy_state.del_value(0));
                Ok(vec![res])
            }
            (DIEM_CORE_ADDR, "BCS", "to_bytes") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                }
                self.native_bcs_to_bytes(dummy_state.del_value(0))
                    .map(|res| vec![res])
            }
            (DIEM_CORE_ADDR, "Event", "write_to_event_store") => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 3);
                }
                self.native_event_write_to_event_store(
                    dummy_state.del_value(0),
                    dummy_state.del_value(1),
                    dummy_state.del_value(2),
                    global_state,
                );
                Ok(vec![])
            }
            _ => unreachable!(),
        }
    }

    fn exec_bytecode(
        &self,
        bytecode: &Bytecode,
        local_state: &mut LocalState,
        global_state: &mut GlobalState,
    ) -> PartialVMResult<()> {
        match bytecode {
            Bytecode::Assign(_, dst, src, kind) => {
                self.handle_assign(*dst, *src, kind, local_state)
            }
            Bytecode::Load(_, dst, constant) => self.handle_load(*dst, constant, local_state),
            Bytecode::Call(_, dsts, op, srcs, on_abort) => {
                self.handle_operation(dsts, op, srcs, on_abort.as_ref(), local_state, global_state)?
            }
            Bytecode::Label(_, label) => {
                if cfg!(debug_assertions) {
                    self.code_offset_by_label(*label);
                }
            }
            Bytecode::Jump(_, label) => {
                local_state.set_pc(self.code_offset_by_label(*label));
            }
            Bytecode::Branch(_, then_label, else_label, cond) => {
                self.handle_conditional_branch(*cond, *then_label, *else_label, local_state)
            }
            Bytecode::Abort(_, index) => self.handle_abort(*index, local_state),
            Bytecode::Ret(_, rets) => self.handle_return(rets, local_state),
            // global memory and expressions (TODO: not supported yet)
            Bytecode::SaveMem(..) | Bytecode::Prop(..) => {}
            // deprecated
            Bytecode::Nop(_) | Bytecode::SaveSpecVar(..) => {}
        }
        local_state.ready_pc_for_next_instruction();
        Ok(())
    }

    //
    // per-bytecode processing
    //

    fn handle_assign(
        &self,
        dst: TempIndex,
        src: TempIndex,
        kind: &AssignKind,
        local_state: &mut LocalState,
    ) {
        let from_val = match kind {
            AssignKind::Move => local_state.del_value(src),
            // TODO (mengxu): what exactly is the semantic of Store here? Why not just use Copy?
            AssignKind::Copy | AssignKind::Store => local_state.get_value(src),
        };
        let into_val = from_val.assign_cast(local_state.get_type(dst).clone());
        local_state.put_value_override(dst, into_val);
    }

    fn handle_load(&self, dst: TempIndex, constant: &Constant, local_state: &mut LocalState) {
        let val = match constant {
            Constant::Bool(v) => TypedValue::mk_bool(*v),
            Constant::U8(v) => TypedValue::mk_u8(*v),
            Constant::U64(v) => TypedValue::mk_u64(*v),
            Constant::U128(v) => TypedValue::mk_u128(*v),
            Constant::Address(v) => TypedValue::mk_address(
                AccountAddress::from_hex_literal(&format!("{:#x}", v)).unwrap(),
            ),
            Constant::ByteArray(v) => {
                let elems = v.iter().map(|e| TypedValue::mk_u8(*e)).collect();
                TypedValue::mk_vector(BaseType::mk_u8(), elems)
            }
        };
        local_state.put_value_override(dst, val);
    }

    fn handle_operation(
        &self,
        dsts: &[TempIndex],
        op: &Operation,
        srcs: &[TempIndex],
        on_abort: Option<&AbortAction>,
        local_state: &mut LocalState,
        global_state: &mut GlobalState,
    ) -> PartialVMResult<()> {
        // check abort handler
        if cfg!(debug_assertions) {
            match on_abort {
                None => (),
                Some(action) => {
                    assert!(op.can_abort());
                    assert!(local_state
                        .get_type(action.1)
                        .is_compatible_for_abort_code());
                }
            }
        }

        // operations that does not need to have the argument in storage
        match op {
            // built-ins
            Operation::Havoc(kind) => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                    let target_ty = local_state.get_type(*srcs.get(0).unwrap());
                    match kind {
                        HavocKind::Value => {
                            assert!(target_ty.is_base());
                        }
                        HavocKind::MutationValue | HavocKind::MutationAll => {
                            assert!(target_ty.is_ref(Some(true)));
                        }
                    }
                }
                return Ok(());
            }
            // debugging
            Operation::TraceLocal(index) => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                    assert_eq!(local_state.get_type(*index), local_state.get_type(srcs[0]));
                }
                return Ok(());
            }
            Operation::TraceReturn(num) => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                    assert!(*num < self.target.get_return_count());
                }
                return Ok(());
            }
            Operation::TraceAbort => {
                if cfg!(debug_assertions) {
                    assert_eq!(srcs.len(), 1);
                    assert!(local_state.get_type(srcs[0]).is_compatible_for_abort_code());
                }
                return Ok(());
            }
            Operation::TraceExp(node_id) => {
                if cfg!(debug_assertions) {
                    let env = self.target.global_env();
                    let node_ty =
                        convert_model_local_type(env, &env.get_node_type(*node_id), &self.ty_args);
                    assert_eq!(srcs.len(), 1);
                    assert_eq!(local_state.get_type(srcs[0]), &node_ty);
                }
                return Ok(());
            }
            // deprecated
            Operation::PackRefDeep | Operation::UnpackRefDeep => {
                unreachable!();
            }
            // all others require args to be collected up front
            _ => (),
        }

        // collect arguments
        let mut typed_args: Vec<_> = srcs.iter().map(|idx| local_state.get_value(*idx)).collect();

        // case on operation type
        let op_result = match op {
            // function call
            Operation::Function(module_id, fun_id, ty_args) => self.handle_call_function(
                *module_id,
                *fun_id,
                ty_args,
                typed_args,
                srcs,
                local_state,
                global_state,
            )?,
            // opaque
            Operation::OpaqueCallBegin(module_id, fun_id, ty_args) => self.handle_call_function(
                *module_id,
                *fun_id,
                ty_args,
                typed_args,
                srcs,
                local_state,
                global_state,
            )?,
            Operation::OpaqueCallEnd(module_id, fun_id, ty_args) => {
                self.handle_opaque_call_end(*module_id, *fun_id, ty_args, typed_args);
                Ok(vec![])
            }
            // struct
            Operation::Pack(module_id, struct_id, ty_args) => {
                let packed = self.handle_pack(*module_id, *struct_id, ty_args, typed_args);
                Ok(vec![packed])
            }
            Operation::Unpack(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let unpacked =
                    self.handle_unpack(*module_id, *struct_id, ty_args, typed_args.remove(0));
                Ok(unpacked)
            }
            Operation::GetField(module_id, struct_id, ty_args, field_num) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let field = self.handle_get_field(
                    *module_id,
                    *struct_id,
                    ty_args,
                    *field_num,
                    typed_args.remove(0),
                );
                Ok(vec![field])
            }
            Operation::BorrowField(module_id, struct_id, ty_args, field_num) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let (is_mut, _) = local_state.get_type(dsts[0]).get_ref_type();
                let field = self.handle_borrow_field(
                    *module_id,
                    *struct_id,
                    ty_args,
                    *field_num,
                    is_mut,
                    srcs[0],
                    typed_args.remove(0),
                );
                Ok(vec![field])
            }
            Operation::MoveTo(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                self.handle_move_to(
                    *module_id,
                    *struct_id,
                    ty_args,
                    typed_args.remove(1),
                    typed_args.remove(0),
                    global_state,
                )
                .map(|_| vec![])
            }
            Operation::MoveFrom(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                self.handle_move_from(
                    *module_id,
                    *struct_id,
                    ty_args,
                    typed_args.remove(0),
                    global_state,
                )
                .map(|object| vec![object])
            }
            Operation::GetGlobal(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                self.handle_get_global(
                    *module_id,
                    *struct_id,
                    ty_args,
                    typed_args.remove(0),
                    global_state,
                )
                .map(|object| vec![object])
            }
            Operation::BorrowGlobal(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let (is_mut, _) = local_state.get_type(dsts[0]).get_ref_type();
                self.handle_borrow_global(
                    *module_id,
                    *struct_id,
                    ty_args,
                    is_mut,
                    typed_args.remove(0),
                    global_state,
                )
                .map(|object| vec![object])
            }
            Operation::Exists(module_id, struct_id, ty_args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let exists = self.handle_exists_global(
                    *module_id,
                    *struct_id,
                    ty_args,
                    typed_args.remove(0),
                    global_state,
                );
                Ok(vec![exists])
            }
            // scope
            Operation::PackRef | Operation::UnpackRef => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                    let arg_ty = typed_args.get(0).unwrap().get_ty();
                    assert!(arg_ty.is_struct() || arg_ty.is_ref_struct(Some(true)));
                }
                Ok(vec![])
            }
            // write-back
            Operation::WriteBack(BorrowNode::ReturnPlaceholder(_), ..) => {
                unreachable!("unexpected intermediate borrow node")
            }
            Operation::IsParent(..) => {
                // TODO: implement write_back check
                Ok(vec![TypedValue::mk_bool(true)])
            }
            Operation::WriteBack(BorrowNode::GlobalRoot(qid), edge) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                match edge {
                    BorrowEdge::Direct => self.handle_write_back_global_struct(
                        qid.module_id,
                        qid.id,
                        &qid.inst,
                        typed_args.remove(0),
                        global_state,
                    ),
                    _ => unreachable!(),
                }
                Ok(vec![])
            }
            Operation::WriteBack(BorrowNode::LocalRoot(idx), edge) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                match edge {
                    BorrowEdge::Direct => {
                        self.handle_write_back_local(*idx, typed_args.remove(0), local_state)
                    }
                    _ => unreachable!(),
                }
                Ok(vec![])
            }
            Operation::WriteBack(BorrowNode::Reference(idx), edge) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                match edge {
                    BorrowEdge::Direct => {
                        self.handle_write_back_ref_whole(*idx, typed_args.remove(0), local_state)
                    }
                    BorrowEdge::Field(qid, field_num) => self.handle_write_back_ref_field(
                        qid.module_id,
                        qid.id,
                        &qid.inst,
                        *idx,
                        *field_num,
                        typed_args.remove(0),
                        local_state,
                    ),
                    BorrowEdge::Index => {
                        self.handle_write_back_ref_element(*idx, typed_args.remove(0), local_state)
                    }
                    BorrowEdge::Hyper(_) => {
                        // TODO: implement
                        unimplemented!("hyper edges")
                    }
                }
                Ok(vec![])
            }
            // references
            Operation::BorrowLoc => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let (is_mut, _) = local_state.get_type(dsts[0]).get_ref_type();
                let object = self.handle_borrow_local(is_mut, typed_args.remove(0), srcs[0]);
                Ok(vec![object])
            }
            Operation::ReadRef => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let object = self.handle_read_ref(typed_args.remove(0));
                Ok(vec![object])
            }
            Operation::WriteRef => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                self.handle_write_ref(typed_args.remove(1), srcs[0], local_state);
                Ok(vec![])
            }
            Operation::FreezeRef => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let object = self.handle_freeze_ref(typed_args.remove(0));
                Ok(vec![object])
            }
            // built-in
            Operation::Destroy => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                self.handle_destroy(srcs[0], local_state);
                Ok(vec![])
            }
            Operation::Stop => {
                // we should never see the Stop operation in interpreter mode
                unreachable!()
            }
            // cast
            Operation::CastU8 | Operation::CastU64 | Operation::CastU128 => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let val = typed_args.remove(0);
                match op {
                    Operation::CastU8 => self.handle_cast_u8(val),
                    Operation::CastU64 => self.handle_cast_u64(val),
                    Operation::CastU128 => self.handle_cast_u128(val),
                    _ => unreachable!(),
                }
                .map(|casted| vec![casted])
            }
            // binary arithmetic
            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div | Operation::Mod => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                self.handle_binary_arithmetic(op, lhs, rhs, local_state.get_type(dsts[0]))
                    .map(|calculated| vec![calculated])
            }
            // binary bitwise
            Operation::BitAnd | Operation::BitOr | Operation::Xor => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                let calculated =
                    self.handle_binary_bitwise(op, lhs, rhs, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // binary bitshift
            Operation::Shl | Operation::Shr => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                let calculated =
                    self.handle_binary_bitshift(op, lhs, rhs, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // binary comparison
            Operation::Lt | Operation::Le | Operation::Ge | Operation::Gt => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                let calculated =
                    self.handle_binary_comparision(op, lhs, rhs, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // binary equality
            Operation::Eq | Operation::Neq => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                let calculated =
                    self.handle_binary_equality(op, lhs, rhs, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // unary boolean
            Operation::Not => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 1);
                }
                let opv = typed_args.remove(0);
                let calculated = self.handle_unary_boolean(op, opv, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // binary boolean
            Operation::And | Operation::Or => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_args.len(), 2);
                }
                let rhs = typed_args.remove(1);
                let lhs = typed_args.remove(0);
                let calculated =
                    self.handle_binary_boolean(op, lhs, rhs, local_state.get_type(dsts[0]));
                Ok(vec![calculated])
            }
            // event (TODO: not supported yet)
            Operation::EmitEvent | Operation::EventStoreDiverge => Ok(vec![]),
            // already handled
            Operation::Havoc(..)
            | Operation::TraceLocal(..)
            | Operation::TraceReturn(..)
            | Operation::TraceAbort
            | Operation::TraceExp(..)
            | Operation::PackRefDeep
            | Operation::UnpackRefDeep => {
                unreachable!();
            }
        };

        // handle result
        match op_result {
            Ok(typed_rets) => {
                if cfg!(debug_assertions) {
                    assert_eq!(typed_rets.len(), dsts.len());
                }
                for (typed_ret, &idx) in typed_rets.into_iter().zip(dsts) {
                    local_state.put_value_override(idx, typed_ret);
                }
            }
            Err(abort_info) => match on_abort {
                None => {
                    return Err(abort_info.into_err());
                }
                Some(action) => {
                    let abort_idx = action.1;
                    let abort_val = if local_state.get_type(abort_idx).is_u64() {
                        TypedValue::mk_u64(abort_info.get_status_code())
                    } else {
                        TypedValue::mk_num(BigUint::from(abort_info.get_status_code()))
                    };
                    local_state.put_value(abort_idx, abort_val);
                    local_state.set_pc(self.code_offset_by_label(action.0));
                    local_state.transit_to_post_abort(abort_info);
                }
            },
        }
        Ok(())
    }

    fn handle_call_function(
        &self,
        module_id: ModuleId,
        fun_id: FunId,
        ty_args: &[MT::Type],
        typed_args: Vec<TypedValue>,
        srcs: &[TempIndex],
        local_state: &mut LocalState,
        global_state: &mut GlobalState,
    ) -> PartialVMResult<Result<Vec<TypedValue>, AbortInfo>> {
        let env = self.target.global_env();
        let callee_env = env.get_function(module_id.qualified(fun_id));
        let callee_ctxt = self.derive_callee_ctxt(&callee_env, ty_args);

        // check argument count and types
        if cfg!(debug_assertions) {
            assert_eq!(callee_ctxt.target.get_parameter_count(), typed_args.len());
        }

        // short-circuit the execution if this is a native function
        if callee_env.is_native() {
            return Ok(callee_ctxt.exec_native_function(
                srcs,
                typed_args,
                local_state,
                global_state,
            ));
        }

        // collect mutable arguments
        let mut_args: Vec<_> = typed_args
            .iter()
            .enumerate()
            .filter(|(_, arg)| arg.get_ty().is_ref(Some(true)))
            .map(|(callee_idx, _)| (callee_idx, *srcs.get(callee_idx).unwrap()))
            .collect();

        // execute the user function
        let mut callee_state = callee_ctxt.exec_user_function(typed_args, global_state)?;

        // update mutable arguments
        for (callee_idx, origin_idx) in mut_args {
            let old_val = local_state.del_value(origin_idx);
            let new_val = if callee_state.has_value(callee_idx) {
                callee_state.del_value(callee_idx)
            } else {
                callee_state.load_destroyed_arg(callee_idx)
            };
            if cfg!(debug_assertions) {
                assert_eq!(old_val.get_ptr(), new_val.get_ptr());
            }
            local_state.put_value(origin_idx, new_val);
        }

        // check callee termination status
        let termination = callee_state.into_termination_status();
        let ok_or_abort = match termination {
            TerminationStatus::Abort(abort_info) => Err(abort_info),
            TerminationStatus::Return(return_vals) => Ok(return_vals),
            TerminationStatus::None | TerminationStatus::PostAbort(_) => unreachable!(),
        };
        Ok(ok_or_abort)
    }

    fn handle_opaque_call_end(
        &self,
        module_id: ModuleId,
        fun_id: FunId,
        ty_args: &[MT::Type],
        typed_args: Vec<TypedValue>,
    ) {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let callee_env = env.get_function(module_id.qualified(fun_id));
            let callee_ctxt = self.derive_callee_ctxt(&callee_env, ty_args);
            assert_eq!(callee_ctxt.target.get_parameter_count(), typed_args.len());
        }
    }

    fn handle_pack(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_fields: Vec<TypedValue>,
    ) -> TypedValue {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        if cfg!(debug_assertions) {
            assert_eq!(inst.fields.len(), op_fields.len());
        }
        TypedValue::mk_struct(inst, op_fields)
    }

    fn handle_unpack(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_struct: TypedValue,
    ) -> Vec<TypedValue> {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
            assert_eq!(&inst, op_struct.get_ty().get_struct_inst());
        }
        op_struct.unpack_struct()
    }

    fn handle_get_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        field_num: usize,
        op_struct: TypedValue,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
            assert!(
                op_struct.get_ty().is_struct_of(&inst)
                    || op_struct.get_ty().is_ref_struct_of(&inst, None)
            );
        }
        if op_struct.get_ty().is_struct() {
            op_struct.unpack_struct_field(field_num)
        } else {
            op_struct.unpack_ref_struct_field(field_num, None)
        }
    }

    fn handle_borrow_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        field_num: usize,
        is_mut: bool,
        local_idx: TempIndex,
        op_struct: TypedValue,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
            assert_eq!(&inst, op_struct.get_ty().get_ref_struct_inst(None));
        }
        op_struct.borrow_ref_struct_field(field_num, is_mut, local_idx)
    }

    fn handle_move_to(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_signer: TypedValue,
        op_struct: TypedValue,
        global_state: &mut GlobalState,
    ) -> Result<(), AbortInfo> {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = op_signer.into_signer();
        if !global_state.put_resource(addr, inst, op_struct) {
            return Err(AbortInfo::sys_abort(StatusCode::RESOURCE_ALREADY_EXISTS));
        }
        Ok(())
    }

    fn handle_move_from(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_addr: TypedValue,
        global_state: &mut GlobalState,
    ) -> Result<TypedValue, AbortInfo> {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = op_addr.into_address();
        match global_state.del_resource(addr, inst) {
            None => Err(AbortInfo::sys_abort(StatusCode::MISSING_DATA)),
            Some(object) => Ok(object),
        }
    }

    fn handle_get_global(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_addr: TypedValue,
        global_state: &mut GlobalState,
    ) -> Result<TypedValue, AbortInfo> {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = op_addr.into_address();
        match global_state.get_resource(None, addr, inst) {
            None => Err(AbortInfo::sys_abort(StatusCode::MISSING_DATA)),
            Some(object) => Ok(object),
        }
    }

    fn handle_borrow_global(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        is_mut: bool,
        op_addr: TypedValue,
        global_state: &mut GlobalState,
    ) -> Result<TypedValue, AbortInfo> {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = op_addr.into_address();
        match global_state.get_resource(Some(is_mut), addr, inst) {
            None => Err(AbortInfo::sys_abort(StatusCode::MISSING_DATA)),
            Some(object) => Ok(object),
        }
    }

    fn handle_exists_global(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_addr: TypedValue,
        global_state: &GlobalState,
    ) -> TypedValue {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = op_addr.into_address();
        TypedValue::mk_bool(global_state.has_resource(&addr, &inst))
    }

    fn handle_write_back_global_struct(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        op_struct: TypedValue,
        global_state: &mut GlobalState,
    ) {
        let env = self.target.global_env();
        let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
        let addr = match op_struct.get_ptr() {
            // TODO (mengxu) this needs to be extended to check for actual address in borrow graph
            // only put the resource back when the address matches
            Pointer::Global(addr) => *addr,
            _ => unreachable!(),
        };
        global_state.put_resource(addr, inst, op_struct.read_ref());
    }

    fn handle_write_back_local(
        &self,
        local_root: TempIndex,
        op_val: TypedValue,
        local_state: &mut LocalState,
    ) {
        if cfg!(debug_assertions) {
            assert!(op_val
                .get_ty()
                .is_ref_of(local_state.get_type(local_root).get_base_type(), Some(true)));
            assert!(local_state.has_value(local_root));
        }
        match op_val.get_ptr() {
            Pointer::Local(root_idx) => {
                if *root_idx == local_root {
                    local_state.put_value_override(local_root, op_val.read_ref());
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_write_back_ref_whole(
        &self,
        local_ref: TempIndex,
        op_val: TypedValue,
        local_state: &mut LocalState,
    ) {
        if cfg!(debug_assertions) {
            let new_ty = op_val.get_ty();
            assert!(new_ty.is_ref(Some(true)));
            assert_eq!(new_ty, local_state.get_type(local_ref));
            assert!(local_state.has_value(local_ref));
        }
        match op_val.get_ptr() {
            Pointer::RefWhole(ref_idx) => {
                if *ref_idx == local_ref {
                    local_state.put_value_override(local_ref, op_val);
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_write_back_ref_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        ty_args: &[MT::Type],
        local_ref: TempIndex,
        field_num: usize,
        op_val: TypedValue,
        local_state: &mut LocalState,
    ) {
        let old_struct = local_state.del_value(local_ref);
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let inst = convert_model_struct_type(env, module_id, struct_id, ty_args, &self.ty_args);
            assert!(old_struct.get_ty().is_ref_struct_of(&inst, Some(true)));
        }
        let new_struct = match op_val.get_ptr() {
            Pointer::RefField(ref_idx, ref_field) => {
                if cfg!(debug_assertions) {
                    assert_eq!(*ref_field, field_num);
                }
                if *ref_idx == local_ref {
                    old_struct.update_ref_struct_field(field_num, op_val)
                } else {
                    old_struct
                }
            }
            _ => unreachable!(),
        };
        local_state.put_value(local_ref, new_struct);
    }

    fn handle_write_back_ref_element(
        &self,
        local_ref: TempIndex,
        op_val: TypedValue,
        local_state: &mut LocalState,
    ) {
        let old_vector = local_state.del_value(local_ref);
        let new_vector = match op_val.get_ptr() {
            Pointer::RefElement(ref_idx, _) => {
                if *ref_idx == local_ref {
                    old_vector.update_ref_vector_element(op_val)
                } else {
                    old_vector
                }
            }
            _ => unreachable!(),
        };
        local_state.put_value(local_ref, new_vector);
    }

    fn handle_borrow_local(
        &self,
        is_mut: bool,
        local_val: TypedValue,
        local_idx: TempIndex,
    ) -> TypedValue {
        local_val.borrow_local(is_mut, local_idx)
    }

    fn handle_read_ref(&self, from_ref: TypedValue) -> TypedValue {
        from_ref.read_ref()
    }

    fn handle_write_ref(
        &self,
        from_val: TypedValue,
        into_ref: TempIndex,
        local_state: &mut LocalState,
    ) {
        let old_val = local_state.del_value(into_ref);
        let (old_ty, _, old_ptr) = old_val.decompose();
        if cfg!(debug_assertions) {
            assert!(old_ty.is_ref_of(from_val.get_ty().get_base_type(), Some(true)));
        }
        let new_val = from_val.write_ref(old_ptr);
        local_state.put_value(into_ref, new_val);
    }

    fn handle_freeze_ref(&self, ref_val: TypedValue) -> TypedValue {
        ref_val.freeze_ref()
    }

    fn handle_destroy(&self, local_idx: TempIndex, local_state: &mut LocalState) {
        let val = local_state.del_value(local_idx);
        if local_idx < self.target.get_parameter_count() {
            local_state.save_destroyed_arg(local_idx, val);
        }
    }

    fn handle_cast_u8(&self, val: TypedValue) -> Result<TypedValue, AbortInfo> {
        let (ty, val, _) = val.decompose();
        let v = if ty.is_u8() {
            val.into_u8()
        } else if ty.is_u64() {
            let v = val.into_u64();
            if v > (u8::MAX as u64) {
                return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
            }
            v as u8
        } else if ty.is_u128() {
            let v = val.into_u128();
            if v > (u8::MAX as u128) {
                return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
            }
            v as u8
        } else {
            let n = val.into_num();
            match n.to_u8() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => v,
            }
        };
        Ok(TypedValue::mk_u8(v))
    }

    fn handle_cast_u64(&self, val: TypedValue) -> Result<TypedValue, AbortInfo> {
        let (ty, val, _) = val.decompose();
        let v = if ty.is_u8() {
            val.into_u8() as u64
        } else if ty.is_u64() {
            val.into_u64()
        } else if ty.is_u128() {
            let v = val.into_u128();
            if v > (u64::MAX as u128) {
                return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
            }
            v as u64
        } else {
            let n = val.into_num();
            match n.to_u64() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => v,
            }
        };
        Ok(TypedValue::mk_u64(v))
    }

    fn handle_cast_u128(&self, val: TypedValue) -> Result<TypedValue, AbortInfo> {
        let (ty, val, _) = val.decompose();
        let v = if ty.is_u8() {
            val.into_u8() as u128
        } else if ty.is_u64() {
            val.into_u64() as u128
        } else if ty.is_u128() {
            val.into_u128()
        } else {
            let n = val.into_num();
            match n.to_u128() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => v,
            }
        };
        Ok(TypedValue::mk_u128(v))
    }

    fn handle_binary_arithmetic(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> Result<TypedValue, AbortInfo> {
        if cfg!(debug_assertions) {
            assert!(res.is_compatible_for_arithmetic(lhs.get_ty(), rhs.get_ty()));
        }

        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::Add => lval + rval,
            Operation::Sub => {
                if lval < rval {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                lval - rval
            }
            Operation::Mul => lval * rval,
            Operation::Div => {
                if rval.is_zero() {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                lval / rval
            }
            Operation::Mod => {
                if rval.is_zero() {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                lval % rval
            }
            _ => unreachable!(),
        };

        let res_val = if res.is_u8() {
            match result.to_u8() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => TypedValue::mk_u8(v),
            }
        } else if res.is_u64() {
            match result.to_u64() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => TypedValue::mk_u64(v),
            }
        } else if res.is_u128() {
            match result.to_u128() {
                None => {
                    return Err(AbortInfo::sys_abort(StatusCode::ARITHMETIC_ERROR));
                }
                Some(v) => TypedValue::mk_u128(v),
            }
        } else {
            assert!(res.is_num());
            TypedValue::mk_num(result)
        };
        Ok(res_val)
    }

    fn handle_binary_bitwise(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(res.is_compatible_for_bitwise(lhs.get_ty(), rhs.get_ty()));
        }

        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::BitAnd => lval & rval,
            Operation::BitOr => lval | rval,
            Operation::Xor => lval ^ rval,
            _ => unreachable!(),
        };

        if res.is_u8() {
            TypedValue::mk_u8(result.to_u8().unwrap())
        } else if res.is_u64() {
            TypedValue::mk_u64(result.to_u64().unwrap())
        } else {
            assert!(res.is_u128());
            TypedValue::mk_u128(result.to_u128().unwrap())
        }
    }

    fn handle_binary_bitshift(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(res.is_compatible_for_bitshift(lhs.get_ty()));
            assert!(rhs.get_ty().is_u8());
        }
        let rval = rhs.into_u8();
        if lhs.get_ty().is_u8() {
            let lval = lhs.into_u8();
            let result = match op {
                Operation::Shl => lval << rval,
                Operation::Shr => lval >> rval,
                _ => unreachable!(),
            };
            TypedValue::mk_u8(result)
        } else if lhs.get_ty().is_u64() {
            let lval = lhs.into_u64();
            let result = match op {
                Operation::Shl => lval << rval,
                Operation::Shr => lval >> rval,
                _ => unreachable!(),
            };
            TypedValue::mk_u64(result)
        } else {
            assert!(lhs.get_ty().is_u128());
            let lval = lhs.into_u128();
            let result = match op {
                Operation::Shl => lval << rval,
                Operation::Shr => lval >> rval,
                _ => unreachable!(),
            };
            TypedValue::mk_u128(result)
        }
    }

    fn handle_binary_comparision(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(lhs.get_ty().is_compatible_for_comparison(rhs.get_ty()));
            assert!(res.is_bool());
        }

        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::Lt => lval < rval,
            Operation::Le => lval <= rval,
            Operation::Ge => lval >= rval,
            Operation::Gt => lval > rval,
            _ => unreachable!(),
        };
        TypedValue::mk_bool(result)
    }

    fn handle_binary_equality(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(lhs.get_ty().is_compatible_for_equality(rhs.get_ty()));
            assert!(res.is_bool());
        }
        let lval = lhs.get_val();
        let rval = rhs.get_val();
        let result = match op {
            Operation::Eq => lval == rval,
            Operation::Neq => lval != rval,
            _ => unreachable!(),
        };
        TypedValue::mk_bool(result)
    }

    fn handle_unary_boolean(&self, op: &Operation, opv: TypedValue, res: &Type) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(opv.get_ty().is_bool());
            assert!(res.is_bool());
        }
        let opval = opv.into_bool();
        let result = match op {
            Operation::Not => !opval,
            _ => unreachable!(),
        };
        TypedValue::mk_bool(result)
    }

    fn handle_binary_boolean(
        &self,
        op: &Operation,
        lhs: TypedValue,
        rhs: TypedValue,
        res: &Type,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(lhs.get_ty().is_bool());
            assert!(rhs.get_ty().is_bool());
            assert!(res.is_bool());
        }
        let lval = lhs.into_bool();
        let rval = rhs.into_bool();
        let result = match op {
            Operation::And => lval && rval,
            Operation::Or => lval || rval,
            _ => unreachable!(),
        };
        TypedValue::mk_bool(result)
    }

    fn handle_conditional_branch(
        &self,
        cond: TempIndex,
        then_label: Label,
        else_label: Label,
        local_state: &mut LocalState,
    ) {
        let cond_val = local_state.get_value(cond);
        if cfg!(debug_assertions) {
            assert!(cond_val.get_ty().is_bool());
        }
        let label = if cond_val.into_bool() {
            then_label
        } else {
            else_label
        };
        local_state.set_pc(self.code_offset_by_label(label));
    }

    fn handle_abort(&self, index: TempIndex, local_state: &mut LocalState) {
        let val = local_state.get_value(index);
        if cfg!(debug_assertions) {
            assert!(val.get_ty().is_compatible_for_abort_code());
        }
        let abort_code = if val.get_ty().is_u64() {
            val.into_u64()
        } else {
            val.into_num().to_u64().unwrap()
        };
        local_state.terminate_with_abort(abort_code);
    }

    fn handle_return(&self, rets: &[TempIndex], local_state: &mut LocalState) {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let decl_ret_types = self.target.get_return_types();
            assert_eq!(rets.len(), decl_ret_types.len());
            for (ret_index, ret_decl_ty) in rets.iter().zip(decl_ret_types) {
                let ret_ty = convert_model_local_type(env, ret_decl_ty, &self.ty_args);
                assert_eq!(&ret_ty, local_state.get_type(*ret_index));
            }
        }
        let ret_vals = rets
            .iter()
            .map(|index| local_state.get_value(*index))
            .collect();
        local_state.terminate_with_return(ret_vals);
    }

    //
    // natives
    //

    fn native_vector_empty(&self) -> TypedValue {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
        }
        TypedValue::mk_vector(self.ty_args.get(0).unwrap().clone(), vec![])
    }

    fn native_vector_length(&self, vec_val: TypedValue) -> TypedValue {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            // NOTE: this function accepts a value instead of a reference!
            // This is different from the Move native implementation.
            assert_eq!(
                vec_val.get_ty().get_vector_elem(),
                self.ty_args.get(0).unwrap()
            );
        }
        TypedValue::mk_u64(vec_val.into_vector().len() as u64)
    }

    fn native_vector_borrow(
        &self,
        is_mut: bool,
        vec_idx: TempIndex,
        vec_val: TypedValue,
        elem_val: TypedValue,
    ) -> Result<TypedValue, AbortInfo> {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                vec_val.get_ty().get_ref_vector_elem(None),
                self.ty_args.get(0).unwrap()
            );
        }
        let elem_num = elem_val.into_u64() as usize;
        vec_val
            .borrow_ref_vector_element(elem_num, is_mut, vec_idx)
            .ok_or_else(|| AbortInfo::usr_abort(INDEX_OUT_OF_BOUNDS))
    }

    fn native_vector_push_back(&self, vec_val: TypedValue, elem_val: TypedValue) -> TypedValue {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                vec_val.get_ty().get_ref_vector_elem(Some(true)),
                self.ty_args.get(0).unwrap()
            );
        }
        vec_val.update_ref_vector_push_back(elem_val)
    }

    fn native_vector_pop_back(
        &self,
        vec_val: TypedValue,
    ) -> Result<(TypedValue, TypedValue), AbortInfo> {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                vec_val.get_ty().get_ref_vector_elem(Some(true)),
                self.ty_args.get(0).unwrap()
            );
        }
        vec_val
            .update_ref_vector_pop_back()
            .ok_or_else(|| AbortInfo::usr_abort(POP_EMPTY_VEC))
    }

    fn native_vector_destroy_empty(&self, vec_val: TypedValue) -> Result<(), AbortInfo> {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                vec_val.get_ty().get_vector_elem(),
                self.ty_args.get(0).unwrap()
            );
        }
        if !vec_val.into_vector().is_empty() {
            return Err(AbortInfo::usr_abort(DESTROY_NON_EMPTY_VEC));
        }
        Ok(())
    }

    fn native_vector_swap(
        &self,
        vec_val: TypedValue,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> Result<TypedValue, AbortInfo> {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                vec_val.get_ty().get_ref_vector_elem(Some(true)),
                self.ty_args.get(0).unwrap()
            );
        }
        vec_val
            .update_ref_vector_swap(lhs.into_u64() as usize, rhs.into_u64() as usize)
            .ok_or_else(|| AbortInfo::usr_abort(INDEX_OUT_OF_BOUNDS))
    }

    fn native_signer_borrow_address(&self, signer_val: TypedValue) -> TypedValue {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 0);
        }
        // NOTE: this function accepts a value instead of a reference!
        // This is different from the Move native implementation.
        let addr = signer_val.into_signer();
        TypedValue::mk_address(addr)
    }

    fn native_hash_sha2_256(&self, bytes_val: TypedValue) -> TypedValue {
        let elem_ty = BaseType::mk_u8();
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 0);
            assert!(bytes_val.get_ty().is_vector_of(&elem_ty));
        }
        let bytes: Vec<_> = bytes_val
            .into_vector()
            .into_iter()
            .map(|e| e.into_u8())
            .collect();
        let digest = Sha256::digest(&bytes).to_vec();
        let hashed = digest.into_iter().map(TypedValue::mk_u8).collect();
        TypedValue::mk_vector(elem_ty, hashed)
    }

    fn native_hash_sha3_256(&self, bytes_val: TypedValue) -> TypedValue {
        let elem_ty = BaseType::mk_u8();
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 0);
            assert!(bytes_val.get_ty().is_vector_of(&elem_ty));
        }
        let bytes: Vec<_> = bytes_val
            .into_vector()
            .into_iter()
            .map(|e| e.into_u8())
            .collect();
        let digest = HashValue::sha3_256_of(&bytes).to_vec();
        let hashed = digest.into_iter().map(TypedValue::mk_u8).collect();
        TypedValue::mk_vector(elem_ty, hashed)
    }

    fn native_bcs_to_bytes(&self, object: TypedValue) -> Result<TypedValue, AbortInfo> {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            object
                .get_ty()
                .is_ref_of(self.ty_args.get(0).unwrap(), Some(false));
        }
        object
            .into_bcs_bytes()
            .map(|bytes| {
                let bcs_val = bytes.into_iter().map(TypedValue::mk_u8).collect();
                TypedValue::mk_vector(BaseType::mk_u8(), bcs_val)
            })
            .ok_or_else(|| AbortInfo::usr_abort(sub_status::NFE_BCS_SERIALIZATION_FAILURE))
    }

    fn native_event_write_to_event_store(
        &self,
        guid_val: TypedValue,
        seq_val: TypedValue,
        msg_val: TypedValue,
        global_state: &mut GlobalState,
    ) {
        if cfg!(debug_assertions) {
            assert_eq!(self.ty_args.len(), 1);
            assert_eq!(
                msg_val.get_ty().get_base_type(),
                self.ty_args.get(0).unwrap()
            );
        }
        let guid = guid_val
            .into_vector()
            .into_iter()
            .map(|e| e.into_u8())
            .collect();
        let seq = seq_val.into_u64();
        global_state.emit_event(guid, seq, msg_val);
    }

    //
    // utilities
    //

    fn code_offset_by_label(&self, label: Label) -> CodeOffset {
        return *self.label_offsets.get(&label).unwrap();
    }

    fn derive_callee_ctxt(
        &self,
        callee_env: &'env FunctionEnv<'env>,
        ty_args: &[MT::Type],
    ) -> FunctionContext<'env> {
        let env = self.target.global_env();
        let callee_target = choose_variant(self.holder, callee_env);

        // check and convert type arguments
        if cfg!(debug_assertions) {
            let callee_ty_params = callee_target.get_type_parameters();
            // TODO (mengxu) verify type constraints
            assert_eq!(callee_ty_params.len(), ty_args.len());
        }
        let callee_ty_insts: Vec<_> = ty_args
            .iter()
            .map(|ty_arg| convert_model_base_type(env, ty_arg, &self.ty_args))
            .collect();

        // build the context
        FunctionContext::new(self.holder, callee_target, callee_ty_insts)
    }

    fn prepare_local_state(&self, typed_args: Vec<TypedValue>) -> LocalState {
        let target = &self.target;
        let env = target.global_env();

        // discover and validate local slots
        let param_decls = target.func_env.get_parameters();
        if cfg!(debug_assertions) {
            assert_eq!(param_decls.len(), typed_args.len());
            assert!(param_decls.len() <= target.get_local_count());
        }

        let mut local_slots = vec![];
        for (i, typed_arg) in typed_args.into_iter().enumerate() {
            let name = env
                .symbol_pool()
                .string(target.get_local_name(i))
                .to_string();

            // check that types for local slots is compatible with the declared parameter type
            if cfg!(debug_assertions) {
                let local_ty = target.get_local_type(i);
                let param_decl_ty = &param_decls.get(i).unwrap().1;
                if local_ty != param_decl_ty {
                    assert!(matches!(
                            param_decl_ty,
                            MT::Type::Reference(false, base_ty)
                            if local_ty == base_ty.as_ref()));
                }
                let ty = convert_model_local_type(env, local_ty, &self.ty_args);
                assert_eq!(&ty, typed_arg.get_ty());
            }

            let slot = LocalSlot::new_arg(name, typed_arg);
            local_slots.push(slot);
        }
        for i in param_decls.len()..target.get_local_count() {
            let name = env
                .symbol_pool()
                .string(target.get_local_name(i))
                .to_string();
            let ty = convert_model_local_type(env, target.get_local_type(i), &self.ty_args);
            let slot = LocalSlot::new_tmp(name, ty);
            local_slots.push(slot);
        }
        LocalState::new(local_slots)
    }
}

//**************************************************************************************************
// Entrypoint
//**************************************************************************************************

/// Entrypoint of the interpretation logic
pub fn entrypoint(
    holder: &FunctionTargetsHolder,
    target: FunctionTarget,
    ty_args: &[BaseType],
    typed_args: Vec<TypedValue>,
    global_state: &mut GlobalState,
) -> PartialVMResult<Vec<TypedValue>> {
    let ctxt = FunctionContext::new(holder, target, ty_args.to_vec());
    let local_state = ctxt.exec_user_function(typed_args, global_state)?;
    let termination = local_state.into_termination_status();
    let return_vals = match termination {
        TerminationStatus::Abort(abort_info) => {
            return Err(abort_info.into_err());
        }
        TerminationStatus::Return(return_vals) => return_vals,
        TerminationStatus::None | TerminationStatus::PostAbort(_) => unreachable!(),
    };
    Ok(return_vals)
}
