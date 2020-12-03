script {
    use 0x1::AccountFreezing;
    use 0x1::ChainId;
    use 0x1::XUS;
    use 0x1::DualAttestation;
    use 0x1::XDX;
    use 0x1::Diem;
    use 0x1::DiemAccount;
    use 0x1::DiemBlock;
    use 0x1::DiemConfig;
    use 0x1::DiemSystem;
    use 0x1::DiemTimestamp;
    use 0x1::DiemTransactionPublishingOption;
    use 0x1::DiemVersion;
    use 0x1::TransactionFee;
    use 0x1::DiemVMConfig;
    use 0x1::Vector;

    fun initialize(
        dr_account: &signer,
        tc_account: &signer,
    ) {
        let dummy_auth_key = x"0000000000000000000000000000000000000000000000000000000000000000";
        let dr_auth_key = copy dummy_auth_key;
        let tc_auth_key = dummy_auth_key;

        // no script allowlist + allow open publishing
        let initial_script_allow_list = Vector::empty();
        let is_open_module = true;
        let instruction_schedule = Vector::empty();
        let native_schedule = Vector::empty();
        let chain_id = 0;

        DiemAccount::initialize(dr_account, x"00000000000000000000000000000000");

        ChainId::initialize(dr_account, chain_id);

        // On-chain config setup
        DiemConfig::initialize(dr_account);

        // Currency setup
        Diem::initialize(dr_account);

        // Currency setup
        XUS::initialize(dr_account, tc_account);

        XDX::initialize(
            dr_account,
            tc_account,
        );

        AccountFreezing::initialize(dr_account);

        TransactionFee::initialize(tc_account);

        DiemSystem::initialize_validator_set(
            dr_account,
        );
        DiemVersion::initialize(
            dr_account,
        );
        DualAttestation::initialize(
            dr_account,
        );
        DiemBlock::initialize_block_metadata(dr_account);

        let dr_rotate_key_cap = DiemAccount::extract_key_rotation_capability(dr_account);
        DiemAccount::rotate_authentication_key(&dr_rotate_key_cap, dr_auth_key);
        DiemAccount::restore_key_rotation_capability(dr_rotate_key_cap);

        DiemTransactionPublishingOption::initialize(
            dr_account,
            initial_script_allow_list,
            is_open_module,
        );

        DiemVMConfig::initialize(
            dr_account,
            instruction_schedule,
            native_schedule,
        );

        let tc_rotate_key_cap = DiemAccount::extract_key_rotation_capability(tc_account);
        DiemAccount::rotate_authentication_key(&tc_rotate_key_cap, tc_auth_key);
        DiemAccount::restore_key_rotation_capability(tc_rotate_key_cap);
        DiemTimestamp::set_time_has_started(dr_account);
    }

}
