/// This module defines the coin type XUS and its initialization function.
module DiemFramework::XUS {
    use DiemFramework::Diem;
    use DiemFramework::DiemTimestamp;
    use DiemFramework::Roles;
    use Std::FixedPoint32;

    /// The type tag representing the `XUS` currency on-chain.
    struct XUS { }

    /// Registers the `XUS` cointype. This can only be called from genesis.
    public fun initialize(
        dr_account: &signer,
        tc_account: &signer,
    ) {
        DiemTimestamp::assert_genesis();
        Roles::assert_treasury_compliance(tc_account);
        Roles::assert_diem_root(dr_account);
        Diem::register_SCS_currency<XUS>(
            dr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to XDX
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"XUS"
        );
    }
    spec initialize {
        use DiemFramework::Roles;
        include Diem::RegisterSCSCurrencyAbortsIf<XUS>{
            currency_code: b"XUS",
            scaling_factor: 1000000
        };
        include Diem::RegisterSCSCurrencyEnsures<XUS>;
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
        /// After genesis, XUS is registered.
        invariant DiemTimestamp::is_operating() ==> Diem::is_currency<XUS>();
    }
}
