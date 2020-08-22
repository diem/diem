script {
    use 0x1::ValidatorConfig;

    /// Set validator's config locally.
    /// Does not emit NewEpochEvent, the config is NOT changed in the validator set.
    fun register_validator_config(
        account: &signer,
        validator_account: address,
        consensus_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_address: vector<u8>,
    ) {
        ValidatorConfig::set_config(
            account,
            validator_account,
            consensus_pubkey,
            validator_network_address,
            fullnodes_network_address
        );
     }
}
