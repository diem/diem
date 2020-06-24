address 0x1 {

module RegisteredCurrencies {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::Roles::Capability;

    // An on-chain config holding all of the currency codes for registered
    // currencies. The inner vector<u8>'s are string representations of
    // currency names.
    struct RegisteredCurrencies {
        currency_codes: vector<vector<u8>>,
    }

    // An operations capability to allow updating of the on-chain config
    resource struct RegistrationCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::RegisteredCurrencies>,
    }

    public fun initialize(
        config_account: &signer,
        create_config_capability: &Capability<CreateOnChainConfig>,
    ): RegistrationCapability {
        // enforce that this is only going to one specific address,
        assert(
            Signer::address_of(config_account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(),
            0
        );
        let cap = LibraConfig::publish_new_config_with_capability(
            config_account,
            create_config_capability,
            empty()
        );

        RegistrationCapability { cap }
    }

    fun empty(): RegisteredCurrencies {
        RegisteredCurrencies { currency_codes: Vector::empty() }
    }

    public fun add_currency_code(
        currency_code: vector<u8>,
        cap: &RegistrationCapability,
    ) {
        let config = LibraConfig::get<RegisteredCurrencies>();
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set_with_capability(&cap.cap, config);
    }

    // **************** Specifications ****************

    /// # Module specifications

    spec module {
        pragma verify = true;

        define spec_singleton_address(): address { 0xA550C18 }

        // spec_is_initialized() is true iff initialize has been called.
        define spec_is_initialized():bool {
            LibraConfig::spec_is_published<RegisteredCurrencies>(spec_singleton_address())
        }
    }

    /// ## Initialization

    spec fun initialize {
        /// After `initialize` is called, the module is initialized.
        pragma aborts_if_is_partial = true;

        // `initialize` aborts if already initialized
        aborts_if spec_is_initialized();
        ensures spec_is_initialized();
    }

    spec schema InitializationPersists {
        /// *Informally:* Once initialize is run, the module continues to be
        /// initialized, forever.
        ensures old(spec_is_initialized()) ==> spec_is_initialized();
    }
    spec module {
        apply InitializationPersists to *;
    }

    /// ## Uniqueness of the RegisteredCurrencies config.

    spec schema OnlySingletonHasRegisteredCurrencies {
        // *Informally:* There is no address with a RegisteredCurrencies value before initialization.
        invariant !spec_is_initialized()
            ==> (forall addr: address: !LibraConfig::spec_is_published<RegisteredCurrencies>(addr));
        // *Informally:* After initialization, only singleton_address() has a RegisteredCurrencies value.
        invariant spec_is_initialized()
            ==> LibraConfig::spec_is_published<RegisteredCurrencies>(spec_singleton_address())
                && (forall addr: address:
                       LibraConfig::spec_is_published<RegisteredCurrencies>(addr)
                                  ==> addr == spec_singleton_address());
    }
    spec module {
        apply OnlySingletonHasRegisteredCurrencies to *;
    }

    /// ## Currency codes

    /// Attempting to specify that only `add_currency` changes the currency_codes
    /// vector.
    /// **Confused:** I think `initialize` should violate this property unless it
    /// checks whether the module is already initialized, because it can be
    /// called a second time, overwriting existing currency_codes.
    spec schema OnlyAddCurrencyChangesT {
        ensures old(spec_is_initialized())
                     ==> old(LibraConfig::spec_get<RegisteredCurrencies>().currency_codes)
                          == LibraConfig::spec_get<RegisteredCurrencies>().currency_codes;
    }
    spec module {
        /// `add_currency_code` and `initialize` change the currency_code vector.
        apply OnlyAddCurrencyChangesT to * except add_currency_code;
    }

    // TODO: currency_code vector is a set (no dups).  (Not satisfied now.)
    // TODO: add_currency just pushes one thing.

}

}
