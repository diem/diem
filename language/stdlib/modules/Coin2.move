address 0x1 {

module Coin2 {
    use 0x1::FixedPoint32;
    use 0x1::Libra::{Self, RegisterNewCurrency};
    use 0x1::Roles::Capability;

    struct Coin2 { }

    public fun initialize(
        account: &signer,
        cap: &Capability<RegisterNewCurrency>,
    ): (Libra::MintCapability<Coin2>, Libra::BurnCapability<Coin2>) {
        // Register the Coin2 currency.
        Libra::register_currency<Coin2>(
            account,
            cap,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            false,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin2",
        )
    }
}

}
