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
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::CoreAddresses;
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::Libra::{Self, Libra,  RegisterNewCurrency};
    use 0x1::Roles::{Capability, TreasuryComplianceRole};
    use 0x1::Signer;

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

    /// Initializes the `LBR` module. This sets up the initial `LBR` ratios and
    /// reserve components, and creates the mint, preburn, and burn
    /// capabilities for `LBR` coins. The `LBR` currency must not already be
    /// registered in order for this to succeed. The sender must both be the
    /// correct address (`CoreAddresses::CURRENCY_INFO_ADDRESS`) and have the
    /// correct permissions (`&Capability<RegisterNewCurrency>`). Both of these
    /// restrictions are enforced in the `Libra::register_currency` function, but also enforced here.
    public fun initialize(
        association: &signer,
        register_currency_capability: &Capability<RegisterNewCurrency>,
        tc_capability: &Capability<TreasuryComplianceRole>,
    ) {
        // Operational constraint
        assert(Signer::address_of(association) == CoreAddresses::CURRENCY_INFO_ADDRESS(), 0);
        // Register the `LBR` currency.
        let (mint_cap, burn_cap) = Libra::register_currency<LBR>(
            association,
            register_currency_capability,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"LBR"
        );
        let preburn_cap = Libra::create_preburn<LBR>(tc_capability);
        let coin1 = ReserveComponent<Coin1> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin1>(),
        };
        let coin2 = ReserveComponent<Coin2> {
            ratio: FixedPoint32::create_from_rational(1, 2),
            backing: Libra::zero<Coin2>(),
        };
        move_to(association, Reserve { mint_cap, burn_cap, preburn_cap, coin1, coin2 });
    }

    /// Returns true if `CoinType` is `LBR::LBR`
    public fun is_lbr<CoinType>(): bool {
        Libra::is_currency<CoinType>() &&
            Libra::currency_code<CoinType>() == Libra::currency_code<LBR>()
    }

    /// Given the constituent coins `coin1` and `coin2` of the `LBR`, this
    /// function calculates the maximum amount of `LBR` that can be returned
    /// for the values of the passed-in coins with respect to the coin ratios
    /// that are specified in the respective currency's  `ReserveComponent`.
    /// Any remaining amounts in the passed-in coins are returned back out
    /// along with any newly-created `LBR` coins.
    /// If either of the coin's values are less than or equal to 1, then the
    /// function returns the passed-in coins along with zero `LBR`.
    /// In order to ensure that the on-chain reserve remains liquid while
    /// minimizing complexity, the values needed to create an `LBR` are the
    /// truncated division of the passed-in coin with `1` base currency unit added.
    /// This is different from rounding, or ceiling; e.g., `ceil(10 /
    /// 2) = 5` whereas we would calculate this as `trunc(10/5) + 1 = 2 + 1 = 3`.
    public fun swap_into(
        coin1: Libra<Coin1>,
        coin2: Libra<Coin2>
    ): (Libra<LBR>, Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        // Grab the reserve
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let coin1_value = Libra::value(&coin1);
        let coin2_value = Libra::value(&coin2);
        // If either of the coin's values is <= 1, then we don't create any LBR
        if (coin1_value <= 1 || coin2_value <= 1) return (Libra::zero<LBR>(), coin1, coin2);
        let lbr_num_coin1 = FixedPoint32::divide_u64(coin1_value - 1, *&reserve.coin1.ratio);
        let lbr_num_coin2 = FixedPoint32::divide_u64(coin2_value - 1, *&reserve.coin2.ratio);
        // The number of `LBR` that can be minted is the minimum of the amount
        // that could be possibly minted according to the value of the coin
        // passed in and that coin's ratio in the reserve.
        let num_lbr = if (lbr_num_coin2 < lbr_num_coin1) {
            lbr_num_coin2
        } else {
            lbr_num_coin1
        };
        create(num_lbr, coin1, coin2)
    }

    /// Create `amount_lbr` number of `LBR` from the passed in coins. If
    /// enough of each coin is passed in, this will return the `LBR` along with any
    /// remaining balances in the passed in coins. If any of the
    /// coins passed-in do not hold a large enough balance--which is calculated as
    /// `truncate(amount_lbr * reserve_component_c_i.ratio) + 1` for each coin
    /// `c_i` passed in--the function will abort.
    public fun create(
        amount_lbr: u64,
        coin1: Libra<Coin1>,
        coin2: Libra<Coin2>
    ): (Libra<LBR>, Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        if (amount_lbr == 0) return (Libra::zero<LBR>(), coin1, coin2);
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        // We take the truncated multiplication + 1 (not ceiling!) to withdraw for each currency.
        // This is because we want to ensure that the reserve is always
        // positive. We could do this with other more complex methods such as
        // bankers rounding, but this adds considerable arithmetic complexity.
        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
        let coin1_exact = Libra::withdraw(&mut coin1, num_coin1);
        let coin2_exact = Libra::withdraw(&mut coin2, num_coin2);
        // Deposit the coins in to the reserve
        Libra::deposit(&mut reserve.coin1.backing, coin1_exact);
        Libra::deposit(&mut reserve.coin2.backing, coin2_exact);
        // Once the coins have been deposited in the reserve, we can mint the LBR
        (Libra::mint_with_capability<LBR>(amount_lbr, &reserve.mint_cap), coin1, coin2)
    }

    /// Unpacks an `LBR` coin, and returns the backing coins that make up the
    /// coin based upon the ratios defined for each `ReserveComponent` in the
    /// `Reserve` resource. The value of each constituent coin that is
    /// returned is the truncated value of the coin to the nearest base
    /// currency unit w.r.t. to the `ReserveComponent` ratio for the currency of
    /// the coin and the value of `coin`. e.g.,, if `coin = 10` and `LBR` is
    /// defined as `2/3` `Coin1` and `1/3` `Coin2`, then the values returned
    /// would be `6` and `3` for `Coin1` and `Coin2` respectively.
    public fun unpack(account: &signer, coin: Libra<LBR>): (Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let ratio_multiplier = Libra::value(&coin);
        let sender = Signer::address_of(account);
        Libra::preburn_with_resource(coin, &mut reserve.preburn_cap, sender);
        Libra::burn_with_resource_cap(&mut reserve.preburn_cap, sender, &reserve.burn_cap);
        let coin1_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin1.ratio);
        let coin2_amount = FixedPoint32::multiply_u64(ratio_multiplier, *&reserve.coin2.ratio);
        let coin1 = Libra::withdraw(&mut reserve.coin1.backing, coin1_amount);
        let coin2 = Libra::withdraw(&mut reserve.coin2.backing, coin2_amount);
        (coin1, coin2)
    }

    /// Creates `amount_lbr` `LBR` using the `MintCapability` for the backing currencies in the reserve.
    /// Calls to this will abort if the caller does not have the appropriate
    /// `MintCapability` for each of the backing currencies.
    public fun mint(account: &signer, amount_lbr: u64): Libra<LBR> acquires Reserve {
        let reserve = borrow_global<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
        let coin1 = Libra::mint<Coin1>(account, num_coin1);
        let coin2 = Libra::mint<Coin2>(account, num_coin2);
        let (lbr, leftover1, leftover2) = create(amount_lbr, coin1, coin2);
        Libra::destroy_zero(leftover1);
        Libra::destroy_zero(leftover2);
        lbr
    }

    // **************** SPECIFICATIONS ****************

    /*
    This module defines the synthetic coin type called LBR and the operations on LBR coins. A global property that this
    module has to satisfy is as follows: LBR must be backed by the reserve of fiat coins in order to exist. In the
    current system, there are two fiat coins called coin1 and coin2. So, there must be a sufficient amounts of coin1
    and coin2 respectively in the reserve to be backing LBR. Here, the "sufficient amount" is determined by the
    pre-defined ratio of each of the fiat coins to the total value of LBR. To define this global property more precisely,

    let reserve_coin1 refer to global<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).coin1 (the reserve of coin1 backing LBR)
    let reserve_coin2 refer to global<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).coin2 (the reserve of coin2 backing LBR).
    Let lbr_total_value be the synthetic variable that represents the total amount of LBR that exists.
    Note: lbr_total_value could refer to global<Libra::CurrencyInfo<LBR::T>>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).total_value, but this may make
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
