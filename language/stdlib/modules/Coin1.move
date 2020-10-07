address 0x1 {

/// This module defines the coin type Coin1 and its initialization function.
module Coin1 {
    use 0x1::AccountLimits;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    use 0x1::FixedPoint32;

    /// The type tag representing the `Coin1` currency on-chain.
    struct Coin1 { }

    /// Registers the `Coin1` cointype. This can only be called from genesis.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Libra::register_SCS_currency<Coin1>(
            lr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin1"
        );
        AccountLimits::publish_unrestricted_limits<Coin1>(lr_account);
    }
    spec fun initialize {
        use 0x1::Roles;
        include Libra::RegisterSCSCurrencyAbortsIf<Coin1>{
            currency_code: b"Coin1",
            scaling_factor: 1000000
        };
        include AccountLimits::PublishUnrestrictedLimitsAbortsIf<Coin1>{publish_account: lr_account};
        include Libra::RegisterSCSCurrencyEnsures<Coin1>;
        include AccountLimits::PublishUnrestrictedLimitsEnsures<Coin1>{publish_account: lr_account};
        /// Registering Coin1 can only be done in genesis.
        include LibraTimestamp::AbortsIfNotGenesis;
        /// Only the LibraRoot account can register a new currency [[H7]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        /// Only a TreasuryCompliance account can have the MintCapability [[H1]][PERMISSION].
        /// Moreover, only a TreasuryCompliance account can have the BurnCapability [[H2]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Persistence of Resources
    spec module {
        /// After genesis, Coin1 is registered.
        invariant [global] LibraTimestamp::is_operating() ==> Libra::is_currency<Coin1>();
    }
}
}
