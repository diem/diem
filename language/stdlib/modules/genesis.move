// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x0 {
module Genesis {
    use 0x0::Association;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Event;
    use 0x0::LBR;
    use 0x0::Libra;
    use 0x0::LibraAccount;
    use 0x0::LibraBlock;
    use 0x0::LibraConfig;
    use 0x0::LibraSystem;
    use 0x0::LibraTimestamp;
    use 0x0::LibraTransactionTimeout;
    use 0x0::LibraVersion;
    use 0x0::LibraWriteSetManager;
    use 0x0::Signer;
    use 0x0::Testnet;
    use 0x0::TransactionFee;
    use 0x0::Unhosted;

    fun initialize(
        association: &signer,
        config_account: &signer,
        tc_addr: address,
        tc_auth_key_prefix: vector<u8>,
        genesis_auth_key: vector<u8>
    ) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000000";

        // Association/cap setup
        Association::initialize(association);
        Association::grant_privilege<Libra::AddCurrency>(association);

        // On-chain config setup
        Event::publish_generator(config_account);
        LibraConfig::initialize(config_account, association);

        // Currency setup
        Libra::initialize(config_account);

        // Set that this is testnet
        Testnet::initialize();

        // Event and currency setup
        Event::publish_generator(association);
        let (coin1_mint_cap, coin1_burn_cap) = Coin1::initialize(association);
        let (coin2_mint_cap, coin2_burn_cap) = Coin2::initialize(association);
        LBR::initialize(association);

        LibraAccount::initialize(association);
        Unhosted::publish_global_limits_definition();
        LibraAccount::create_genesis_account<LBR::T>(
            Signer::address_of(association),
            copy dummy_auth_key_prefix,
        );
        Libra::grant_mint_capability_to_association<Coin1::T>(association);
        Libra::grant_mint_capability_to_association<Coin2::T>(association);

        // Register transaction fee accounts
        LibraAccount::create_testnet_account<LBR::T>(0xFEE, copy dummy_auth_key_prefix);


        // Create the treasury compliance account
        LibraAccount::create_treasury_compliance_account<LBR::T>(
            tc_addr,
            tc_auth_key_prefix,
            coin1_mint_cap,
            coin1_burn_cap,
            coin2_mint_cap,
            coin2_burn_cap,
        );

        // Create the config account
        LibraAccount::create_genesis_account<LBR::T>(
            LibraConfig::default_config_address(),
            dummy_auth_key_prefix
        );

        LibraTransactionTimeout::initialize(association);
        LibraSystem::initialize_validator_set(config_account);
        LibraVersion::initialize(config_account);

        LibraBlock::initialize_block_metadata(association);
        LibraWriteSetManager::initialize(association);
        LibraTimestamp::initialize(association);
        LibraAccount::rotate_authentication_key(genesis_auth_key);
    }

    // TODO: use signer for this and combine with the above once add_currency and
    // initialize_transaction accept a signer parameter
    fun initialize_txn_fee_account(auth_key: vector<u8>) {
        LibraAccount::add_currency<Coin1::T>();
        LibraAccount::add_currency<Coin2::T>();
        TransactionFee::initialize_transaction_fees();
        LibraAccount::rotate_authentication_key(auth_key);
    }

}
}
