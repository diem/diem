address 0x0 {

module ValidatorConfig {
    use 0x0::LibraAccount;
    use 0x0::Option;
    use 0x0::Transaction;

    struct Config {
        consensus_pubkey: vector<u8>,
        // TODO(philiphayes): restructure
        network_signing_pubkey: vector<u8>,
        network_identity_pubkey: vector<u8>,
    }

    struct DiscoveryConfig {
        // TODO(philiphayes): restructure
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>,
    }

    // A current or prospective validator should publish one of these under their accounts.
    // If delegation is enabled, only the delegated_account can change the config.
    // If delegation is disabled, only the owner of this resource can change the config.
    // The entity that can change the config is called a "validator operator".
    // The owner of this resource can enable delegation and set delegated account.
    // The owner can also disable delegation at any time.
    resource struct T {
        config: Config,
        discovery_config: DiscoveryConfig,
        delegated_account: Option::T<address>,
    }

    // TODO(valerini): add events here

    // Returns true if addr has a published ValidatorConfig::T resource
    public fun has(addr: address): bool {
        exists<T>(addr)
    }

    // Get Config
    public fun get_config(addr: address): Config acquires T {
        *&borrow_global<T>(addr).config
    }

    // Get DiscoveryConfig
    public fun get_discovery_config(addr: address): DiscoveryConfig acquires T {
        *&borrow_global<T>(addr).discovery_config
    }

    // Get consensus_pubkey from Config
    public fun get_consensus_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.consensus_pubkey
    }

    // Returns the address of the entity who is now in charge of managing the config.
    public fun get_validator_operator_account(addr: address): address acquires T {
        Option::get_with_default(&borrow_global<T>(addr).delegated_account, addr)
    }

    // Register the transaction sender as a candidate validator by creating a ValidatorConfig
    // resource under their account. Note that only one such resource can be
    // instantiated under an account.
    public fun register_candidate_validator(
        consensus_pubkey: vector<u8>,
        validator_network_signing_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>) {

        move_to_sender<T>(
            T {
                config: Config {
                    consensus_pubkey: consensus_pubkey,
                    network_signing_pubkey: *&validator_network_signing_pubkey,
                    network_identity_pubkey: *&validator_network_identity_pubkey,
                },
                discovery_config: DiscoveryConfig {
                    validator_network_identity_pubkey,
                    validator_network_address,
                    fullnodes_network_identity_pubkey,
                    fullnodes_network_address,
                },
                delegated_account: Option::none()
            }
        );
    }

    // If the sender of the transaction has a ValidatorConfig::T resource,
    // this function will delegate management of the ValidatorConfig::T resource
    // to a delegated_account.
    public fun set_delegated_account(delegated_account: address) acquires T {
        Transaction::assert(LibraAccount::exists(delegated_account), 5);
        // check delegated address is different from transaction's sender
        Transaction::assert(delegated_account != Transaction::sender(), 6);
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        t_ref.delegated_account = Option::some(delegated_account)
    }

    // If the sender of the transaction has a ValidatorConfig::T resource,
    // this function will revoke management capabilities from the delegated_account account.
    public fun remove_delegated_account() acquires T {
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        t_ref.delegated_account = Option::none()
    }

    // Rotate validator's config.
    // Here validator_account - is the account of the validator whose
    // consensus_pubkey is going to be rotated.
    public fun rotate_consensus_pubkey(
        validator_account: address,
        new_consensus_pubkey: vector<u8>,
        // _proof: vector<u8>
    ) acquires T {
        let addr = get_validator_operator_account(validator_account);
        Transaction::assert(Transaction::sender() == addr, 1);

        // TODO(valerini): verify the proof of posession of new_consensus_secretkey

        let t_ref = borrow_global_mut<T>(validator_account);
        // Set the new key
        t_ref.config.consensus_pubkey = new_consensus_pubkey;
    }

    // Simplified arguments when the sender is the validators itself
    public fun rotate_consensus_pubkey_of_sender(new_consensus_pubkey: vector<u8>) acquires T {
        rotate_consensus_pubkey(Transaction::sender(), new_consensus_pubkey);
    }

    // TODO(philiphayes): add necessary rotation methods for discovery_config

    // Public accessor for validator's network_identity_pubkey
    public fun get_validator_network_identity_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.network_identity_pubkey
    }

    // Public accessor for validator_network_address
    public fun get_validator_network_address(config_ref: &DiscoveryConfig): vector<u8> {
        *&config_ref.validator_network_address
    }

    // Rotate the network public key for validator discovery. This change will be
    // committed in the next reconfiguration.
    public fun rotate_validator_network_identity_pubkey(
        validator_account: address,
        validator_network_identity_pubkey: vector<u8>
    ) acquires T {
        let addr = get_validator_operator_account(validator_account);
        Transaction::assert(Transaction::sender() == addr, 1);

        let t_ref = borrow_global_mut<T>(validator_account);
        t_ref.config.network_identity_pubkey = *&validator_network_identity_pubkey;
        t_ref.discovery_config.validator_network_identity_pubkey =
            validator_network_identity_pubkey;
    }

    // Rotate the network address for validator discovery. This change will be
    // committed in the next reconfiguration.
    public fun rotate_validator_network_address(
        validator_account: address,
        validator_network_address: vector<u8>
    ) acquires T {
        let addr = get_validator_operator_account(validator_account);
        Transaction::assert(Transaction::sender() == addr, 1);

        let t_ref = borrow_global_mut<T>(validator_account);
        t_ref.discovery_config.validator_network_address = validator_network_address;
    }
}
}
