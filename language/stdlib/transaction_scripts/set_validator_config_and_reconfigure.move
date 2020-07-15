script {
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;

    /// Set validator's config and updates the config in the validator set.
    /// NewEpochEvent is emitted.
    fun set_validator_config_and_reconfigure(
        account: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>,
    ) {
        ValidatorConfig::set_config(
            account,
            validator_account,
            consensus_pubkey,
            validator_network_identity_pubkey,
            validator_network_address,
            fullnodes_network_identity_pubkey,
            fullnodes_network_address
        );
        LibraSystem::update_config_and_reconfigure(account, validator_account);
     }
}
