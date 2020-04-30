address 0x0:

module Coin2 {
    use 0x0::Libra;
    use 0x0::FixedPoint32;
    use 0x0::Sender;

    struct T { }

    public fun initialize(sender: &Sender::T) {
        // Register the LBR currency.
        Libra::register_currency<T>(
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            false,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            x"436F696E32", // UTF8 encoding of "Coin2" in hex
            sender,
        );
    }
}
