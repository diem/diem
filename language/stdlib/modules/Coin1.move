address 0x1 {

module Coin1 {
    use 0x1::AccountLimits;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    use 0x1::FixedPoint32;

    struct Coin1 { }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> Libra::is_currency<Coin1>();
    }

    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Libra::register_SCS_currency<Coin1>(
            lr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin1"
        );
        AccountLimits::publish_unrestricted_limits<Coin1>(lr_account);
    }
}
}
