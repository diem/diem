script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;
use 0x1::Errors;

/// # Summary
/// Creates a Validator Operator account. This transaction can only be sent by the Libra
/// Root account.
///
/// # Technical Description
/// Creates an account with a Validator Operator role at `new_account_address`, with authentication key
/// `auth_key_prefix` | `new_account_address`. It publishes a
/// `ValidatorOperatorConfig::ValidatorOperatorConfig` resource with the specified `human_name`.
/// This script does not assign the validator operator to any validator accounts but only creates the account.
///
/// # Parameters
/// | Name                  | Type         | Description                                                                                     |
/// | ------                | ------       | -------------                                                                                   |
/// | `lr_account`          | `&signer`    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
/// | `sliding_nonce`       | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                      |
/// | `new_account_address` | `address`    | Address of the to-be-created Validator account.                                                 |
/// | `auth_key_prefix`     | `vector<u8>` | The authentication key prefix that will be used initially for the newly created account.        |
/// | `human_name`          | `vector<u8>` | ASCII-encoded human name for the validator.                                                     |
///
/// # Common Abort Conditions
/// | Error Category | Error Reason | Description |
/// |----------------|--------------|-------------|
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::NOT_PUBLISHED`     | `SlidingNonce::ESLIDING_NONCE`          | The sending account is not the Libra Root account or Treasury Compliance account           |
/// | `Errors::REQUIRES_ADDRESS`  | `CoreAddresses::ELIBRA_ROOT`            | The sending account is not the Libra Root account.                                         |
/// | `Errors::ALREADY_PUBLISHED` | `Roles::EROLE_ID`                       | The `new_account_address` address is already taken.                                        |
///
/// # Related Scripts
/// * `Script::create_validator_account`
/// * `Script::add_validator_and_reconfigure`
/// * `Script::register_validator_config`
/// * `Script::remove_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::set_validator_config_and_reconfigure`

fun create_validator_operator_account(
    lr_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    LibraAccount::create_validator_operator_account(
        lr_account,
        new_account_address,
        auth_key_prefix,
        human_name,
    );
}

/// Only Libra root may create Validator Operator accounts
/// Authentication: ValidatorAccountAbortsIf includes AbortsIfNotLibraRoot.
/// Checks that above table includes all error categories.
/// The verifier find aborts that are not documented, and cannot occur in practice:
/// * INVALID_STATE comes from `assert_operating()` in `ValidatorOperatorConfig::publish`, which should
///   always be true during a transaction.
/// * REQUIRES_ROLE comes from `Roles::assert_libra_root`. However, assert_libra_root checks the literal
///   Libra root address before checking the role, and the role abort is unreachable in practice, since
///   only Libra root has the Libra root role.
spec fun create_validator_operator_account {
    aborts_with [check] Errors::INVALID_ARGUMENT, Errors::NOT_PUBLISHED, Errors::REQUIRES_ADDRESS, Errors::ALREADY_PUBLISHED, Errors::INVALID_STATE, Errors::REQUIRES_ROLE;
    include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: lr_account};
    include LibraAccount::CreateValidatorOperatorAccountAbortsIf;
    include LibraAccount::CreateValidatorOperatorAccountEnsures;
    }

}
