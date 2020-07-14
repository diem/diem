address 0x1 {

/// Module managing the registered currencies in the Libra framework.
module RegisteredCurrencies {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp::{Self, spec_is_genesis};
    use 0x1::Signer;
    use 0x1::Vector;

    /// An on-chain config holding all of the currency codes for registered
    /// currencies. The inner vector<u8>'s are string representations of
    /// currency names.
    struct RegisteredCurrencies {
        currency_codes: vector<vector<u8>>,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ECURRENCY_CODE_ALREADY_TAKEN: u64 = 2;

    /// Initializes this module. Can only be called from genesis.
    public fun initialize(config_account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);

        assert(
            Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );
        LibraConfig::publish_new_config(
            config_account,
            RegisteredCurrencies { currency_codes: Vector::empty() }
        );


    }
    spec fun initialize {
        pragma aborts_if_is_partial; // because of Roles import problem below
        /// Function aborts if already initialized or not in genesis.
        // TODO: I don't think there is any way to import the next function for spec only.
        // aborts_if !Roles::spec_has_libra_root_role_addr(Signer::spec_address_of(config_account));
        aborts_if Signer::spec_address_of(config_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if LibraConfig::spec_is_published<RegisteredCurrencies>();
        aborts_if !spec_is_genesis();
        ensures LibraConfig::spec_is_published<RegisteredCurrencies>();
        ensures len(get_currency_codes()) == 0;
    }


    /// Adds a new currency code. The currency code must not yet exist.
    public fun add_currency_code(
        lr_account: &signer,
        currency_code: vector<u8>,
    ) {
        let config = LibraConfig::get<RegisteredCurrencies>();
        assert(
            !Vector::contains(&config.currency_codes, &currency_code),
            ECURRENCY_CODE_ALREADY_TAKEN
        );
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set(lr_account, config);
    }
    spec fun add_currency_code {
        include AddCurrencyCodeAbortsIf;
        /// The resulting currency_codes is the one before this function is called, with the new one added to the end.
        ensures Vector::eq_push_back(get_currency_codes(), old(get_currency_codes()), currency_code);
    }
    spec schema AddCurrencyCodeAbortsIf {
        lr_account: &signer;
        currency_code: vector<u8>;

        aborts_if !LibraConfig::spec_is_published<RegisteredCurrencies>();
        aborts_if !exists<LibraConfig::ModifyConfigCapability<RegisteredCurrencies>>(Signer::spec_address_of(lr_account));

        /// The same currency code can be only added once.
        aborts_if Vector::spec_contains(
            LibraConfig::spec_get<RegisteredCurrencies>().currency_codes,
            currency_code
        );
    }

    // **************** Global Specification ****************

    // spec schema OnlyConfigAddressHasRegisteredCurrencies {
    //     /// There is no address with a RegisteredCurrencies value before initialization.
    //     invariant module !spec_is_initialized()
    //         ==> (forall addr: address: !LibraConfig::spec_is_published<RegisteredCurrencies>(addr));

    //     /// *Informally:* After initialization, only singleton_address() has a RegisteredCurrencies value.
    //     invariant module spec_is_initialized()
    //         ==> LibraConfig::spec_is_published<RegisteredCurrencies>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
    //             && (forall addr: address:
    //                    LibraConfig::spec_is_published<RegisteredCurrencies>(addr)
    //                               ==> addr == CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS());
    // }

    spec module {
        pragma verify = true;

        /// Helper to get the currency code vector.
        define get_currency_codes(): vector<vector<u8>> {
            LibraConfig::spec_get<RegisteredCurrencies>().currency_codes
        }

        /// Global invariant that currency config is always available after genesis.
        invariant !spec_is_genesis() ==> LibraConfig::spec_is_published<RegisteredCurrencies>();
    }

}

}
