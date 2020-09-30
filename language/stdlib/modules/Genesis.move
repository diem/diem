// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x1 {
module Genesis {
    use 0x1::AccountFreezing;
    use 0x1::ChainId;
    use 0x1::Coin1;
    use 0x1::Coin2;
    use 0x1::DualAttestation;
    use 0x1::LBR;
    use 0x1::Libra;
    use 0x1::LibraAccount;
    use 0x1::LibraBlock;
    use 0x1::LibraConfig;
    use 0x1::LibraSystem;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionPublishingOption;
    use 0x1::LibraVersion;
    use 0x1::TransactionFee;
    use 0x1::LibraVMConfig;

    fun initialize(
        lr_account: &signer,
        tc_account: &signer,
        lr_auth_key: vector<u8>,
        tc_auth_key: vector<u8>,
        initial_script_allow_list: vector<vector<u8>>,
        is_open_module: bool,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        chain_id: u8,
    ) {

        LibraAccount::initialize(lr_account, x"00000000000000000000000000000000");

        ChainId::initialize(lr_account, chain_id);

        // On-chain config setup
        LibraConfig::initialize(lr_account);

        // Currency setup
        Libra::initialize(lr_account);

        // Currency setup
        Coin1::initialize(lr_account, tc_account);
        Coin2::initialize(lr_account, tc_account);

        LBR::initialize(
            lr_account,
            tc_account,
        );

        AccountFreezing::initialize(lr_account);

        // Register transaction fee resource
        TransactionFee::initialize(
            lr_account,
            tc_account,
        );

        LibraSystem::initialize_validator_set(
            lr_account,
        );
        LibraVersion::initialize(
            lr_account,
        );
        DualAttestation::initialize(
            lr_account,
        );
        LibraBlock::initialize_block_metadata(lr_account);

        let lr_rotate_key_cap = LibraAccount::extract_key_rotation_capability(lr_account);
        LibraAccount::rotate_authentication_key(&lr_rotate_key_cap, lr_auth_key);
        LibraAccount::restore_key_rotation_capability(lr_rotate_key_cap);

        LibraTransactionPublishingOption::initialize(
            lr_account,
            initial_script_allow_list,
            is_open_module,
        );

        LibraVMConfig::initialize(
            lr_account,
            instruction_schedule,
            native_schedule,
        );

        let tc_rotate_key_cap = LibraAccount::extract_key_rotation_capability(tc_account);
        LibraAccount::rotate_authentication_key(&tc_rotate_key_cap, tc_auth_key);
        LibraAccount::restore_key_rotation_capability(tc_rotate_key_cap);
        LibraTimestamp::set_time_has_started(lr_account);
    }
    spec fun initialize {
        pragma verify = false; // TODO: times out
    }

}
}
