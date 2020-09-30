script {
use 0x1::SlidingNonce;
use 0x1::ValidatorConfig;
use 0x1::ValidatorOperatorConfig;

/// # Summary
/// Sets the validator operator for a validator in the validator's configuration resource "locally"
/// and does not reconfigure the system. Changes from this transaction will not picked up by the
/// system until a reconfiguration of the system is triggered. May only be sent by the Libra Root
/// account as a write set transaction.
///
/// # Technical Description
/// Sets the account at `operator_account` address and with the specified `human_name` as an
/// operator for the validator `account`. The account at `operator_account` address must have a
/// Validator Operator role and have a `ValidatorOperatorConfig::ValidatorOperatorConfig` resource
/// published under it. The account represented by the `account` signer must be a Validator and
/// have a `ValidatorConfig::ValidatorConfig` resource published under it. No reconfiguration of
/// the system is initiated by this script.
///
/// # Parameters
/// | Name               | Type         | Description                                                                                                  |
/// | ------             | ------       | -------------                                                                                                |
/// | `lr_account`       | `&signer`    | The signer reference of the sending account of the write set transaction. May only be the Libra Root signer. |
/// | `account`          | `&signer`    | Signer reference of account specified in the `execute_as` field of the write set transaction.                |
/// | `sliding_nonce`    | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction for Libra Root.                    |
/// | `operator_name`    | `vector<u8>` | Validator operator's human name.                                                                             |
/// | `operator_account` | `address`    | Address of the validator operator account to be added as the `account` validator's operator.                 |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                          | Description                                                                                                                                                  |
/// | ----------------           | --------------                                        | -------------                                                                                                                                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                        | The `sliding_nonce` in `lr_account` is too old and it's impossible to determine if it's duplicated or not.                                                   |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                        | The `sliding_nonce` in `lr_account` is too far in the future.                                                                                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`               | The `sliding_nonce` in` lr_account` has been previously recorded.                                                                                            |
/// | `Errors::NOT_PUBLISHED`    | `ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG` | The `ValidatorOperatorConfig::ValidatorOperatorConfig` resource is not published under `operator_account`.                                                   |
/// | 0                          | 0                                                     | The `human_name` field of the `ValidatorOperatorConfig::ValidatorOperatorConfig` resource under `operator_account` does not match the provided `human_name`. |
/// | `Errors::REQUIRES_ROLE`    | `Roles::EVALIDATOR`                                   | `account` does not have a Validator account role.                                                                                                            |
/// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR`          | The account at `operator_account` does not have a `ValidatorOperatorConfig::ValidatorOperatorConfig` resource.                                               |
/// | `Errors::NOT_PUBLISHED`    | `ValidatorConfig::EVALIDATOR_CONFIG`                  | A `ValidatorConfig::ValidatorConfig` is not published under `account`.                                                                                       |
///
/// # Related Scripts
/// * `Script::create_validator_account`
/// * `Script::create_validator_operator_account`
/// * `Script::register_validator_config`
/// * `Script::remove_validator_and_reconfigure`
/// * `Script::add_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_config_and_reconfigure`

fun set_validator_operator_with_nonce_admin(
    lr_account: &signer,
    account: &signer,
    sliding_nonce: u64,
    operator_name: vector<u8>,
    operator_account: address
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    assert(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
    ValidatorConfig::set_operator(account, operator_account);
}
}
