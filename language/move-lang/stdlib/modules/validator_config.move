address 0x0:

module ValidatorConfig {
    use 0x0::Transaction;

    struct Config {
        consensus_pubkey: bytearray,
        network_identity_pubkey: bytearray,
        network_signing_pubkey: bytearray,
    }

    // A current or prospective validator should publish one of these under their address
    resource struct T { config: Self::Config }

    // The following are public accessors for retrieving config information about Validators

    // Retrieve a read-only instance of a specific accounts ValidatorConfig::T::config
    public config(addr: address): Config acquires T {
        *&borrow_global<T>(addr).config
    }

    // Public accessor for consensus_pubkey
    public consensus_pubkey(config: &Config): bytearray {
        config.consensus_pubkey
    }

    // Public accessor for network_identity_pubkey
    public network_identity_pubkey(config: &Config): bytearray {
        config.network_identity_pubkey
    }

    // Public accessor for network_signing_pubkey
    public network_signing_pubkey(config: &Config): bytearray {
        config.network_signing_pubkey
    }

    // The following are self methods for initializing and maintaining a Validator's config

    // Register the transaction sender as a candidate validator by creating a ValidatorConfig
    // resource under their account
    public register_candidate_validator(
        network_signing_pubkey: bytearray,
        network_identity_pubkey: bytearray,
        consensus_pubkey: bytearray,
    ) {
        move_to_sender(
            T {
                config: Config {
                    consensus_pubkey,
                    network_identity_pubkey,
                    network_signing_pubkey,
                }
            }
        )
    }

    // Rotate a validator candidate's consensus public key. The change will not take effect until
    // the next reconfiguration.
    public rotate_consensus_pubkey(consensus_pubkey: bytearray) acquires T {
        borrow_global_mut<T>(Transaction::sender()).config.consensus_pubkey = consensus_pubkey;
    }

}
