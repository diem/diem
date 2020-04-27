address 0x0:

module RegisteredCurrencies {
    use 0x0::Vector;
    use 0x0::Transaction;
    use 0x0::LibraConfig;

    // An on-chain config holding all of the currency codes for registered
    // currencies. Must be named "T" for an on-chain config.
    struct T {
        currency_codes: vector<vector<u8>>,
    }

    // An operations capability to allow updating of the on-chain config
    resource struct RegistrationCapability {}

    public fun grant_registration_capability(): RegistrationCapability {
        // enforce that this is only going to one specific address,
        Transaction::assert(Transaction::sender() == singleton_address(), 0);
        RegistrationCapability{}
    }

    public fun empty(): T {
        T { currency_codes: Vector::empty() }
    }

    public fun add_currency_code(
        currency_code: vector<u8>,
        _cap: &RegistrationCapability,
    ) {
        let sender = Transaction::sender();
        let config = LibraConfig::get<T>(sender);
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set(sender, config);
    }

    fun singleton_address(): address {
        0xA550C18
    }
}
