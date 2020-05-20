// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x0 {
module Genesis {
    use 0x0::AccountTrack;
    use 0x0::AccountType;
    use 0x0::Association;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Empty;
    use 0x0::Event;
    use 0x0::LBR;
    use 0x0::Libra;
    use 0x0::LibraAccount;
    use 0x0::LibraBlock;
    use 0x0::LibraConfig;
    use 0x0::LibraTransactionTimeout;
    use 0x0::LibraWriteSetManager;
    use 0x0::TransactionFee;
    use 0x0::Unhosted;
    use 0x0::VASP;
    use 0x0::Testnet;

    fun initialize_association(association_root_addr: address) {
        // Association/cap setup
        Association::initialize();
        Association::apply_for_privilege<Libra::AddCurrency>();
        Association::grant_privilege<Libra::AddCurrency>(association_root_addr);
    }

    fun initialize_accounts(
        association_root_addr: address,
        burn_addr: address,
        genesis_auth_key: vector<u8>,
    ) {
        let dummy_auth_key = x"00000000000000000000000000000000";

        // Set that this is testnet
        Testnet::initialize();

        // Event and currency setup
        Event::grant_event_generator();
        Coin1::initialize();
        Coin2::initialize();
        LBR::initialize();
        LibraConfig::apply_for_creator_privilege();
        LibraConfig::grant_creator_privilege(0xA550C18);

        //// Account type setup
        AccountType::register<Unhosted::T>();
        AccountType::register<Empty::T>();
        VASP::initialize();

        AccountTrack::initialize();
        LibraAccount::initialize();
        Unhosted::publish_global_limits_definition();
        LibraAccount::create_account<LBR::T>(
            association_root_addr,
            copy dummy_auth_key,
        );

        // Create the burn account
        LibraAccount::create_account<LBR::T>(burn_addr, copy dummy_auth_key);

        // Register transaction fee accounts
        // TODO: Need to convert this to a different account type than unhosted.
        LibraAccount::create_testnet_account<LBR::T>(0xFEE, copy dummy_auth_key);

        // Create the config account
        LibraAccount::create_account<LBR::T>(LibraConfig::default_config_address(), dummy_auth_key);

        LibraTransactionTimeout::initialize();
        LibraBlock::initialize_block_metadata();
        LibraWriteSetManager::initialize();
        LibraAccount::rotate_authentication_key(genesis_auth_key);
    }

    fun initalize_tc_account() {
        Association::apply_for_association();
        Association::apply_for_privilege<LibraAccount::FreezingPrivilege>();
    }

    fun grant_tc_account(tc_addr: address) {
        Association::grant_association_address(tc_addr);
        Association::grant_privilege<LibraAccount::FreezingPrivilege>(tc_addr);
    }

    fun grant_tc_capabilities_for_sender(auth_key: vector<u8>) {
        Libra::grant_burn_capability_for_sender<Coin1::T>();
        Libra::grant_burn_capability_for_sender<Coin2::T>();
        Libra::grant_burn_capability_for_sender<LBR::T>();
        LibraAccount::rotate_authentication_key(auth_key);
    }

    fun initialize_txn_fee_account(auth_key: vector<u8>) {
        LibraAccount::add_currency<Coin1::T>();
        LibraAccount::add_currency<Coin2::T>();
        TransactionFee::initialize_transaction_fees();
        LibraAccount::rotate_authentication_key(auth_key);
    }

    fun initialize_config() {
        Event::grant_event_generator();
        LibraConfig::initialize_configuration();
        LibraConfig::apply_for_creator_privilege();
    }
}
}
