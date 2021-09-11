module 0x1::Vote {
    use Std::Vector;

    const ROOT_ADDR: address = @0x1;

    struct Ballot<Params: store> has store {
        // parameters for the action being voted on. Specific to each ballot.
        params: Params,
        possible_voters: vector<address>,
        votes_received: vector<address>,
        // a globally unique ballot id that is created
        ballot_id: u64,
        //
        expiration_date: u64,
        start_time: u64,
    }

    struct Ballots<Params: store> has key {
        ballots: vector<Ballot<Params>>
    }

    struct BallotCounter {
        counter: u64,
    }

    struct BallotValidation<Params> has key {
        ballot_id: u64
    }

    struct BallotIsValidated {}

    public fun gc<param: store>(anyone: address) {
        ..
    }

    public fun create_ballot<Params: store>(ballot_account: &signer, params: Params, possible_voters: vector<address>): BallotValidation<Params> {
        Event::emit_event(singleton, CreatedBallot {
            ballot_id,

        });
        move_to(ballot_account, Ballot {
            params,
            possible_voters,
            votes_received: Vector::empty(),
            ballot_id: incr_counter(),
        })
    }

    public fun vote<ParamType>(voting_account: &signer, ballot_id: u64, ballot_addr: address, cap: &BallotValidation<Params>) acquires Ballot {
        let ballot = borrow_global_mut<Ballot<ParamType>>(ballot_addr);
        let voting_addr = Signer::address_of(voting_account);
        let (has_elem, index) = Vector::index_of(&ballot.possible_voters, voting_addr);
        assert(has_elem, 0);
        Vector::swap_remove(&mut ballot.possible_voters, index);
        Vector::push_back(&mut ballot.votes_received, voting_addr);
    }

    public fun get_votes<ParamType>(ballot_addr: address): (vector<address>, vector<address>) acquires Ballot {
        let ballot = borrow_global<Ballot<ParamType>>(ballot_addr);
        (*&ballot.possible_voters, *& ballot.votes_received)
    }

    // Restrict that Params must be used in the same transaction by giving back a hot potato `BallotIsValidated`
    public fun validate_and_return_ballot<Params: store>(ballot_address: address, ballot_validation: BallotValidation<Params>): (Params, BallotIsValidated) {
        let Ballot { params, ballot_id, .. } = move_from<Ballot<Params>>(ballot_address);
        assert(ballot_id == ballot_validation.ballot_id, 0);
        (params, BallotIsValidated{})
    }

    fun incr_counter(): u64 acquires BallotCounter {
        let counter = &mut borrow_global_mut<BallotCounter>(@ROOT_ADDR).counter;
        let count = *counter;
        *counter = *counter + 1;
        count
    }
}