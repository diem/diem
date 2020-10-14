script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// # Summary
/// Creates a Validator account. This transaction can only be sent by the Libra
/// Root account.
///
/// # Technical Description
/// Creates an account with a Validator role at `new_account_address`, with authentication key
/// `auth_key_prefix` | `new_account_address`. It publishes a
/// `ValidatorConfig::ValidatorConfig` resource with empty `config`, and
/// `operator_account` fields. The `human_name` field of the
/// `ValidatorConfig::ValidatorConfig` is set to the passed in `human_name`.
/// This script does not add the validator to the validator set or the system,
/// but only creates the account.
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
/// | Error Category              | Error Reason                            | Description                                                                                |
/// | ----------------            | --------------                          | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`     | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `lr_account`.                             |
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT`  | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS`  | `CoreAddresses::ELIBRA_ROOT`            | The sending account is not the Libra Root account.                                         |
/// | `Errors::REQUIRES_ROLE`     | `Roles::ELIBRA_ROOT`                    | The sending account is not the Libra Root account.                                         |
/// | `Errors::ALREADY_PUBLISHED` | `Roles::EROLE_ID`                       | The `new_account_address` address is already taken.                                        |
///
/// # Related Scripts
/// * `Script::add_validator_and_reconfigure`
/// * `Script::create_validator_operator_account`
/// * `Script::register_validator_config`
/// * `Script::remove_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::set_validator_config_and_reconfigure`

fun create_validator_account(
    lr_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>,
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    LibraAccount::create_validator_account(
        lr_account,
        new_account_address,
        auth_key_prefix,
        human_name,
    );
  }


/// Only Libra root may create Validator accounts
/// Authentication: ValidatorAccountAbortsIf includes AbortsIfNotLibraRoot.
/// Checks that above table includes all error categories.
/// The verifier finds an abort that is not documented, and cannot occur in practice:
/// * REQUIRES_ROLE comes from `Roles::assert_libra_root`. However, assert_libra_root checks the literal
///   Libra root address before checking the role, and the role abort is unreachable in practice, since
///   only Libra root has the Libra root role.
spec fun create_validator_account {
    use 0x1::Errors;
    use 0x1::Roles;

    include LibraAccount::TransactionChecks{sender: lr_account}; // properties checked by the prologue.
    include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: lr_account};
    include LibraAccount::CreateValidatorAccountAbortsIf;
    include LibraAccount::CreateValidatorAccountEnsures;

    aborts_with [check]
        Errors::INVALID_ARGUMENT,
        Errors::NOT_PUBLISHED,
        Errors::REQUIRES_ADDRESS,
        Errors::ALREADY_PUBLISHED,
        Errors::REQUIRES_ROLE;

    /// **Access Control:**
    /// Only the Libra Root account can create Validator accounts [[A3]][ROLE].
    include Roles::AbortsIfNotLibraRoot{account: lr_account};
}
}
