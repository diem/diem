script {
    use 0x1::LibraSystem;
    use 0x1::Signer;
    use 0x1::ValidatorConfig;

    // Here the sender's address should already be certified as both a Validator.
    // This tx sets the config and adds the validator to the Validator Set.
    fun main(
        account: &signer,
        consensus_pubkey: vector<u8>,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>,
    ) {
        let sender = Signer::address_of(account);
        ValidatorConfig::set_config(
            account,
            sender,
            consensus_pubkey,
            validator_network_identity_pubkey,
            validator_network_address,
            fullnodes_network_identity_pubkey,
            fullnodes_network_address
        );
        LibraSystem::add_validator(account, sender)
}
}
