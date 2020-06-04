address 0x0 {

module Coin1 {
    use 0x0::Libra;
    use 0x0::Association;
    use 0x0::FixedPoint32;

    struct Coin1 { }

    public fun initialize(account: &signer): (Libra::MintCapability<Coin1>, Libra::BurnCapability<Coin1>) {
        Association::assert_is_association(account);
        // Register the Coin1 currency.
        Libra::register_currency<Coin1>(
            account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            false,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin1",
        )
    }
}

}
