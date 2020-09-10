script {
    use 0x1::ValidatorConfig;

    /// Set validator's config locally.
    /// Does not emit NewEpochEvent, the config is NOT changed in the validator set.
    fun register_validator_config(
        account: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_addresses: vector<u8>,
        fullnode_network_addresses: vector<u8>,
    ) {
        ValidatorConfig::set_config(
            account,
            validator_account,
            consensus_pubkey,
            validator_network_addresses,
            fullnode_network_addresses
        );
     }
}
