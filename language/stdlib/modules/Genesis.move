// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x1 {
module Genesis {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::Coin1;
    use 0x1::Coin2;
    use 0x1::DualAttestationLimit;
    use 0x1::Event;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, RegisterNewCurrency};
    use 0x1::LibraAccount;
    use 0x1::LibraBlock;
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::LibraSystem;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionTimeout;
    use 0x1::LibraVersion;
    use 0x1::LibraWriteSetManager;
    use 0x1::Signer;
    use 0x1::Testnet;
    use 0x1::TransactionFee;
    use 0x1::Roles::{Self, AssociationRootRole, TreasuryComplianceRole};
    use 0x1::SlidingNonce::{Self, CreateSlidingNonce};
    use 0x1::ValidatorConfig;
    use 0x1::LibraVMConfig;


    fun initialize(
        association: &signer,
        config_account: &signer,
        fee_account: &signer,
        core_code_account: &signer,
        tc_account: &signer,
        tc_addr: address,
        genesis_auth_key: vector<u8>,
        publishing_option: vector<u8>,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
    ) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000000";

        Roles::grant_root_association_role(association);
        LibraConfig::grant_privileges(association);
        LibraAccount::grant_association_privileges(association);
        SlidingNonce::grant_privileges(association);
        ValidatorConfig::grant_privileges(association);
        let assoc_root_capability = Roles::extract_privilege_to_capability<AssociationRootRole>(association);
        let create_config_capability = Roles::extract_privilege_to_capability<CreateOnChainConfig>(association);
        let create_sliding_nonce_capability = Roles::extract_privilege_to_capability<CreateSlidingNonce>(association);

        Roles::grant_treasury_compliance_role(tc_account, &assoc_root_capability);
        LibraAccount::grant_treasury_compliance_privileges(tc_account);
        Libra::grant_privileges(tc_account);
        DualAttestationLimit::grant_privileges(tc_account);
        let currency_registration_capability = Roles::extract_privilege_to_capability<RegisterNewCurrency>(tc_account);
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);

        // On-chain config setup
        Event::publish_generator(config_account);
        LibraConfig::initialize(
            config_account,
            &create_config_capability,
        );

        // Currency setup
        Libra::initialize(config_account, &create_config_capability);

        // Set that this is testnet
        Testnet::initialize(association);

        // Event and currency setup
        Event::publish_generator(association);
        let (coin1_mint_cap, coin1_burn_cap) = Coin1::initialize(
            association,
            &currency_registration_capability,
        );
        let (coin2_mint_cap, coin2_burn_cap) = Coin2::initialize(
            association,
            &currency_registration_capability,
        );
        LBR::initialize(
            association,
            &currency_registration_capability,
            &tc_capability,
        );

        LibraAccount::initialize(association);
        LibraAccount::create_root_association_account<LBR>(
            Signer::address_of(association),
            copy dummy_auth_key_prefix,
        );

        LibraAccount::create_testnet_account<LBR>(
            association,
            &assoc_root_capability,
            Signer::address_of(core_code_account),
            copy dummy_auth_key_prefix,
        );

        // Register transaction fee accounts
        TransactionFee::initialize(
            association,
            fee_account,
            &assoc_root_capability,
            &tc_capability,
            copy dummy_auth_key_prefix
        );

        // Create the treasury compliance account
        LibraAccount::create_treasury_compliance_account<LBR>(
            &assoc_root_capability,
            &tc_capability,
            &create_sliding_nonce_capability,
            tc_addr,
            copy dummy_auth_key_prefix,
            coin1_mint_cap,
            coin1_burn_cap,
            coin2_mint_cap,
            coin2_burn_cap,
        );
        AccountLimits::publish_unrestricted_limits(tc_account);
        AccountLimits::certify_limits_definition(&tc_capability, tc_addr);

        // Create the config account
        LibraAccount::create_config_account<LBR>(
            association,
            &create_config_capability,
            CoreAddresses::DEFAULT_CONFIG_ADDRESS(),
            dummy_auth_key_prefix
        );

        LibraTransactionTimeout::initialize(association);
        LibraSystem::initialize_validator_set(config_account, &create_config_capability);
        LibraVersion::initialize(config_account, &create_config_capability);

        DualAttestationLimit::initialize(config_account, tc_account, &create_config_capability);
        LibraBlock::initialize_block_metadata(association);
        LibraWriteSetManager::initialize(association);
        LibraTimestamp::initialize(association);

        let assoc_rotate_key_cap = LibraAccount::extract_key_rotation_capability(association);
        LibraAccount::rotate_authentication_key(&assoc_rotate_key_cap, copy genesis_auth_key);
        LibraAccount::restore_key_rotation_capability(assoc_rotate_key_cap);

        LibraVMConfig::initialize(
            config_account,
            association,
            &create_config_capability,
            publishing_option,
            instruction_schedule,
            native_schedule,
        );

        LibraConfig::grant_privileges_for_config_TESTNET_HACK_REMOVE(config_account);

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

        // Restore privileges
        Roles::restore_capability_to_privilege(association, create_config_capability);
        Roles::restore_capability_to_privilege(association, create_sliding_nonce_capability);
        Roles::restore_capability_to_privilege(association, assoc_root_capability);

        Roles::restore_capability_to_privilege(tc_account, currency_registration_capability);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }

}
}
