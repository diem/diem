address 0x1 {
module DesignatedDealer {
    use 0x1::Libra::{Self, Libra};
    use 0x1::LibraTimestamp;
    use 0x1::Vector;
    use 0x1::Event;
    use 0x1::Roles;

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

    const MAX_NUM_TIERS: u64 = 4;

    const EACCOUNT_NOT_TREASURY_COMPLIANCE: u64 = 0;
    const EINVALID_TIER_ADDITION: u64 = 1;
    const EINVALID_TIER_START: u64 = 2;
    const EINVALID_TIER_INDEX: u64 = 3;
    const EINVALID_MINT_AMOUNT: u64 = 4;
    const ENOT_A_DD: u64 = 5;
    const EINVALID_AMOUNT_FOR_TIER: u64 = 6;

    ///////////////////////////////////////////////////////////////////////////
    // To-be designated-dealer called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun publish_designated_dealer_credential(
        dd: &signer,
        tc_account: &signer,
    ) {
        assert(Roles::has_treasury_compliance_role(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
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
        assert(number_of_tiers + 1 <= MAX_NUM_TIERS, EINVALID_TIER_ADDITION);
        if (number_of_tiers > 0) {
            let last_tier = *Vector::borrow(tiers, number_of_tiers - 1);
            assert(last_tier < next_tier_upperbound, EINVALID_TIER_START);
        };
        Vector::push_back(tiers, next_tier_upperbound);
    }

    public fun add_tier(
        tc_account: &signer,
        dd_addr: address,
        tier_upperbound: u64
    ) acquires Dealer {
        assert(Roles::has_treasury_compliance_role(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
        let dealer = borrow_global_mut<Dealer>(dd_addr);
        add_tier_(dealer, tier_upperbound)
    }

    spec fun add_tier {
        // modifies global<Dealer>@dd_addr.tiers;
        ensures len(global<Dealer>(dd_addr).tiers) == len(old(global<Dealer>(dd_addr)).tiers) + 1;
        ensures global<Dealer>(dd_addr).tiers[len(global<Dealer>(dd_addr).tiers) - 1] == tier_upperbound;
    }

    fun update_tier_(dealer: &mut Dealer, tier_index: u64, new_upperbound: u64) {
        let tiers = &mut dealer.tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index < number_of_tiers, EINVALID_TIER_INDEX);
        // Make sure that this new start for the tier is consistent
        // with the tier above and below it.
        let tier = Vector::borrow(tiers, tier_index);
        if (*tier == new_upperbound) return;
        if (*tier < new_upperbound) {
            let next_tier_index = tier_index + 1;
            if (next_tier_index < number_of_tiers) {
                assert(new_upperbound < *Vector::borrow(tiers, next_tier_index), EINVALID_TIER_START);
            };
        };
        if (*tier > new_upperbound && tier_index > 0) {
            let prev_tier_index = tier_index - 1;
            assert(new_upperbound > *Vector::borrow(tiers, prev_tier_index), EINVALID_TIER_START);
        };
        *Vector::borrow_mut(tiers, tier_index) = new_upperbound;
    }

    public fun update_tier(
        tc_account: &signer,
        dd_addr: address,
        tier_index: u64,
        new_upperbound: u64
    ) acquires Dealer {
        assert(Roles::has_treasury_compliance_role(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
        let dealer = borrow_global_mut<Dealer>(dd_addr);
        update_tier_(dealer, tier_index, new_upperbound)
    }

    spec fun update_tier {
        // modifies global<Dealer>@dd_addr.tiers;
        ensures len(global<Dealer>(dd_addr).tiers) == len(old(global<Dealer>(dd_addr)).tiers);
        ensures global<Dealer>(dd_addr).tiers[tier_index] == new_upperbound;
    }

    fun tiered_mint_(dealer: &mut Dealer, amount: u64, tier_index: u64) {
        reset_window(dealer);
        let cur_inflow = dealer.window_inflow;
        let new_inflow = cur_inflow + amount;
        let tiers = &mut dealer.tiers;
        let number_of_tiers = Vector::length(tiers);
        assert(tier_index < number_of_tiers, EINVALID_TIER_INDEX);
        let tier_upperbound: u64 = *Vector::borrow(tiers, tier_index);
        assert(new_inflow <= tier_upperbound, EINVALID_AMOUNT_FOR_TIER);
        dealer.window_inflow = new_inflow;
    }

    public fun tiered_mint<CoinType>(
        tc_account: &signer,
        amount: u64,
        dd_addr: address,
        tier_index: u64,
    ): Libra<CoinType> acquires Dealer {
        assert(Roles::has_treasury_compliance_role(tc_account), EACCOUNT_NOT_TREASURY_COMPLIANCE);
        assert(amount > 0, EINVALID_MINT_AMOUNT);
        assert(exists_at(dd_addr), ENOT_A_DD);

        tiered_mint_(borrow_global_mut<Dealer>(dd_addr), amount, tier_index);

        // Send ReceivedMintEvent
        Event::emit_event<ReceivedMintEvent>(
            &mut borrow_global_mut<Dealer>(dd_addr).mint_event_handle,
            ReceivedMintEvent {
                destination_address: dd_addr,
                amount: amount,
            },
        );
        Libra::mint<CoinType>(tc_account, amount)
    }

    spec fun tiered_mint {
        // modifies global<Dealer>@dd_addr.{window_start, window_inflow, mint_event_handle}
        ensures {let dealer = global<Dealer>(dd_addr); old(dealer.window_start) <= dealer.window_start};
        ensures {let dealer = global<Dealer>(dd_addr);
                {let current_time = LibraTimestamp::spec_now_microseconds();
                    (dealer.window_start == current_time && dealer.window_inflow == amount) ||
                    (old(dealer.window_start) == dealer.window_start && dealer.window_inflow == old(dealer.window_inflow) + amount)
                }};
        ensures tier_index < len(old(global<Dealer>(dd_addr)).tiers);
        ensures global<Dealer>(dd_addr).window_inflow <= old(global<Dealer>(dd_addr)).tiers[tier_index];
    }

    public fun exists_at(dd_addr: address): bool {
        exists<Dealer>(dd_addr)
    }

    // If the time window starting at `dealer.window_start` and lasting for window_length()
    // has elapsed, resets the window and the inflow and outflow records.
    fun reset_window(dealer: &mut Dealer) {
        let current_time = LibraTimestamp::now_microseconds();
        if (current_time >= dealer.window_start + window_length()) {
            dealer.window_start = current_time;
            dealer.window_inflow = 0;
        }
    }

    fun window_length(): u64 {
        // number of microseconds in a day
        86400000000
    }

    spec schema SpecSchema {
        invariant module forall x: address where exists<Dealer>(x): len(global<Dealer>(x).tiers) <= SPEC_MAX_NUM_TIERS();
        invariant module forall x: address where exists<Dealer>(x):
                         forall i: u64, j: u64 where 0 <= i && i < j && j < len(global<Dealer>(x).tiers):
                            global<Dealer>(x).tiers[i] < global<Dealer>(x).tiers[j];
    }

    spec module {
        pragma verify = false;
        define SPEC_MAX_NUM_TIERS(): u64 { 4 }
        define spec_window_length(): u64 { 86400000000 }
        apply SpecSchema to *, *<CoinType>;
    }
}
}
