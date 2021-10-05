address 0x1 {
module ValidatorAdministrationScripts {
    use DiemFramework::DiemSystem;
    use DiemFramework::SlidingNonce;
    use DiemFramework::ValidatorConfig;
    use DiemFramework::ValidatorOperatorConfig;

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
    /// | `dr_account`        | `signer`     | The signer of the sending account of this transaction. Must be the Diem Root signer.                                               |
    /// | `sliding_nonce`     | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                         |
    /// | `validator_name`    | `vector<u8>` | ASCII-encoded human name for the validator. Must match the human name in the `ValidatorConfig::ValidatorConfig` for the validator. |
    /// | `validator_address` | `address`    | The validator account address to be added to the validator set.                                                                    |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                 | Description                                                                                                                               |
    /// | ----------------           | --------------                               | -------------                                                                                                                             |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`               | A `SlidingNonce` resource is not published under `dr_account`.                                                                            |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`               | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.                                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`               | The `sliding_nonce` is too far in the future.                                                                                             |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`      | The `sliding_nonce` has been previously recorded.                                                                                         |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                  | The sending account is not the Diem Root account.                                                                                         |
    /// | `Errors::REQUIRES_ROLE`    | `Roles::EDIEM_ROOT`                          | The sending account is not the Diem Root account.                                                                                         |
    /// | 0                          | 0                                            | The provided `validator_name` does not match the already-recorded human name for the validator.                                           |
    /// | `Errors::INVALID_ARGUMENT` | `DiemSystem::EINVALID_PROSPECTIVE_VALIDATOR` | The validator to be added does not have a `ValidatorConfig::ValidatorConfig` resource published under it, or its `config` field is empty. |
    /// | `Errors::INVALID_ARGUMENT` | `DiemSystem::EALREADY_A_VALIDATOR`           | The `validator_address` account is already a registered validator.                                                                        |
    /// | `Errors::INVALID_STATE`    | `DiemConfig::EINVALID_BLOCK_TIME`            | An invalid time value was encountered in reconfiguration. Unlikely to occur.                                                              |
    /// | `Errors::LIMIT_EXCEEDED`   | `DiemSystem::EMAX_VALIDATORS`                | The validator set is already at its maximum size. The validator could not be added.                                                       |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::register_validator_config`
    /// * `ValidatorAdministrationScripts::remove_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator`
    /// * `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`
    /// * `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`

    public(script) fun add_validator_and_reconfigure(
        dr_account: signer,
        sliding_nonce: u64,
        validator_name: vector<u8>,
        validator_address: address
    ) {
        SlidingNonce::record_nonce_or_abort(&dr_account, sliding_nonce);
        assert!(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
        DiemSystem::add_validator(&dr_account, validator_address);
    }


    spec add_validator_and_reconfigure {
        use DiemFramework::DiemAccount;
        use Std::Errors;
        use DiemFramework::Roles;
        use DiemFramework::DiemConfig;

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
            Errors::LIMIT_EXCEEDED,
            Errors::REQUIRES_ROLE;

        include DiemConfig::ReconfigureEmits;

        /// **Access Control:**
        /// Only the Diem Root account can add Validators [[H14]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
    }

    /// # Summary
    /// Updates a validator's configuration. This does not reconfigure the system and will not update
    /// the configuration in the validator set that is seen by other validators in the network. Can
    /// only be successfully sent by a Validator Operator account that is already registered with a
    /// validator.
    ///
    /// # Technical Description
    /// This updates the fields with corresponding names held in the `ValidatorConfig::ValidatorConfig`
    /// config resource held under `validator_account`. It does not emit a `DiemConfig::NewEpochEvent`
    /// so the copy of this config held in the validator set will not be updated, and the changes are
    /// only "locally" under the `validator_account` account address.
    ///
    /// # Parameters
    /// | Name                          | Type         | Description                                                                                                        |
    /// | ------                        | ------       | -------------                                                                                                      |
    /// | `validator_operator_account`  | `signer`     | Signer of the sending account. Must be the registered validator operator for the validator at `validator_address`. |
    /// | `validator_account`           | `address`    | The address of the validator's `ValidatorConfig::ValidatorConfig` resource being updated.                          |
    /// | `consensus_pubkey`            | `vector<u8>` | New Ed25519 public key to be used in the updated `ValidatorConfig::ValidatorConfig`.                               |
    /// | `validator_network_addresses` | `vector<u8>` | New set of `validator_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.             |
    /// | `fullnode_network_addresses`  | `vector<u8>` | New set of `fullnode_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.              |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                   | Description                                                                                           |
    /// | ----------------           | --------------                                 | -------------                                                                                         |
    /// | `Errors::NOT_PUBLISHED`    | `ValidatorConfig::EVALIDATOR_CONFIG`           | `validator_address` does not have a `ValidatorConfig::ValidatorConfig` resource published under it.   |
    /// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_TRANSACTION_SENDER` | `validator_operator_account` is not the registered operator for the validator at `validator_address`. |
    /// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::EINVALID_CONSENSUS_KEY`      | `consensus_pubkey` is not a valid ed25519 public key.                                                 |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::add_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::remove_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator`
    /// * `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`
    /// * `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`

    public(script) fun register_validator_config(
        validator_operator_account: signer,
        // TODO Rename to validator_addr, since it is an address.
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<u8>,
        fullnode_network_addresses: vector<u8>,
    ) {
        ValidatorConfig::set_config(
            &validator_operator_account,
            validator_account,
            consensus_pubkey,
            validator_network_addresses,
            fullnode_network_addresses
        );
     }

    /// Access control rule is that only the validator operator for a validator may set
    /// call this, but there is an aborts_if in SetConfigAbortsIf that tests that directly.
    spec register_validator_config {
        use Std::Errors;
        use DiemFramework::DiemAccount;
        use Std::Signer;

        include DiemAccount::TransactionChecks{sender: validator_operator_account}; // properties checked by the prologue.
        include ValidatorConfig::SetConfigAbortsIf {validator_addr: validator_account};
        ensures ValidatorConfig::is_valid(validator_account);

        aborts_with [check]
            Errors::INVALID_ARGUMENT,
            Errors::NOT_PUBLISHED;

        /// **Access Control:**
        /// Only the Validator Operator account which has been registered with the validator can
        /// update the validator's configuration [[H15]][PERMISSION].
        aborts_if Signer::address_of(validator_operator_account) !=
                    ValidatorConfig::get_operator(validator_account)
                        with Errors::INVALID_ARGUMENT;
    }

    /// # Summary
    /// This script removes a validator account from the validator set, and triggers a reconfiguration
    /// of the system to remove the validator from the system. This transaction can only be
    /// successfully called by the Diem Root account.
    ///
    /// # Technical Description
    /// This script removes the account at `validator_address` from the validator set. This transaction
    /// emits a `DiemConfig::NewEpochEvent` event. Once the reconfiguration triggered by this event
    /// has been performed, the account at `validator_address` is no longer considered to be a
    /// validator in the network. This transaction will fail if the validator at `validator_address`
    /// is not in the validator set.
    ///
    /// # Parameters
    /// | Name                | Type         | Description                                                                                                                        |
    /// | ------              | ------       | -------------                                                                                                                      |
    /// | `dr_account`        | `signer`     | The signer of the sending account of this transaction. Must be the Diem Root signer.                                               |
    /// | `sliding_nonce`     | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                         |
    /// | `validator_name`    | `vector<u8>` | ASCII-encoded human name for the validator. Must match the human name in the `ValidatorConfig::ValidatorConfig` for the validator. |
    /// | `validator_address` | `address`    | The validator account address to be removed from the validator set.                                                                |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                            | Description                                                                                     |
    /// | ----------------           | --------------                          | -------------                                                                                   |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `dr_account`.                                  |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.      |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                                   |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                               |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | The sending account is not the Diem Root account or Treasury Compliance account                |
    /// | 0                          | 0                                       | The provided `validator_name` does not match the already-recorded human name for the validator. |
    /// | `Errors::INVALID_ARGUMENT` | `DiemSystem::ENOT_AN_ACTIVE_VALIDATOR` | The validator to be removed is not in the validator set.                                        |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`            | The sending account is not the Diem Root account.                                              |
    /// | `Errors::REQUIRES_ROLE`    | `Roles::EDIEM_ROOT`                    | The sending account is not the Diem Root account.                                              |
    /// | `Errors::INVALID_STATE`    | `DiemConfig::EINVALID_BLOCK_TIME`      | An invalid time value was encountered in reconfiguration. Unlikely to occur.                    |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::register_validator_config`
    /// * `ValidatorAdministrationScripts::add_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator`
    /// * `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`
    /// * `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`

    public(script) fun remove_validator_and_reconfigure(
        dr_account: signer,
        sliding_nonce: u64,
        validator_name: vector<u8>,
        validator_address: address
    ) {
        SlidingNonce::record_nonce_or_abort(&dr_account, sliding_nonce);
        // TODO: Use an error code from Errors.move
        assert!(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
        DiemSystem::remove_validator(&dr_account, validator_address);
    }

    spec remove_validator_and_reconfigure {
        use DiemFramework::DiemAccount;
        use Std::Errors;
        use DiemFramework::Roles;
        use DiemFramework::DiemConfig;

        include DiemAccount::TransactionChecks{sender: dr_account}; // properties checked by the prologue.
        include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: dr_account};
        // next is due to abort in get_human_name
        include ValidatorConfig::AbortsIfNoValidatorConfig{addr: validator_address};
        // TODO: use an error code from Errors.move instead of 0.
        aborts_if ValidatorConfig::get_human_name(validator_address) != validator_name with 0;
        include DiemSystem::RemoveValidatorAbortsIf{validator_addr: validator_address};
        include DiemSystem::RemoveValidatorEnsures{validator_addr: validator_address};

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

        include DiemConfig::ReconfigureEmits;

        /// **Access Control:**
        /// Only the Diem Root account can remove Validators [[H14]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
    }

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
    /// | Name                          | Type         | Description                                                                                                        |
    /// | ------                        | ------       | -------------                                                                                                      |
    /// | `validator_operator_account`  | `signer`     | Signer of the sending account. Must be the registered validator operator for the validator at `validator_address`. |
    /// | `validator_account`           | `address`    | The address of the validator's `ValidatorConfig::ValidatorConfig` resource being updated.                          |
    /// | `consensus_pubkey`            | `vector<u8>` | New Ed25519 public key to be used in the updated `ValidatorConfig::ValidatorConfig`.                               |
    /// | `validator_network_addresses` | `vector<u8>` | New set of `validator_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.             |
    /// | `fullnode_network_addresses`  | `vector<u8>` | New set of `fullnode_network_addresses` to be used in the updated `ValidatorConfig::ValidatorConfig`.              |
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
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::add_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::remove_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator`
    /// * `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`
    /// * `ValidatorAdministrationScripts::register_validator_config`

    public(script) fun set_validator_config_and_reconfigure(
        validator_operator_account: signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<u8>,
        fullnode_network_addresses: vector<u8>,
    ) {
        ValidatorConfig::set_config(
            &validator_operator_account,
            validator_account,
            consensus_pubkey,
            validator_network_addresses,
            fullnode_network_addresses
        );
        DiemSystem::update_config_and_reconfigure(&validator_operator_account, validator_account);
     }

    spec set_validator_config_and_reconfigure {
        use DiemFramework::DiemAccount;
        use DiemFramework::DiemConfig;
        use DiemFramework::DiemSystem;
        use DiemFramework::DiemTimestamp;
        use Std::Errors;
        use Std::Signer;

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
        let validator_index = DiemSystem::spec_index_of_validator(DiemSystem::spec_get_validators(), validator_account);
        let last_config_time = DiemSystem::spec_get_validators()[validator_index].last_config_update_time;
        aborts_if is_validator_info_updated && last_config_time > DiemSystem::MAX_U64 - DiemSystem::FIVE_MINUTES
            with Errors::LIMIT_EXCEEDED;
        aborts_if is_validator_info_updated && DiemTimestamp::spec_now_microseconds() <= last_config_time + DiemSystem::FIVE_MINUTES
            with Errors::LIMIT_EXCEEDED;

        /// This reports a possible INVALID_STATE abort, which comes from an assert in DiemConfig::reconfigure_
        /// that config.last_reconfiguration_time is not in the future. This is a system error that a user
        /// for which there is no useful recovery except to resubmit the transaction.
        aborts_with [check]
            Errors::NOT_PUBLISHED,
            Errors::REQUIRES_ROLE,
            Errors::INVALID_ARGUMENT,
            Errors::INVALID_STATE;

        include is_validator_info_updated ==> DiemConfig::ReconfigureEmits;

        /// **Access Control:**
        /// Only the Validator Operator account which has been registered with the validator can
        /// update the validator's configuration [[H15]][PERMISSION].
        aborts_if Signer::address_of(validator_operator_account) !=
                    ValidatorConfig::get_operator(validator_account)
                        with Errors::INVALID_ARGUMENT;
    }

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
    /// `DiemConfig::NewEpochEvent` and no reconfiguration of the system is initiated by this script.
    ///
    /// # Parameters
    /// | Name               | Type         | Description                                                                                  |
    /// | ------             | ------       | -------------                                                                                |
    /// | `account`          | `signer`     | The signer of the sending account of the transaction.                                        |
    /// | `operator_name`    | `vector<u8>` | Validator operator's human name.                                                             |
    /// | `operator_account` | `address`    | Address of the validator operator account to be added as the `account` validator's operator. |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                          | Description                                                                                                                                                  |
    /// | ----------------           | --------------                                        | -------------                                                                                                                                                |
    /// | `Errors::NOT_PUBLISHED`    | `ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG` | The `ValidatorOperatorConfig::ValidatorOperatorConfig` resource is not published under `operator_account`.                                                   |
    /// | 0                          | 0                                                     | The `human_name` field of the `ValidatorOperatorConfig::ValidatorOperatorConfig` resource under `operator_account` does not match the provided `human_name`. |
    /// | `Errors::REQUIRES_ROLE`    | `Roles::EVALIDATOR`                                   | `account` does not have a Validator account role.                                                                                                            |
    /// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR`          | The account at `operator_account` does not have a `ValidatorOperatorConfig::ValidatorOperatorConfig` resource.                                               |
    /// | `Errors::NOT_PUBLISHED`    | `ValidatorConfig::EVALIDATOR_CONFIG`                  | A `ValidatorConfig::ValidatorConfig` is not published under `account`.                                                                                       |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::register_validator_config`
    /// * `ValidatorAdministrationScripts::remove_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::add_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator_with_nonce_admin`
    /// * `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`

    public(script) fun set_validator_operator(
        account: signer,
        operator_name: vector<u8>,
        operator_account: address
    ) {
        assert!(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
        ValidatorConfig::set_operator(&account, operator_account);
    }

    spec set_validator_operator {
        use DiemFramework::DiemAccount;
        use Std::Signer;
        use Std::Errors;
        use DiemFramework::Roles;

        let account_addr = Signer::address_of(account);
        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        // next is due to abort in get_human_name
        include ValidatorConfig::AbortsIfNoValidatorConfig{addr: account_addr};
        // TODO: use an error code from Errors.move instead of 0.
        aborts_if ValidatorOperatorConfig::get_human_name(operator_account) != operator_name with 0;
        include ValidatorConfig::SetOperatorAbortsIf{validator_account: account, operator_addr: operator_account};
        include ValidatorConfig::SetOperatorEnsures{validator_account: account, operator_addr: operator_account};

        /// Reports INVALID_STATE because of !exists<DiemSystem::CapabilityHolder>, but that can't happen
        /// because CapabilityHolder is published during initialization (Genesis).
        aborts_with [check]
            0, // Odd error code in assert on second statement in add_validator_and_reconfigure
            Errors::INVALID_ARGUMENT,
            Errors::NOT_PUBLISHED,
            Errors::REQUIRES_ROLE;

        /// **Access Control:**
        /// Only a Validator account can set its Validator Operator [[H16]][PERMISSION].
        include Roles::AbortsIfNotValidator;
    }

    /// # Summary
    /// Sets the validator operator for a validator in the validator's configuration resource "locally"
    /// and does not reconfigure the system. Changes from this transaction will not picked up by the
    /// system until a reconfiguration of the system is triggered. May only be sent by the Diem Root
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
    /// | Name               | Type         | Description                                                                                   |
    /// | ------             | ------       | -------------                                                                                 |
    /// | `dr_account`       | `signer`     | Signer of the sending account of the write set transaction. May only be the Diem Root signer. |
    /// | `account`          | `signer`     | Signer of account specified in the `execute_as` field of the write set transaction.           |
    /// | `sliding_nonce`    | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction for Diem Root.      |
    /// | `operator_name`    | `vector<u8>` | Validator operator's human name.                                                              |
    /// | `operator_account` | `address`    | Address of the validator operator account to be added as the `account` validator's operator.  |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                          | Description                                                                                                                                                  |
    /// | ----------------           | --------------                                        | -------------                                                                                                                                                |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                        | A `SlidingNonce` resource is not published under `dr_account`.                                                                                               |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                        | The `sliding_nonce` in `dr_account` is too old and it's impossible to determine if it's duplicated or not.                                                   |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                        | The `sliding_nonce` in `dr_account` is too far in the future.                                                                                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`               | The `sliding_nonce` in` dr_account` has been previously recorded.                                                                                            |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                        | The sending account is not the Diem Root account or Treasury Compliance account                                                                             |
    /// | `Errors::NOT_PUBLISHED`    | `ValidatorOperatorConfig::EVALIDATOR_OPERATOR_CONFIG` | The `ValidatorOperatorConfig::ValidatorOperatorConfig` resource is not published under `operator_account`.                                                   |
    /// | 0                          | 0                                                     | The `human_name` field of the `ValidatorOperatorConfig::ValidatorOperatorConfig` resource under `operator_account` does not match the provided `human_name`. |
    /// | `Errors::REQUIRES_ROLE`    | `Roles::EVALIDATOR`                                   | `account` does not have a Validator account role.                                                                                                            |
    /// | `Errors::INVALID_ARGUMENT` | `ValidatorConfig::ENOT_A_VALIDATOR_OPERATOR`          | The account at `operator_account` does not have a `ValidatorOperatorConfig::ValidatorOperatorConfig` resource.                                               |
    /// | `Errors::NOT_PUBLISHED`    | `ValidatorConfig::EVALIDATOR_CONFIG`                  | A `ValidatorConfig::ValidatorConfig` is not published under `account`.                                                                                       |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_validator_account`
    /// * `AccountCreationScripts::create_validator_operator_account`
    /// * `ValidatorAdministrationScripts::register_validator_config`
    /// * `ValidatorAdministrationScripts::remove_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::add_validator_and_reconfigure`
    /// * `ValidatorAdministrationScripts::set_validator_operator`
    /// * `ValidatorAdministrationScripts::set_validator_config_and_reconfigure`

    public(script) fun set_validator_operator_with_nonce_admin(
        dr_account: signer,
        account: signer,
        sliding_nonce: u64,
        operator_name: vector<u8>,
        operator_account: address
    ) {
        SlidingNonce::record_nonce_or_abort(&dr_account, sliding_nonce);
        assert!(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
        ValidatorConfig::set_operator(&account, operator_account);
    }

    spec set_validator_operator_with_nonce_admin {
        use DiemFramework::DiemAccount;
        use Std::Signer;
        use Std::Errors;
        use DiemFramework::Roles;

        let account_addr = Signer::address_of(account);
        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: dr_account};
        // next is due to abort in get_human_name
        include ValidatorConfig::AbortsIfNoValidatorConfig{addr: account_addr};
        // TODO: use an error code from Errors.move instead of 0.
        aborts_if ValidatorOperatorConfig::get_human_name(operator_account) != operator_name with 0;
        include ValidatorConfig::SetOperatorAbortsIf{validator_account: account, operator_addr: operator_account};
        include ValidatorConfig::SetOperatorEnsures{validator_account: account, operator_addr: operator_account};

        aborts_with [check]
            0, // Odd error code in assert on second statement in add_validator_and_reconfigure
            Errors::INVALID_ARGUMENT,
            Errors::NOT_PUBLISHED,
            Errors::REQUIRES_ROLE;

        /// **Access Control:**
        /// Only the Diem Root account can process the admin scripts [[H9]][PERMISSION].
        requires Roles::has_diem_root_role(dr_account); /// This is ensured by DiemAccount::writeset_prologue.
        /// Only a Validator account can set its Validator Operator [[H16]][PERMISSION].
        include Roles::AbortsIfNotValidator;
    }
}
}
