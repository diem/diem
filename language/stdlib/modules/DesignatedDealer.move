address 0x1 {
module DesignatedDealer {
    use 0x1::Libra::{Self, Libra};
    use 0x1::LibraTimestamp;
    use 0x1::Vector;
    use 0x1::Event;
    use 0x1::Roles::{Capability, TreasuryComplianceRole};

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
        // INVALID_TIER_ADDITION
        assert(number_of_tiers <= 4, 31);
        if (number_of_tiers > 1) {
            let prev_tier = *Vector::borrow(tiers, number_of_tiers - 1);
            // INVALID_TIER_START
            assert(prev_tier < next_tier_upperbound, 4);
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
        // INVALID_TIER_INDEX
        assert(tier_index <= 3, 3); // max 4 tiers allowed
        assert(tier_index < number_of_tiers, 3);
        // Make sure that this new start for the tier is consistent
        // with the tier above it.
        let next_tier = tier_index + 1;
        if (next_tier < number_of_tiers) {
            // INVALID_TIER_START
            assert(new_upperbound < *Vector::borrow(tiers, next_tier), 4);
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
        // INVALID TIER_INDEX (if tier is 4, can mint unlimited)
        assert(tier_index <= 4, 66);
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

        // INVALID_MINT_AMOUNT
        assert(amount > 0, 6);

        // NOT_A_DD
        assert(exists_at(dd_addr), 1);
        let tier_check = tiered_mint_(borrow_global_mut<Dealer>(dd_addr), amount, tier_index);
        // INVALID_AMOUNT_FOR_TIER
        assert(tier_check, 5);
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
