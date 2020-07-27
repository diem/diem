address 0x1 {
module DesignatedDealer {
    use 0x1::Errors;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    use 0x1::Vector;
    use 0x1::Event;
    use 0x1::Roles;
    use 0x1::Signer;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;

    /// A `DesignatedDealer` always holds this `Dealer` resource regardless of the
    /// currencies it can hold. All `ReceivedMintEvent` events for all
    /// currencies will be emitted on `mint_event_handle`.
    resource struct Dealer {
        /// Handle for mint events
        mint_event_handle: Event::EventHandle<ReceivedMintEvent>,
    }

    /// The `TierInfo` resource holds the information needed to track which
    /// tier a mint to a DD needs to be in.
    // Preburn published at top level in Libra.move
    resource struct TierInfo<CoinType> {
        /// Time window start in microseconds
        window_start: u64,
        /// The minted inflow during this time window
        window_inflow: u64,
        /// 0-indexed array of tier upperbounds
        tiers: vector<u64>,
    }
    spec struct TierInfo {
        invariant len(tiers) <= SPEC_MAX_NUM_TIERS();
        invariant forall i in 0..len(tiers), j in 0..len(tiers) where i < j: tiers[i] < tiers[j];
    }

    /// Message for mint events
    struct ReceivedMintEvent {
        /// The currency minted
        currency_code: vector<u8>,
        /// The address that receives the mint
        destination_address: address,
        /// The amount minted (in base units of `currency_code`)
        amount: u64,
    }

    /// Error codes
    const EDEALER: u64 = 0;
    const ELIMIT: u64 = 1;
    const EINVALID_TIER_ADDITION: u64 = 2;
    const EINVALID_TIER_START: u64 = 3;
    const EINVALID_TIER_INDEX: u64 = 4;
    const EINVALID_MINT_AMOUNT: u64 = 5;
    const EINVALID_AMOUNT_FOR_TIER: u64 = 6;

    /// Number of microseconds in a day
    const ONE_DAY: u64 = 86400000000;

    /// The maximum number of tiers allowed
    const MAX_NUM_TIERS: u64 = 4;

    /// Default FIAT amounts for tiers when a DD is created
    /// These get scaled by coin specific scaling factor on tier setting
    const TIER_0_DEFAULT: u64 = 500000; // ex $500,000
    const TIER_1_DEFAULT: u64 = 5000000;
    const TIER_2_DEFAULT: u64 = 50000000;
    const TIER_3_DEFAULT: u64 = 500000000;

    ///////////////////////////////////////////////////////////////////////////
    // To-be designated-dealer called functions
    ///////////////////////////////////////////////////////////////////////////

    /// Publishes a `Dealer` resource under `dd` with a `TierInfo`, `Preburn`, and default tiers for `CoinType`.
    /// If `add_all_currencies = true` this will add a `TierInfo`, `Preburn`,
    /// and default tiers for each known currency at launch.
    public fun publish_designated_dealer_credential<CoinType>(
        dd: &signer,
        tc_account: &signer,
        add_all_currencies: bool,
    ) acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert(!exists<Dealer>(Signer::address_of(tc_account)), Errors::already_published(EDEALER));
        move_to(dd, Dealer { mint_event_handle: Event::new_event_handle<ReceivedMintEvent>(dd) });
        if (add_all_currencies) {
            add_currency<Coin1>(dd, tc_account);
            add_currency<Coin2>(dd, tc_account);
        } else {
            add_currency<CoinType>(dd, tc_account);
        };
    }
    spec fun publish_designated_dealer_credential {
        /// TODO(wrwg): takes a long time but verifies.
        pragma verify_duration_estimate = 80;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs by Treasury Compliance Account
    ///////////////////////////////////////////////////////////////////////////

    /// Adds the needed resources to the DD account `dd` in order to work with `CoinType`.
    /// Public so that a currency can be added to a DD later on. Will require
    /// multi-signer transactions in order to add a new currency to an existing DD.
    public fun add_currency<CoinType>(dd: &signer, tc_account: &signer)
    acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        let dd_addr = Signer::address_of(dd);
        assert(exists_at(dd_addr), Errors::not_published(EDEALER));
        Libra::publish_preburn_to_account<CoinType>(dd, tc_account);
        assert(!exists<TierInfo<CoinType>>(dd_addr), Errors::already_published(EDEALER));
        move_to(dd, TierInfo<CoinType> {
            window_start: LibraTimestamp::now_microseconds(),
            window_inflow: 0,
            tiers: Vector::empty(),
        });
        // Add tier amounts in base_units of CoinType
        let coin_scaling_factor = Libra::scaling_factor<CoinType>();
        add_tier<CoinType>(tc_account, dd_addr, TIER_0_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_1_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_2_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_3_DEFAULT * coin_scaling_factor);
    }
    spec fun add_currency {
        /// TODO(wrwg): sort out strange behavior: verifies wo/ problem locally, but times out in Ci
        pragma verify = false;
    }

    public fun add_tier<CoinType>(
        tc_account: &signer,
        dd_addr: address,
        tier_upperbound: u64
    ) acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        let tiers = &mut borrow_global_mut<TierInfo<CoinType>>(dd_addr).tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(number_of_tiers + 1 <= MAX_NUM_TIERS, Errors::invalid_argument(EINVALID_TIER_ADDITION));
        if (number_of_tiers > 0) {
            let last_tier = *Vector::borrow(tiers, number_of_tiers - 1);
            assert(last_tier < tier_upperbound, Errors::invalid_argument(EINVALID_TIER_START));
        };
        Vector::push_back(tiers, tier_upperbound);
    }

    spec fun add_tier {
        // modifies global<TierInfo<CoinType>>@dd_addr.tiers;
        ensures len(global<TierInfo<CoinType>>(dd_addr).tiers) == len(old(global<TierInfo<CoinType>>(dd_addr)).tiers) + 1;
        ensures global<TierInfo<CoinType>>(dd_addr).tiers[len(global<TierInfo<CoinType>>(dd_addr).tiers) - 1] == tier_upperbound;
    }

    public fun update_tier<CoinType>(
        tc_account: &signer,
        dd_addr: address,
        tier_index: u64,
        new_upperbound: u64
    ) acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        let tiers = &mut borrow_global_mut<TierInfo<CoinType>>(dd_addr).tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index < number_of_tiers, Errors::invalid_argument(EINVALID_TIER_INDEX));
        // Make sure that this new start for the tier is consistent with the tier above and below it.
        let tier = Vector::borrow(tiers, tier_index);
        if (*tier == new_upperbound) return;
        if (*tier < new_upperbound) {
            let next_tier_index = tier_index + 1;
            if (next_tier_index < number_of_tiers) {
                assert(
                    new_upperbound < *Vector::borrow(tiers, next_tier_index),
                    Errors::invalid_argument(EINVALID_TIER_START)
                );
            };
        };
        if (*tier > new_upperbound && tier_index > 0) {
            let prev_tier_index = tier_index - 1;
            assert(
                new_upperbound > *Vector::borrow(tiers, prev_tier_index),
                Errors::invalid_argument(EINVALID_TIER_START)
            );
        };
        *Vector::borrow_mut(tiers, tier_index) = new_upperbound;
    }

    spec fun update_tier {
        // modifies global<TierInfo<CoinType>>@dd_addr.tiers;
        ensures len(global<TierInfo<CoinType>>(dd_addr).tiers) == len(old(global<TierInfo<CoinType>>(dd_addr)).tiers);
        ensures global<TierInfo<CoinType>>(dd_addr).tiers[tier_index] == new_upperbound;
    }

    public fun tiered_mint<CoinType>(
        tc_account: &signer,
        amount: u64,
        dd_addr: address,
        tier_index: u64,
    ): Libra::Libra<CoinType> acquires Dealer, TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert(amount > 0, Errors::invalid_argument(EINVALID_MINT_AMOUNT));
        assert(exists_at(dd_addr), Errors::not_published(EDEALER));

        validate_and_record_mint<CoinType>(dd_addr, amount, tier_index);
        // Send ReceivedMintEvent
        Event::emit_event<ReceivedMintEvent>(
            &mut borrow_global_mut<Dealer>(dd_addr).mint_event_handle,
            ReceivedMintEvent {
                currency_code: Libra::currency_code<CoinType>(),
                destination_address: dd_addr,
                amount: amount,
            },
        );
        Libra::mint<CoinType>(tc_account, amount)
    }

    spec fun tiered_mint {
        // modifies global<TierInfo<CoinType>>@dd_addr.{window_start, window_inflow, mint_event_handle}
        let dealer = global<TierInfo<CoinType>>(dd_addr);
        let current_time = LibraTimestamp::spec_now_microseconds();
        ensures old(dealer.window_start) <= dealer.window_start;
        ensures
            dealer.window_start == current_time && dealer.window_inflow == amount ||
            (old(dealer.window_start) == dealer.window_start &&
                dealer.window_inflow == old(dealer.window_inflow) + amount);
        ensures tier_index < len(old(dealer).tiers);
        ensures dealer.window_inflow <= old(dealer).tiers[tier_index];
    }

    public fun exists_at(dd_addr: address): bool {
        exists<Dealer>(dd_addr)
    }
    spec schema AbortsIfNotExistAt{
        dd_addr: address;
        aborts_if !exists<Dealer>(dd_addr) with Errors::NOT_PUBLISHED;
    }

    /// Validate and record the minting of `amount` of `CoinType` coins against
    /// the `TierInfo<CoinType>` for the DD at `dd_addr`. Aborts if an invalid
    /// `tier_index` is supplied, or if the inflow over the time period exceeds
    /// that amount that can be minted according to the bounds for the `tier_index` tier.
    fun validate_and_record_mint<CoinType>(dd_addr: address, amount: u64, tier_index: u64)
    acquires TierInfo {
        let tier = borrow_global_mut<TierInfo<CoinType>>(dd_addr);
        reset_window(tier);
        let cur_inflow = tier.window_inflow;
        let new_inflow = cur_inflow + amount;
        let tiers = &mut tier.tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index < number_of_tiers, Errors::invalid_argument(EINVALID_TIER_INDEX));
        let tier_upperbound: u64 = *Vector::borrow(tiers, tier_index);
        assert(new_inflow <= tier_upperbound, Errors::invalid_argument(EINVALID_AMOUNT_FOR_TIER));
        tier.window_inflow = new_inflow;
    }

    // If the time window starting at `dealer.window_start` and lasting for
    // `ONE_DAY` has elapsed, resets the window and the inflow and outflow records.
    fun reset_window<CoinType>(dealer: &mut TierInfo<CoinType>) {
        let current_time = LibraTimestamp::now_microseconds();
        if (current_time > dealer.window_start + ONE_DAY) {
            dealer.window_start = current_time;
            dealer.window_inflow = 0;
        }
    }

    spec module {
        pragma verify = true;
        define SPEC_MAX_NUM_TIERS(): u64 { 4 }
        define spec_window_length(): u64 { 86400000000 }
    }
}
}
