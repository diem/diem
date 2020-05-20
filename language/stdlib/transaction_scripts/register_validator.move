script {
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::LBR;
    use 0x0::LibraAccount;
    use 0x0::LibraSystem;
    use 0x0::Signer;
    use 0x0::Transaction;
    use 0x0::ValidatorConfig;

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
        ValidatorConfig::set_config(
            account,
            Transaction::sender(),
            consensus_pubkey,
            validator_network_identity_pubkey,
            validator_network_address,
            fullnodes_network_identity_pubkey,
            fullnodes_network_address
        );
        LibraSystem::add_validator(Transaction::sender());


        let sender = Signer::address_of(account);
        // Validating nodes need to accept all currencies in order to receive txn fees
        if (!LibraAccount::accepts_currency<Coin1::T>(sender)) {
            LibraAccount::add_currency<Coin1::T>(account)
        };
        if (!LibraAccount::accepts_currency<Coin2::T>(sender)) {
            LibraAccount::add_currency<Coin2::T>(account)
        };
        if (!LibraAccount::accepts_currency<LBR::T>(sender)) {
            LibraAccount::add_currency<LBR::T>(account)
        };
    }
}
