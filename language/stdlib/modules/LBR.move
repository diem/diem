address 0x1 {
/// NB: This module is a stub of the `LBR` at the moment.
///
/// Once the component makeup of the LBR has been chosen the
/// `Reserve` will be updated to hold the backing coins in the correct ratios.

module LBR {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::FixedPoint32;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;

    /// The type tag representing the `LBR` currency on-chain.
    resource struct LBR { }

    /// Note: Currently only holds the mint, burn, and preburn capabilities for
    /// LBR. Once the makeup of the LBR has been determined this resource will
    /// be updated to hold the backing LBR reserve compnents on-chain.
    ///
    /// The on-chain reserve for the `LBR` holds both the capability for minting `LBR`
    /// coins, and also each reserve component that holds the backing for these coins on-chain.
    /// Currently this holds no coins since LBR is not able to be minted/created.
    resource struct Reserve {
        /// The mint capability allowing minting of `LBR` coins.
        mint_cap: Libra::MintCapability<LBR>,
        /// The burn capability for `LBR` coins. This is used for the unpacking
        /// of `LBR` coins into the underlying backing currencies.
        burn_cap: Libra::BurnCapability<LBR>,
        /// The preburn for `LBR`. This is an administrative field since we
        /// need to alway preburn before we burn.
        preburn_cap: Libra::Preburn<LBR>,
        // TODO: Once the reserve has been determined this resource will
        // contain a ReserveComponent<Currency> for every currency that makes
        // up the reserve.
    }

    /// The `Reserve` resource is in an invalid state
    const ERESERVE: u64 = 0;

    /// Initializes the `LBR` module. This sets up the initial `LBR` ratios and
    /// reserve components, and creates the mint, preburn, and burn
    /// capabilities for `LBR` coins. The `LBR` currency must not already be
    /// registered in order for this to succeed. The sender must both be the
    /// correct address (`CoreAddresses::CURRENCY_INFO_ADDRESS`) and have the
    /// correct permissions (`&Capability<RegisterNewCurrency>`). Both of these
    /// restrictions are enforced in the `Libra::register_currency` function, but also enforced here.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_currency_info(lr_account);
        // Reserve must not exist.
        assert(!exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ERESERVE));
        let (mint_cap, burn_cap) = Libra::register_currency<LBR>(
            lr_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"LBR"
        );
        // LBR cannot be minted.
        Libra::update_minting_ability<LBR>(tc_account, false);
        AccountLimits::publish_unrestricted_limits<LBR>(lr_account);
        let preburn_cap = Libra::create_preburn<LBR>(tc_account);
        move_to(lr_account, Reserve { mint_cap, burn_cap, preburn_cap });
    }
    spec fun initialize {
       use 0x1::Roles;
        include CoreAddresses::AbortsIfNotCurrencyInfo{account: lr_account};
        aborts_if exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
        include Libra::RegisterCurrencyAbortsIf<LBR>{
            currency_code: b"LBR",
            scaling_factor: 1000000
        };
        include AccountLimits::PublishUnrestrictedLimitsAbortsIf<LBR>{publish_account: lr_account};

        include Libra::RegisterCurrencyEnsures<LBR>;
        include Libra::UpdateMintingAbilityEnsures<LBR>{can_mint: false};
        include AccountLimits::PublishUnrestrictedLimitsEnsures<LBR>{publish_account: lr_account};
        ensures exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        /// Registering LBR can only be done in genesis.
        include LibraTimestamp::AbortsIfNotGenesis;
        /// Only the LibraRoot account can register a new currency [[H7]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        /// Only the TreasuryCompliance role can update the `can_mint` field of CurrencyInfo.
        /// Moreover, only the TreasuryCompliance role can create Preburn.
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

    /// Returns true if `CoinType` is `LBR::LBR`
    public fun is_lbr<CoinType>(): bool {
        Libra::is_currency<CoinType>() &&
            Libra::currency_code<CoinType>() == Libra::currency_code<LBR>()
    }

    spec fun is_lbr {
        pragma opaque, verify = false;
        include Libra::spec_is_currency<CoinType>() ==> Libra::AbortsIfNoCurrency<LBR>;
        /// The following is correct because currency codes are unique; however, we
        /// can currently not prove it, therefore verify is false.
        ensures result == Libra::spec_is_currency<CoinType>() && spec_is_lbr<CoinType>();
    }

    /// Return the account address where the globally unique LBR::Reserve resource is stored
    public fun reserve_address(): address {
        CoreAddresses::CURRENCY_INFO_ADDRESS()
    }

    // =================================================================
    // Module Specification

    spec module {} // switch documentation context back to module level

    /// # Persistence of Resources

    spec module {
        /// After genesis, the Reserve resource exists.
        invariant [global] LibraTimestamp::is_operating() ==> reserve_exists();

        /// After genesis, LBR is registered.
        invariant [global] LibraTimestamp::is_operating() ==> Libra::is_currency<LBR>();
    }

    /// # Helper Functions
    spec module {
        /// Checks whether the Reserve resource exists.
        define reserve_exists(): bool {
           exists<Reserve>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Returns true if CoinType is LBR.
        define spec_is_lbr<CoinType>(): bool {
            type<CoinType>() == type<LBR>()
        }
    }

}
}
