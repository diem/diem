address 0x1 {
/// NB: This module is a stub of the `XDX` at the moment.
///
/// Once the component makeup of the XDX has been chosen the
/// `Reserve` will be updated to hold the backing coins in the correct ratios.

module XDX {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::FixedPoint32;
    use 0x1::Diem;
    use 0x1::DiemTimestamp;

    /// The type tag representing the `XDX` currency on-chain.
    struct XDX { }

    /// Note: Currently only holds the mint, burn, and preburn capabilities for
    /// XDX. Once the makeup of the XDX has been determined this resource will
    /// be updated to hold the backing XDX reserve compnents on-chain.
    ///
    /// The on-chain reserve for the `XDX` holds both the capability for minting `XDX`
    /// coins, and also each reserve component that holds the backing for these coins on-chain.
    /// Currently this holds no coins since XDX is not able to be minted/created.
    resource struct Reserve {
        /// The mint capability allowing minting of `XDX` coins.
        mint_cap: Diem::MintCapability<XDX>,
        /// The burn capability for `XDX` coins. This is used for the unpacking
        /// of `XDX` coins into the underlying backing currencies.
        burn_cap: Diem::BurnCapability<XDX>,
        /// The preburn for `XDX`. This is an administrative field since we
        /// need to alway preburn before we burn.
        preburn_cap: Diem::Preburn<XDX>,
        // TODO: Once the reserve has been determined this resource will
        // contain a ReserveComponent<Currency> for every currency that makes
        // up the reserve.
    }

    /// The `Reserve` resource is in an invalid state
    const ERESERVE: u64 = 0;

    /// Initializes the `XDX` module. This sets up the initial `XDX` ratios and
    /// reserve components, and creates the mint, preburn, and burn
    /// capabilities for `XDX` coins. The `XDX` currency must not already be
    /// registered in order for this to succeed. The sender must both be the
    /// correct address (`CoreAddresses::CURRENCY_INFO_ADDRESS`) and have the
    /// correct permissions (`&Capability<RegisterNewCurrency>`). Both of these
    /// restrictions are enforced in the `Diem::register_currency` function, but also enforced here.
    public fun initialize(
        dr_account: &signer,
        tc_account: &signer,
    ) {
        DiemTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_currency_info(dr_account);
        // Reserve must not exist.
        assert(!exists<Reserve>(CoreAddresses::DIEM_ROOT_ADDRESS()), Errors::already_published(ERESERVE));
        let (mint_cap, burn_cap) = Diem::register_currency<XDX>(
            dr_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to XDX
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"XDX"
        );
        // XDX cannot be minted.
        Diem::update_minting_ability<XDX>(tc_account, false);
        AccountLimits::publish_unrestricted_limits<XDX>(dr_account);
        let preburn_cap = Diem::create_preburn<XDX>(tc_account);
        move_to(dr_account, Reserve { mint_cap, burn_cap, preburn_cap });
    }
    spec fun initialize {
       use 0x1::Roles;
        include CoreAddresses::AbortsIfNotCurrencyInfo{account: dr_account};
        aborts_if exists<Reserve>(CoreAddresses::DIEM_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
        include Diem::RegisterCurrencyAbortsIf<XDX>{
            currency_code: b"XDX",
            scaling_factor: 1000000
        };
        include AccountLimits::PublishUnrestrictedLimitsAbortsIf<XDX>{publish_account: dr_account};

        include Diem::RegisterCurrencyEnsures<XDX>;
        include Diem::UpdateMintingAbilityEnsures<XDX>{can_mint: false};
        include AccountLimits::PublishUnrestrictedLimitsEnsures<XDX>{publish_account: dr_account};
        ensures exists<Reserve>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// Registering XDX can only be done in genesis.
        include DiemTimestamp::AbortsIfNotGenesis;
        /// Only the DiemRoot account can register a new currency [[H8]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        /// Only the TreasuryCompliance role can update the `can_mint` field of CurrencyInfo [[H2]][PERMISSION].
        /// Moreover, only the TreasuryCompliance role can create Preburn.
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

    /// Returns true if `CoinType` is `XDX::XDX`
    public fun is_xdx<CoinType>(): bool {
        Diem::is_currency<CoinType>() &&
            Diem::currency_code<CoinType>() == Diem::currency_code<XDX>()
    }

    spec fun is_xdx {
        pragma opaque, verify = false;
        include Diem::spec_is_currency<CoinType>() ==> Diem::AbortsIfNoCurrency<XDX>;
        /// The following is correct because currency codes are unique; however, we
        /// can currently not prove it, therefore verify is false.
        ensures result == Diem::spec_is_currency<CoinType>() && spec_is_xdx<CoinType>();
    }

    /// Return the account address where the globally unique XDX::Reserve resource is stored
    public fun reserve_address(): address {
        CoreAddresses::CURRENCY_INFO_ADDRESS()
    }

    // =================================================================
    // Module Specification

    spec module {} // switch documentation context back to module level

    /// # Persistence of Resources

    spec module {
        /// After genesis, the Reserve resource exists.
        invariant [global] DiemTimestamp::is_operating() ==> reserve_exists();

        /// After genesis, XDX is registered.
        invariant [global] DiemTimestamp::is_operating() ==> Diem::is_currency<XDX>();
    }

    /// # Helper Functions
    spec module {
        /// Checks whether the Reserve resource exists.
        define reserve_exists(): bool {
           exists<Reserve>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Returns true if CoinType is XDX.
        define spec_is_xdx<CoinType>(): bool {
            type<CoinType>() == type<XDX>()
        }

        /// After genesis, `LimitsDefinition<XDX>` is published at Diem root. It's published by
        /// AccountLimits::publish_unrestricted_limits, but we can't prove the condition there because
        /// it does not hold for all types (but does hold for XDX).
        invariant [global] DiemTimestamp::is_operating()
            ==> exists<AccountLimits::LimitsDefinition<XDX>>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// `LimitsDefinition<XDX>` is not published at any other address
        invariant [global] forall addr: address where exists<AccountLimits::LimitsDefinition<XDX>>(addr):
            addr == CoreAddresses::DIEM_ROOT_ADDRESS();

        /// `Reserve` is persistent
        invariant update [global] old(exists<Reserve>(reserve_address()))
            ==> exists<Reserve>(reserve_address());
    }

}
}
