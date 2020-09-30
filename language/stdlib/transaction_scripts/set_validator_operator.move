script {
use 0x1::ValidatorConfig;
use 0x1::ValidatorOperatorConfig;

/// # Summary
/// Sets the validator operator for a validator in the validator's configuration resource "locally"
/// and does not reconfigure the system. Changes from this transaction will not picked up by the
/// system until a reconfiguration of the system is triggered. May only be sent by an account with
/// Validator role.
///
/// # Technical Description
/// Sets the account at `operator_account` address and with the specified `human_name` as an
/// operator for the sending validator account. The account at `operator_account` address must have
/// a Validator Operator role and have a `ValidatorOperatorConfig::ValidatorOperatorConfig`
/// resource published under it. The sending `account` must be a Validator and have a
/// `ValidatorConfig::ValidatorConfig` resource published under it. This script does not emit a
/// `LibraConfig::NewEpochEvent` and no reconfiguration of the system is initiated by this script.
///
/// # Parameters
/// | Name               | Type         | Description                                                                                  |
/// | ------             | ------       | -------------                                                                                |
/// | `account`          | `&signer`    | The signer reference of the sending account of the transaction.                              |
/// | `operator_name`    | `vector<u8>` | Validator operator's human name.                                                             |
/// | `operator_account` | `address`    | Address of the validator operator account to be added as the `account` validator's operator. |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                          | Description                                                                                                                                                  |
/// | ----------------           | --------------                                        | -------------                                                                                                                                                |
/// | `Errors::NOT_PUBLISHED`    | `ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG` | The `ValidatorOperatorConfig::ValidatorOperatorConfig` resource is not published under `operator_account`.                                                   |
/// | EMPTY                      | 0                                                     | The `human_name` field of the `ValidatorOperatorConfig::ValidatorOperatorConfig` resource under `operator_account` does not match the provided `human_name`. |
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
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::set_validator_config_and_reconfigure`

fun set_validator_operator(
    account: &signer,
    operator_name: vector<u8>,
    operator_account: address
) {
    assert(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
    ValidatorConfig::set_operator(account, operator_account);
}
}
