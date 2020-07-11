address 0x1 {

module Coin1 {
    use 0x1::Libra;
    use 0x1::FixedPoint32;
    use 0x1::Roles;
    use 0x1::RegisteredCurrencies;

    struct Coin1 { }

    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        // Register the Coin1 currency.
        let (coin1_mint_cap, coin1_burn_cap) =
            Libra::register_currency<Coin1>(
                lr_account,
                FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
                false,   // is_synthetic
                1000000, // scaling_factor = 10^6
                100,     // fractional_part = 10^2
                b"Coin1"
            );
        Libra::publish_mint_capability<Coin1>(tc_account, coin1_mint_cap, tc_account);
        Libra::publish_burn_capability<Coin1>(tc_account, coin1_burn_cap, tc_account);
    }
    spec fun initialize {
        include Libra::RegisterCurrencyAbortsIf<Coin1>;
        include RegisteredCurrencies::AddCurrencyCodeAbortsIf{currency_code: b"Coin1"};
        aborts_if !Roles::spec_has_treasury_compliance_role(tc_account);
        aborts_if Libra::spec_has_mint_capability<Coin1>(tc_account);
        aborts_if Libra::spec_has_burn_capability<Coin1>(tc_account);
        ensures Libra::spec_is_currency<Coin1>();
        ensures Libra::spec_has_mint_capability<Coin1>(tc_account);
        ensures Libra::spec_has_burn_capability<Coin1>(tc_account);
    }

    /// **************** MODULE SPECIFICATION ****************

    /// # Module Specification

    spec module {
        /// Verify all functions in this module.
        pragma verify = true;

        // TODO: A global property to verify is that once Coin1 is initialized, it remains registered.
        // This property may need to be verified in Libra not in Coin1.
    }

}

}
