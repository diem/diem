address 0x1 {

/// Module managing the registered currencies in the Libra framework.
module RegisteredCurrencies {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::LibraTimestamp::{assert_is_genesis, spec_is_genesis};
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::Roles::Capability;

    /// An on-chain config holding all of the currency codes for registered
    /// currencies. The inner vector<u8>'s are string representations of
    /// currency names.
    struct RegisteredCurrencies {
        currency_codes: vector<vector<u8>>,
    }

    /// A capability which allows updating of the currency on-chain configuration.
    resource struct RegistrationCapability {
        cap: LibraConfig::ModifyConfigCapability<Self::RegisteredCurrencies>,
    }

    /// Initializes this module. Can only be called from genesis.
    public fun initialize(
        config_account: &signer,
        create_config_capability: &Capability<CreateOnChainConfig>,
    ): RegistrationCapability {
        assert_is_genesis();

        // enforce that this is only going to one specific address,
        assert(
            Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            0
        );
        let cap = LibraConfig::publish_new_config_with_capability(
            config_account,
            create_config_capability,
            RegisteredCurrencies { currency_codes: Vector::empty() }
        );

        RegistrationCapability { cap }
    }
    spec fun initialize {
        /// Function aborts if already initialized or not in genesis.
        aborts_if Signer::spec_address_of(config_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if LibraConfig::spec_is_published<RegisteredCurrencies>();
        aborts_if !spec_is_genesis();
        ensures LibraConfig::spec_is_published<RegisteredCurrencies>();
        ensures len(get_currency_codes()) == 0;
    }


    /// Adds a new currency code. The currency code must not yet exist.
    public fun add_currency_code(
        currency_code: vector<u8>,
        cap: &RegistrationCapability,
    ) {
        let config = LibraConfig::get<RegisteredCurrencies>();
        assert(
            !Vector::contains(&config.currency_codes, &currency_code),
            1
        );
        Vector::push_back(&mut config.currency_codes, currency_code);
        LibraConfig::set_with_capability(&cap.cap, config);
    }
    spec fun add_currency_code {
        include AddCurrencyCodeAbortsIf;
        /// The resulting currency_codes is the one before this function is called, with the new one added to the end.
        ensures Vector::eq_push_back(get_currency_codes(), old(get_currency_codes()), currency_code);
    }
    spec schema AddCurrencyCodeAbortsIf {
        currency_code: vector<u8>;

        aborts_if !LibraConfig::spec_is_published<RegisteredCurrencies>();

        /// The same currency code can be only added once.
        aborts_if Vector::spec_contains(
            LibraConfig::spec_get<RegisteredCurrencies>().currency_codes,
            currency_code
        );
    }

    // **************** Global Specification ****************

    /// # Module specifications

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
