address 0x0 {

module Unhosted {
    use 0x0::AccountLimits;
    use 0x0::Signer;
    use 0x0::Testnet;
    use 0x0::Transaction;

    // An unhosted account is subject to account holding/velocity limits.
    // This holds the metadata about the account transactions during a
    // given time period that tracks this and enforces these limits.
    struct Unhosted {
    }

    public fun publish_global_limits_definition(account: &signer) {
        Transaction::assert(Signer::address_of(account) == limits_addr(), 100042);
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

    public fun create(): Unhosted {
        Transaction::assert(Testnet::is_testnet(), 10041);
        Unhosted {  }
    }

    public fun limits_addr(): address {
        0xA550C18
    }
}
}
