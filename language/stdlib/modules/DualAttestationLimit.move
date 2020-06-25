address 0x1 {

module DualAttestationLimit {
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Roles::{Self, Capability};

    resource struct UpdateDualAttestationThreshold {}

    struct DualAttestationLimit {
        micro_lbr_limit: u64,
    }

    // A wrapped capability to allow updating of limit on-chain config by Blessed
    resource struct ModifyLimitCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::DualAttestationLimit>,
    }

    /// Will fail if `account` does not have the treasury-compliance role
    public fun grant_privileges(account: &signer) {
        Roles::add_privilege_to_account_treasury_compliance_role(account, UpdateDualAttestationThreshold{});
    }

    /// Travel rule limit set during genesis
    public fun initialize(
        account: &signer,
        tc_account: &signer,
        create_on_chain_config_capability: &Capability<CreateOnChainConfig>,
    ) {
        assert(Signer::address_of(account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
        let cap = LibraConfig::publish_new_config_with_capability<DualAttestationLimit>(
            account,
            create_on_chain_config_capability,
            DualAttestationLimit { micro_lbr_limit: 1000000 },
        );
        move_to(tc_account, ModifyLimitCapability { cap })
    }

    // Anyone can fetch the current set limit
    public fun get_cur_microlibra_limit(): u64 {
        LibraConfig::get<DualAttestationLimit>().micro_lbr_limit
    }

    public fun set_microlibra_limit(
        _: &Capability<UpdateDualAttestationThreshold>,
        tc_address: address,
        micro_lbr_limit: u64
    ) acquires ModifyLimitCapability {
        assert(
            micro_lbr_limit >= 1000,
            4
        );
        let modify_cap = &borrow_global<ModifyLimitCapability>(tc_address).cap;
        LibraConfig::set_with_capability<DualAttestationLimit>(
            modify_cap,
            DualAttestationLimit { micro_lbr_limit },
        );
    }
}

}
