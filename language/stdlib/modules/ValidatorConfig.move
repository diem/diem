address 0x1 {

module ValidatorConfig {
    use 0x1::Option::{Self, Option};
    use 0x1::Signature;
    use 0x1::Signer;
    use 0x1::Roles;

    resource struct UpdateValidatorConfig {}

    struct Config {
        consensus_pubkey: vector<u8>,
        /// TODO(philiphayes): restructure
        ///   3) remove validator_network_identity_pubkey
        ///   4) remove full_node_network_identity_pubkey
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        full_node_network_identity_pubkey: vector<u8>,
        full_node_network_address: vector<u8>,
    }

    resource struct ValidatorConfig {
        /// set and rotated by the operator_account
        config: Option<Config>,
        operator_account: Option<address>,
    }

    /// TODO(valerini): add events here

    const ENOT_LIBRA_ROOT: u64 = 0;
    const EINVALID_TRANSACTION_SENDER: u64 = 1;
    const EVALIDATOR_RESOURCE_DOES_NOT_EXIST: u64 = 2;
    const EINVALID_CONSENSUS_KEY: u64 = 3;

    ///////////////////////////////////////////////////////////////////////////
    // Validator setup methods
    ///////////////////////////////////////////////////////////////////////////

    public fun publish(
        account: &signer,
        lr_account: &signer,
        ) {
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        move_to(account, ValidatorConfig {
            config: Option::none(),
            operator_account: Option::none(),
        });
    }

    spec fun publish {
        aborts_if !Roles::spec_has_libra_root_role_addr(Signer::spec_address_of(lr_account));
        aborts_if spec_exists_config(Signer::spec_address_of(account));
        ensures spec_exists_config(Signer::spec_address_of(account));
    }

    spec module {
        /// Returns true if a ValidatorConfig resource exists under addr.
        define spec_exists_config(addr: address): bool {
            exists<ValidatorConfig>(addr)
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig owner
    ///////////////////////////////////////////////////////////////////////////

    /// Sets a new operator account, preserving the old config.
    public fun set_operator(account: &signer, operator_account: address) acquires ValidatorConfig {
        let sender = Signer::address_of(account);
        (borrow_global_mut<ValidatorConfig>(sender)).operator_account = Option::some(operator_account);
    }

    spec fun set_operator {
        aborts_if !spec_exists_config(Signer::spec_address_of(account));
        ensures spec_has_operator(Signer::spec_address_of(account));
        ensures spec_get_operator(Signer::spec_address_of(account)) == operator_account;
    }

    spec module {
        /// Returns true if addr has an operator account.
        define spec_has_operator(addr: address): bool {
            Option::spec_is_some(global<ValidatorConfig>(addr).operator_account)
        }

        /// Returns the operator account of a validator if it has one,
        /// and returns the addr itself otherwise.
        define spec_get_operator(addr: address): address {
            if (spec_has_operator(addr)) {
                Option::spec_value_inside(global<ValidatorConfig>(addr).operator_account)
            } else {
                addr
            }
        }
    }

    /// Removes an operator account, setting a corresponding field to Option::none.
    /// The old config is preserved.
    public fun remove_operator(account: &signer) acquires ValidatorConfig {
        let sender = Signer::address_of(account);
        // Config field remains set
        (borrow_global_mut<ValidatorConfig>(sender)).operator_account = Option::none();
    }

    spec fun remove_operator {
        aborts_if !spec_exists_config(Signer::spec_address_of(account));
        ensures !spec_has_operator(Signer::spec_address_of(account));
        ensures spec_get_operator(Signer::spec_address_of(account))
            == Signer::spec_address_of(account);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig.operator_account
    ///////////////////////////////////////////////////////////////////////////

    /// Rotate the config in the validator_account
    /// NB! Once the config is set, it can not go to Option::none - this is crucial for validity
    ///     of the LibraSystem's code
    public fun set_config(
        signer: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        full_node_network_identity_pubkey: vector<u8>,
        full_node_network_address: vector<u8>,
    ) acquires ValidatorConfig {
        assert(
            Signer::address_of(signer) == get_operator(validator_account),
            EINVALID_TRANSACTION_SENDER
        );
        assert(Signature::ed25519_validate_pubkey(copy consensus_pubkey), EINVALID_CONSENSUS_KEY);
        // TODO(valerini): verify the proof of posession for consensus_pubkey
        let t_ref = borrow_global_mut<ValidatorConfig>(validator_account);
        t_ref.config = Option::some(Config {
            consensus_pubkey,
            validator_network_identity_pubkey,
            validator_network_address,
            full_node_network_identity_pubkey,
            full_node_network_address,
        });
    }

    spec fun set_config {
        aborts_if Signer::spec_address_of(signer) != spec_get_operator(validator_account);
        aborts_if !spec_exists_config(validator_account);
        aborts_if !Signature::spec_ed25519_validate_pubkey(consensus_pubkey);
        ensures spec_has_config(validator_account);
    }

    spec module {
        /// Returns true if there a config published under addr.
        define spec_has_config(addr: address): bool {
            Option::spec_is_some(global<ValidatorConfig>(addr).config)
        }
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
        aborts_if false;
        ensures result == spec_is_valid(addr);
    }

    spec module {
        /// Returns true if addr is a valid validator.
        define spec_is_valid(addr: address): bool {
            spec_exists_config(addr) && spec_has_config(addr)
        }
    }

    /// ## Validator stays valid once it becomes valid

    spec schema ValidatorStaysValid {
        ensures forall validator: address:
            old(spec_is_valid(validator)) ==> spec_is_valid(validator);
    }

    spec module {
        apply ValidatorStaysValid to *;
    }

    /// Get Config
    /// Aborts if there is no ValidatorConfig resource of if its config is empty
    public fun get_config(addr: address): Config acquires ValidatorConfig {
        assert(exists<ValidatorConfig>(addr), EVALIDATOR_RESOURCE_DOES_NOT_EXIST);
        let config = &borrow_global<ValidatorConfig>(addr).config;
        *Option::borrow(config)
    }

    spec fun get_config {
        aborts_if !spec_exists_config(addr);
        aborts_if !spec_has_config(addr);
        ensures result == spec_get_config(addr);
    }

    spec module {
        /// Returns the config published under addr.
        define spec_get_config(addr: address): Config {
            Option::spec_value_inside(global<ValidatorConfig>(addr).config)
        }
    }

    /// Get operator's account
    /// Aborts if there is no ValidatorConfig resource, if its operator_account is
    /// empty, returns the input
    public fun get_operator(addr: address): address acquires ValidatorConfig {
        assert(exists<ValidatorConfig>(addr), EVALIDATOR_RESOURCE_DOES_NOT_EXIST);
        let t_ref = borrow_global<ValidatorConfig>(addr);
        *Option::borrow_with_default(&t_ref.operator_account, &addr)
    }

    spec fun get_operator {
        aborts_if !spec_exists_config(addr);
        ensures result == spec_get_operator(addr);
    }

    /// Get consensus_pubkey from Config
    /// Never aborts
    public fun get_consensus_pubkey(config_ref: &Config): &vector<u8> {
        &config_ref.consensus_pubkey
    }

    /// Get validator's network identity pubkey from Config
    /// Never aborts
    public fun get_validator_network_identity_pubkey(config_ref: &Config): &vector<u8> {
        &config_ref.validator_network_identity_pubkey
    }

    /// Get validator's network address from Config
    /// Never aborts
    public fun get_validator_network_address(config_ref: &Config): &vector<u8> {
        &config_ref.validator_network_address
    }

    spec module {
        pragma verify = true, aborts_if_is_strict = true;
    }
}
}
