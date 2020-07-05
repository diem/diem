address 0x1 {

module Coin1 {
    use 0x1::Libra;
    use 0x1::FixedPoint32;
    use 0x1::Roles;

    struct Coin1 { }

    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        // Register the Coin1 currency.
        let (coin1_mint_cap, coin1_burn_cap) =
            Libra::register_currency<Coin1>(
                lr_account,
                tc_account,
                FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
                false,   // is_synthetic
                1000000, // scaling_factor = 10^6
                100,     // fractional_part = 10^2
                b"Coin1"
            );
        Libra::publish_mint_capability<Coin1>(tc_account, coin1_mint_cap, tc_account);
        Libra::publish_burn_capability<Coin1>(tc_account, coin1_burn_cap, tc_account);
    }

// **************** SPECIFICATIONS  ****************

/// Only TreasuryComplianceRole may mint Coin1

/// BUG: This property verifies, but it does not necessarily hold.
/// Libra.move has a function `remove_mint_capability<CoinType>` that removes
/// a published mint capability and returns it as a value.  Unless I'm missing
/// something, another module (in a transaction signed by TREASURY_COMPLIANCE)
/// could use `remove_mint_capability` to obtain the capability and transfer
/// it in various ways so that another account could obtain references to it,
/// allowing that account to mint.
/// First priority is to make sure the prover (or our use of it) is sound.
/// Then we should fix the code, somehow.
spec schema Coin1MintBurnCapabilitiesStayAtTreasuryCompliance {
    invariant module
        !Libra::spec_is_currency<Coin1>()
            ==> (forall addr: address where true: !exists<Libra::MintCapability<Coin1>>(addr));
    invariant module
        Libra::spec_is_currency<Coin1>()
            ==> (forall addr: address
                     where exists<Libra::MintCapability<Coin1>>(addr): Roles::spec_has_treasury_compliance_role(addr));
}

spec module {
    apply Coin1MintBurnCapabilitiesStayAtTreasuryCompliance to *<T>, *;
}

// Really not sure how to do this!
// only TC account has MintCapability<Coin1>
// Only accounts with MintCapability<Token> may mint for Token
//  (unless they are LBR?).

}

}
