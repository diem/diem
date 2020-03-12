// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{hash, lcs, signature};
use crate::{
    loaded_data::types::Type,
    values::{debug, vector, Value},
};
use libra_types::{
    account_address::AccountAddress,
    account_config::{
        account_balance_struct_name, account_module_name, account_struct_name, CORE_CODE_ADDRESS,
    },
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::IdentStr;
use std::collections::VecDeque;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{FunctionSignature, Kind, Signature, SignatureToken, StructHandleIndex},
    gas_schedule::{
        AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits, NativeCostIndex,
    },
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
}

impl NativeFunction {
    pub fn resolve(module: &ModuleId, function_name: &IdentStr) -> Option<Self> {
        use NativeFunction::*;

        let case = (
            module.address(),
            module.name().as_str(),
            function_name.as_str(),
        );
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
            (&CORE_CODE_ADDRESS, "LibraAccount", "write_to_event_store") => AccountWriteEvent,
            (&CORE_CODE_ADDRESS, "LibraAccount", "save_account") => AccountSaveAccount,
            (&CORE_CODE_ADDRESS, "Debug", "print") => DebugPrint,
            _ => return None,
        })
    }
}

impl NativeFunction {
    /// Given the vector of aguments, it executes the native function.
    pub fn dispatch(
        self,
        t: Vec<Type>,
        v: VecDeque<Value>,
        c: &CostTable,
    ) -> VMResult<NativeResult> {
        match self {
            Self::HashSha2_256 => hash::native_sha2_256(t, v, c),
            Self::HashSha3_256 => hash::native_sha3_256(t, v, c),
            Self::LCSToBytes => lcs::native_to_bytes(t, v, c),
            Self::SigED25519Verify => signature::native_ed25519_signature_verification(t, v, c),
            Self::SigED25519ThresholdVerify => {
                signature::native_ed25519_threshold_signature_verification(t, v, c)
            }
            Self::VectorLength => vector::native_length(t, v, c),
            Self::VectorEmpty => vector::native_empty(t, v, c),
            Self::VectorBorrow => vector::native_borrow(t, v, c),
            Self::VectorBorrowMut => vector::native_borrow(t, v, c),
            Self::VectorPushBack => vector::native_push_back(t, v, c),
            Self::VectorPopBack => vector::native_pop(t, v, c),
            Self::VectorDestroyEmpty => vector::native_destroy_empty(t, v, c),
            Self::VectorSwap => vector::native_swap(t, v, c),
            Self::AccountWriteEvent => Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(
                "write_to_event_store does not have a native implementation".to_string(),
            )),
            Self::AccountSaveAccount => Err(VMStatus::new(StatusCode::UNREACHABLE)
                .with_message("save_account does not have a native implementation".to_string())),
            Self::DebugPrint => debug::native_print(t, v, c),
        }
    }

    /// The number of arguments to the native function,
    /// It is checked at publishing of the module that this matches the expected signature.
    pub fn num_args(self) -> usize {
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
            Self::AccountSaveAccount => 3,
            Self::DebugPrint => 1,
        }
    }

    pub fn parameters<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<Signature>> {
        Ok(self.signature(m)?.map(|res| Signature(res.parameters)))
    }

    pub fn return_<T: ModuleAccess>(
        self,
        m: Option<&ModuleView<T>>,
    ) -> VMResult<Option<Signature>> {
        Ok(self.signature(m)?.map(|res| Signature(res.return_)))
    }

    pub fn type_parameters<T: ModuleAccess>(
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
    pub fn signature<T: ModuleAccess>(
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
                let type_parameters = vec![Kind::All];
                let self_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    account_module_name().as_str(),
                    account_struct_name().as_str(),
                )?;
                let balance_t_idx = struct_handle_idx(
                    m?,
                    &CORE_CODE_ADDRESS,
                    account_module_name().as_str(),
                    account_balance_struct_name().as_str(),
                )?;
                let parameters = vec![
                    StructInstantiation(balance_t_idx, vec![TypeParameter(0)]),
                    Struct(self_t_idx),
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
