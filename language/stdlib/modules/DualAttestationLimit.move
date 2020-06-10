address 0x1 {

module DualAttestationLimit {
    use 0x1::LibraConfig;
    use 0x1::Signer;
    use 0x1::Association;
    use 0x1::CoreAddresses;

    struct DualAttestationLimit {
        micro_lbr_limit: u64,
    }

    // A wrapped capability to allow updating of limit on-chain config by Blessed
    resource struct ModifyLimitCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::DualAttestationLimit>,
    }

    /// Travel rule limit set during genesis
    public fun initialize(account: &signer, tc_account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::DEFAULT_CONFIG_ADDRESS(), 1);
        let cap = LibraConfig::publish_new_config_with_capability<DualAttestationLimit>(
            account,
            DualAttestationLimit { micro_lbr_limit: 1000000 },
        );
        move_to(tc_account, ModifyLimitCapability { cap })
    }

    // Anyone can fetch the current set limit
    public fun get_cur_microlibra_limit(): u64 {
        LibraConfig::get<DualAttestationLimit>().micro_lbr_limit
    }

    public fun set_microlibra_limit(tc_account: &signer, micro_lbr_limit: u64) acquires ModifyLimitCapability {
        assert(
            micro_lbr_limit >= 1000,
            4
        );
        let tc_address = Signer::address_of(tc_account);
        assert(tc_address == Association::treasury_compliance_account(), 3);
        let modify_cap = &borrow_global<ModifyLimitCapability>(tc_address).cap;
        LibraConfig::set_with_capability<DualAttestationLimit>(
            modify_cap,
            DualAttestationLimit { micro_lbr_limit },
        );
    }
}

}
