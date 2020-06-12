address 0x1 {

module LBR {
    use 0x1::CoreAddresses;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::Libra::{Self, Libra};
    use 0x1::Signer;

    // The type tag for this coin type.
    resource struct LBR { }

    // A reserve component holds one part of the LBR. It holds
    // both backing currency itself, along with the ratio of the backing
    // asset to the LBR (i.e. 1Coin1 to 1LBR ~> ratio = 1).
    resource struct ReserveComponent<CoinType> {
        // Specifies the relative ratio between `CoinType` and LBR (i.e. how
        // many `CoinType`s makeup one LBR).
        ratio: FixedPoint32,
        backing: Libra<CoinType>
    }

    // The reserve for the LBR holds both the capability of minting LBR
    // coins, and also each reserve component that backs these coins
    // on-chain.
    resource struct Reserve {
        mint_cap: Libra::MintCapability<LBR>,
        burn_cap: Libra::BurnCapability<LBR>,
        preburn_cap: Libra::Preburn<LBR>,
        coin1: ReserveComponent<Coin1>,
        coin2: ReserveComponent<Coin2>,
    }

    // Initialize the LBR module. This sets up the initial LBR ratios, and
    // creates the mint capability for LBR coins. The LBR currency must not
    // already be registered in order for this to succeed. The sender must
    // both be the correct address and have the correct permissions. These
    // restrictions are enforced in the Libra::register_currency function.
    public fun initialize(association: &signer) {
        assert(Signer::address_of(association) == 0xA550C18, 0);
        // Register the LBR currency.
        let (mint_cap, burn_cap) = Libra::register_currency<LBR>(
            association,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"LBR"
        );
        let preburn_cap = Libra::create_preburn<LBR>(association);
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

    /// Return true if CoinType is LBR::LBR
    public fun is_lbr<CoinType>(): bool {
        Libra::is_currency<CoinType>() &&
            Libra::currency_code<CoinType>() == Libra::currency_code<LBR>()
    }

    // Given the constituent coins return as much LBR as possible, with any
    // remainder in those coins returned back along with the newly minted LBR.
    public fun swap_into(
        coin1: Libra<Coin1>,
        coin2: Libra<Coin2>
    ): (Libra<LBR>, Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let coin1_value = Libra::value(&coin1);
        let coin2_value = Libra::value(&coin2);
        if (coin1_value <= 1 || coin2_value <= 1) return (Libra::zero<LBR>(), coin1, coin2);
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
        coin1: Libra<Coin1>,
        coin2: Libra<Coin2>
    ): (Libra<LBR>, Libra<Coin1>, Libra<Coin2>)
    acquires Reserve {
        if (amount_lbr == 0) return (Libra::zero<LBR>(), coin1, coin2);
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let num_coin1 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin1.ratio);
        let num_coin2 = 1 + FixedPoint32::multiply_u64(amount_lbr, *&reserve.coin2.ratio);
        let coin1_exact = Libra::withdraw(&mut coin1, num_coin1);
        let coin2_exact = Libra::withdraw(&mut coin2, num_coin2);
        Libra::deposit(&mut reserve.coin1.backing, coin1_exact);
        Libra::deposit(&mut reserve.coin2.backing, coin2_exact);
        (Libra::mint_with_capability<LBR>(amount_lbr, &reserve.mint_cap), coin1, coin2)
    }

    // Unpack a LBR coin and return the backing currencies (in the correct
    // amounts).
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

    // Create `amount_lbr` LBR using the `MintCapability` for the coin types in the reserve.
    // Aborts if the caller does not have the appropriate `MintCapability`'s
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
