script {
use 0x1::LibraSystem;
use 0x1::SlidingNonce;
use 0x1::ValidatorConfig;

/// # Summary
/// Adds a validator account to the validator set, and triggers a
/// reconfiguration of the system to admit the account to the validator set for the system. This
/// transaction can only be successfully called by the Libra Root account.
///
/// # Technical Description
/// This script adds the account at `validator_address` to the validator set.
/// This transaction emits a `LibraConfig::NewEpochEvent` event and triggers a
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
/// | `lr_account`        | `&signer`    | The signer reference of the sending account of this transaction. Must be the Libra Root signer.                                    |
/// | `sliding_nonce`     | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                         |
/// | `validator_name`    | `vector<u8>` | ASCII-encoded human name for the validator. Must match the human name in the `ValidatorConfig::ValidatorConfig` for the validator. |
/// | `validator_address` | `address`    | The validator account address to be added to the validator set.                                                                    |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                  | Description                                                                                                                               |
/// | ----------------           | --------------                                | -------------                                                                                                                             |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ELIBRA_ROOT`                  | The sending account is not the Libra Root account.                                                                                        |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.                                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                                                                             |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                                                                         |
/// | 0                          | 0                                             | The provided `validator_name` does not match the already-recorded human name for the validator.                                           |
/// | `Errors::INVALID_ARGUMENT` | `LibraSystem::EINVALID_PROSPECTIVE_VALIDATOR` | The validator to be added does not have a `ValidatorConfig::ValidatorConfig` resource published under it, or its `config` field is empty. |
/// | `Errors::INVALID_ARGUMENT` | `LibraSystem::EALREADY_A_VALIDATOR`           | The `validator_address` account is already a registered validator.                                                                        |
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
    lr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector<u8>,
    validator_address: address
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    assert(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
    LibraSystem::add_validator(lr_account, validator_address);
}
}
