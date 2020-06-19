address 0x1 {

module Unhosted {
    use 0x1::CoreErrors;
    use 0x1::CoreAddresses;
    use 0x1::AccountLimits;
    use 0x1::Signer;
    use 0x1::Testnet;
    use 0x1::Roles::{Capability, TreasuryComplianceRole};

    fun MODULE_ERROR_BASE(): u64 { 16000 }

    // An unhosted account is subject to account holding/velocity limits.
    // This holds the metadata about the account transactions during a
    // given time period that tracks this and enforces these limits.
    struct Unhosted {
    }

    // Global limits published during genesis
    public fun publish_global_limits_definition(account: &signer, cap: &Capability<TreasuryComplianceRole>) {
        // Operational constraint
        assert(
            Signer::address_of(account) == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(),
            MODULE_ERROR_BASE() + CoreErrors::INVALID_SINGLETON_ADDRESS()
        );
        // These are limits for testnet _only_.
        AccountLimits::publish_unrestricted_limits(account);
        /*AccountLimits::publish_limits_definition(
            10000,
            10000,
            50000,
            31540000000000
        );*/
        AccountLimits::certify_limits_definition(cap, CoreAddresses::TREASURY_COMPLIANCE_ADDRESS());
    }

    // Regular unhosted wallet accounts are currently Testnet only
    public fun create(): Unhosted {
        assert(Testnet::is_testnet(), 10041);
        Unhosted {  }
    }

    fun window_length(): u64 {
        // number of microseconds in a day
        86400000000
    }

}
}
