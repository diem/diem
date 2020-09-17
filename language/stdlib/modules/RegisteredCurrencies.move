address 0x1 {

/// Module managing the registered currencies in the Libra framework.
module RegisteredCurrencies {
    use 0x1::Errors;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
    use 0x1::Vector;

    /// An on-chain config holding all of the currency codes for registered
    /// currencies. The inner vector<u8>'s are string representations of
    /// currency names.
    struct RegisteredCurrencies {
        currency_codes: vector<vector<u8>>,
    }

    /// Attempted to add a currency code that is already in use
    const ECURRENCY_CODE_ALREADY_TAKEN: u64 = 0;

    /// Initializes this module. Can only be called from genesis.
    public fun initialize(lr_account: &signer) {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);
        LibraConfig::publish_new_config(
            lr_account,
            RegisteredCurrencies { currency_codes: Vector::empty() }
        );
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include LibraConfig::PublishNewConfigAbortsIf<RegisteredCurrencies>;
        include LibraConfig::PublishNewConfigEnsures<RegisteredCurrencies>{
            payload: RegisteredCurrencies { currency_codes: Vector::empty() }
        };
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
            Errors::invalid_argument(ECURRENCY_CODE_ALREADY_TAKEN)
        );
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set(lr_account, config);
    }
    spec fun add_currency_code {
        include AddCurrencyCodeAbortsIf;
        include AddCurrencyCodeEnsures;
    }
    spec schema AddCurrencyCodeAbortsIf {
        lr_account: &signer;
        currency_code: vector<u8>;
        include LibraConfig::SetAbortsIf<RegisteredCurrencies>{ account: lr_account };
        /// The same currency code can be only added once.
        aborts_if Vector::spec_contains(
            LibraConfig::get<RegisteredCurrencies>().currency_codes,
            currency_code
        ) with Errors::INVALID_ARGUMENT;
    }
    spec schema AddCurrencyCodeEnsures {
        currency_code: vector<u8>;
        /// The resulting currency_codes is the one before this function is called, with the new one added to the end.
        ensures Vector::eq_push_back(get_currency_codes(), old(get_currency_codes()), currency_code);
        include LibraConfig::SetEnsures<RegisteredCurrencies> {payload: LibraConfig::get<RegisteredCurrencies>()};
    }

    // **************** Global Specification ****************

    spec module {
        /// Helper to get the currency code vector.
        define get_currency_codes(): vector<vector<u8>> {
            LibraConfig::get<RegisteredCurrencies>().currency_codes
        }

        /// Global invariant that currency config is always available after genesis.
        invariant [global] LibraTimestamp::is_operating() ==> LibraConfig::spec_is_published<RegisteredCurrencies>();

        /*
        /// Global invariant that only LIBRA_ROOT can have a currency registration.
        /// TODO(wrwg): deactivated because it is causing inconsistencies(?)
        invariant [global] LibraTimestamp::is_operating() ==> (
            forall holder: address where exists<LibraConfig::LibraConfig<RegisteredCurrencies>>(holder):
                holder == CoreAddresses::LIBRA_ROOT_ADDRESS()
        );
        */
    }

}

}
