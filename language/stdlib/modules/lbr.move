address 0x0 {

module LBR {
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::FixedPoint32;
    use 0x0::Libra;
    use 0x0::Transaction;

    // The type tag for this coin type.
    resource struct T { }

    // A reserve component holds one part of the LBR. It holds
    // both backing currency itself, along with the ratio of the backing
    // asset to the LBR (i.e. 1Coin1 to 1LBR ~> ratio = 1).
    resource struct ReserveComponent<CoinType> {
        // Specifies the relative ratio between `CoinType` and LBR (i.e. how
        // many `CoinType`s makeup one LBR).
        ratio: FixedPoint32::T,
        backing: Libra::T<CoinType>
    }

    // The reserve for the LBR holds both the capability of minting LBR
    // coins, and also each reserve component that backs these coins
    // on-chain.
    resource struct Reserve {
        mint_cap: Libra::MintCapability<T>,
        burn_cap: Libra::BurnCapability<T>,
        preburn_cap: Libra::Preburn<T>,
        coin1: ReserveComponent<Coin1::T>,
        coin2: ReserveComponent<Coin2::T>,
    }

    // Initialize the LBR module. This sets up the initial LBR ratios, and
    // creates the mint capability for LBR coins. The LBR currency must not
    // already be registered in order for this to succeed. The sender must
    // both be the correct address and have the correct permissions. These
    // restrictions are enforced in the Libra::register_currency function.
    public fun initialize() {
        // Register the LBR currency.
        Libra::register_currency<T>(
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            x"4C4252" // UTF8-encoded "LBR" as a hex string
        );
        let mint_cap = Libra::grant_mint_capability();
        let burn_cap = Libra::grant_burn_capability();
        let preburn_cap = Libra::new_preburn_with_capability(&burn_cap);
        let coin1 = ReserveComponent<Coin1::T> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin1::T>(),
        };
        let coin2 = ReserveComponent<Coin2::T> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin2::T>(),
        };
        move_to_sender(Reserve{ mint_cap, burn_cap, preburn_cap, coin1, coin2});
    }

    // Given the constituent coins return as much LBR as possible, with any
    // remainder in those coins returned back along with the newly minted LBR.
    public fun swap_into(
        coin1: Libra::T<Coin1::T>,
        coin2: Libra::T<Coin2::T>
    ): (Libra::T<T>, Libra::T<Coin1::T>, Libra::T<Coin2::T>)
    acquires Reserve {
        let reserve = borrow_global_mut<Reserve>(0xA550C18);
        let coin1_value = Libra::value(&coin1);
        let coin2_value = Libra::value(&coin2);
        if (coin1_value <= 1 || coin2_value <= 1) return (Libra::zero<T>(), coin1, coin2);
        let lbr_num_coin1 = FixedPoint32::divide_u64(coin1_value - 1, *&reserve.coin1.ratio);
        let lbr_num_coin2 = FixedPoint32::divide_u64(coin2_value - 1, *&reserve.coin2.ratio);
        let num_lbr = if (lbr_num_coin2 < lbr_num_coin1) {
            lbr_num_coin2
        } else {
            lbr_num_coin1
        };
        create(num_lbr, coin1, coin2)
    }

    // Create `amount_lbr` number of LBR from the passed in Coin1 and Coin2
    // coins. If enough is passed in, return the LBR along with any
    // remaining Coin1 and Coin2 coins.
    public fun create(
        amount_lbr: u64,
        coin1: Libra::T<Coin1::T>,
        coin2: Libra::T<Coin2::T>
    ): (Libra::T<T>, Libra::T<Coin1::T>, Libra::T<Coin2::T>)
    acquires Reserve {
        if (amount_lbr == 0) return (Libra::zero<T>(), coin1, coin2);
        let reserve = borrow_global_mut<Reserve>(0xA550C18);
        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
        let coin1_exact = Libra::withdraw(&mut coin1, num_coin1);
        let coin2_exact = Libra::withdraw(&mut coin2, num_coin2);
        Libra::deposit(&mut reserve.coin1.backing, coin1_exact);
        Libra::deposit(&mut reserve.coin2.backing, coin2_exact);
        (Libra::mint_with_capability<T>(amount_lbr, &reserve.mint_cap), coin1, coin2)
    }

    // Unpack a LBR coin and return the backing currencies (in the correct
    // amounts).
    public fun unpack(coin: Libra::T<T>): (Libra::T<Coin1::T>, Libra::T<Coin2::T>)
    acquires Reserve {
        let reserve = borrow_global_mut<Reserve>(0xA550C18);
        let ratio_multiplier = Libra::value(&coin);
        let sender = Transaction::sender();
        Libra::preburn_with_resource(coin, &mut reserve.preburn_cap, sender);
        Libra::burn_with_resource_cap(&mut reserve.preburn_cap, sender, &reserve.burn_cap);
        let coin1_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin1.ratio);
        let coin2_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin2.ratio);
        let coin1 = Libra::withdraw(&mut reserve.coin1.backing, coin1_amount);
        let coin2 = Libra::withdraw(&mut reserve.coin2.backing, coin2_amount);
        (coin1, coin2)
    }
}
}
