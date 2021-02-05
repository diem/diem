script {
use 0x1::DiemSystem;
use 0x1::SlidingNonce;
use 0x1::ValidatorConfig;

/// # Summary
/// Adds a validator account to the validator set, and triggers a
/// reconfiguration of the system to admit the account to the validator set for the system. This
/// transaction can only be successfully called by the Diem Root account.
///
/// # Technical Description
/// This script adds the account at `validator_address` to the validator set.
/// This transaction emits a `DiemConfig::NewEpochEvent` event and triggers a
/// reconfiguration. Once the reconfiguration triggered by this script's
/// execution has been performed, the account at the `validator_address` is
/// considered to be a validator in the network.
///
/// This transaction script will fail if the `validator_address` address is already in the validator set
/// or does not have a `ValidatorConfig::ValidatorConfig` resource already published under it.
///
/// # Parameters
/// | Name                | Type         | Description                                                                                                                        |
/// | ------              | ------       | -------------                                                                                                                      |
/// | `dr_account`        | `&signer`    | The signer reference of the sending account of this transaction. Must be the Diem Root signer.                                    |
/// | `sliding_nonce`     | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                         |
/// | `validator_name`    | `vector<u8>` | ASCII-encoded human name for the validator. Must match the human name in the `ValidatorConfig::ValidatorConfig` for the validator. |
/// | `validator_address` | `address`    | The validator account address to be added to the validator set.                                                                    |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                  | Description                                                                                                                               |
/// | ----------------           | --------------                                | -------------                                                                                                                             |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                | A `SlidingNonce` resource is not published under `dr_account`.                                                                            |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.                                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                                                                             |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                                                                         |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                  | The sending account is not the Diem Root account.                                                                                        |
/// | `Errors::REQUIRES_ROLE`    | `Roles::EDIEM_ROOT`                          | The sending account is not the Diem Root account.                                                                                        |
/// | 0                          | 0                                             | The provided `validator_name` does not match the already-recorded human name for the validator.                                           |
/// | `Errors::INVALID_ARGUMENT` | `DiemSystem::EINVALID_PROSPECTIVE_VALIDATOR` | The validator to be added does not have a `ValidatorConfig::ValidatorConfig` resource published under it, or its `config` field is empty. |
/// | `Errors::INVALID_ARGUMENT` | `DiemSystem::EALREADY_A_VALIDATOR`           | The `validator_address` account is already a registered validator.                                                                        |
/// | `Errors::INVALID_STATE`    | `DiemConfig::EINVALID_BLOCK_TIME`            | An invalid time value was encountered in reconfiguration. Unlikely to occur.                                                              |
///
/// # Related Scripts
/// * `Script::create_validator_account`
/// * `Script::create_validator_operator_account`
/// * `Script::register_validator_config`
/// * `Script::remove_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::set_validator_config_and_reconfigure`

fun add_validator_and_reconfigure(
    dr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector<u8>,
    validator_address: address
) {
    SlidingNonce::record_nonce_or_abort(dr_account, sliding_nonce);
    assert(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
    DiemSystem::add_validator(dr_account, validator_address);
}


spec fun add_validator_and_reconfigure {
    use 0x1::DiemAccount;
    use 0x1::Errors;
    use 0x1::Roles;

    include DiemAccount::TransactionChecks{sender: dr_account}; // properties checked by the prologue.
    include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: dr_account};
    // next is due to abort in get_human_name
    include ValidatorConfig::AbortsIfNoValidatorConfig{addr: validator_address};
    // TODO: use an error code from Errors.move instead of 0.
    aborts_if ValidatorConfig::get_human_name(validator_address) != validator_name with 0;
    include DiemSystem::AddValidatorAbortsIf{validator_addr: validator_address};
    include DiemSystem::AddValidatorEnsures{validator_addr: validator_address};

    /// Reports INVALID_STATE because of is_operating() and !exists<DiemSystem::CapabilityHolder>.
    /// is_operating() is always true during transactions, and CapabilityHolder is published
    /// during initialization (Genesis).
    /// Reports REQUIRES_ROLE if dr_account is not Diem root, but that can't happen
    /// in practice because it aborts with NOT_PUBLISHED or REQUIRES_ADDRESS, first.
    aborts_with [check]
        0, // Odd error code in assert on second statement in add_validator_and_reconfigure
        Errors::INVALID_ARGUMENT,
        Errors::NOT_PUBLISHED,
        Errors::REQUIRES_ADDRESS,
        Errors::INVALID_STATE,
        Errors::REQUIRES_ROLE;

    /// **Access Control:**
    /// Only the Diem Root account can add Validators [[H13]][PERMISSION].
    include Roles::AbortsIfNotDiemRoot{account: dr_account};
}
}
