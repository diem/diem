address 0x1 {

/// This module defines the coin type XUS and its initialization function.
module XUS {
    use 0x1::AccountLimits;
    use 0x1::Diem;
    use 0x1::DiemTimestamp;
    use 0x1::FixedPoint32;

    /// The type tag representing the `XUS` currency on-chain.
    struct XUS has store { }

    /// Registers the `XUS` cointype. This can only be called from genesis.
    public fun initialize(
        dr_account: &signer,
        tc_account: &signer,
    ) {
        DiemTimestamp::assert_genesis();
        Diem::register_SCS_currency<XUS>(
            dr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to XDX
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"XUS"
        );
        AccountLimits::publish_unrestricted_limits<XUS>(dr_account);
    }
    spec fun initialize {
        use 0x1::Roles;
        include Diem::RegisterSCSCurrencyAbortsIf<XUS>{
            currency_code: b"XUS",
            scaling_factor: 1000000
        };
        include AccountLimits::PublishUnrestrictedLimitsAbortsIf<XUS>{publish_account: dr_account};
        include Diem::RegisterSCSCurrencyEnsures<XUS>;
        include AccountLimits::PublishUnrestrictedLimitsEnsures<XUS>{publish_account: dr_account};
        /// Registering XUS can only be done in genesis.
        include DiemTimestamp::AbortsIfNotGenesis;
        /// Only the DiemRoot account can register a new currency [[H8]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        /// Only a TreasuryCompliance account can have the MintCapability [[H1]][PERMISSION].
        /// Moreover, only a TreasuryCompliance account can have the BurnCapability [[H3]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Persistence of Resources
    spec module {
        use 0x1::CoreAddresses;

        /// After genesis, XUS is registered.
        invariant [global] DiemTimestamp::is_operating() ==> Diem::is_currency<XUS>();

        /// After genesis, `LimitsDefinition<XUS>` is published at Diem root. It's published by
        /// AccountLimits::publish_unrestricted_limits, but we can't prove the condition there because
        /// it does not hold for all types (but does hold for XUS).
        invariant [global] DiemTimestamp::is_operating()
            ==> exists<AccountLimits::LimitsDefinition<XUS>>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// `LimitsDefinition<XUS>` is not published at any other address
        invariant [global] forall addr: address where exists<AccountLimits::LimitsDefinition<XUS>>(addr):
            addr == CoreAddresses::DIEM_ROOT_ADDRESS();

    }
}
}
