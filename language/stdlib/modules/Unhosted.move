address 0x1 {

module Unhosted {
    use 0x1::CoreAddresses;
    use 0x1::AccountLimits;
    use 0x1::Signer;
    use 0x1::Testnet;

    // An unhosted account is subject to account holding/velocity limits.
    // This holds the metadata about the account transactions during a
    // given time period that tracks this and enforces these limits.
    struct Unhosted {
    }

    // Global limits published during genesis
    public fun publish_global_limits_definition(account: &signer) {
        assert(Signer::address_of(account) == limits_addr(), 100042);
        // These are limits for testnet _only_.
        AccountLimits::publish_unrestricted_limits(account);
        /*AccountLimits::publish_limits_definition(
            10000,
            10000,
            50000,
            31540000000000
        );*/
        AccountLimits::certify_limits_definition(account, limits_addr());
    }

    // Regular unhosted wallet accounts are currently Testnet only
    public fun create(): Unhosted {
        assert(Testnet::is_testnet(), 10041);
        Unhosted {  }
    }

    public fun limits_addr(): address {
        CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    }

    fun window_length(): u64 {
        // number of microseconds in a day
        86400000000
    }

}
}
