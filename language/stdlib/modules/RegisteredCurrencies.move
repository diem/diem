address 0x1 {

/// Module managing the registered currencies in the Libra framework.
module RegisteredCurrencies {
    use 0x1::CoreAddresses;
    use 0x1::CoreErrors;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
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
        assert(LibraTimestamp::is_genesis(), CoreErrors::NOT_GENESIS());
        assert(
            Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            CoreErrors::NOT_LIBRA_ROOT_ADDRESS()
        );
        LibraConfig::publish_new_config(
            config_account,
            RegisteredCurrencies { currency_codes: Vector::empty() }
        );
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: config_account};
        include Roles::AbortsIfNotLibraRoot{account: config_account};
        aborts_if LibraConfig::spec_is_published<RegisteredCurrencies>();
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

    spec module {
        pragma verify = true;

        /// Helper to get the currency code vector.
        define get_currency_codes(): vector<vector<u8>> {
            LibraConfig::spec_get<RegisteredCurrencies>().currency_codes
        }

        /// Global invariant that currency config is always available after genesis.
        invariant [global] LibraTimestamp::is_operating() ==> LibraConfig::spec_is_published<RegisteredCurrencies>();

        /// Global invariant that only LIBRA_ROOT can have a currency registration.
        invariant [global] LibraTimestamp::is_operating() ==> (
            forall holder: address where exists<LibraConfig::LibraConfig<RegisteredCurrencies>>(holder):
                holder == CoreAddresses::LIBRA_ROOT_ADDRESS()
        );
    }

}

}
