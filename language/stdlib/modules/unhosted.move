address 0x0:

module Unhosted {
    use 0x0::AccountLimits;
    use 0x0::Sender;
    use 0x0::Transaction;

    // An unhosted account is subject to account holding/velocity limits.
    // This holds the metadata about the account transactions during a
    // given time period that tracks this and enforces these limits.
    struct T {
        limit_tracker: AccountLimits::Window,
    }

    // This is _not_ meant to be called in genesis. Once this
    public fun publish_global_limits_definition(sender: &Sender::T) {
        // TODO: error code
        Transaction::assert(Sender::address_(sender) == limits_addr(), 0);
        // These are limits for testnet _only_.
        AccountLimits::publish_unrestricted_limits(sender);
        AccountLimits::certify_limits_definition(limits_addr(), sender);
    }

    public fun create(): T {
        T { limit_tracker: AccountLimits::create() }
    }

    public fun account_limits(t: T): AccountLimits::Window {
        *&t.limit_tracker
    }

    // this is OK just as in the VASP case: you can create these all you
    // want, but the only way these can be stored in a meaningful way is
    // via permissioned operations.
    public fun update_account_limits(limit_tracker: AccountLimits::Window): T {
        T { limit_tracker }
    }

    public fun limits_addr(): address {
        0xA550C18
    }
}
