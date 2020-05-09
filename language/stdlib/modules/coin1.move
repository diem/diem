address 0x0 {

module Coin1 {
    use 0x0::Libra;
    use 0x0::Association;
    use 0x0::FixedPoint32;

    struct T { }

    public fun initialize(account: &signer): (Libra::MintCapability<T>, Libra::BurnCapability<T>) {
        Association::assert_sender_is_association();
        // Register the Coin1 currency.
        Libra::register_currency<T>(
            account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            false,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            x"436F696E31", // UTF8 encoding of "Coin1" in hex
        )
    }
}

}
