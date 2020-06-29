address 0x1 {

module DualAttestationLimit {
    use 0x1::LBR::LBR;
    use 0x1::Libra;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Roles::{has_update_dual_attestation_threshold_privilege};


    resource struct UpdateDualAttestationThreshold {}

    struct DualAttestationLimit {
        micro_lbr_limit: u64,
    }

    // A wrapped capability to allow updating of limit on-chain config by Blessed
    resource struct ModifyLimitCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::DualAttestationLimit>,
    }

    // Will fail if `account` does not have the treasury-compliance role
    // public fun grant_privileges(account: &signer) {
    // }

    /// Travel rule limit set during genesis
    ///
    /// >TODO: add in_genesis assertion here.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), 0);
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
        let cap = LibraConfig::publish_new_config_with_capability<DualAttestationLimit>(
            lr_account,
            DualAttestationLimit { micro_lbr_limit: 1000 * Libra::scaling_factor<LBR>() },
        );
        move_to(tc_account, ModifyLimitCapability { cap })
    }

    // Anyone can fetch the current set limit
    public fun get_cur_microlibra_limit(): u64 {
        LibraConfig::get<DualAttestationLimit>().micro_lbr_limit
    }

    public fun set_microlibra_limit(
        tc_account: &signer,
        micro_lbr_limit: u64
    ) acquires ModifyLimitCapability {
        // TODO: abort code
        assert(has_update_dual_attestation_threshold_privilege(tc_account), 919401);
        assert(
            micro_lbr_limit >= 1000,
            4
        );
        let tc_address = Signer::address_of(tc_account);
        let modify_cap = &borrow_global<ModifyLimitCapability>(tc_address).cap;
        LibraConfig::set_with_capability<DualAttestationLimit>(
            modify_cap,
            DualAttestationLimit { micro_lbr_limit },
        );
    }

    // **************************** SPECIFICATION ********************************

    spec module {
        pragma verify = true;

        /// Helper function to determine whether the DualAttestationLimit is published.
        define spec_is_published(): bool {
            LibraConfig::spec_is_published<DualAttestationLimit>()
        }

        /// Mirrors `Self::get_cur_microlibra_limit`.
        define spec_get_cur_microlibra_limit(): u64 {
            LibraConfig::spec_get<DualAttestationLimit>().micro_lbr_limit
        }

        /// After genesis, the DualAttestationLimit is always published.
        invariant !LibraTimestamp::spec_is_genesis() ==> spec_is_published();
    }
}
}
