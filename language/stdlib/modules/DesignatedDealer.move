address 0x1 {

/// Module providing functionality for designated dealers.
module DesignatedDealer {
    use 0x1::Errors;
    use 0x1::Diem;
    use 0x1::DiemTimestamp;
    use 0x1::Vector;
    use 0x1::Event;
    use 0x1::Roles;
    use 0x1::Signer;
    use 0x1::XUS::XUS;

    /// A `DesignatedDealer` always holds this `Dealer` resource regardless of the
    /// currencies it can hold. All `ReceivedMintEvent` events for all
    /// currencies will be emitted on `mint_event_handle`.
    resource struct Dealer {
        /// Handle for mint events
        mint_event_handle: Event::EventHandle<ReceivedMintEvent>,
    }

    spec schema AbortsIfNoDealer {
        dd_addr: address;
        aborts_if !exists<Dealer>(dd_addr) with Errors::NOT_PUBLISHED;
    }

    /// The `TierInfo` resource holds the information needed to track which
    /// tier a mint to a DD needs to be in.
    // Preburn published at top level in Diem.move
    resource struct TierInfo<CoinType> {
        /// Time window start in microseconds
        window_start: u64,
        /// The minted inflow during this time window
        window_inflow: u64,
        /// 0-indexed array of tier upperbounds
        tiers: vector<u64>,
    }
    spec struct TierInfo {
        /// The number of tiers is limited.
        invariant len(tiers) <= MAX_NUM_TIERS;

        /// Tiers are ordered.
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

    /// The `DesignatedDealer` resource is in an invalid state
    const EDEALER: u64 = 0;
    /// The maximum number of tiers (4) has already been reached
    const EINVALID_TIER_ADDITION: u64 = 1;
    /// The starting value for the tier overlaps with the tier below or above it
    const EINVALID_TIER_START: u64 = 2;
    /// The tier index is out-of-bounds
    const EINVALID_TIER_INDEX: u64 = 3;
    /// A zero mint amount was provided
    const EINVALID_MINT_AMOUNT: u64 = 4;
    /// The maximum amount of money that can be minted for the tier has been reached
    const EINVALID_AMOUNT_FOR_TIER: u64 = 5;

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
        Roles::assert_designated_dealer(dd);
        assert(!exists<Dealer>(Signer::address_of(dd)), Errors::already_published(EDEALER));
        move_to(dd, Dealer { mint_event_handle: Event::new_event_handle<ReceivedMintEvent>(dd) });
        if (add_all_currencies) {
            add_currency<XUS>(dd, tc_account);
        } else {
            add_currency<CoinType>(dd, tc_account);
        };
    }
    spec fun publish_designated_dealer_credential {
        pragma opaque;

        let dd_addr = Signer::spec_address_of(dd);

        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include Roles::AbortsIfNotDesignatedDealer{account: dd};
        aborts_if exists<Dealer>(dd_addr) with Errors::ALREADY_PUBLISHED;
        include if (add_all_currencies) AddCurrencyAbortsIf<XUS>{dd_addr: dd_addr}
                else AddCurrencyAbortsIf<CoinType>{dd_addr: dd_addr};

        modifies global<Dealer>(dd_addr);
        ensures exists<Dealer>(dd_addr);
        modifies global<TierInfo<CoinType>>(dd_addr), global<TierInfo<XUS>>(dd_addr);
        ensures if (add_all_currencies) exists<TierInfo<XUS>>(dd_addr) else exists<TierInfo<CoinType>>(dd_addr);
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
        Diem::publish_preburn_to_account<CoinType>(dd, tc_account);
        assert(!exists<TierInfo<CoinType>>(dd_addr), Errors::already_published(EDEALER));
        move_to(dd, TierInfo<CoinType> {
            window_start: DiemTimestamp::now_microseconds(),
            window_inflow: 0,
            tiers: Vector::empty(),
        });
        // Add tier amounts in base_units of CoinType
        let coin_scaling_factor = Diem::scaling_factor<CoinType>();
        add_tier<CoinType>(tc_account, dd_addr, TIER_0_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_1_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_2_DEFAULT * coin_scaling_factor);
        add_tier<CoinType>(tc_account, dd_addr, TIER_3_DEFAULT * coin_scaling_factor);
    }
    spec fun add_currency {
        pragma opaque;

        let dd_addr = Signer::spec_address_of(dd);

        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include Roles::AbortsIfNotDesignatedDealer{account: dd};
        include AbortsIfNoDealer{dd_addr: dd_addr};
        include AddCurrencyAbortsIf<CoinType>{dd_addr: dd_addr};

        modifies global<Diem::Preburn<CoinType>>(dd_addr);
        modifies global<TierInfo<CoinType>>(dd_addr);
        ensures exists<TierInfo<CoinType>>(dd_addr);
        ensures global<TierInfo<CoinType>>(dd_addr) ==
            TierInfo<CoinType> {
                window_start: DiemTimestamp::spec_now_microseconds(),
                window_inflow: 0,
                tiers: global<TierInfo<CoinType>>(dd_addr).tiers,
            };
        ensures len(global<TierInfo<CoinType>>(dd_addr).tiers) == MAX_NUM_TIERS;
    }
    spec schema AddCurrencyAbortsIf<CoinType> {
        dd_addr: address;
        aborts_if exists<TierInfo<CoinType>>(dd_addr) with Errors::ALREADY_PUBLISHED;
        include Diem::AbortsIfNoCurrency<CoinType>;
        aborts_if Diem::is_synthetic_currency<CoinType>() with Errors::INVALID_ARGUMENT;
        aborts_if exists<Diem::Preburn<CoinType>>(dd_addr) with Errors::ALREADY_PUBLISHED;
        include DiemTimestamp::AbortsIfNotOperating;
    }

    fun add_tier<CoinType>(
        tc_account: &signer,
        dd_addr: address,
        tier_upperbound: u64
    ) acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert(exists<TierInfo<CoinType>>(dd_addr), Errors::not_published(EDEALER));
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
        pragma opaque;

        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AbortsIfNoTierInfo<CoinType>;
        let tier_info = global<TierInfo<CoinType>>(dd_addr);
        let number_of_tiers = len(tier_info.tiers);
        aborts_if number_of_tiers == MAX_NUM_TIERS with Errors::INVALID_ARGUMENT;
        aborts_if number_of_tiers > 0 && tier_info.tiers[number_of_tiers - 1] >= tier_upperbound with Errors::INVALID_ARGUMENT;

        modifies global<TierInfo<CoinType>>(dd_addr);
        ensures exists<TierInfo<CoinType>>(dd_addr);
        ensures global<TierInfo<CoinType>>(dd_addr) ==
            TierInfo<CoinType> {
                window_start: old(global<TierInfo<CoinType>>(dd_addr)).window_start,
                window_inflow: old(global<TierInfo<CoinType>>(dd_addr)).window_inflow,
                tiers: concat_vector(old(global<TierInfo<CoinType>>(dd_addr)).tiers, singleton_vector(tier_upperbound)),
            };
    }
    spec schema AbortsIfNoTierInfo<CoinType> {
        dd_addr: address;
        aborts_if !exists<TierInfo<CoinType>>(dd_addr) with Errors::NOT_PUBLISHED;
    }


    public fun update_tier<CoinType>(
        tc_account: &signer,
        dd_addr: address,
        tier_index: u64,
        new_upperbound: u64
    ) acquires TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert(exists<TierInfo<CoinType>>(dd_addr), Errors::not_published(EDEALER));
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
        pragma opaque;

        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AbortsIfNoTierInfo<CoinType>;
        let tier_info = global<TierInfo<CoinType>>(dd_addr);
        aborts_if tier_index >= len(tier_info.tiers) with Errors::INVALID_ARGUMENT;
        aborts_if tier_index > 0 && tier_info.tiers[tier_index - 1] >= new_upperbound with Errors::INVALID_ARGUMENT;
        aborts_if tier_index + 1 < len(tier_info.tiers) && tier_info.tiers[tier_index + 1] <= new_upperbound with Errors::INVALID_ARGUMENT;

        modifies global<TierInfo<CoinType>>(dd_addr);
        ensures exists<TierInfo<CoinType>>(dd_addr);
        ensures global<TierInfo<CoinType>>(dd_addr) ==
            TierInfo<CoinType> {
                window_start: old(global<TierInfo<CoinType>>(dd_addr)).window_start,
                window_inflow: old(global<TierInfo<CoinType>>(dd_addr)).window_inflow,
                tiers: update_vector(old(global<TierInfo<CoinType>>(dd_addr)).tiers, tier_index, new_upperbound),
            };
    }

    public fun tiered_mint<CoinType>(
        tc_account: &signer,
        amount: u64,
        dd_addr: address,
        tier_index: u64,
    ): Diem::Diem<CoinType> acquires Dealer, TierInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert(amount > 0, Errors::invalid_argument(EINVALID_MINT_AMOUNT));
        assert(exists_at(dd_addr), Errors::not_published(EDEALER));
        assert(exists<TierInfo<CoinType>>(dd_addr), Errors::not_published(EDEALER));

        validate_and_record_mint<CoinType>(dd_addr, amount, tier_index);
        // Send ReceivedMintEvent
        Event::emit_event<ReceivedMintEvent>(
            &mut borrow_global_mut<Dealer>(dd_addr).mint_event_handle,
            ReceivedMintEvent {
                currency_code: Diem::currency_code<CoinType>(),
                destination_address: dd_addr,
                amount: amount,
            },
        );
        Diem::mint<CoinType>(tc_account, amount)
    }
    spec fun tiered_mint {
        use 0x1::CoreAddresses;
        pragma opaque;

        include TieredMintAbortsIf<CoinType>;

        modifies global<Diem::CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures exists<Diem::CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        modifies global<TierInfo<CoinType>>(dd_addr);
        ensures exists<TierInfo<CoinType>>(dd_addr);
        ensures global<TierInfo<CoinType>>(dd_addr).tiers == old(global<TierInfo<CoinType>>(dd_addr).tiers);
        let dealer = global<TierInfo<CoinType>>(dd_addr);
        let current_time = DiemTimestamp::spec_now_microseconds();
        let currency_info = global<Diem::CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures old(dealer.window_start) <= dealer.window_start;
        ensures
            dealer.window_start == current_time && dealer.window_inflow == amount ||
            (old(dealer.window_start) == dealer.window_start &&
                dealer.window_inflow == old(dealer.window_inflow) + amount);
        ensures tier_index < len(old(dealer).tiers);
        ensures dealer.window_inflow <= old(dealer).tiers[tier_index];
        ensures result.value == amount;
        ensures currency_info == update_field(old(currency_info), total_value, old(currency_info.total_value) + amount);
    }
    spec schema TieredMintAbortsIf<CoinType> {
        tc_account: signer;
        dd_addr: address;
        amount: u64;
        tier_index: u64;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if amount == 0 with Errors::INVALID_ARGUMENT;
        include AbortsIfNoDealer;
        include AbortsIfNoTierInfo<CoinType>;
        let tier_info = global<TierInfo<CoinType>>(dd_addr);
        aborts_if tier_index >= len(tier_info.tiers) with Errors::INVALID_ARGUMENT;
        let new_amount = if (DiemTimestamp::spec_now_microseconds() <= tier_info.window_start + ONE_DAY) { tier_info.window_inflow + amount } else { amount };
        aborts_if new_amount > tier_info.tiers[tier_index] with Errors::INVALID_ARGUMENT;
        include DiemTimestamp::AbortsIfNotOperating;
        aborts_if !exists<Diem::MintCapability<CoinType>>(Signer::spec_address_of(tc_account)) with Errors::REQUIRES_CAPABILITY;
        include Diem::MintAbortsIf<CoinType>{value: amount};
    }

    public fun exists_at(dd_addr: address): bool {
        exists<Dealer>(dd_addr)
    }
    spec fun exists_at {
        pragma opaque;
        aborts_if false;
        ensures result == exists<Dealer>(dd_addr);
    }

    /// Validate and record the minting of `amount` of `CoinType` coins against
    /// the `TierInfo<CoinType>` for the DD at `dd_addr`. Aborts if an invalid
    /// `tier_index` is supplied, or if the inflow over the time period exceeds
    /// that amount that can be minted according to the bounds for the `tier_index` tier.
    fun validate_and_record_mint<CoinType>(dd_addr: address, amount: u64, tier_index: u64)
    acquires TierInfo {
        let tier_info = borrow_global_mut<TierInfo<CoinType>>(dd_addr);
        reset_window(tier_info);
        let cur_inflow = tier_info.window_inflow;
        let tiers = &tier_info.tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index < number_of_tiers, Errors::invalid_argument(EINVALID_TIER_INDEX));
        let tier_upperbound: u64 = *Vector::borrow(tiers, tier_index);
        assert(amount <= tier_upperbound, Errors::invalid_argument(EINVALID_AMOUNT_FOR_TIER));
        assert(cur_inflow <= tier_upperbound - amount, Errors::invalid_argument(EINVALID_AMOUNT_FOR_TIER));
        tier_info.window_inflow = cur_inflow + amount;
    }
    spec fun validate_and_record_mint {
        modifies global<TierInfo<CoinType>>(dd_addr);
    }

    // If the time window starting at `dealer.window_start` and lasting for
    // `ONE_DAY` has elapsed, resets the window and the inflow and outflow records.
    fun reset_window<CoinType>(tier_info: &mut TierInfo<CoinType>) {
        let current_time = DiemTimestamp::now_microseconds();
        if (current_time > ONE_DAY && current_time - ONE_DAY > tier_info.window_start) {
            tier_info.window_start = current_time;
            tier_info.window_inflow = 0;
        }
    }
    spec fun reset_window {
        pragma opaque;
        include DiemTimestamp::AbortsIfNotOperating;
        let current_time = DiemTimestamp::spec_now_microseconds();
        ensures
            if (current_time > ONE_DAY && current_time - ONE_DAY > old(tier_info).window_start)
                tier_info == update_field(update_field(old(tier_info),
                    window_start, current_time),
                    window_inflow, 0)
            else
                tier_info == old(tier_info);
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    spec module {
        /// resource struct Dealer persists after publication
        invariant update [global] forall addr: address where old(exists<Dealer>(addr)): exists<Dealer>(addr);

        /// TierInfo persists
        invariant update [global] forall addr: address, coin_type: type where old(exists<TierInfo<coin_type>>(addr)):
            exists<TierInfo<coin_type>>(addr);

    }

}
}
