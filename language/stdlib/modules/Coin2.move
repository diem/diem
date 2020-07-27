address 0x1 {

module Coin2 {
    use 0x1::AccountLimits;
    use 0x1::FixedPoint32;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;

    struct Coin2 { }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> Libra::is_currency<Coin2>();
    }

    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Libra::register_SCS_currency<Coin2>(
            lr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin2",
        );
        AccountLimits::publish_unrestricted_limits<Coin2>(lr_account);
    }
}
}
