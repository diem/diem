address 0x1 {

module ValidatorConfig {
    use 0x1::LibraTimestamp;
    use 0x1::Errors;
    use 0x1::Option::{Self, Option};
    use 0x1::Signature;
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::ValidatorOperatorConfig;

    struct Config {
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<u8>,
        fullnode_network_addresses: vector<u8>,
    }

    resource struct ValidatorConfig {
        /// set and rotated by the operator_account
        config: Option<Config>,
        operator_account: Option<address>,
        /// The human readable name of this entity. Immutable.
        human_name: vector<u8>,
    }

    // TODO(valerini): add events here

    /// The `ValidatorConfig` resource was not in the required state
    const EVALIDATOR_CONFIG: u64 = 0;
    /// The sender is not the operator for the specified validator
    const EINVALID_TRANSACTION_SENDER: u64 = 1;
    /// The provided consensus public key is malformed
    const EINVALID_CONSENSUS_KEY: u64 = 2;
    /// Tried to set an account without the correct operator role as a Validator Operator
    const ENOT_A_VALIDATOR_OPERATOR: u64 = 3;

    ///////////////////////////////////////////////////////////////////////////
    // Validator setup methods
    ///////////////////////////////////////////////////////////////////////////

    /// Publishes a mostly empty ValidatorConfig struct. Eventually, it
    /// will have critical info such as keys, network addresses for validators,
    /// and the address of the validator operator.
    public fun publish(
        validator_account: &signer,
        lr_account: &signer,
        human_name: vector<u8>,
    ) {
        LibraTimestamp::assert_operating();
        Roles::assert_libra_root(lr_account);
        Roles::assert_validator(validator_account);
        assert(
            !exists<ValidatorConfig>(Signer::address_of(validator_account)),
            Errors::already_published(EVALIDATOR_CONFIG)
        );
        move_to(validator_account, ValidatorConfig {
            config: Option::none(),
            operator_account: Option::none(),
            human_name,
        });
    }

    spec fun publish {
        include LibraTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include Roles::AbortsIfNotValidator;
        aborts_if exists_config(Signer::spec_address_of(validator_account))
            with Errors::ALREADY_PUBLISHED;
        ensures exists_config(Signer::spec_address_of(validator_account));
    }

    /// Returns true if a ValidatorConfig resource exists under addr.
    fun exists_config(addr: address): bool {
        exists<ValidatorConfig>(addr)
    }

    /// Describes abort if ValidatorConfig does not exist.
    spec schema AbortsIfNoValidatorConfig {
        addr: address;
        aborts_if !exists_config(addr) with Errors::NOT_PUBLISHED;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig owner
    ///////////////////////////////////////////////////////////////////////////

    /// Sets a new operator account, preserving the old config.
    /// Note: Access control.  No one but the owner of the account may change .operator_account
    public fun set_operator(validator_account: &signer, operator_account: address) acquires ValidatorConfig {
        Roles::assert_validator(validator_account);
        // Check for validator role is not necessary since the role is checked when the config
        // resource is published.
        // TODO (dd): Probably need to prove an invariant about role.
        assert(
            ValidatorOperatorConfig::has_validator_operator_config(operator_account),
            Errors::invalid_argument(ENOT_A_VALIDATOR_OPERATOR)
        );
        let sender = Signer::address_of(validator_account);
        assert(exists_config(sender), Errors::not_published(EVALIDATOR_CONFIG));
        (borrow_global_mut<ValidatorConfig>(sender)).operator_account = Option::some(operator_account);
    }
    spec fun set_operator {
        /// Must abort if the signer does not have the Validator role [B24].
        include Roles::AbortsIfNotValidator;

        aborts_if !ValidatorOperatorConfig::has_validator_operator_config(operator_account)
            with Errors::INVALID_ARGUMENT;
        let sender = Signer::spec_address_of(validator_account);
        include AbortsIfNoValidatorConfig{addr: sender};
        aborts_if !ValidatorOperatorConfig::has_validator_operator_config(operator_account) with Errors::NOT_PUBLISHED;
        ensures spec_has_operator(sender);
        ensures spec_get_operator(sender) == operator_account;

        /// The signer can only change its own operator account [B24].
        ensures forall addr: address where addr != sender:
            global<ValidatorConfig>(addr).operator_account == old(global<ValidatorConfig>(addr).operator_account);
    }

    spec module {
        /// Returns true if addr has an operator account.
        define spec_has_operator(addr: address): bool {
            Option::is_some(global<ValidatorConfig>(addr).operator_account)
        }

        /// Returns the operator account of a validator if it has one,
        /// and returns the addr itself otherwise.
        define spec_get_operator(addr: address): address {
            if (spec_has_operator(addr)) {
                Option::borrow(global<ValidatorConfig>(addr).operator_account)
            } else {
                addr
            }
        }

        /// Returns the human name of the validator
        define spec_get_human_name(addr: address): vector<u8> {
            global<ValidatorConfig>(addr).human_name
        }
    }

    /// Removes an operator account, setting a corresponding field to Option::none.
    /// The old config is preserved.
    public fun remove_operator(validator_account: &signer) acquires ValidatorConfig {
        Roles::assert_validator(validator_account);
        let sender = Signer::address_of(validator_account);
        // Config field remains set
        assert(exists_config(sender), Errors::not_published(EVALIDATOR_CONFIG));
        (borrow_global_mut<ValidatorConfig>(sender)).operator_account = Option::none();
    }

    spec fun remove_operator {
        /// Must abort if the signer does not have the Validator role [B24].
        include Roles::AbortsIfNotValidator;

        let sender = Signer::spec_address_of(validator_account);
        include AbortsIfNoValidatorConfig{addr: sender};
        ensures !spec_has_operator(Signer::spec_address_of(validator_account));
        ensures spec_get_operator(sender) == sender;

        /// The signer can only change its own operator account [B24].
        ensures forall addr: address where addr != sender:
            global<ValidatorConfig>(addr).operator_account == old(global<ValidatorConfig>(addr).operator_account);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig.operator_account
    ///////////////////////////////////////////////////////////////////////////

    /// Rotate the config in the validator_account
    /// NB! Once the config is set, it can not go to Option::none - this is crucial for validity
    ///     of the LibraSystem's code
    /// label: ValidatorConfigRemainsValid
    public fun set_config(
        signer: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<u8>,
        fullnode_network_addresses: vector<u8>,
    ) acquires ValidatorConfig {
        assert(
            Signer::address_of(signer) == get_operator(validator_account),
            Errors::invalid_argument(EINVALID_TRANSACTION_SENDER)
        );
        assert(
            Signature::ed25519_validate_pubkey(copy consensus_pubkey),
            Errors::invalid_argument(EINVALID_CONSENSUS_KEY)
        );
        // TODO(valerini): verify the proof of posession for consensus_pubkey
        assert(exists_config(validator_account), Errors::not_published(EVALIDATOR_CONFIG));
        let t_ref = borrow_global_mut<ValidatorConfig>(validator_account);
        t_ref.config = Option::some(Config {
            consensus_pubkey,
            validator_network_addresses,
            fullnode_network_addresses,
        });
    }

    spec fun set_config {
        let sender = Signer::spec_address_of(signer);
        aborts_if sender != spec_get_operator(validator_account) with Errors::INVALID_ARGUMENT;
        include AbortsIfNoValidatorConfig{addr: validator_account};
        aborts_if !Signature::ed25519_validate_pubkey(consensus_pubkey) with Errors::INVALID_ARGUMENT;
        ensures spec_has_config(validator_account);
    }

    /// Returns true if there a config published under addr.
    spec define spec_has_config(addr: address): bool {
        Option::is_some(global<ValidatorConfig>(addr).config)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs: getters
    ///////////////////////////////////////////////////////////////////////////

    /// Returns true if all of the following is true:
    /// 1) there is a ValidatorConfig resource under the address, and
    /// 2) the config is set, and
    /// NB! currently we do not require the the operator_account to be set
    public fun is_valid(addr: address): bool acquires ValidatorConfig {
        exists<ValidatorConfig>(addr) && Option::is_some(&borrow_global<ValidatorConfig>(addr).config)
    }

    spec fun is_valid {
        pragma opaque = true;
        aborts_if false;
        ensures result == is_valid(addr);
    }

    /// # Validator stays valid once it becomes valid
    /// See comment on ValidatorConfig::set_config -- LibraSystem depends on this.
    /// ref: (#ValidatorConfigRemainsValid)
    spec module {
        invariant update [global]
            forall validator: address where old(is_valid(validator)): is_valid(validator);
    }

    /// Get Config
    /// Aborts if there is no ValidatorConfig resource of if its config is empty
    public fun get_config(addr: address): Config acquires ValidatorConfig {
        assert(exists_config(addr), Errors::not_published(EVALIDATOR_CONFIG));
        let config = &borrow_global<ValidatorConfig>(addr).config;
        assert(Option::is_some(config), Errors::invalid_argument(EVALIDATOR_CONFIG));
        *Option::borrow(config)
    }

    spec fun get_config {
        pragma opaque = true;
        include AbortsIfNoValidatorConfig;
        aborts_if Option::spec_is_none(global<ValidatorConfig>(addr).config) with Errors::INVALID_ARGUMENT;
        ensures result == spec_get_config(addr);
    }

    /// Returns the config published under addr.
    spec define spec_get_config(addr: address): Config {
        Option::borrow(global<ValidatorConfig>(addr).config)
    }

    /// Get validator's account human name
    /// Aborts if there is no ValidatorConfig resource
    public fun get_human_name(addr: address): vector<u8> acquires ValidatorConfig {
        assert(exists<ValidatorConfig>(addr), Errors::not_published(EVALIDATOR_CONFIG));
        let t_ref = borrow_global<ValidatorConfig>(addr);
        *&t_ref.human_name
    }

    spec fun get_human_name {
        pragma opaque = true;
        include AbortsIfNoValidatorConfig;
        ensures result == spec_get_human_name(addr);
    }

    /// Get operator's account
    /// Aborts if there is no ValidatorConfig resource, if its operator_account is
    /// empty, returns the input
    public fun get_operator(addr: address): address acquires ValidatorConfig {
        assert(exists<ValidatorConfig>(addr), Errors::not_published(EVALIDATOR_CONFIG));
        let t_ref = borrow_global<ValidatorConfig>(addr);
        *Option::borrow_with_default(&t_ref.operator_account, &addr)
    }

    spec fun get_operator {
        pragma opaque = true;
        include AbortsIfNoValidatorConfig;
        ensures result == spec_get_operator(addr);
    }

    /// Get consensus_pubkey from Config
    /// Never aborts
    public fun get_consensus_pubkey(config_ref: &Config): &vector<u8> {
        &config_ref.consensus_pubkey
    }

    /// Get validator's network address from Config
    /// Never aborts
    public fun get_validator_network_addresses(config_ref: &Config): &vector<u8> {
        &config_ref.validator_network_addresses
    }

    spec module {
        pragma aborts_if_is_strict = true;
    }

    /// Specifies that only set_operator and remove_operator may change the operator for a
    /// particular (validator owner) address. Those two functions have a &signer argument for
    /// the validator account, so we know that the change has been authorized by the validator
    /// owner via signing the transaction. But other functions in this module could also
    /// change the operator_account field of ValidatorConfig, and this shows that they do not.
    spec schema OperatorRemainsSame {
        ensures forall addr1: address where old(exists<ValidatorConfig>(addr1)):
            global<ValidatorConfig>(addr1).operator_account == old(global<ValidatorConfig>(addr1).operator_account);
    }
    spec module {
        ///  set_operator, remove_operator can change the operator account [B24].
        apply OperatorRemainsSame to * except set_operator, remove_operator;
    }

    /// LIP-6 Property: If address has a ValidatorConfig, it has a validator role.  This invariant is useful
    /// in LibraSystem so we don't have to check whether every validator address has a validator role.
    /// <a name="ValidatorConfigImpliesValidatorRole"></a>
    spec module {
        invariant [global] forall addr1: address where exists_config(addr1):
            Roles::spec_has_validator_role_addr(addr1);
    }
}
}
