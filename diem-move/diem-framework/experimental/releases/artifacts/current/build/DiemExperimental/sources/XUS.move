/// This module defines the coin type XUS and its initialization function.
module DiemFramework::XUS {
    use DiemFramework::AccountLimits;
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
        AccountLimits::publish_unrestricted_limits<XUS>(dr_account);
    }
    spec initialize {
        use DiemFramework::Roles;
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
        /// After genesis, XUS is registered.
        invariant [suspendable] DiemTimestamp::is_operating() ==> Diem::is_currency<XUS>();

        /// After genesis, `LimitsDefinition<XUS>` is published at Diem root. It's published by
        /// AccountLimits::publish_unrestricted_limits, but we can't prove the condition there because
        /// it does not hold for all types (but does hold for XUS).
        invariant [suspendable] DiemTimestamp::is_operating()
            ==> exists<AccountLimits::LimitsDefinition<XUS>>(@DiemRoot);

        /// `LimitsDefinition<XUS>` is not published at any other address
        invariant [suspendable] forall addr: address where exists<AccountLimits::LimitsDefinition<XUS>>(addr):
            addr == @DiemRoot;

        /// After genesis, XUS is always a non-synthetic currency.
        invariant [suspendable] DiemTimestamp::is_operating()
            ==> !Diem::is_synthetic_currency<XUS>();

        /// After genesis, the scaling factor for XUS is always equal to 1,000,000.
        invariant [suspendable] DiemTimestamp::is_operating()
            ==> Diem::spec_currency_info<XUS>().scaling_factor == 1000000;

        /// After genesis, the fractional part for XUS is always equal to 100.
        invariant [suspendable] DiemTimestamp::is_operating()
            ==> Diem::spec_currency_info<XUS>().fractional_part == 100;

        /// After genesis, the currency code for XUS is always "XUS".
        invariant [suspendable] DiemTimestamp::is_operating()
            ==> Diem::spec_currency_code<XUS>() == b"XUS";
    }

    /// # Access Control

    /// ## Minting

    spec module {
        /// Only TreasuryCompliance can have MintCapability<XUS> [[H1]][PERMISSION].
        /// If an account has MintCapability<XUS>, it is a TreasuryCompliance account.
        invariant
            forall a: address:
                Diem::spec_has_mint_capability<XUS>(a) ==>
                    Roles::spec_has_treasury_compliance_role_addr(a);

        /// Only the owner of MintCapability<XUS> can mint XUS [[H1]][PERMISSION].
        /// If the `total_value` for XUS is increased, the transaction should be
        /// signed by the owner of MintCapability<XUS>.
        invariant update [suspendable] (
                old(Diem::spec_is_currency<XUS>()) &&
                Diem::spec_is_currency<XUS>() &&
                old(Diem::spec_currency_info<XUS>().total_value) < Diem::spec_currency_info<XUS>().total_value
            ) ==> Diem::spec_signed_by_mint_capability_owner<XUS>();

        /// The permission to mint XUS is unique [[I1]][PERMISSION].
        /// At most one address has a MintCapability<XUS>.
        invariant
            forall a1: address, a2: address:
                (Diem::spec_has_mint_capability<XUS>(a1) && Diem::spec_has_mint_capability<XUS>(a2)) ==> a1 == a2;

        /// MintCapability<XUS> is not transferrable [[J1]][PERMISSION].
        /// MintCapability<XUS> is not copiable, and once it's published, it's not removed.
        invariant update
            forall a: address
                where old(exists<Diem::MintCapability<XUS>>(a)):
                    exists<Diem::MintCapability<XUS>>(a);
    }

    /// ## Burning

    spec module {
        /// Only TreasuryCompliance can have BurnCapability [[H3]][PERMISSION].
        /// If an account has BurnCapability<XUS>, it is a TreasuryCompliance account.
        invariant
            forall a: address:
                Diem::spec_has_burn_capability<XUS>(a) ==>
                    Roles::spec_has_treasury_compliance_role_addr(a);

        /// Only the owner of BurnCapability<XUS> can burn XUS [[H3]][PERMISSION].
        /// If the `total_value` or `preburn_value` for XUS is decreased, the
        /// transaction should be signed by the owner of BurnCapability<XUS>.
        invariant update [suspendable] (
                old(Diem::spec_is_currency<XUS>()) &&
                Diem::spec_is_currency<XUS>() &&
                old(Diem::spec_currency_info<XUS>().total_value) > Diem::spec_currency_info<XUS>().total_value
            ) ==> Diem::spec_signed_by_burn_capability_owner<XUS>();
        invariant update [suspendable] (
                old(Diem::spec_is_currency<XUS>()) &&
                Diem::spec_is_currency<XUS>() &&
                old(Diem::spec_currency_info<XUS>().preburn_value) > Diem::spec_currency_info<XUS>().preburn_value
            ) ==> Diem::spec_signed_by_burn_capability_owner<XUS>();

        /// The permission to burn XUS is unique [[I3]][PERMISSION].
        /// At most one address has a BurnCapability<XUS>.
        invariant
            forall a1: address, a2: address:
                (Diem::spec_has_burn_capability<XUS>(a1) && Diem::spec_has_burn_capability<XUS>(a2)) ==> a1 == a2;

        /// BurnCapability<XUS> is not transferrable [[J3]][PERMISSION].
        /// BurnCapability<XUS> is not copiable, and once it's published, it's not removed.
        invariant update [suspendable]
            forall a: address
                where old(exists<Diem::BurnCapability<XUS>>(a)):
                    exists<Diem::BurnCapability<XUS>>(a);
    }

    /// ## Preburning

    spec module {
        /// Only DesignatedDealer can has the "preburn" permission [[H4]][PERMISSION].
        /// If an account has PreburnQueue<XUS> or Preburn<XUS>, it is a DesignatedDealer account.
        invariant
            forall a: address:
                (Diem::spec_has_preburn_queue<XUS>(a) || Diem::spec_has_preburn<XUS>(a)) ==>
                    Roles::spec_has_designated_dealer_role_addr(a);

        /// Only the owner of PreburnQueue<XUS> can preburn XUS [[H4]][PERMISSION].
        /// If the `preburn_value` for XUS is increased, the transaction should be
        /// signed by the owner of PreburnQueue<XUS> or Preburn<XUS>.
        invariant update [suspendable] (
                old(Diem::spec_is_currency<XUS>()) &&
                Diem::spec_is_currency<XUS>() &&
                old(Diem::spec_currency_info<XUS>().preburn_value) < Diem::spec_currency_info<XUS>().preburn_value
            ) ==> (Diem::spec_signed_by_preburn_queue_owner<XUS>() || Diem::spec_signed_by_preburn_owner<XUS>());

        /// PreburnQueue<XUS> is not transferrable [[J4]][PERMISSION].
        /// PreburnQueue<XUS> is not copiable, and once it's published, it's not removed.
        invariant update [suspendable]
            forall a: address
                where old(exists<Diem::PreburnQueue<XUS>>(a)):
                    exists<Diem::PreburnQueue<XUS>>(a);
    }
}
