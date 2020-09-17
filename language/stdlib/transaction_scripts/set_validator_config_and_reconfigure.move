script {
use 0x1::LibraSystem;
use 0x1::ValidatorConfig;

/// # Summary
/// Updates a validator's configuration, and triggers a reconfiguration of the system to update the
/// validator set with this new validator configuration.  Can only be successfully sent by a
/// Validator Operator account that is already registered with a validator.
///
/// # Technical Description
/// This updates the fields with corresponding names held in the `ValidatorConfig::ValidatorConfig`
/// config resource held under `validator_account`. It then emits a `LibraConfig::NewEpochEvent` to
/// trigger a reconfiguration of the system.  This reconfiguration will update the validator set
/// on-chain with the updated `ValidatorConfig::ValidatorConfig`.
///
/// # Parameters
/// | Name                          | Type         | Description                                                                                                                  |
/// | ------                        | ------       | -------------                                                                                                                |
/// | `validator_operator_account`  | `&signer`    | Signer reference of the sending account. Must be the registered validator operator for the validator at `validator_address`. |
/// | `validator_account`           | `address`    | The address of the validator's `ValidatorConfig::ValidatorConfig` resource being updated.                                    |
/// | `consensus_pubkey`            | `vector<u8>` | New Ed25519 public key to be used in the updated `ValidatorConfig::ValidatorConfig`.                                         |
/// | `validator_network_addresses` | `vector<u8>` | New set of `validator_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.                       |
/// | `fullnode_network_addresses`  | `vector<u8>` | New set of `fullnode_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.                        |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                   | Description                                                                                           |
/// | ----------------           | --------------                                 | -------------                                                                                         |
/// | `Errors::NOT_PUBLISHED`    | `ValidatorConfig::EVALIDATOR_CONFIG`           | `validator_address` does not have a `ValidatorConfig::ValidatorConfig` resource published under it.   |
/// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_TRANSACTION_SENDER` | `validator_operator_account` is not the registered operator for the validator at `validator_address`. |
/// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_CONSENSUS_KEY`      | `consensus_pubkey` is not a valid ed25519 public key.                                                 |
///
/// # Related Scripts
/// * `Script::create_validator_account`
/// * `Script::create_validator_operator_account`
/// * `Script::add_validator_and_reconfigure`
/// * `Script::remove_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::register_validator_config`

fun set_validator_config_and_reconfigure(
    validator_operator_account: &signer,
    validator_account: address,
    consensus_pubkey: vector<u8>,
    validator_network_addresses: vector<u8>,
    fullnode_network_addresses: vector<u8>,
) {
    ValidatorConfig::set_config(
        validator_operator_account,
        validator_account,
        consensus_pubkey,
        validator_network_addresses,
        fullnode_network_addresses
    );
    LibraSystem::update_config_and_reconfigure(validator_operator_account, validator_account);
 }
}
