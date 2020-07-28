script {
    use 0x1::ValidatorConfig;

    /// Set validator's config locally.
    /// Does not emit NewEpochEvent, the config is NOT changed in the validator set.
    /// TODO(valerini): rename to register_validator_config to avoid confusion with
    ///                 set_validator_config_and_reconfigure script.
    fun set_validator_config(
        account: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<vector<u8>>,
        full_node_network_addresses: vector<vector<u8>>,
    ) {
        ValidatorConfig::set_config(
            account,
            validator_account,
            consensus_pubkey,
            validator_network_addresses,
            full_node_network_addresses
        );
     }
}
