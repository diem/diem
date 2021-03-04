script {
use 0x1::DiemSystem;
use 0x1::ValidatorConfig;

/// # Summary
/// Updates a validator's configuration, and triggers a reconfiguration of the system to update the
/// validator set with this new validator configuration.  Can only be successfully sent by a
/// Validator Operator account that is already registered with a validator.
///
/// # Technical Description
/// This updates the fields with corresponding names held in the `ValidatorConfig::ValidatorConfig`
/// config resource held under `validator_account`. It then emits a `DiemConfig::NewEpochEvent` to
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
/// | `Errors::REQUIRES_ROLE`    | `Roles::EVALIDATOR_OPERATOR`                   | `validator_operator_account` does not have a Validator Operator role.                                 |
/// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_TRANSACTION_SENDER` | `validator_operator_account` is not the registered operator for the validator at `validator_address`. |
/// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_CONSENSUS_KEY`      | `consensus_pubkey` is not a valid ed25519 public key.                                                 |
/// | `Errors::INVALID_STATE`    | `DiemConfig::EINVALID_BLOCK_TIME`             | An invalid time value was encountered in reconfiguration. Unlikely to occur.                          |
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
    DiemSystem::update_config_and_reconfigure(validator_operator_account, validator_account);
 }

spec fun set_validator_config_and_reconfigure {
    use 0x1::DiemAccount;
    use 0x1::DiemConfig;
    use 0x1::DiemSystem;
    use 0x1::Errors;
    use 0x1::Signer;

     // properties checked by the prologue.
   include DiemAccount::TransactionChecks{sender: validator_operator_account};
    // UpdateConfigAndReconfigureEnsures includes properties such as not changing info
    // for validators in the validator_set other than that for validator_account, etc.
    // These properties are important for access control. See notes in DiemSystem.
    include DiemSystem::UpdateConfigAndReconfigureEnsures{validator_addr: validator_account};
    ensures ValidatorConfig::is_valid(validator_account);
    // The published ValidatorConfig has the correct data, according to the arguments to
    // set_validator_config_and_reconfigure
    ensures ValidatorConfig::spec_get_config(validator_account)
        == ValidatorConfig::Config {
                    consensus_pubkey,
                    validator_network_addresses,
                    fullnode_network_addresses,
    };

    include ValidatorConfig::SetConfigAbortsIf{validator_addr: validator_account};
    include DiemSystem::UpdateConfigAndReconfigureAbortsIf{validator_addr: validator_account};

    // Reconfiguration only happens if validator info changes for a validator in the validator set.
    // v_info.config is the old config (if it exists) and the Config with args from set_config is
    // the new config
    let is_validator_info_updated =
        (exists v_info in DiemSystem::spec_get_validators():
            v_info.addr == validator_account
            && v_info.config != ValidatorConfig::Config {
                    consensus_pubkey,
                    validator_network_addresses,
                    fullnode_network_addresses,
               });
    include is_validator_info_updated ==> DiemConfig::ReconfigureAbortsIf;


    /// This reports a possible INVALID_STATE abort, which comes from an assert in DiemConfig::reconfigure_
    /// that config.last_reconfiguration_time is not in the future. This is a system error that a user
    /// for which there is no useful recovery except to resubmit the transaction.
    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::REQUIRES_ROLE,
        Errors::INVALID_ARGUMENT,
        Errors::INVALID_STATE;

    /// **Access Control:**
    /// Only the Validator Operator account which has been registered with the validator can
    /// update the validator's configuration [[H14]][PERMISSION].
    aborts_if Signer::address_of(validator_operator_account) !=
                ValidatorConfig::get_operator(validator_account)
                    with Errors::INVALID_ARGUMENT;
}
}
