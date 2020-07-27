address 0x1 {
/// This module defines the `LBR` currency as an on-chain reserve. The
/// `LBR` currency differs from other currencies on-chain, since anyone can
/// "atomically" swap into, and out-of the `LBR` as long as they hold the
/// underlying currencies. This is done by specifying the make up of, and
/// holding the reserve of backing currencies for the `LBR` on-chain.
/// Users can create `LBR` coins by passing in the backing
/// currencies, and can likewise "unpack" `LBR` to get the backing coins
/// for that coin. The liquidity of the reserve is enforced by the logic in
/// this module that ensures that the correct amount of each backing currency
/// is withdrawn on creation of an `LBR` coin, and that only the appropriate
/// amount of each coin is returned when an `LBR` coin is "unpacked."

module LBR {
    use 0x1::AccountLimits;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::Libra::{Self, Libra,
    // RegisterNewCurrency
    };
    use 0x1::LibraTimestamp;

    spec module {
        pragma verify;
    }

    /// The type tag representing the `LBR` currency on-chain.
    resource struct LBR { }

    /// A `ReserveComponent` holds one part of the on-chain reserve that backs
    /// `LBR` coins. Each `ReserveComponent` holds both the backing currency
    /// itself, along with the ratio of the backing currency to the `LBR` coin.
    /// For example, if `Coin1` makes up 1/2 of an `LBR`, then the `ratio` field would be 0.5.
    resource struct ReserveComponent<CoinType> {
        /// Specifies the relative ratio between the `CoinType` and `LBR` (i.e., how
        /// many `CoinType`s make up one `LBR`).
        ratio: FixedPoint32,
        /// Holds the `CoinType` backing coins for the on-chain reserve.
        backing: Libra<CoinType>
    }
    spec struct ReserveComponent {
        // The ratio cannot be zero.
        invariant !FixedPoint32::is_zero(ratio);
    }

    /// The on-chain reserve for the `LBR` holds both the capability for minting `LBR`
    /// coins, and also each reserve component that holds the backing for these coins on-chain.
    /// A crucial invariant of this on-chain reserve is that for each component
    /// `c_i`, `c_i.value/c_i.ratio >= LBR.market_cap`.
    /// e.g., if `coin1.ratio = 1/2` and `coin2.ratio = 1/2` and `LBR.market_cap ==
    /// 100`, then `coin1.value >= 50`, and `coin2.value >= 50`.
    resource struct Reserve {
        /// The mint capability allowing minting of `LBR` coins.
        mint_cap: Libra::MintCapability<LBR>,
        /// The burn capability for `LBR` coins. This is used for the unpacking
        /// of `LBR` coins into the underlying backing currencies.
        burn_cap: Libra::BurnCapability<LBR>,
        /// The preburn for `LBR`. This is an administrative field since we
        /// need to alway preburn before we burn.
        preburn_cap: Libra::Preburn<LBR>,
        /// The `Coin1` reserve component, holds the backing coins and ratio
        /// that needs to be held for the `Coin1` currency.
        coin1: ReserveComponent<Coin1>,
        /// The `Coin2` reserve component, holds the backing coins and ratio
        /// that needs to be held for the `Coin2` currency.
        coin2: ReserveComponent<Coin2>,
    }

    /// Global invariant that the Reserve resource exists after genesis.
    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> reserve_exists() && Libra::is_currency<LBR>();

        define reserve_exists(): bool {
           exists<Reserve>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }
    }

    /// TODO(wrwg): This should be provided somewhere centrally in the framework.
    const MAX_U64: u64 = 18446744073709551615;

    const ERESERVE: u64 = 0;
    const ECOIN1: u64 = 1;
    const ECOIN2: u64 = 2;
    const EZERO_LBR_MINT_NOT_ALLOWED: u64 = 3;

    /// Initializes the `LBR` module. This sets up the initial `LBR` ratios and
    /// reserve components, and creates the mint, preburn, and burn
    /// capabilities for `LBR` coins. The `LBR` currency must not already be
    /// registered in order for this to succeed. The sender must both be the
    /// correct address (`CoreAddresses::CURRENCY_INFO_ADDRESS`) and have the
    /// correct permissions (`&Capability<RegisterNewCurrency>`). Both of these
    /// restrictions are enforced in the `Libra::register_currency` function, but also enforced here.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_currency_info(lr_account);
        // Reserve must not exist.
        assert(!exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ERESERVE));
        let (mint_cap, burn_cap) = Libra::register_currency<LBR>(
            lr_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"LBR"
        );
        AccountLimits::publish_unrestricted_limits<LBR>(lr_account);
        let preburn_cap = Libra::create_preburn<LBR>(tc_account);
        let coin1 = ReserveComponent<Coin1> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin1>(),
        };
        let coin2 = ReserveComponent<Coin2> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin2>(),
        };
        move_to(lr_account, Reserve { mint_cap, burn_cap, preburn_cap, coin1, coin2 });
    }

    /// Returns true if `CoinType` is `LBR::LBR`
    public fun is_lbr<CoinType>(): bool {
        Libra::is_currency<CoinType>() &&
            Libra::currency_code<CoinType>() == Libra::currency_code<LBR>()
    }

    spec fun is_lbr {
        pragma verify = false, opaque = true;
        /// The following is correct because currency codes are unique.
        ensures result == spec_is_lbr<CoinType>();
    }

    /// Returns true if CoinType is LBR.
    spec define spec_is_lbr<CoinType>(): bool {
        type<CoinType>() == type<LBR>()
    }

    /// We take the truncated multiplication + 1 (not ceiling!) to withdraw for each currency that makes up the `LBR`.
    /// We do this to ensure that the reserve is always positive. We could do this with other more complex methods such as
    /// banker's rounding, but this adds considerable arithmetic complexity.
    public fun calculate_component_amounts_for_lbr(amount_lbr: u64): (u64, u64)
    acquires Reserve {
        LibraTimestamp::assert_operating();
        let reserve = borrow_global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let amount1 = FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
        let amount2 = FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
        assert(amount1 != MAX_U64, Errors::limit_exceeded(ECOIN1));
        assert(amount2 != MAX_U64, Errors::limit_exceeded(ECOIN2));
        (amount1 + 1, amount2 + 1)
    }

    /// Create `amount_lbr` number of `LBR` from the passed in coins. If
    /// enough of each coin is passed in, this will return the `LBR`.
    /// * If the passed in coins are not the exact amount needed to mint `amount_lbr` LBR, the function will abort.
    /// * If any of the coins passed-in do not hold a large enough balance--which is calculated as
    ///   `truncate(amount_lbr * reserve_component_c_i.ratio) + 1` for each coin
    ///   `c_i` passed in--the function will abort.
    /// * If `amount_lbr` is zero the function will abort.
    public fun create(
        amount_lbr: u64,
        coin1: Libra<Coin1>,
        coin2: Libra<Coin2>
    ): Libra<LBR>
    acquires Reserve {
        LibraTimestamp::assert_operating();
        assert(amount_lbr > 0, Errors::invalid_argument(EZERO_LBR_MINT_NOT_ALLOWED));
        let (num_coin1, num_coin2) = calculate_component_amounts_for_lbr(amount_lbr);
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        assert(num_coin1 == Libra::value(&coin1), Errors::invalid_argument(ECOIN1));
        assert(num_coin2 == Libra::value(&coin2), Errors::invalid_argument(ECOIN2));
        // Deposit the coins in to the reserve
        Libra::deposit(&mut reserve.coin1.backing, coin1);
        Libra::deposit(&mut reserve.coin2.backing, coin2);
        // Once the coins have been deposited in the reserve, we can mint the LBR
        Libra::mint_with_capability<LBR>(amount_lbr, &reserve.mint_cap)
    }

    /// Unpacks an `LBR` coin, and returns the backing coins that make up the
    /// coin based upon the ratios defined for each `ReserveComponent` in the
    /// `Reserve` resource. The value of each constituent coin that is
    /// returned is the truncated value of the coin to the nearest base
    /// currency unit w.r.t. to the `ReserveComponent` ratio for the currency of
    /// the coin and the value of `coin`. e.g.,, if `coin = 10` and `LBR` is
    /// defined as `2/3` `Coin1` and `1/3` `Coin2`, then the values returned
    /// would be `6` and `3` for `Coin1` and `Coin2` respectively.
    public fun unpack(coin: Libra<LBR>): (Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        LibraTimestamp::assert_operating();
        let reserve = borrow_global_mut<Reserve>(reserve_address());
        let ratio_multiplier = Libra::value(&coin);
        let sender = reserve_address();
        Libra::burn_now(coin, &mut reserve.preburn_cap, sender, &reserve.burn_cap);
        let coin1_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin1.ratio);
        let coin2_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin2.ratio);
        let coin1 = Libra::withdraw(&mut reserve.coin1.backing, coin1_amount);
        let coin2 = Libra::withdraw(&mut reserve.coin2.backing, coin2_amount);
        (coin1, coin2)
    }
    spec fun unpack {
        pragma verify_duration_estimate = 100; // TODO: occasionally times out
        include UnpackAbortsIf;
        ensures Libra::spec_market_cap<LBR>() == old(Libra::spec_market_cap<LBR>()) - coin.value;
        ensures result_1.value == spec_unpack_coin1(coin);
        ensures result_2.value == spec_unpack_coin2(coin);
    }
    spec schema UnpackAbortsIf {
        coin: Libra<LBR>;
        include LibraTimestamp::AbortsIfNotOperating;
        let reserve = global<Reserve>(reserve_address());
        include Libra::BurnNowAbortsIf<LBR>{preburn: reserve.preburn_cap};
        /// > TODO(wrwg): It appears the next couple of aborts inclusions are redundant, i.e. they can be
        /// > removed but still no abort is reported. It is unclear why this is the case. For example,
        /// > the coin value could be so larged that multiply overflows, or the reserve could not have backing.
        //  > Need to investigate why this is the case. Notice that keeping them also does not produce an error,
        /// > indicating the the solver determines their conditions can never become true.
        include FixedPoint32::MultiplyAbortsIf{val: coin.value, multiplier: reserve.coin1.ratio};
        include FixedPoint32::MultiplyAbortsIf{val: coin.value, multiplier: reserve.coin2.ratio};
        include Libra::WithdrawAbortsIf<Coin1>{coin: reserve.coin1.backing, amount: spec_unpack_coin1(coin)};
        include Libra::WithdrawAbortsIf<Coin2>{coin: reserve.coin2.backing, amount: spec_unpack_coin2(coin)};
    }
    spec define spec_unpack_coin1(coin: Libra<LBR>): u64 {
        let reserve = global<Reserve>(reserve_address());
        FixedPoint32::spec_multiply_u64(coin.value, reserve.coin1.ratio)
    }
    spec define spec_unpack_coin2(coin: Libra<LBR>): u64 {
        let reserve = global<Reserve>(reserve_address());
        FixedPoint32::spec_multiply_u64(coin.value, reserve.coin2.ratio)
    }

    /// Return the account address where the globally unique LBR::Reserve resource is stored
    public fun reserve_address(): address {
        CoreAddresses::CURRENCY_INFO_ADDRESS()
    }

    // **************** SPECIFICATIONS ****************

    /*
    This module defines the synthetic coin type called LBR and the operations on LBR coins. A global property that this
    module has to satisfy is as follows: LBR must be backed by the reserve of fiat coins in order to exist. In the
    current system, there are two fiat coins called coin1 and coin2. So, there must be a sufficient amounts of coin1
    and coin2 respectively in the reserve to be backing LBR. Here, the "sufficient amount" is determined by the
    pre-defined ratio of each of the fiat coins to the total value of LBR. To define this global property more precisely,

    let reserve_coin1 refer to global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()).coin1 (the reserve of coin1 backing LBR)
    let reserve_coin2 refer to global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()).coin2 (the reserve of coin2 backing LBR).
    Let lbr_total_value be the synthetic variable that represents the total amount of LBR that exists.
    Note: lbr_total_value could refer to global<Libra::CurrencyInfo<LBR::T>>(CoreAddresses::LIBRA_ROOT_ADDRESS()).total_value, but this may make
    verification harder because one need prove a relational invariant of two modules (such as Libra and LBR).
    The module invariant can be formulated as:
    (1) lbr_total_value * r_coin1.ratio <= reserve_coin1.backing.value, and
    (2) lbr_total_value * r_coin2.ratio <= reserve_coin2.backing.value
    where '*' is the multiplication over real numbers. (Yet, it could be the FP multiplication. It should not matter.)

    Note that to argue this, the system needs to satisfy the following property (beyond the scope of this module):
    LBR coins should be created only through LBR::create, and there is no other way in the system. Specifically,
    Libra::mint<LBR::T> should not be able to create LBR coins because if so, the invariant above may not be guaranteed.
    */
}
}
