address 0x0:

module ValidatorConfig {

    use 0x0::Transaction;
    // TODO(philiphayes): We should probably enforce a max length for these fields

    struct Config {
        consensus_pubkey: vector<u8>,
        validator_network_signing_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>,
    }

    // A current or prospective validator should publish one of these under their address
    resource struct T {
        config: Config,
    }

    // Returns true if addr has a published ValidatorConfig::T resource
    public fun has(addr: address): bool {
        exists<T>(addr)
    }

    // The following are public accessors for retrieving config information about Validators

    // Retrieve a read-only instance of a specific accounts ValidatorConfig::T.config
    public fun config(addr: address): Config acquires T {
      *&borrow_global<T>(addr).config
    }

    // Public accessor for consensus_pubkey
    public fun consensus_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.consensus_pubkey
    }

    // Public accessor for validator_network_signing_pubkey
    public fun validator_network_signing_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.validator_network_signing_pubkey
    }

    // Public accessor for validator_network_identity_pubkey
    public fun validator_network_identity_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.validator_network_identity_pubkey
    }

    // Public accessor for validator_network_address
    public fun validator_network_address(config_ref: &Config): vector<u8> {
        *&config_ref.validator_network_address
    }

    // Public accessor for fullnodes_network_identity_pubkey
    public fun fullnodes_network_identity_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.fullnodes_network_identity_pubkey
    }

    // Public accessor for fullnodes_network_address
    public fun fullnodes_network_address(config_ref: &Config): vector<u8> {
        *&config_ref.fullnodes_network_address
    }

    // The following are self methods for initializing and maintaining a Validator's config

    // Register the transaction sender as a candidate validator by creating a ValidatorConfig
    // resource under their account
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
                     consensus_pubkey,
                     validator_network_signing_pubkey,
                     validator_network_identity_pubkey,
                     validator_network_address,
                     fullnodes_network_identity_pubkey,
                     fullnodes_network_address,
                }
            }
        );
    }

    // Rotate a validator candidate's consensus public key. The change will not take effect until
    // the next reconfiguration.
    public fun rotate_consensus_pubkey(consensus_pubkey: vector<u8>) acquires T {
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        let key_ref = &mut t_ref.config.consensus_pubkey;
        *key_ref = consensus_pubkey;
    }

    // TODO(philiphayes): fill out the rest of the rotate methods

    // Rotate the network public key for validator discovery. This change will be
    // committed in the next reconfiguration.
    public fun rotate_validator_network_identity_pubkey(
        validator_network_identity_pubkey: vector<u8>
    ) acquires T {
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        let key_ref = &mut t_ref.config.validator_network_identity_pubkey;
        *key_ref = validator_network_identity_pubkey;
    }

    // Rotate the network address for validator discovery. This change will be
    // committed in the next reconfiguration.
    public fun rotate_validator_network_address(
        validator_network_address: vector<u8>
    ) acquires T {
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        let key_ref = &mut t_ref.config.validator_network_address;
        *key_ref = validator_network_address;
    }
}
