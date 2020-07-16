// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To re-generate this code, run: `(cd language/stdlib && cargo run --release)`

use libra_types::{
    account_address::AccountAddress,
    transaction::{Script, TransactionArgument},
};
use move_core_types::language_storage::TypeTag;

/// Add a `Currency` balance to `account`, which will enable `account` to send and receive
/// `Libra<Currency>`. Aborts with NOT_A_CURRENCY if `Currency` is not an accepted
/// currency type in the Libra system Aborts with `LibraAccount::ADD_EXISTING_CURRENCY` if
/// the account already holds a balance in `Currency`. Aborts with
/// `LibraAccount::PARENT_VASP_CURRENCY_LIMITS_DNE` if `account` is a `ChildVASP` whose
/// parent does not have an `AccountLimits<Currency>` resource.
pub fn encode_add_currency_to_account_script(currency: TypeTag) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 7, 7, 17, 26, 8, 43,
            16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 1, 6, 12, 0, 1, 9, 0, 12, 76, 105, 98, 114, 97, 65,
            99, 99, 111, 117, 110, 116, 12, 97, 100, 100, 95, 99, 117, 114, 114, 101, 110, 99, 121,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 3, 11, 0, 56, 0, 2,
        ],
        vec![currency],
        vec![],
    )
}

/// Add the `KeyRotationCapability` for `to_recover_account` to the `RecoveryAddress`
/// resource under `recovery_address`. ## Aborts * Aborts with
/// `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if `account` has already
/// delegated its `KeyRotationCapability`. * Aborts with
/// `RecoveryAddress:ENOT_A_RECOVERY_ADDRESS` if `recovery_address` does not have a
/// `RecoveryAddress` resource. * Aborts with
/// `RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION` if `to_recover_account` and
/// `recovery_address` do not belong to the same VASP.
pub fn encode_add_recovery_rotation_capability_script(recovery_address: AccountAddress) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 2, 4, 4, 3, 8, 10, 5, 18, 15, 7, 33, 107, 8,
            140, 1, 16, 0, 0, 0, 1, 0, 2, 1, 0, 0, 3, 0, 1, 0, 1, 4, 2, 3, 0, 1, 6, 12, 1, 8, 0, 2,
            8, 0, 5, 0, 2, 6, 12, 5, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 15,
            82, 101, 99, 111, 118, 101, 114, 121, 65, 100, 100, 114, 101, 115, 115, 21, 75, 101,
            121, 82, 111, 116, 97, 116, 105, 111, 110, 67, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 31, 101, 120, 116, 114, 97, 99, 116, 95, 107, 101, 121, 95, 114, 111, 116, 97,
            116, 105, 111, 110, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 23, 97, 100, 100,
            95, 114, 111, 116, 97, 116, 105, 111, 110, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 4, 3, 5, 11, 0, 17, 0, 10, 1,
            17, 1, 2,
        ],
        vec![],
        vec![TransactionArgument::Address(recovery_address)],
    )
}

/// Add `new_validator` to the validator set. Fails if the `new_validator` address is
/// already in the validator set or does not have a `ValidatorConfig` resource stored at
/// the address. Emits a NewEpochEvent. TODO(valerini): rename to
/// add_validator_and_reconfigure?
pub fn encode_add_validator_script(validator_address: AccountAddress) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 5, 7, 12, 26, 8, 38, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 5, 0, 11, 76, 105, 98, 114, 97, 83, 121, 115, 116, 101, 109,
            13, 97, 100, 100, 95, 118, 97, 108, 105, 100, 97, 116, 111, 114, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11, 0, 10, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::Address(validator_address)],
    )
}

/// Permanently destroy the `Token`s stored in the oldest burn request under the `Preburn`
/// resource. This will only succeed if `account` has a `MintCapability<Token>`, a
/// `Preburn<Token>` resource exists under `preburn_address`, and there is a pending burn
/// request. sliding_nonce is a unique nonce for operation, see sliding_nonce.move for
/// details
pub fn encode_burn_script(
    token: TypeTag,
    sliding_nonce: u64,
    preburn_address: AccountAddress,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 3, 4, 11, 4, 15, 2, 5, 17, 17, 7, 34, 46, 8,
            80, 16, 0, 0, 0, 1, 1, 2, 0, 1, 0, 0, 3, 2, 1, 1, 1, 1, 4, 2, 6, 12, 3, 0, 2, 6, 12, 5,
            3, 6, 12, 3, 5, 1, 9, 0, 5, 76, 105, 98, 114, 97, 12, 83, 108, 105, 100, 105, 110, 103,
            78, 111, 110, 99, 101, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111, 110, 99, 101, 95,
            111, 114, 95, 97, 98, 111, 114, 116, 4, 98, 117, 114, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 1, 3, 1, 7, 10, 0, 10, 1, 17, 0, 11, 0, 10, 2, 56, 0, 2,
        ],
        vec![token],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::Address(preburn_address),
        ],
    )
}

/// Burn transaction fees that have been collected in the given `currency` and relinquish
/// to the association. The currency must be non-synthetic.
pub fn encode_burn_txn_fees_script(coin_type: TypeTag) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 7, 7, 17, 25, 8, 42,
            16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 1, 6, 12, 0, 1, 9, 0, 14, 84, 114, 97, 110, 115, 97,
            99, 116, 105, 111, 110, 70, 101, 101, 9, 98, 117, 114, 110, 95, 102, 101, 101, 115, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 3, 11, 0, 56, 0, 2,
        ],
        vec![coin_type],
        vec![],
    )
}

/// Cancel the oldest burn request from `preburn_address` and return the funds. Fails if
/// the sender does not have a published `BurnCapability<Token>`.
pub fn encode_cancel_burn_script(token: TypeTag, preburn_address: AccountAddress) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 8, 7, 18, 25, 8, 43,
            16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 2, 6, 12, 5, 0, 1, 9, 0, 12, 76, 105, 98, 114, 97,
            65, 99, 99, 111, 117, 110, 116, 11, 99, 97, 110, 99, 101, 108, 95, 98, 117, 114, 110,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 4, 11, 0, 10, 1, 56, 0, 2,
        ],
        vec![token],
        vec![TransactionArgument::Address(preburn_address)],
    )
}

/// Create a `ChildVASP` account for sender `parent_vasp` at `child_address` with a
/// balance of `child_initial_balance` in `CoinType` and an initial authentication_key
/// `auth_key_prefix | child_address`. If `add_all_currencies` is true, the child address
/// will have a zero balance in all available currencies in the system. This account will
/// a child of the transaction sender, which must be a ParentVASP. ## Aborts The
/// transaction will abort: * If `parent_vasp` is not a parent vasp with error:
/// `Roles::EINVALID_PARENT_ROLE` * If `child_address` already exists with error:
/// `Roles::EROLE_ALREADY_ASSIGNED` * If `parent_vasp` already has 256 child accounts with
/// error: `VASP::ETOO_MANY_CHILDREN` * If `parent_vasp` does not hold limits for
/// `CoinType` with error: `VASP::ENOT_A_PARENT_VASP` * If `CoinType` is not a registered
/// currency with error: `LibraAccount::ENOT_A_CURRENCY` * If `parent_vasp`'s withdrawal
/// capability has been extracted with error:
/// `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` * If `parent_vasp` doesn't
/// hold `CoinType` and `child_initial_balance > 0` with error:
/// `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY` * If `parent_vasp` doesn't at least
/// `child_initial_balance` of `CoinType` in its account balance with error:
/// `LibraAccount::EINSUFFICIENT_BALANCE`
pub fn encode_create_child_vasp_account_script(
    coin_type: TypeTag,
    child_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
    add_all_currencies: bool,
    child_initial_balance: u64,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 8, 1, 0, 2, 2, 2, 4, 3, 6, 22, 4, 28, 4, 5, 32, 35, 7,
            67, 123, 8, 190, 1, 16, 6, 206, 1, 4, 0, 0, 0, 1, 1, 0, 0, 2, 0, 1, 1, 1, 0, 3, 2, 3,
            0, 0, 4, 4, 1, 1, 1, 0, 5, 3, 1, 0, 0, 6, 2, 6, 4, 6, 12, 5, 10, 2, 1, 0, 1, 6, 12, 1,
            8, 0, 5, 6, 8, 0, 5, 3, 10, 2, 10, 2, 5, 6, 12, 5, 10, 2, 1, 3, 1, 9, 0, 12, 76, 105,
            98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 18, 87, 105, 116, 104, 100, 114, 97, 119,
            67, 97, 112, 97, 98, 105, 108, 105, 116, 121, 25, 99, 114, 101, 97, 116, 101, 95, 99,
            104, 105, 108, 100, 95, 118, 97, 115, 112, 95, 97, 99, 99, 111, 117, 110, 116, 27, 101,
            120, 116, 114, 97, 99, 116, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112,
            97, 98, 105, 108, 105, 116, 121, 8, 112, 97, 121, 95, 102, 114, 111, 109, 27, 114, 101,
            115, 116, 111, 114, 101, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112,
            97, 98, 105, 108, 105, 116, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 2,
            1, 0, 1, 1, 5, 3, 25, 10, 0, 10, 1, 11, 2, 10, 3, 56, 0, 10, 4, 6, 0, 0, 0, 0, 0, 0, 0,
            0, 36, 3, 10, 5, 22, 11, 0, 17, 1, 12, 5, 14, 5, 10, 1, 10, 4, 7, 0, 7, 0, 56, 1, 11,
            5, 17, 3, 5, 24, 11, 0, 1, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::Address(child_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::Bool(add_all_currencies),
            TransactionArgument::U64(child_initial_balance),
        ],
    )
}

/// Create an account with the DesignatedDealer role at `addr` with authentication key
/// `auth_key_prefix` | `addr` and a 0 balance of type `Currency`. If `add_all_currencies`
/// is true, 0 balances for all available currencies in the system will also be added.
/// This can only be invoked by an account with the TreasuryCompliance role.
pub fn encode_create_designated_dealer_script(
    currency: TypeTag,
    sliding_nonce: u64,
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    human_name: Vec<u8>,
    base_url: Vec<u8>,
    compliance_public_key: Vec<u8>,
    add_all_currencies: bool,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 3, 4, 11, 4, 15, 2, 5, 17, 35, 7, 52, 73, 8,
            125, 16, 0, 0, 0, 1, 1, 2, 0, 1, 0, 0, 3, 2, 1, 1, 1, 1, 4, 2, 6, 12, 3, 0, 7, 6, 12,
            5, 10, 2, 10, 2, 10, 2, 10, 2, 1, 8, 6, 12, 3, 5, 10, 2, 10, 2, 10, 2, 10, 2, 1, 1, 9,
            0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 12, 83, 108, 105, 100,
            105, 110, 103, 78, 111, 110, 99, 101, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111,
            110, 99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 24, 99, 114, 101, 97, 116, 101,
            95, 100, 101, 115, 105, 103, 110, 97, 116, 101, 100, 95, 100, 101, 97, 108, 101, 114,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 3, 1, 12, 10, 0, 10, 1, 17, 0,
            11, 0, 10, 2, 11, 3, 11, 4, 11, 5, 11, 6, 10, 7, 56, 0, 2,
        ],
        vec![currency],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::Address(addr),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U8Vector(human_name),
            TransactionArgument::U8Vector(base_url),
            TransactionArgument::U8Vector(compliance_public_key),
            TransactionArgument::Bool(add_all_currencies),
        ],
    )
}

/// Create an account with the ParentVASP role at `address` with authentication key
/// `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If
/// `add_all_currencies` is true, 0 balances for all available currencies in the system
/// will also be added. This can only be invoked by an Association account.
pub fn encode_create_parent_vasp_account_script(
    coin_type: TypeTag,
    new_account_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
    human_name: Vec<u8>,
    base_url: Vec<u8>,
    compliance_public_key: Vec<u8>,
    add_all_currencies: bool,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 17, 7, 27, 40, 8,
            67, 16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 7, 6, 12, 5, 10, 2, 10, 2, 10, 2, 10, 2, 1, 0, 1,
            9, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 26, 99, 114, 101, 97,
            116, 101, 95, 112, 97, 114, 101, 110, 116, 95, 118, 97, 115, 112, 95, 97, 99, 99, 111,
            117, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 9, 11, 0,
            10, 1, 11, 2, 11, 3, 11, 4, 11, 5, 10, 6, 56, 0, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::Address(new_account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U8Vector(human_name),
            TransactionArgument::U8Vector(base_url),
            TransactionArgument::U8Vector(compliance_public_key),
            TransactionArgument::Bool(add_all_currencies),
        ],
    )
}

/// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
/// `RecoveryAddress` resource under `account`. ## Aborts * Aborts with
/// `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if `account` has already
/// delegated its `KeyRotationCapability`. * Aborts with `RecoveryAddress::ENOT_A_VASP` if
/// `account` is not a ParentVASP or ChildVASP
pub fn encode_create_recovery_address_script() -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 2, 4, 4, 3, 8, 10, 5, 18, 12, 7, 30, 91, 8,
            121, 16, 0, 0, 0, 1, 0, 2, 1, 0, 0, 3, 0, 1, 0, 1, 4, 2, 3, 0, 1, 6, 12, 1, 8, 0, 2, 6,
            12, 8, 0, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 15, 82, 101, 99,
            111, 118, 101, 114, 121, 65, 100, 100, 114, 101, 115, 115, 21, 75, 101, 121, 82, 111,
            116, 97, 116, 105, 111, 110, 67, 97, 112, 97, 98, 105, 108, 105, 116, 121, 31, 101,
            120, 116, 114, 97, 99, 116, 95, 107, 101, 121, 95, 114, 111, 116, 97, 116, 105, 111,
            110, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 7, 112, 117, 98, 108, 105, 115,
            104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 3, 5, 10, 0, 11, 0, 17, 0,
            17, 1, 2,
        ],
        vec![],
        vec![],
    )
}

/// Create an account with the ParentVASP role at `address` with authentication key
/// `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If
/// `add_all_currencies` is true, 0 balances for all available currencies in the system
/// will also be added. This can only be invoked by an Association account. The
/// `human_name`, `base_url`, and compliance_public_key` fields of the ParentVASP are
/// filled in with dummy information.
pub fn encode_create_testing_account_script(
    coin_type: TypeTag,
    new_account_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
    add_all_currencies: bool,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 7, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 24, 7, 34, 40, 8,
            74, 16, 6, 90, 68, 0, 0, 0, 1, 0, 1, 1, 1, 0, 3, 7, 6, 12, 5, 10, 2, 10, 2, 10, 2, 10,
            2, 1, 0, 4, 6, 12, 5, 10, 2, 1, 1, 9, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111,
            117, 110, 116, 26, 99, 114, 101, 97, 116, 101, 95, 112, 97, 114, 101, 110, 116, 95,
            118, 97, 115, 112, 95, 97, 99, 99, 111, 117, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 10, 2, 8, 7, 116, 101, 115, 116, 110, 101, 116, 10, 2, 18, 17, 104, 116,
            116, 112, 115, 58, 47, 47, 108, 105, 98, 114, 97, 46, 111, 114, 103, 10, 2, 33, 32,
            183, 163, 193, 45, 192, 200, 199, 72, 171, 7, 82, 91, 112, 17, 34, 184, 139, 215, 143,
            96, 12, 118, 52, 45, 39, 242, 94, 95, 146, 68, 76, 222, 1, 1, 2, 1, 9, 11, 0, 10, 1,
            11, 2, 7, 0, 7, 1, 7, 2, 10, 3, 56, 0, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::Address(new_account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::Bool(add_all_currencies),
        ],
    )
}

/// Create a validator account at `new_validator_address` with `auth_key_prefix`.
pub fn encode_create_validator_account_script(
    new_account_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 7, 7, 14, 38, 8, 52, 16, 0, 0,
            0, 1, 0, 1, 0, 3, 6, 12, 5, 10, 2, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 24, 99, 114, 101, 97, 116, 101, 95, 118, 97, 108, 105, 100, 97, 116, 111,
            114, 95, 97, 99, 99, 111, 117, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 1, 5, 11, 0, 10, 1, 11, 2, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::Address(new_account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
        ],
    )
}

/// Create a validator operator account at `new_validator_address` with `auth_key_prefix`.
pub fn encode_create_validator_operator_account_script(
    new_account_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 7, 7, 14, 47, 8, 61, 16, 0, 0,
            0, 1, 0, 1, 0, 3, 6, 12, 5, 10, 2, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 33, 99, 114, 101, 97, 116, 101, 95, 118, 97, 108, 105, 100, 97, 116, 111,
            114, 95, 111, 112, 101, 114, 97, 116, 111, 114, 95, 97, 99, 99, 111, 117, 110, 116, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 5, 11, 0, 10, 1, 11, 2, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::Address(new_account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
        ],
    )
}

/// Freeze account `address`. Initiator must be authorized. `sliding_nonce` is a unique
/// nonce for operation, see sliding_nonce.move for details.
pub fn encode_freeze_account_script(
    sliding_nonce: u64,
    to_freeze_account: AccountAddress,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 4, 3, 4, 10, 5, 14, 14, 7, 28, 66, 8, 94, 16, 0,
            0, 0, 1, 0, 2, 0, 1, 0, 1, 3, 2, 1, 0, 2, 6, 12, 5, 0, 2, 6, 12, 3, 3, 6, 12, 3, 5, 15,
            65, 99, 99, 111, 117, 110, 116, 70, 114, 101, 101, 122, 105, 110, 103, 12, 83, 108,
            105, 100, 105, 110, 103, 78, 111, 110, 99, 101, 14, 102, 114, 101, 101, 122, 101, 95,
            97, 99, 99, 111, 117, 110, 116, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111, 110, 99,
            101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 3, 1, 7, 10, 0, 10, 1, 17, 1, 11, 0, 10, 2, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::Address(to_freeze_account),
        ],
    )
}

/// Mint `amount_lbr` LBR from the sending account's constituent coins and deposits the
/// resulting LBR into the sending account.
pub fn encode_mint_lbr_script(amount_lbr: u64) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 2, 2, 4, 3, 6, 15, 5, 21, 16, 7, 37, 99, 8,
            136, 1, 16, 0, 0, 0, 1, 1, 0, 0, 2, 0, 1, 0, 0, 3, 1, 2, 0, 0, 4, 3, 2, 0, 1, 6, 12, 1,
            8, 0, 0, 2, 6, 8, 0, 3, 2, 6, 12, 3, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 18, 87, 105, 116, 104, 100, 114, 97, 119, 67, 97, 112, 97, 98, 105, 108, 105,
            116, 121, 27, 101, 120, 116, 114, 97, 99, 116, 95, 119, 105, 116, 104, 100, 114, 97,
            119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 27, 114, 101, 115, 116, 111,
            114, 101, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108,
            105, 116, 121, 10, 115, 116, 97, 112, 108, 101, 95, 108, 98, 114, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 4, 1, 9, 11, 0, 17, 0, 12, 2, 14, 2, 10, 1, 17, 2, 11, 2,
            17, 1, 2,
        ],
        vec![],
        vec![TransactionArgument::U64(amount_lbr)],
    )
}

/// Modify publishing options. Takes the LCS bytes of a `VMPublishingOption` object as
/// input.
pub fn encode_modify_publishing_option_script(args: Vec<u8>) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 6, 7, 13, 36, 8, 49, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 10, 2, 0, 13, 76, 105, 98, 114, 97, 86, 77, 67, 111, 110, 102,
            105, 103, 21, 115, 101, 116, 95, 112, 117, 98, 108, 105, 115, 104, 105, 110, 103, 95,
            111, 112, 116, 105, 111, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
            4, 11, 0, 11, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::U8Vector(args)],
    )
}

/// Transfer `amount` coins of type `Currency` from `payer` to `payee` with (optional)
/// associated `metadata` and an (optional) `metadata_signature` on the message `metadata`
/// | `Signer::address_of(payer)` | `amount` | `DualAttestation::DOMAIN_SEPARATOR`. The
/// `metadata` and `metadata_signature` parameters are only required if `amount` >=
/// `DualAttestation::get_cur_microlibra_limit` LBR and `payer` and `payee` are distinct
/// entities (e.g., different VASPs, or a VASP and a DesignatedDealer). Standardized
/// `metadata` LCS format can be found in `libra_types::transaction::metadata::Metadata`.
/// ## Events When this script executes without aborting, it emits two events:
/// `SentPaymentEvent { amount, currency_code = Currency, payee, metadata }` on `payer`'s
/// `LibraAccount::sent_events` handle, and `ReceivedPaymentEvent { amount, currency_code
/// = Currency, payer, metadata }` on `payee`'s `LibraAccount::received_events` handle. ##
/// Common Aborts These aborts can in occur in any payment. * Aborts with
/// `LibraAccount::EINSUFFICIENT_BALANCE` if `amount` is greater than `payer`'s balance in
/// `Currency`. * Aborts with `LibraAccount::ECOIN_DEPOSIT_IS_ZERO` if `amount` is zero. *
/// Aborts with `LibraAccount::EPAYEE_DOES_NOT_EXIST` if no account exists at the address
/// `payee`. * Aborts with `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` if an account
/// exists at `payee`, but it does not accept payments in `Currency`. ## Dual Attestation
/// Aborts These aborts can occur in any payment subject to dual attestation. * Aborts
/// with `DualAttestation::EMALFORMED_METADATA_SIGNATURE` if `metadata_signature`'s is not
/// 64 bytes. * Aborts with `DualAttestation:EINVALID_METADATA_SIGNATURE` if
/// `metadata_signature` does not verify on the message `metadata` | `payer` | `value` |
/// `DOMAIN_SEPARATOR` using the `compliance_public_key` published in the `payee`'s
/// `DualAttestation::Credential` resource. ## Other Aborts These aborts should only
/// happen when `payer` or `payee` have account limit restrictions or have been frozen by
/// Libra administrators. * Aborts with `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS` if
/// `payer` has exceeded their daily withdrawal limits. * Aborts with
/// `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS` if `payee` has exceeded their daily deposit
/// limits. * Aborts with `LibraAccount::EACCOUNT_FROZEN` if `payer`'s account is frozen.
pub fn encode_peer_to_peer_with_metadata_script(
    currency: TypeTag,
    payee: AccountAddress,
    amount: u64,
    metadata: Vec<u8>,
    metadata_signature: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 7, 1, 0, 2, 2, 2, 4, 3, 6, 16, 4, 22, 2, 5, 24, 29, 7,
            53, 97, 8, 150, 1, 16, 0, 0, 0, 1, 1, 0, 0, 2, 0, 1, 0, 0, 3, 2, 3, 1, 1, 0, 4, 1, 3,
            0, 1, 5, 1, 6, 12, 1, 8, 0, 5, 6, 8, 0, 5, 3, 10, 2, 10, 2, 0, 5, 6, 12, 5, 3, 10, 2,
            10, 2, 1, 9, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 18, 87, 105,
            116, 104, 100, 114, 97, 119, 67, 97, 112, 97, 98, 105, 108, 105, 116, 121, 27, 101,
            120, 116, 114, 97, 99, 116, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112,
            97, 98, 105, 108, 105, 116, 121, 8, 112, 97, 121, 95, 102, 114, 111, 109, 27, 114, 101,
            115, 116, 111, 114, 101, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112,
            97, 98, 105, 108, 105, 116, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            4, 1, 12, 11, 0, 17, 0, 12, 5, 14, 5, 10, 1, 10, 2, 11, 3, 11, 4, 56, 0, 11, 5, 17, 2,
            2,
        ],
        vec![currency],
        vec![
            TransactionArgument::Address(payee),
            TransactionArgument::U64(amount),
            TransactionArgument::U8Vector(metadata),
            TransactionArgument::U8Vector(metadata_signature),
        ],
    )
}

/// Preburn `amount` `Token`s from `account`. This will only succeed if `account` already
/// has a published `Preburn<Token>` resource.
pub fn encode_preburn_script(token: TypeTag, amount: u64) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 7, 1, 0, 2, 2, 2, 4, 3, 6, 16, 4, 22, 2, 5, 24, 21, 7,
            45, 96, 8, 141, 1, 16, 0, 0, 0, 1, 1, 0, 0, 2, 0, 1, 0, 0, 3, 2, 3, 1, 1, 0, 4, 1, 3,
            0, 1, 5, 1, 6, 12, 1, 8, 0, 3, 6, 12, 6, 8, 0, 3, 0, 2, 6, 12, 3, 1, 9, 0, 12, 76, 105,
            98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 18, 87, 105, 116, 104, 100, 114, 97, 119,
            67, 97, 112, 97, 98, 105, 108, 105, 116, 121, 27, 101, 120, 116, 114, 97, 99, 116, 95,
            119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 7, 112, 114, 101, 98, 117, 114, 110, 27, 114, 101, 115, 116, 111, 114, 101, 95,
            119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 4, 1, 10, 10, 0, 17, 0, 12,
            2, 11, 0, 14, 2, 10, 1, 56, 0, 11, 2, 17, 2, 2,
        ],
        vec![token],
        vec![TransactionArgument::U64(amount)],
    )
}

/// Publishes an unrestricted `LimitsDefintion<CoinType>` under `account`. Will abort if a
/// resource with the same type already exists under `account`. No windows will point to
/// this limit at the time it is published.
pub fn encode_publish_account_limit_definition_script(coin_type: TypeTag) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 7, 7, 17, 42, 8, 59,
            16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 1, 6, 12, 0, 1, 9, 0, 13, 65, 99, 99, 111, 117, 110,
            116, 76, 105, 109, 105, 116, 115, 27, 112, 117, 98, 108, 105, 115, 104, 95, 117, 110,
            114, 101, 115, 116, 114, 105, 99, 116, 101, 100, 95, 108, 105, 109, 105, 116, 115, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 3, 11, 0, 56, 0, 2,
        ],
        vec![coin_type],
        vec![],
    )
}

/// (1) Rotate the authentication key of the sender to `public_key` (2) Publish a resource
/// containing a 32-byte ed25519 public key and the rotation capability of the sender
/// under the sender's address. Aborts if the sender already has a
/// `SharedEd25519PublicKey` resource. Aborts if the length of `new_public_key` is not 32.
pub fn encode_publish_shared_ed25519_public_key_script(public_key: Vec<u8>) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 6, 7, 13, 31, 8, 44, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 10, 2, 0, 22, 83, 104, 97, 114, 101, 100, 69, 100, 50, 53, 53,
            49, 57, 80, 117, 98, 108, 105, 99, 75, 101, 121, 7, 112, 117, 98, 108, 105, 115, 104,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11, 0, 11, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::U8Vector(public_key)],
    )
}

/// Removes a validator from the validator set. Fails if the validator_address is not in
/// the validator set. Emits a NewEpochEvent. TODO(valerini): rename to
/// remove_validator_and_reconfigure?
pub fn encode_remove_validator_script(validator_address: AccountAddress) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 5, 7, 12, 29, 8, 41, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 5, 0, 11, 76, 105, 98, 114, 97, 83, 121, 115, 116, 101, 109,
            16, 114, 101, 109, 111, 118, 101, 95, 118, 97, 108, 105, 100, 97, 116, 111, 114, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11, 0, 10, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::Address(validator_address)],
    )
}

/// Rotate the sender's authentication key to `new_key`. `new_key` should be a 256 bit
/// sha3 hash of an ed25519 public key. * Aborts with
/// `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if the
/// `KeyRotationCapability` for `account` has already been extracted. * Aborts with `0` if
/// the key rotation capability held by the account doesn't match the sender's address. *
/// Aborts with `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` if the length of `new_key`
/// != 32.
pub fn encode_rotate_authentication_key_script(new_key: Vec<u8>) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 2, 4, 4, 3, 8, 25, 5, 33, 32, 7, 65, 175, 1,
            8, 240, 1, 16, 0, 0, 0, 1, 0, 3, 1, 0, 1, 2, 0, 1, 0, 0, 4, 0, 2, 0, 0, 5, 3, 4, 0, 0,
            6, 2, 5, 0, 0, 7, 6, 5, 0, 1, 6, 12, 1, 5, 1, 8, 0, 1, 6, 8, 0, 1, 6, 5, 0, 2, 6, 8, 0,
            10, 2, 2, 6, 12, 10, 2, 3, 8, 0, 1, 3, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 6, 83, 105, 103, 110, 101, 114, 10, 97, 100, 100, 114, 101, 115, 115, 95,
            111, 102, 21, 75, 101, 121, 82, 111, 116, 97, 116, 105, 111, 110, 67, 97, 112, 97, 98,
            105, 108, 105, 116, 121, 31, 101, 120, 116, 114, 97, 99, 116, 95, 107, 101, 121, 95,
            114, 111, 116, 97, 116, 105, 111, 110, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 31, 107, 101, 121, 95, 114, 111, 116, 97, 116, 105, 111, 110, 95, 99, 97, 112, 97,
            98, 105, 108, 105, 116, 121, 95, 97, 100, 100, 114, 101, 115, 115, 31, 114, 101, 115,
            116, 111, 114, 101, 95, 107, 101, 121, 95, 114, 111, 116, 97, 116, 105, 111, 110, 95,
            99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 25, 114, 111, 116, 97, 116, 101, 95, 97,
            117, 116, 104, 101, 110, 116, 105, 99, 97, 116, 105, 111, 110, 95, 107, 101, 121, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 7, 8, 20, 10, 0, 17, 1, 12, 2, 14, 2, 17,
            2, 20, 11, 0, 17, 0, 33, 12, 3, 11, 3, 3, 14, 6, 0, 0, 0, 0, 0, 0, 0, 0, 39, 14, 2, 11,
            1, 17, 4, 11, 2, 17, 3, 2,
        ],
        vec![],
        vec![TransactionArgument::U8Vector(new_key)],
    )
}

/// Rotate the sender's authentication key to `new_key`. `new_key` should be a 256 bit
/// sha3 hash of an ed25519 public key. This script also takes `sliding_nonce`, as a
/// unique nonce for this operation. See sliding_nonce.move for details.
pub fn encode_rotate_authentication_key_with_nonce_script(
    sliding_nonce: u64,
    new_key: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 2, 4, 4, 3, 8, 20, 5, 28, 23, 7, 51, 160, 1,
            8, 211, 1, 16, 0, 0, 0, 1, 0, 3, 1, 0, 1, 2, 0, 1, 0, 0, 4, 2, 3, 0, 0, 5, 3, 1, 0, 0,
            6, 4, 1, 0, 2, 6, 12, 3, 0, 1, 6, 12, 1, 8, 0, 2, 6, 8, 0, 10, 2, 3, 6, 12, 3, 10, 2,
            12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 12, 83, 108, 105, 100, 105,
            110, 103, 78, 111, 110, 99, 101, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111, 110,
            99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 21, 75, 101, 121, 82, 111, 116, 97,
            116, 105, 111, 110, 67, 97, 112, 97, 98, 105, 108, 105, 116, 121, 31, 101, 120, 116,
            114, 97, 99, 116, 95, 107, 101, 121, 95, 114, 111, 116, 97, 116, 105, 111, 110, 95, 99,
            97, 112, 97, 98, 105, 108, 105, 116, 121, 31, 114, 101, 115, 116, 111, 114, 101, 95,
            107, 101, 121, 95, 114, 111, 116, 97, 116, 105, 111, 110, 95, 99, 97, 112, 97, 98, 105,
            108, 105, 116, 121, 25, 114, 111, 116, 97, 116, 101, 95, 97, 117, 116, 104, 101, 110,
            116, 105, 99, 97, 116, 105, 111, 110, 95, 107, 101, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 5, 3, 12, 10, 0, 10, 1, 17, 0, 11, 0, 17, 1, 12, 3, 14, 3, 11, 2,
            17, 3, 11, 3, 17, 2, 2,
        ],
        vec![],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::U8Vector(new_key),
        ],
    )
}

/// Rotate the authentication key of `account` to `new_key` using the
/// `KeyRotationCapability` stored under `recovery_address`. ## Aborts * Aborts with
/// `RecoveryAddress::ENOT_A_RECOVERY_ADDRESS` if `recovery_address` does not have a
/// `RecoveryAddress` resource * Aborts with `RecoveryAddress::ECANNOT_ROTATE_KEY` if
/// `account` is not `recovery_address` or `to_recover`. * Aborts with
/// `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` if `new_key` is not 32 bytes. * Aborts
/// with `RecoveryAddress::ECANNOT_ROTATE_KEY` if `account` has not delegated its
/// `KeyRotationCapability` to `recovery_address`.
pub fn encode_rotate_authentication_key_with_recovery_address_script(
    recovery_address: AccountAddress,
    to_recover: AccountAddress,
    new_key: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 8, 7, 15, 42, 8, 57, 16, 0, 0,
            0, 1, 0, 1, 0, 4, 6, 12, 5, 5, 10, 2, 0, 15, 82, 101, 99, 111, 118, 101, 114, 121, 65,
            100, 100, 114, 101, 115, 115, 25, 114, 111, 116, 97, 116, 101, 95, 97, 117, 116, 104,
            101, 110, 116, 105, 99, 97, 116, 105, 111, 110, 95, 107, 101, 121, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 6, 11, 0, 10, 1, 10, 2, 11, 3, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::Address(recovery_address),
            TransactionArgument::Address(to_recover),
            TransactionArgument::U8Vector(new_key),
        ],
    )
}

/// Rotate `account`'s base URL to `new_url` and its compliance public key to `new_key`.
/// Aborts if `account` is not a ParentVASP or DesignatedDealer Aborts if `new_key` is not
/// a well-formed public key
pub fn encode_rotate_dual_attestation_info_script(new_url: Vec<u8>, new_key: Vec<u8>) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 10, 5, 12, 13, 7, 25, 61, 8, 86, 16, 0,
            0, 0, 1, 0, 1, 0, 0, 2, 0, 1, 0, 2, 6, 12, 10, 2, 0, 3, 6, 12, 10, 2, 10, 2, 15, 68,
            117, 97, 108, 65, 116, 116, 101, 115, 116, 97, 116, 105, 111, 110, 15, 114, 111, 116,
            97, 116, 101, 95, 98, 97, 115, 101, 95, 117, 114, 108, 28, 114, 111, 116, 97, 116, 101,
            95, 99, 111, 109, 112, 108, 105, 97, 110, 99, 101, 95, 112, 117, 98, 108, 105, 99, 95,
            107, 101, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 2, 1, 7, 10, 0, 11,
            1, 17, 0, 11, 0, 11, 2, 17, 1, 2,
        ],
        vec![],
        vec![
            TransactionArgument::U8Vector(new_url),
            TransactionArgument::U8Vector(new_key),
        ],
    )
}

/// (1) Rotate the public key stored in `account`'s `SharedEd25519PublicKey` resource to
/// `new_public_key` (2) Rotate the authentication key using the capability stored in
/// `account`'s `SharedEd25519PublicKey` to a new value derived from `new_public_key`
/// Aborts if `account` does not have a `SharedEd25519PublicKey` resource. Aborts if the
/// length of `new_public_key` is not 32.
pub fn encode_rotate_shared_ed25519_public_key_script(public_key: Vec<u8>) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 6, 7, 13, 34, 8, 47, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 10, 2, 0, 22, 83, 104, 97, 114, 101, 100, 69, 100, 50, 53, 53,
            49, 57, 80, 117, 98, 108, 105, 99, 75, 101, 121, 10, 114, 111, 116, 97, 116, 101, 95,
            107, 101, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11, 0, 11,
            1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::U8Vector(public_key)],
    )
}

/// Set validator's config locally. Does not emit NewEpochEvent, the config is NOT changed
/// in the validator set. TODO(valerini): rename to register_validator_config to avoid
/// confusion with set_validator_config_and_reconfigure script.
pub fn encode_set_validator_config_script(
    validator_account: AccountAddress,
    consensus_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 15, 7, 22, 27, 8, 49, 16, 0,
            0, 0, 1, 0, 1, 0, 7, 6, 12, 5, 10, 2, 10, 2, 10, 2, 10, 2, 10, 2, 0, 15, 86, 97, 108,
            105, 100, 97, 116, 111, 114, 67, 111, 110, 102, 105, 103, 10, 115, 101, 116, 95, 99,
            111, 110, 102, 105, 103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 9,
            11, 0, 10, 1, 11, 2, 11, 3, 11, 4, 11, 5, 11, 6, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::Address(validator_account),
            TransactionArgument::U8Vector(consensus_pubkey),
            TransactionArgument::U8Vector(validator_network_identity_pubkey),
            TransactionArgument::U8Vector(validator_network_address),
            TransactionArgument::U8Vector(fullnodes_network_identity_pubkey),
            TransactionArgument::U8Vector(fullnodes_network_address),
        ],
    )
}

/// Set validator's config and updates the config in the validator set. NewEpochEvent is
/// emitted.
pub fn encode_set_validator_config_and_reconfigure_script(
    validator_account: AccountAddress,
    consensus_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 4, 3, 4, 10, 5, 14, 19, 7, 33, 69, 8, 102, 16,
            0, 0, 0, 1, 1, 2, 0, 1, 0, 0, 3, 2, 1, 0, 7, 6, 12, 5, 10, 2, 10, 2, 10, 2, 10, 2, 10,
            2, 0, 2, 6, 12, 5, 11, 76, 105, 98, 114, 97, 83, 121, 115, 116, 101, 109, 15, 86, 97,
            108, 105, 100, 97, 116, 111, 114, 67, 111, 110, 102, 105, 103, 10, 115, 101, 116, 95,
            99, 111, 110, 102, 105, 103, 29, 117, 112, 100, 97, 116, 101, 95, 99, 111, 110, 102,
            105, 103, 95, 97, 110, 100, 95, 114, 101, 99, 111, 110, 102, 105, 103, 117, 114, 101,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 12, 10, 0, 10, 1, 11, 2, 11,
            3, 11, 4, 11, 5, 11, 6, 17, 0, 11, 0, 10, 1, 17, 1, 2,
        ],
        vec![],
        vec![
            TransactionArgument::Address(validator_account),
            TransactionArgument::U8Vector(consensus_pubkey),
            TransactionArgument::U8Vector(validator_network_identity_pubkey),
            TransactionArgument::U8Vector(validator_network_address),
            TransactionArgument::U8Vector(fullnodes_network_identity_pubkey),
            TransactionArgument::U8Vector(fullnodes_network_address),
        ],
    )
}

/// Set validator's operator
pub fn encode_set_validator_operator_script(operator_account: AccountAddress) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 5, 7, 12, 29, 8, 41, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 5, 0, 15, 86, 97, 108, 105, 100, 97, 116, 111, 114, 67, 111,
            110, 102, 105, 103, 12, 115, 101, 116, 95, 111, 112, 101, 114, 97, 116, 111, 114, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11, 0, 10, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::Address(operator_account)],
    )
}

/// Send `amount` coins of type `Token` to `payee`.
pub fn encode_testnet_mint_script(token: TypeTag, payee: AccountAddress, amount: u64) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 8, 1, 0, 8, 2, 8, 4, 3, 12, 37, 4, 49, 4, 5, 53, 40, 7,
            93, 193, 1, 8, 158, 2, 16, 6, 174, 2, 22, 0, 0, 0, 1, 0, 2, 0, 3, 2, 7, 1, 0, 0, 4, 0,
            1, 0, 3, 5, 2, 3, 0, 1, 6, 1, 1, 1, 1, 2, 8, 3, 4, 0, 2, 9, 2, 5, 0, 2, 10, 6, 0, 1, 1,
            2, 11, 5, 0, 0, 2, 9, 5, 9, 0, 1, 3, 1, 6, 12, 1, 5, 1, 1, 1, 8, 0, 5, 6, 8, 0, 5, 3,
            10, 2, 10, 2, 3, 6, 12, 5, 3, 7, 8, 0, 1, 3, 1, 3, 1, 3, 1, 9, 0, 15, 68, 117, 97, 108,
            65, 116, 116, 101, 115, 116, 97, 116, 105, 111, 110, 5, 76, 105, 98, 114, 97, 12, 76,
            105, 98, 114, 97, 65, 99, 99, 111, 117, 110, 116, 6, 83, 105, 103, 110, 101, 114, 24,
            103, 101, 116, 95, 99, 117, 114, 95, 109, 105, 99, 114, 111, 108, 105, 98, 114, 97, 95,
            108, 105, 109, 105, 116, 10, 97, 100, 100, 114, 101, 115, 115, 95, 111, 102, 20, 97,
            112, 112, 114, 111, 120, 95, 108, 98, 114, 95, 102, 111, 114, 95, 118, 97, 108, 117,
            101, 18, 87, 105, 116, 104, 100, 114, 97, 119, 67, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 9, 101, 120, 105, 115, 116, 115, 95, 97, 116, 27, 101, 120, 116, 114, 97, 99, 116,
            95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 8, 112, 97, 121, 95, 102, 114, 111, 109, 27, 114, 101, 115, 116, 111, 114, 101,
            95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116,
            121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 5, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 221, 10, 2, 1, 0, 1, 1, 7, 8, 43, 10, 1, 17, 3, 12, 4, 11, 4, 3, 9,
            11, 0, 1, 6, 203, 21, 122, 0, 0, 0, 0, 0, 39, 10, 0, 17, 1, 7, 0, 33, 12, 6, 11, 6, 3,
            20, 11, 0, 1, 6, 204, 21, 122, 0, 0, 0, 0, 0, 39, 10, 2, 56, 0, 17, 0, 35, 12, 8, 11,
            8, 3, 31, 11, 0, 1, 6, 205, 21, 122, 0, 0, 0, 0, 0, 39, 11, 0, 17, 4, 12, 3, 14, 3, 10,
            1, 10, 2, 7, 1, 7, 1, 56, 1, 11, 3, 17, 6, 2,
        ],
        vec![token],
        vec![
            TransactionArgument::Address(payee),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier. Max valid
/// tier index is 3 since there are max 4 tiers per DD. Sender should be treasury
/// compliance account and receiver authorized DD. `sliding_nonce` is a unique nonce for
/// operation, see sliding_nonce.move for details.
pub fn encode_tiered_mint_script(
    coin_type: TypeTag,
    sliding_nonce: u64,
    designated_dealer_address: AccountAddress,
    mint_amount: u64,
    tier_index: u64,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 3, 4, 11, 4, 15, 2, 5, 17, 21, 7, 38, 60, 8,
            98, 16, 0, 0, 0, 1, 1, 2, 0, 1, 0, 0, 3, 2, 1, 1, 1, 1, 4, 2, 6, 12, 3, 0, 4, 6, 12, 5,
            3, 3, 5, 6, 12, 3, 5, 3, 3, 1, 9, 0, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 12, 83, 108, 105, 100, 105, 110, 103, 78, 111, 110, 99, 101, 21, 114, 101,
            99, 111, 114, 100, 95, 110, 111, 110, 99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116,
            11, 116, 105, 101, 114, 101, 100, 95, 109, 105, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 1, 1, 3, 1, 9, 10, 0, 10, 1, 17, 0, 11, 0, 10, 2, 10, 3, 10, 4, 56,
            0, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::Address(designated_dealer_address),
            TransactionArgument::U64(mint_amount),
            TransactionArgument::U64(tier_index),
        ],
    )
}

/// Unfreeze account `address`. Initiator must be authorized. `sliding_nonce` is a unique
/// nonce for operation, see sliding_nonce.move for details.
pub fn encode_unfreeze_account_script(
    sliding_nonce: u64,
    to_unfreeze_account: AccountAddress,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 4, 3, 4, 10, 5, 14, 14, 7, 28, 68, 8, 96, 16, 0,
            0, 0, 1, 0, 2, 0, 1, 0, 1, 3, 2, 1, 0, 2, 6, 12, 5, 0, 2, 6, 12, 3, 3, 6, 12, 3, 5, 15,
            65, 99, 99, 111, 117, 110, 116, 70, 114, 101, 101, 122, 105, 110, 103, 12, 83, 108,
            105, 100, 105, 110, 103, 78, 111, 110, 99, 101, 16, 117, 110, 102, 114, 101, 101, 122,
            101, 95, 97, 99, 99, 111, 117, 110, 116, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111,
            110, 99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 3, 1, 7, 10, 0, 10, 1, 17, 1, 11, 0, 10, 2, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::Address(to_unfreeze_account),
        ],
    )
}

/// Unmints `amount_lbr` LBR from the sending account into the constituent coins and
/// deposits the resulting coins into the sending account."
pub fn encode_unmint_lbr_script(amount_lbr: u64) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 2, 2, 4, 3, 6, 15, 5, 21, 16, 7, 37, 101, 8,
            138, 1, 16, 0, 0, 0, 1, 1, 0, 0, 2, 0, 1, 0, 0, 3, 1, 2, 0, 0, 4, 3, 2, 0, 1, 6, 12, 1,
            8, 0, 0, 2, 6, 8, 0, 3, 2, 6, 12, 3, 12, 76, 105, 98, 114, 97, 65, 99, 99, 111, 117,
            110, 116, 18, 87, 105, 116, 104, 100, 114, 97, 119, 67, 97, 112, 97, 98, 105, 108, 105,
            116, 121, 27, 101, 120, 116, 114, 97, 99, 116, 95, 119, 105, 116, 104, 100, 114, 97,
            119, 95, 99, 97, 112, 97, 98, 105, 108, 105, 116, 121, 27, 114, 101, 115, 116, 111,
            114, 101, 95, 119, 105, 116, 104, 100, 114, 97, 119, 95, 99, 97, 112, 97, 98, 105, 108,
            105, 116, 121, 12, 117, 110, 115, 116, 97, 112, 108, 101, 95, 108, 98, 114, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 4, 1, 9, 11, 0, 17, 0, 12, 2, 14, 2, 10, 1, 17,
            2, 11, 2, 17, 1, 2,
        ],
        vec![],
        vec![TransactionArgument::U64(amount_lbr)],
    )
}

/// Optionally update thresholds of max balance, inflow, outflow for any limits-bound
/// accounts with their limits defined at `limit_address`. Limits are defined in terms of
/// base (on-chain) currency units for `CoinType`. If a new threshold is 0, that
/// particular config does not get updated. `sliding_nonce` is a unique nonce for
/// operation, see SlidingNonce.move for details.
pub fn encode_update_account_limit_definition_script(
    coin_type: TypeTag,
    limit_address: AccountAddress,
    sliding_nonce: u64,
    new_max_inflow: u64,
    new_max_outflow: u64,
    new_max_holding_balance: u64,
    new_time_period: u64,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 4, 3, 4, 11, 4, 15, 2, 5, 17, 25, 7, 42, 74, 8,
            116, 16, 0, 0, 0, 1, 0, 2, 0, 1, 1, 1, 1, 3, 2, 1, 0, 0, 4, 6, 6, 12, 5, 3, 3, 3, 3, 0,
            2, 6, 12, 3, 7, 6, 12, 5, 3, 3, 3, 3, 3, 1, 9, 0, 13, 65, 99, 99, 111, 117, 110, 116,
            76, 105, 109, 105, 116, 115, 12, 83, 108, 105, 100, 105, 110, 103, 78, 111, 110, 99,
            101, 24, 117, 112, 100, 97, 116, 101, 95, 108, 105, 109, 105, 116, 115, 95, 100, 101,
            102, 105, 110, 105, 116, 105, 111, 110, 21, 114, 101, 99, 111, 114, 100, 95, 110, 111,
            110, 99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 3, 1, 11, 10, 0, 10, 2, 17, 1, 11, 0, 10, 1, 10, 3, 10, 4, 10, 5,
            10, 6, 56, 0, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::Address(limit_address),
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::U64(new_max_inflow),
            TransactionArgument::U64(new_max_outflow),
            TransactionArgument::U64(new_max_holding_balance),
            TransactionArgument::U64(new_time_period),
        ],
    )
}

/// * Sets the account limits window `tracking_balance` field for `CoinType` at
/// `window_address` to `aggregate_balance` if `aggregate_balance != 0`. * Sets the
/// account limits window `limit_address` field for `CoinType` at `window_address` to
/// `new_limit_address`.
pub fn encode_update_account_limit_window_info_script(
    coin_type: TypeTag,
    window_address: AccountAddress,
    aggregate_balance: u64,
    new_limit_address: AccountAddress,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 10, 7, 20, 33, 8,
            53, 16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 4, 6, 12, 5, 3, 5, 0, 1, 9, 0, 13, 65, 99, 99,
            111, 117, 110, 116, 76, 105, 109, 105, 116, 115, 18, 117, 112, 100, 97, 116, 101, 95,
            119, 105, 110, 100, 111, 119, 95, 105, 110, 102, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 0, 1, 6, 11, 0, 10, 1, 10, 2, 10, 3, 56, 0, 2,
        ],
        vec![coin_type],
        vec![
            TransactionArgument::Address(window_address),
            TransactionArgument::U64(aggregate_balance),
            TransactionArgument::Address(new_limit_address),
        ],
    )
}

/// Update the dual attesation limit to `new_micro_lbr_limit`.
pub fn encode_update_dual_attestation_limit_script(
    sliding_nonce: u64,
    new_micro_lbr_limit: u64,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 4, 3, 4, 10, 5, 14, 10, 7, 24, 72, 8, 96, 16, 0,
            0, 0, 1, 0, 2, 0, 1, 0, 1, 3, 0, 1, 0, 2, 6, 12, 3, 0, 3, 6, 12, 3, 3, 15, 68, 117, 97,
            108, 65, 116, 116, 101, 115, 116, 97, 116, 105, 111, 110, 12, 83, 108, 105, 100, 105,
            110, 103, 78, 111, 110, 99, 101, 20, 115, 101, 116, 95, 109, 105, 99, 114, 111, 108,
            105, 98, 114, 97, 95, 108, 105, 109, 105, 116, 21, 114, 101, 99, 111, 114, 100, 95,
            110, 111, 110, 99, 101, 95, 111, 114, 95, 97, 98, 111, 114, 116, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 2, 1, 7, 10, 0, 10, 1, 17, 1, 11, 0, 10, 2, 17, 0, 2,
        ],
        vec![],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::U64(new_micro_lbr_limit),
        ],
    )
}

/// Update the on-chain exchange rate to LBR for the given `currency` to be given by
/// `new_exchange_rate_numerator/new_exchange_rate_denominator`.
pub fn encode_update_exchange_rate_script(
    currency: TypeTag,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 7, 1, 0, 6, 2, 6, 4, 3, 10, 16, 4, 26, 2, 5, 28, 25, 7,
            53, 100, 8, 153, 1, 16, 0, 0, 0, 1, 0, 2, 0, 0, 2, 0, 0, 3, 0, 1, 0, 2, 4, 2, 3, 0, 1,
            5, 4, 3, 1, 1, 2, 6, 2, 3, 3, 1, 8, 0, 2, 6, 12, 3, 0, 2, 6, 12, 8, 0, 4, 6, 12, 3, 3,
            3, 1, 9, 0, 12, 70, 105, 120, 101, 100, 80, 111, 105, 110, 116, 51, 50, 5, 76, 105, 98,
            114, 97, 12, 83, 108, 105, 100, 105, 110, 103, 78, 111, 110, 99, 101, 20, 99, 114, 101,
            97, 116, 101, 95, 102, 114, 111, 109, 95, 114, 97, 116, 105, 111, 110, 97, 108, 21,
            114, 101, 99, 111, 114, 100, 95, 110, 111, 110, 99, 101, 95, 111, 114, 95, 97, 98, 111,
            114, 116, 24, 117, 112, 100, 97, 116, 101, 95, 108, 98, 114, 95, 101, 120, 99, 104, 97,
            110, 103, 101, 95, 114, 97, 116, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 5, 1, 11, 10, 0, 10, 1, 17, 1, 10, 2, 10, 3, 17, 0, 12, 4, 11, 0, 11, 4, 56, 0,
            2,
        ],
        vec![currency],
        vec![
            TransactionArgument::U64(sliding_nonce),
            TransactionArgument::U64(new_exchange_rate_numerator),
            TransactionArgument::U64(new_exchange_rate_denominator),
        ],
    )
}

/// Update Libra version.
pub fn encode_update_libra_version_script(major: u64) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 5, 1, 0, 2, 3, 2, 5, 5, 7, 5, 7, 12, 17, 8, 29, 16, 0, 0,
            0, 1, 0, 1, 0, 2, 6, 12, 3, 0, 12, 76, 105, 98, 114, 97, 86, 101, 114, 115, 105, 111,
            110, 3, 115, 101, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 4, 11,
            0, 10, 1, 17, 0, 2,
        ],
        vec![],
        vec![TransactionArgument::U64(major)],
    )
}

/// Allows--true--or disallows--false--minting of `currency` based upon `allow_minting`.
pub fn encode_update_minting_ability_script(currency: TypeTag, allow_minting: bool) -> Script {
    Script::new(
        vec![
            161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 6, 4, 8, 2, 5, 10, 8, 7, 18, 29, 8, 47,
            16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 2, 6, 12, 1, 0, 1, 9, 0, 5, 76, 105, 98, 114, 97, 22,
            117, 112, 100, 97, 116, 101, 95, 109, 105, 110, 116, 105, 110, 103, 95, 97, 98, 105,
            108, 105, 116, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 4, 11,
            0, 10, 1, 56, 0, 2,
        ],
        vec![currency],
        vec![TransactionArgument::Bool(allow_minting)],
    )
}
