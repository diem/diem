address 0x1 {

module Coin1 {
    use 0x1::Libra;
    use 0x1::FixedPoint32;

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
}

}
