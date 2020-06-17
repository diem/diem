// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x1 {
module Genesis {
    use 0x1::AccountLimits;
    use 0x1::Association::{Self, PublishModule};
    use 0x1::CoreAddresses;
    use 0x1::Coin1;
    use 0x1::Coin2;
    use 0x1::DualAttestationLimit;
    use 0x1::Event;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, AddCurrency};
    use 0x1::LibraAccount;
    use 0x1::LibraBlock;
    use 0x1::LibraConfig;
    use 0x1::LibraSystem;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionTimeout;
    use 0x1::LibraVersion;
    use 0x1::LibraWriteSetManager;
    use 0x1::Signer;
    use 0x1::Testnet;
    use 0x1::TransactionFee;

    fun initialize(
        association: &signer,
        config_account: &signer,
        fee_account: &signer,
        core_code_account: &signer,
        tc_account: &signer,
        tc_addr: address,
        genesis_auth_key: vector<u8>,
    ) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000000";

        // Association root setup
        Association::initialize(association);
        Association::grant_privilege<AddCurrency>(association, association);
        Association::grant_privilege<PublishModule>(association, association);

        // On-chain config setup
        Event::publish_generator(config_account);
        LibraConfig::initialize(config_account, association);

        // Currency setup
        Libra::initialize(config_account);

        // Set that this is testnet
        Testnet::initialize(association);

        // Event and currency setup
        Event::publish_generator(association);
        let (coin1_mint_cap, coin1_burn_cap) = Coin1::initialize(association);
        let (coin2_mint_cap, coin2_burn_cap) = Coin2::initialize(association);
        LBR::initialize(association);

        LibraAccount::initialize(association);
        LibraAccount::create_genesis_account<LBR>(
            Signer::address_of(association),
            copy dummy_auth_key_prefix,
        );

        Event::publish_generator(core_code_account);
        LibraAccount::create_genesis_account<LBR>(
            Signer::address_of(core_code_account),
            copy dummy_auth_key_prefix,
        );

        // Register transaction fee accounts
        TransactionFee::initialize(association, fee_account, copy dummy_auth_key_prefix);

        // Create the treasury compliance account
        LibraAccount::create_treasury_compliance_account<LBR>(
            association,
            tc_addr,
            copy dummy_auth_key_prefix,
            coin1_mint_cap,
            coin1_burn_cap,
            coin2_mint_cap,
            coin2_burn_cap,
        );
        AccountLimits::publish_unrestricted_limits(tc_account);
        AccountLimits::certify_limits_definition(tc_account, tc_addr);

        // Create the config account
        LibraAccount::create_genesis_account<LBR>(
            CoreAddresses::DEFAULT_CONFIG_ADDRESS(),
            dummy_auth_key_prefix
        );

        LibraTransactionTimeout::initialize(association);
        LibraSystem::initialize_validator_set(config_account);
        LibraVersion::initialize(config_account);

        DualAttestationLimit::initialize(config_account, tc_account);
        LibraBlock::initialize_block_metadata(association);
        LibraWriteSetManager::initialize(association);
        LibraTimestamp::initialize(association);

        let assoc_rotate_key_cap = LibraAccount::extract_key_rotation_capability(association);
        LibraAccount::rotate_authentication_key(&assoc_rotate_key_cap, copy genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(assoc_rotate_key_cap);

        let config_rotate_key_cap = LibraAccount::extract_key_rotation_capability(config_account);
        LibraAccount::rotate_authentication_key(&config_rotate_key_cap, copy genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(config_rotate_key_cap);

        let fee_rotate_key_cap = LibraAccount::extract_key_rotation_capability(fee_account);
        LibraAccount::rotate_authentication_key(&fee_rotate_key_cap, copy genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(fee_rotate_key_cap);

        let tc_rotate_key_cap = LibraAccount::extract_key_rotation_capability(tc_account);
        LibraAccount::rotate_authentication_key(&tc_rotate_key_cap, copy genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(tc_rotate_key_cap);

        let core_code_rotate_key_cap = LibraAccount::extract_key_rotation_capability(core_code_account);
        LibraAccount::rotate_authentication_key(&core_code_rotate_key_cap, genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(core_code_rotate_key_cap);
    }

}
}
