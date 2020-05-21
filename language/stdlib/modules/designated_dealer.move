address 0x0 {
module DesignatedDealer {
    use 0x0::LibraTimestamp;
    use 0x0::Vector;
    use 0x0::Transaction as Txn;
    use 0x0::Signer;

    struct Dealer {
        /// Time window start in microseconds
        window_start: u64,
        /// The minted inflow during this time window
        window_inflow: u64,
        /// Association grants
        is_certified: bool,
        /// 0-indexed array of tier upperbounds
        tiers: vector<u64>
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be designated-dealer called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun create_designated_dealer(
    ): Dealer {
        Dealer {
            window_start: LibraTimestamp::now_microseconds(),
            window_inflow: 0,
            is_certified: true,
            tiers: Vector::empty(),
        }
    }


    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs by Treasury Compliance Account
    ///////////////////////////////////////////////////////////////////////////

    public fun add_tier(dealer: &mut Dealer, next_tier_upperbound: u64)
    {
        let tiers = &mut dealer.tiers;
        let number_of_tiers: u64 = Vector::length(tiers);
        // INVALID_TIER_ADDITION
        Txn::assert(number_of_tiers <= 4, 3);
        if (number_of_tiers > 1) {
            let prev_tier = *Vector::borrow(tiers, number_of_tiers - 1);
            // INVALID_TIER_START
            Txn::assert(prev_tier < next_tier_upperbound, 4);
        };
        Vector::push_back(tiers, next_tier_upperbound);
    }

    public fun update_tier(dealer: &mut Dealer, tier_index: u64, new_upperbound: u64)
    {
        let tiers = &mut dealer.tiers;
        let number_of_tiers = Vector::length(tiers);
        // INVALID_TIER_INDEX
        Txn::assert(tier_index <= 4, 3);
        Txn::assert(tier_index < number_of_tiers, 3);
        // Make sure that this new start for the tier is consistent
        // with the tier above it.
        let next_tier = tier_index + 1;
        if (next_tier < number_of_tiers) {
            // INVALID_TIER_START
            Txn::assert(new_upperbound < *Vector::borrow(tiers, next_tier), 4);
        };
        let tier_mut = Vector::borrow_mut(tiers, tier_index);
        *tier_mut = new_upperbound;
    }

    public fun tiered_mint(dealer: &mut Dealer, amount: u64, tier_index: u64): bool {
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

    public fun is_designated_dealer(dealer: &Dealer): bool
    {
        *&dealer.is_certified
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

    public fun assert_account_is_blessed(sender_account: &signer) {
        // Verify that the sender is treasury compliant account
        Txn::assert(Signer::address_of(sender_account) == treasury_compliance_account(), 0)
    }

    fun window_length(): u64 {
        // number of microseconds in a day
        86400000000
    }

    fun treasury_compliance_account(): address {
        0xB1E55ED
    }
}
}
