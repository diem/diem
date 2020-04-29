// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{lcs, signature};
use crate::{
    loaded_data::runtime_types::Type,
    native_functions::{account, context::NativeContext, event, hash},
    values::{debug, vector, Value},
};
use libra_types::{
    account_address::AccountAddress,
    account_config::{
        account_type_module_name, account_type_struct_name, event_handle_generator_struct_name,
        event_module_name, AccountResource, BalanceResource, CORE_CODE_ADDRESS,
    },
    language_storage::ModuleId,
    move_resource::MoveResource,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::gas_schedule::{
    AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits, NativeCostIndex,
};
use std::collections::VecDeque;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{FunctionSignature, Kind, Signature, SignatureToken, StructHandleIndex},
    views::ModuleView,
};

/// Result of a native function execution that requires charges for execution cost.
///
/// An execution that causes an invariant violation would not return a `NativeResult` but
/// return a `VMResult` error directly.
/// All native functions must return a `VMResult<NativeResult>` where an `Err` is returned
/// when an error condition is met that should not charge for the execution. A common example
/// is a VM invariant violation which should have been forbidden by the verifier.
/// Errors (typically user errors and aborts) that are logically part of the function execution
/// must be expressed in a `NativeResult` via a cost and a VMStatus.
pub struct NativeResult {
    /// The cost for running that function, whether successfully or not.
    pub cost: GasUnits<GasCarrier>,
    /// Result of execution. This is either the return values or the error to report.
    pub result: VMResult<Vec<Value>>,
}

impl NativeResult {
    /// Return values of a successful execution.
    pub fn ok(cost: GasUnits<GasCarrier>, values: Vec<Value>) -> Self {
        NativeResult {
            cost,
            result: Ok(values),
        }
    }

    /// `VMStatus` of a failed execution. The failure is a runtime failure in the function
    /// and not an invariant failure of the VM which would raise a `VMResult` error directly.
    pub fn err(cost: GasUnits<GasCarrier>, err: VMStatus) -> Self {
        NativeResult {
            cost,
            result: Err(err),
        }
    }
}

pub fn native_gas(table: &CostTable, key: NativeCostIndex, size: usize) -> GasUnits<GasCarrier> {
    let gas_amt = table.native_cost(key);
    let memory_size = AbstractMemorySize::new(size as GasCarrier);
    gas_amt.total().mul(memory_size)
}

#[derive(Debug, Clone, Copy)]
pub enum NativeFunction {
    HashSha2_256,
    HashSha3_256,
    LCSToBytes,
    SigED25519Verify,
    SigED25519ThresholdVerify,
    VectorLength,
    VectorEmpty,
    VectorBorrow,
    VectorBorrowMut,
    VectorPushBack,
    VectorPopBack,
    VectorDestroyEmpty,
    VectorSwap,
    AccountWriteEvent,
    AccountSaveAccount,
    DebugPrint,
    DebugPrintStackTrace,
}

/// Function.
pub trait Function: Copy {
    /// Given the vector of aguments, it executes the native function.
    fn dispatch(
        self,
        ctx: &mut impl NativeContext,
        t: Vec<Type>,
        v: VecDeque<Value>,
    ) -> VMResult<NativeResult>;

    /// The number of arguments to the native function,
    /// It is checked at publishing of the module that this matches the expected signature.
    fn num_args(&self) -> usize;

    fn parameters<T: ModuleAccess>(self, m: Option<&ModuleView<T>>) -> VMResult<Option<Signature>>;

    fn return_<T: ModuleAccess>(self, m: Option<&ModuleView<T>>) -> VMResult<Option<Signature>>;

    fn type_parameters<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<Vec<Kind>>>;

    /// The signature as defined in it's declaring module.
    /// It should NOT be generally inspected outside of it's declaring module as the various
    /// struct handle indexes are not remapped into the local context.
    /// Returns:
    /// - `Err(NATIVE_FUNCTION_INTERNAL_INCONSISTENCY)` if the produced function signature
    ///   is inconsistent with `self.num_args()`. This only fails if the native function was
    ///   implemented incorrectly
    /// - `Ok(None)` if a function signature could not be generated for the native function with
    ///   the given `ModuleView`, `m`
    /// - `Ok(Some(expected_function_signature))` otherwise
    fn signature<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<FunctionSignature>>;
}

pub struct FunctionResolver();

impl FunctionResolver {
    pub fn resolve(module: &ModuleId, function_name: &str) -> Option<impl Function> {
        use NativeFunction::*;

        let case = (module.address(), module.name().as_str(), function_name);
        Some(match case {
            (&CORE_CODE_ADDRESS, "Hash", "sha2_256") => HashSha2_256,
            (&CORE_CODE_ADDRESS, "Hash", "sha3_256") => HashSha3_256,
            (&CORE_CODE_ADDRESS, "LCS", "to_bytes") => LCSToBytes,
            (&CORE_CODE_ADDRESS, "Signature", "ed25519_verify") => SigED25519Verify,
            (&CORE_CODE_ADDRESS, "Signature", "ed25519_threshold_verify") => {
                SigED25519ThresholdVerify
            }
            (&CORE_CODE_ADDRESS, "Vector", "length") => VectorLength,
            (&CORE_CODE_ADDRESS, "Vector", "empty") => VectorEmpty,
            (&CORE_CODE_ADDRESS, "Vector", "borrow") => VectorBorrow,
            (&CORE_CODE_ADDRESS, "Vector", "borrow_mut") => VectorBorrowMut,
            (&CORE_CODE_ADDRESS, "Vector", "push_back") => VectorPushBack,
            (&CORE_CODE_ADDRESS, "Vector", "pop_back") => VectorPopBack,
            (&CORE_CODE_ADDRESS, "Vector", "destroy_empty") => VectorDestroyEmpty,
            (&CORE_CODE_ADDRESS, "Vector", "swap") => VectorSwap,
            (&CORE_CODE_ADDRESS, "Event", "write_to_event_store") => AccountWriteEvent,
            (&CORE_CODE_ADDRESS, "LibraAccount", "save_account") => AccountSaveAccount,
            (&CORE_CODE_ADDRESS, "Debug", "print") => DebugPrint,
            (&CORE_CODE_ADDRESS, "Debug", "print_stack_trace") => DebugPrintStackTrace,
            _ => return None,
        })
    }
}

impl Function for NativeFunction {
    /// Given the vector of aguments, it executes the native function.
    fn dispatch(
        self,
        ctx: &mut impl NativeContext,
        t: Vec<Type>,
        v: VecDeque<Value>,
    ) -> VMResult<NativeResult> {
        match self {
            Self::HashSha2_256 => hash::native_sha2_256(ctx, t, v),
            Self::HashSha3_256 => hash::native_sha3_256(ctx, t, v),
            Self::SigED25519Verify => signature::native_ed25519_signature_verification(ctx, t, v),
            Self::SigED25519ThresholdVerify => {
                signature::native_ed25519_threshold_signature_verification(ctx, t, v)
            }
            Self::VectorLength => vector::native_length(ctx, t, v),
            Self::VectorEmpty => vector::native_empty(ctx, t, v),
            Self::VectorBorrow => vector::native_borrow(ctx, t, v),
            Self::VectorBorrowMut => vector::native_borrow(ctx, t, v),
            Self::VectorPushBack => vector::native_push_back(ctx, t, v),
            Self::VectorPopBack => vector::native_pop(ctx, t, v),
            Self::VectorDestroyEmpty => vector::native_destroy_empty(ctx, t, v),
            Self::VectorSwap => vector::native_swap(ctx, t, v),
            Self::AccountWriteEvent => event::native_emit_event(ctx, t, v),
            Self::AccountSaveAccount => account::native_save_account(ctx, t, v),
            Self::LCSToBytes => lcs::native_to_bytes(ctx, t, v),
            Self::DebugPrint => debug::native_print(ctx, t, v),
            Self::DebugPrintStackTrace => debug::native_print_stack_trace(ctx, t, v),
        }
    }

    /// The number of arguments to the native function,
    /// It is checked at publishing of the module that this matches the expected signature.
    fn num_args(&self) -> usize {
        match self {
            Self::HashSha2_256 => 1,
            Self::HashSha3_256 => 1,
            Self::LCSToBytes => 1,
            Self::SigED25519Verify => 3,
            Self::SigED25519ThresholdVerify => 4,
            Self::VectorLength => 1,
            Self::VectorEmpty => 0,
            Self::VectorBorrow => 2,
            Self::VectorBorrowMut => 2,
            Self::VectorPushBack => 2,
            Self::VectorPopBack => 1,
            Self::VectorDestroyEmpty => 1,
            Self::VectorSwap => 3,
            Self::AccountWriteEvent => 3,
            Self::AccountSaveAccount => 5,
            Self::DebugPrint => 1,
            Self::DebugPrintStackTrace => 0,
        }
    }

    fn parameters<T: ModuleAccess>(self, m: Option<&ModuleView<T>>) -> VMResult<Option<Signature>> {
        Ok(self.signature(m)?.map(|res| Signature(res.parameters)))
    }

    fn return_<T: ModuleAccess>(self, m: Option<&ModuleView<T>>) -> VMResult<Option<Signature>> {
        Ok(self.signature(m)?.map(|res| Signature(res.return_)))
    }

    fn type_parameters<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<Vec<Kind>>> {
        Ok(self.signature(m)?.map(|res| res.type_parameters))
    }

    /// The signature as defined in it's declaring module.
    /// It should NOT be generally inspected outside of it's declaring module as the various
    /// struct handle indexes are not remapped into the local context.
    /// Returns:
    /// - `Err(NATIVE_FUNCTION_INTERNAL_INCONSISTENCY)` if the produced function signature
    ///   is inconsistent with `self.num_args()`. This only fails if the native function was
    ///   implemented incorrectly
    /// - `Ok(None)` if a function signature could not be generated for the native function with
    ///   the given `ModuleView`, `m`
    /// - `Ok(Some(expected_function_signature))` otherwise
    fn signature<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<FunctionSignature>> {
        let res = match self.signature_(m) {
            None => return Ok(None),
            Some(res) => res,
        };
        if self.num_args() == res.parameters.len() {
            Ok(Some(res))
        } else {
            Err(
                VMStatus::new(StatusCode::NATIVE_FUNCTION_INTERNAL_INCONSISTENCY).with_message(
                    "Invalid native function declaration. Declared number of args does not match \
                     produced signature"
                        .to_owned(),
                ),
            )
        }
    }
}

impl NativeFunction {
    fn signature_<T: ModuleAccess>(self, m: Option<&ModuleView<T>>) -> Option<FunctionSignature> {
        use SignatureToken::*;
        macro_rules! simple {
            ($args:expr, $ret:expr) => {{
                simple!(vec![], $args, $ret)
            }};
            ($kinds:expr, $args:expr, $ret:expr) => {{
                FunctionSignature {
                    return_: $ret,
                    parameters: $args,
                    type_parameters: $kinds,
                }
            }};
        }
        Some(match self {
            Self::HashSha2_256 => simple!(vec![Vector(Box::new(U8))], vec![Vector(Box::new(U8))]),
            Self::HashSha3_256 => simple!(vec![Vector(Box::new(U8))], vec![Vector(Box::new(U8))]),
            Self::LCSToBytes => {
                let type_parameters = vec![Kind::All];
                let parameters = vec![Reference(Box::new(TypeParameter(0)))];
                let return_ = vec![Vector(Box::new(U8))];
                FunctionSignature {
                    type_parameters,
                    parameters,
                    return_,
                }
            }
            Self::SigED25519Verify => simple!(
                vec![
                    Vector(Box::new(U8)),
                    Vector(Box::new(U8)),
                    Vector(Box::new(U8))
                ],
                vec![Bool]
            ),
            Self::SigED25519ThresholdVerify => simple!(
                vec![
                    Vector(Box::new(U8)),
                    Vector(Box::new(U8)),
                    Vector(Box::new(U8)),
                    Vector(Box::new(U8))
                ],
                vec![U64]
            ),
            Self::VectorLength => simple!(
                vec![Kind::All],
                vec![Reference(Box::new(Vector(Box::new(TypeParameter(0)))))],
                vec![U64]
            ),
            Self::VectorEmpty => simple!(
                vec![Kind::All],
                vec![],
                vec![Vector(Box::new(TypeParameter(0)))]
            ),
            Self::VectorBorrow => simple!(
                vec![Kind::All],
                vec![Reference(Box::new(Vector(Box::new(TypeParameter(0))))), U64],
                vec![Reference(Box::new(TypeParameter(0)))]
            ),
            Self::VectorBorrowMut => simple!(
                vec![Kind::All],
                vec![
                    MutableReference(Box::new(Vector(Box::new(TypeParameter(0))))),
                    U64
                ],
                vec![MutableReference(Box::new(TypeParameter(0)))]
            ),
            Self::VectorPushBack => simple!(
                vec![Kind::All],
                vec![
                    MutableReference(Box::new(Vector(Box::new(TypeParameter(0))))),
                    TypeParameter(0),
                ],
                vec![]
            ),
            Self::VectorPopBack => simple!(
                vec![Kind::All],
                vec![MutableReference(Box::new(Vector(Box::new(TypeParameter(
                    0
                )))))],
                vec![TypeParameter(0)]
            ),
            Self::VectorDestroyEmpty => simple!(
                vec![Kind::All],
                vec![Vector(Box::new(TypeParameter(0)))],
                vec![]
            ),
            Self::VectorSwap => simple!(
                vec![Kind::All],
                vec![
                    MutableReference(Box::new(Vector(Box::new(TypeParameter(0))))),
                    U64,
                    U64,
                ],
                vec![]
            ),
            Self::AccountWriteEvent => simple!(
                vec![Kind::Copyable],
                vec![Vector(Box::new(U8)), U64, TypeParameter(0)],
                vec![]
            ),
            Self::AccountSaveAccount => {
                let type_parameters = vec![Kind::All, Kind::Copyable];
                let self_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    AccountResource::MODULE_NAME,
                    AccountResource::STRUCT_NAME,
                )?;
                let balance_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    AccountResource::MODULE_NAME,
                    BalanceResource::STRUCT_NAME,
                )?;
                let assoc_cap_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    account_type_module_name().as_str(),
                    account_type_struct_name().as_str(),
                )?;
                let event_generator_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    event_module_name().as_str(),
                    event_handle_generator_struct_name().as_str(),
                )?;
                let parameters = vec![
                    StructInstantiation(assoc_cap_t_idx, vec![TypeParameter(1)]),
                    StructInstantiation(balance_t_idx, vec![TypeParameter(0)]),
                    Struct(self_t_idx),
                    Struct(event_generator_t_idx),
                    Address,
                ];
                let return_ = vec![];
                FunctionSignature {
                    type_parameters,
                    parameters,
                    return_,
                }
            }
            Self::DebugPrint => simple!(
                vec![Kind::All],
                vec![Reference(Box::new(TypeParameter(0)))],
                vec![]
            ),
            Self::DebugPrintStackTrace => simple!(vec![], vec![], vec![]),
        })
    }
}

/// Helper for finding non-native struct handle index
fn struct_handle_idx<T: ModuleAccess>(
    m: &ModuleView<T>,
    module_address: &AccountAddress,
    module_name: &str,
    name: &str,
) -> Option<StructHandleIndex> {
    m.struct_handles().enumerate().find_map(|(idx, handle)| {
        if handle.name().as_str() == name
            && handle.module_id().name().as_str() == module_name
            && handle.module_id().address() == module_address
        {
            Some(StructHandleIndex(idx as u16))
        } else {
            None
        }
    })
}

#[macro_export]
macro_rules! pop_arg {
    ($arguments:ident, $t:ty) => {{
        $arguments.pop_back().unwrap().value_as::<$t>()?
    }};
}
