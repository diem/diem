// Error codes:
// 1100 -> OPERATOR_ACCOUNT_DOES_NOT_EXIST
// 1101 -> INVALID_TRANSACTION_SENDER
// 1102 -> VALIDATOR_NOT_CERTIFIED
// 1103 -> VALIDATOR_OPERATOR_NOT_CERTIFIED
// 1104 -> VALIDATOR_CONFIG_IS_NOT_SET
// 1105 -> VALIDATOR_OPERATOR_IS_NOT_SET
// 1106 -> VALIDATOR_RESOURCE_DOES_NOT_EXIST
// 1107 -> INVALID_NET
address 0x0 {

module ValidatorConfig {
    use 0x0::Option;
    use 0x0::Transaction;
    use 0x0::Signer;

    struct Config {
        consensus_pubkey: vector<u8>,
        // TODO(philiphayes): restructure
        //   1) make validator_network_address[es] of type vector<vector<u8>>,
        //   2) make fullnodes_network_address[es] of type vector<vector<u8>>,
        //   3) remove validator_network_identity_pubkey
        //   4) remove full_node_network_identity_pubkey
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        full_node_network_identity_pubkey: vector<u8>,
        full_node_network_address: vector<u8>,
    }

    resource struct T {
        // set and rotated by the operator_account
        config: Option::T<Config>,
        operator_account: Option::T<address>,
    }

    // TODO(valerini): add events here

    ///////////////////////////////////////////////////////////////////////////
    // Validator setup methods
    ///////////////////////////////////////////////////////////////////////////

    public fun publish(signer: &signer) {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1101);
        move_to(signer, T {
            config: Option::none(),
            operator_account: Option::none(),
        });
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig::T owner
    ///////////////////////////////////////////////////////////////////////////

    // Sets a new operator account, preserving the old config.
    public fun set_operator(operator_account: address) acquires T {
        (borrow_global_mut<T>(Transaction::sender())).operator_account = Option::some(operator_account);
    }

    // Removes an operator account, setting a corresponding field to Opetion::none.
    // The old config is preserved.
    public fun remove_operator() acquires T {
        // Config field remains set
        (borrow_global_mut<T>(Transaction::sender())).operator_account = Option::none();
    }

    ///////////////////////////////////////////////////////////////////////////
    // Rotation methods callable by ValidatorConfig::T.operator_account
    ///////////////////////////////////////////////////////////////////////////

    // Rotate the config in the validator_account
    // NB! Once the config is set, it can not go to Option::none - this is crucial for validity
    //     of the LibraSystem's code
    public fun set_config(
        signer: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        full_node_network_identity_pubkey: vector<u8>,
        full_node_network_address: vector<u8>,
    ) acquires T {
        Transaction::assert(Signer::address_of(signer) ==
                            get_operator(validator_account), 1101);
        // TODO(valerini): verify the validity of new_config.consensus_pubkey and
        // the proof of posession
        let t_ref = borrow_global_mut<T>(validator_account);
        t_ref.config = Option::some(Config {
            consensus_pubkey,
            validator_network_identity_pubkey,
            validator_network_address,
            full_node_network_identity_pubkey,
            full_node_network_address,
        });
    }

    // TODO(valerini): to remove and call into set_config instead
    public fun set_consensus_pubkey(
        validator_account: address,
        consensus_pubkey: vector<u8>,
    ) acquires T {
        Transaction::assert(Transaction::sender() ==
                            get_operator(validator_account), 1101);
        let t_config_ref = Option::borrow_mut(&mut borrow_global_mut<T>(validator_account).config);
        t_config_ref.consensus_pubkey = consensus_pubkey;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs: getters
    ///////////////////////////////////////////////////////////////////////////

    // Returns true if all of the following is true:
    // 1) there is a ValidatorConfig resource under the address, and
    // 2) the config is set, and
    // NB! currently we do not require the the operator_account to be set
    public fun is_valid(addr: address): bool acquires T {
        exists<T>(addr) && Option::is_some(&borrow_global<T>(addr).config)
    }

    // Get Config
    // Aborts if there is no ValidatorConfig::T resource of if its config is empty
    public fun get_config(addr: address): Config acquires T {
        Transaction::assert(exists<T>(addr), 1106);
        let config = &borrow_global<T>(addr).config;
        *Option::borrow(config)
    }

    // Get operator's account
    // Aborts if there is no ValidatorConfig::T resouce, if its operator_account is
    // empty, returns the input
    public fun get_operator(addr: address): address acquires T {
        Transaction::assert(exists<T>(addr), 1106);
        let t_ref = borrow_global<T>(addr);
        *Option::borrow_with_default(&t_ref.operator_account, &addr)
    }

    // Get consensus_pubkey from Config
    // Never aborts
    public fun get_consensus_pubkey(config_ref: &Config): &vector<u8> {
        &config_ref.consensus_pubkey
    }

    // Get validator's network identity pubkey from Config
    // Never aborts
    public fun get_validator_network_identity_pubkey(config_ref: &Config): &vector<u8> {
        &config_ref.validator_network_identity_pubkey
    }

    // Get validator's network address from Config
    // Never aborts
    public fun get_validator_network_address(config_ref: &Config): &vector<u8> {
        &config_ref.validator_network_address
    }
}
}
