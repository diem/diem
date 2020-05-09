address 0x0 {

module Unhosted {
    use 0x0::AccountLimits;
    use 0x0::Testnet;
    use 0x0::Transaction;

    // An unhosted account is subject to account holding/velocity limits.
    // This holds the metadata about the account transactions during a
    // given time period that tracks this and enforces these limits.
    struct T {
    }

    public fun publish_global_limits_definition() {
        Transaction::assert(Transaction::sender() == limits_addr(), 0);
        // These are limits for testnet _only_.
        AccountLimits::publish_unrestricted_limits();
        /*AccountLimits::publish_limits_definition(
            10000,
            10000,
            50000,
            31540000000000
        );*/
        AccountLimits::certify_limits_definition(limits_addr());
    }

    public fun create(): T {
        Transaction::assert(Testnet::is_testnet(), 10041);
        T {  }
    }

    public fun limits_addr(): address {
        0xA550C18
    }
}
}
