address 0x1 {
module DesignatedDealer {
    use 0x1::CoreErrors;
    use 0x1::Libra::{Self, Libra};
    use 0x1::LibraTimestamp;
    use 0x1::Vector;
    use 0x1::Event;
    use 0x1::Roles::{Capability, TreasuryComplianceRole};

    // TODO: make these constants
    fun MODULE_ERROR_BASE(): u64 { 14000 }
    public fun NOT_A_DD(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 1 }
    public fun INVALID_TIER_INDEX(): u64 { MODULE_ERROR_BASE() + 2 }
    public fun INVALID_TIER_START(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 3 }
    public fun INVALID_AMOUNT_FOR_TIER(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 4 }
    public fun INVALID_MINT_AMOUNT(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 5 }
    public fun INVALID_TIER_ADDITION(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 6 }

    resource struct Dealer {
        /// Time window start in microseconds
        window_start: u64,
        /// The minted inflow during this time window
        window_inflow: u64,
        /// 0-indexed array of tier upperbounds
        tiers: vector<u64>,
        /// Handle for mint events
        mint_event_handle: Event::EventHandle<ReceivedMintEvent>,
    }
    // Preburn published at top level in Libra.move

    // Message for mint events
    struct ReceivedMintEvent {
        // The address that receives the mint
        destination_address: address,
        // The amount minted
        amount: u64,
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be designated-dealer called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun publish_designated_dealer_credential(dd: &signer, _: &Capability<TreasuryComplianceRole>) {
        move_to(
            dd,
            Dealer {
                window_start: LibraTimestamp::now_microseconds(),
                window_inflow: 0,
                tiers: Vector::empty(),
                mint_event_handle: Event::new_event_handle<ReceivedMintEvent>(dd),
            }
        )
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs by Treasury Compliance Account
    ///////////////////////////////////////////////////////////////////////////


    fun add_tier_(dealer: &mut Dealer, next_tier_upperbound: u64) {
        let tiers = &mut dealer.tiers;
        let number_of_tiers: u64 = Vector::length(tiers);
        assert(number_of_tiers <= 4, INVALID_TIER_ADDITION());
        if (number_of_tiers > 1) {
            let prev_tier = *Vector::borrow(tiers, number_of_tiers - 1);
            assert(prev_tier < next_tier_upperbound, INVALID_TIER_START());
        };
        Vector::push_back(tiers, next_tier_upperbound);
    }

    public fun add_tier(
        _: &Capability<TreasuryComplianceRole>,
        addr: address,
        tier_upperbound: u64
    ) acquires Dealer {
        let dealer = borrow_global_mut<Dealer>(addr);
        add_tier_(dealer, tier_upperbound)
    }

    fun update_tier_(dealer: &mut Dealer, tier_index: u64, new_upperbound: u64) {
        let tiers = &mut dealer.tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index <= 3, INVALID_TIER_INDEX()); // max 4 tiers allowed
        assert(tier_index < number_of_tiers, INVALID_TIER_INDEX());
        // Make sure that this new start for the tier is consistent
        // with the tier above it.
        let next_tier = tier_index + 1;
        if (next_tier < number_of_tiers) {
            assert(new_upperbound < *Vector::borrow(tiers, next_tier), INVALID_TIER_START());
        };
        let tier_mut = Vector::borrow_mut(tiers, tier_index);
        *tier_mut = new_upperbound;
    }

    public fun update_tier(
        _update_tier_capability: &Capability<TreasuryComplianceRole>,
        addr: address,
        tier_index: u64,
        new_upperbound: u64
    ) acquires Dealer {
        let dealer = borrow_global_mut<Dealer>(addr);
        update_tier_(dealer, tier_index, new_upperbound)
    }

    fun tiered_mint_(dealer: &mut Dealer, amount: u64, tier_index: u64): bool {
        // if tier is 4, can mint unlimited)
        assert(tier_index <= 4, INVALID_TIER_INDEX());
        reset_window(dealer);
        let cur_inflow = *&dealer.window_inflow;
        let tiers = &mut dealer.tiers;
        // If the tier_index is one past the bounded tiers, minting is unbounded
        let number_of_tiers = Vector::length(tiers);
        let tier_check = &mut false;
        if (tier_index == number_of_tiers) {
            *tier_check = true;
        } else {
            let tier_upperbound: u64 = *Vector::borrow(tiers, tier_index);
            *tier_check = (cur_inflow + amount <= tier_upperbound);
        };
        if (*tier_check) {
            dealer.window_inflow = cur_inflow + amount;
        };
        *tier_check
    }

    public fun tiered_mint<CoinType>(
        blessed: &signer,
        _: &Capability<TreasuryComplianceRole>,
        amount: u64,
        dd_addr: address,
        tier_index: u64,
    ): Libra<CoinType> acquires Dealer {

        assert(amount > 0, INVALID_MINT_AMOUNT());
        assert(exists_at(dd_addr), NOT_A_DD());
        let tier_check = tiered_mint_(borrow_global_mut<Dealer>(dd_addr), amount, tier_index);
        assert(tier_check, INVALID_AMOUNT_FOR_TIER());
        // Send ReceivedMintEvent
        Event::emit_event<ReceivedMintEvent>(
            &mut borrow_global_mut<Dealer>(dd_addr).mint_event_handle,
            ReceivedMintEvent {
                destination_address: dd_addr,
                amount: amount,
            },
        );
        Libra::mint<CoinType>(blessed, amount)
    }

    public fun exists_at(addr: address): bool {
        exists<Dealer>(addr)
    }

    // If the time window starting at `dealer.window_start` and lasting for
    // window_length() has elapsed, resets the window and
    // the inflow and outflow records.
    fun reset_window(dealer: &mut Dealer) {
        let current_time = LibraTimestamp::now_microseconds();
        if (current_time > dealer.window_start + window_length()) {
            dealer.window_start = current_time;
            dealer.window_inflow = 0;
        }
    }

    fun window_length(): u64 {
        // number of microseconds in a day
        86400000000
    }

}
}
