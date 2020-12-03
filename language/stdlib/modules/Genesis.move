address 0x1 {

/// The `Genesis` module defines the Move initialization entry point of the Diem framework
/// when executing from a fresh state.
///
/// > TODO: Currently there are a few additional functions called from Rust during genesis.
/// > Document which these are and in which order they are called.
module Genesis {
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

    /// Initializes the Diem framework.
    fun initialize(
        dr_account: &signer,
        tc_account: &signer,
        dr_auth_key: vector<u8>,
        tc_auth_key: vector<u8>,
        initial_script_allow_list: vector<vector<u8>>,
        is_open_module: bool,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        chain_id: u8,
    ) {

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

        // After we have called this function, all invariants which are guarded by
        // `DiemTimestamp::is_operating() ==> ...` will become active and a verification condition.
        // See also discussion at function specification.
        DiemTimestamp::set_time_has_started(dr_account);
    }

    /// For verification of genesis, the goal is to prove that all the invariants which
    /// become active after the end of this function hold. This cannot be achieved with
    /// modular verification as we do in regular continuous testing. Rather, this module must
    /// be verified **together** with the module(s) which provides the invariant.
    ///
    /// > TODO: currently verifying this module together with modules providing invariants
    /// > (see above) times out. This can likely be solved by making more of the initialize
    /// > functions called by this function opaque, and prove the according invariants locally to
    /// > each module.
    spec fun initialize {
        /// Assume that this is called in genesis state (no timestamp).
        requires DiemTimestamp::is_genesis();
    }

}
}
