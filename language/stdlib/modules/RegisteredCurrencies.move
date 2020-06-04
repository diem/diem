address 0x0 {

module RegisteredCurrencies {
    use 0x0::LibraConfig;
    use 0x0::Signer;
    use 0x0::Transaction;
    use 0x0::Vector;

    // An on-chain config holding all of the currency codes for registered
    // currencies. Must be named "T" for an on-chain config.
    // The inner vector<u8>'s are string representations of currency names.
    struct T {
        currency_codes: vector<vector<u8>>,
    }

    // An operations capability to allow updating of the on-chain config
    resource struct RegistrationCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::T>,
    }

    public fun initialize(config_account: &signer): RegistrationCapability {
        // enforce that this is only going to one specific address,
        Transaction::assert(
            Signer::address_of(config_account) == LibraConfig::default_config_address(),
            0
        );
        let cap = LibraConfig::publish_new_config_with_capability(config_account, empty());

        RegistrationCapability { cap }
    }

    fun empty(): T {
        T { currency_codes: Vector::empty() }
    }

    public fun add_currency_code(
        currency_code: vector<u8>,
        cap: &RegistrationCapability,
    ) {
        let config = LibraConfig::get<T>();
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set_with_capability(&cap.cap, config);
    }

    // DD: What value does this redundant wrapper provide?
    fun singleton_address(): address {
        LibraConfig::default_config_address()
    }


    // **************** Specifications ****************

    /// # Module specifications

    spec module {
        // Verification is disabled because I'm running into errors
        // from an invariant in association.move
        pragma verify = false;

        // spec_singleton_address() is the spec version of singleton_address, which is
        // defined in the LibraConfig module we are in.
        define spec_singleton_address():address { LibraConfig::spec_default_config_address() }

        // spec_is_initialized() is true iff initialize has been called.
        define spec_is_initialized():bool { LibraConfig::spec_is_published<T>(spec_singleton_address()) }
    }

    // Check spec_is_initialized on initialize()
    spec fun initialize {
        ensures !spec_is_initialized();
        ensures spec_is_initialized();
    }

    spec schema OnlySingletonHasT {
        // *Informally:* There is no address with a T value before initialization.
        invariant !spec_is_initialized()
            ==> all(domain<address>(), |addr| !LibraConfig::spec_is_published<T>(addr));

        // *Informally:* After initialization, only singleton_address() has a T value.
        invariant spec_is_initialized()
            ==> LibraConfig::spec_is_published<T>(sender())
                && all(domain<address>(),
                       |addr| LibraConfig::spec_is_published<T>(addr)
                                  ==> addr == spec_singleton_address());
    }
    spec module {
        apply OnlySingletonHasT to *;
    }

    spec schema OnlyAddCurrencyChangesT {
        ensures spec_is_initialized()
                     ==> old(LibraConfig::spec_get<T>().currency_codes)
                          == LibraConfig::spec_get<T>().currency_codes;
    }
    spec module {
        apply OnlyAddCurrencyChangesT to * except add_currency_code;
    }

    // TODO: currency_code vector is a set (no dups).  (Not enforced)

}

}
