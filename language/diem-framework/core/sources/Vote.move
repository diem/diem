/// The Vote module is used to allow voting on proposals on the chain.
/// It is typically not going to be used directly, but is intended to be
/// used as a library for modules which want to perform operations which
/// require m-of-n approvals from accounts on chain.
/// A typical workflow would look like the following
/// * Module M creates a ballot with a given `Proposal` and an approval policy using `create_ballot`
/// * It receives the BallotID corresponding to the ballot
/// * It submits votes using the `vote` function from the voters
/// * If a vote causes a ballot to be approved, `vote` returns `true` and Module M can proceed with the operation requested by the `Proposal`
module DiemFramework::Vote {

    use Std::BCS;
    use Std::Errors;
    use Std::Event;
    use Std::Signer;
    use Std::Vector;
    use DiemFramework::DiemTimestamp;
    #[test_only]
    friend DiemFramework::VoteTests;

    /// An unique identifier for a ballot. A counter is stored
    /// under each proposers address which is incremented
    /// every time a new ballot is created. The proposers
    /// address is also part of the ballot id.
    struct BallotID has store, copy, drop {
        counter: u64,
        proposer: address,
    }

    /// WeightedVoter represents a voter with a weight
    /// The voter is represented by the bcs serialization of address
    struct WeightedVoter has store, copy, drop {
        weight: u64,
        voter: vector<u8>,
    }

    /// Ballot is a struct which contains a Proposal on which
    /// votes are gathered. A ballot is started by a proposer
    /// and it carries the proposal, the approval policy,
    /// expiration timestamp
    struct Ballot<Proposal: store + drop> has store, copy, drop {
        /// Details for the proposal being voted on.
        proposal: Proposal,
        /// A human readable type for the proposal - ex: "create_validator_owner",
        /// "freeze_account", "create_vasp", etc.
        /// This lives outside the `proposal: Proposal` to make it easy for
        /// indexers to index the ballots which have been proposed and
        /// categorize them into "types"
        proposal_type: vector<u8>,
        /// The num_votes_required for this proposal to be approved
        num_votes_required: u64,
        /// A vector of addresses which are allowed to vote on this ballot.
        allowed_voters: vector<WeightedVoter>,
        /// Votes received so far
        votes_received: vector<WeightedVoter>,
        /// Total number of weighted votes received
        total_weighted_votes_received: u64,
        // A globally unique ballot id that is created for every proposal
        ballot_id: BallotID,
        // Votes rejected after this time
        expiration_timestamp_secs: u64,
    }

    /// Ballots stores a list of ballots under a proposers address.
    /// It is type parametrized by the Proposal. Some of these may
    /// be expired and can be cleaned up using the `gc_ballots` function
    /// It also contains the handles for all the events associated
    /// with the ballots created by a proposer
    struct Ballots<Proposal: store + drop> has key {
        ballots: vector<Ballot<Proposal>>,
        create_ballot_handle: Event::EventHandle<CreateBallotEvent<Proposal>>,
        remove_ballot_handle: Event::EventHandle<RemoveBallotEvent>,
        voted_handle: Event::EventHandle<VotedEvent>,
        ballot_approved_handle: Event::EventHandle<BallotApprovedEvent>,
    }

    /// A counter which is stored under the proposers address and gets incremented
    /// everytime they create a new ballot. This is used for creating unique
    /// global identifiers for Ballots
    struct BallotCounter has key {
        counter: u64,
    }

    /// CreateBallotEvent is emitted when a ballot is
    /// created by a proposer
    struct CreateBallotEvent<Proposal: store + drop> has drop, store {
        ballot_id: BallotID,
        ballot: Ballot<Proposal>,
    }

    /// RemoveBallotEvent is emitted when a ballot has
    /// been removed. This can either happen because:
    /// * ballot was approved
    /// * ballot was manually removed by the proposer
    /// * ballot was expired and garbage collected
    struct RemoveBallotEvent has drop, store {
        ballot_id: BallotID,
    }

    /// VotedEvent is emitted when a valid vote has
    /// been accepted by the ballot
    struct VotedEvent has drop, store {
        ballot_id: BallotID,
        voter: address,
        vote_weight: u64,
    }

    /// BallotApprovedEvent is emitted when a ballot has
    /// been approved by the voters
    struct BallotApprovedEvent has drop, store {
        ballot_id: BallotID,
    }

    /// The maximum number of ballots allowed per proposal type
    /// per address.
    const MAX_BALLOTS_PER_PROPOSAL_TYPE_PER_ADDRESS: u64 = 256;

    /// The provided timestamp(s) were invalid
    const EINVALID_TIMESTAMP: u64 = 1;
    /// The address already contains has the maximum of ballots allowed
    /// MAX_BALLOTS_PER_PROPOSAL_TYPE_PER_ADDRESS
    const ETOO_MANY_BALLOTS: u64 = 2;
    /// Ballot with the provided id was not found
    const EBALLOT_NOT_FOUND: u64 = 3;
    /// Proposal details in the vote do not match the proposal details
    /// in the ballot
    const EBALLOT_PROPOSAL_MISMATCH: u64 = 4;
    /// Voter not allowed to vote in the ballot
    const EINVALID_VOTER: u64 = 5;
    /// current timestamp > ballot expiration time
    const EBALLOT_EXPIRED: u64 = 7;
    /// Voter has already voted in this ballot
    const EALREADY_VOTED: u64 = 8;

    /// A constructor for BallotID
    public fun new_ballot_id(
        counter: u64,
        proposer: address,
    ): BallotID {
        BallotID {
            counter,
            proposer,
        }
    }

    /// A constructor for WeightedVoter
    public fun new_weighted_voter(
        weight: u64,
        voter: vector<u8>,
    ): WeightedVoter {
        WeightedVoter {
            weight,
            voter,
        }
    }

    /// Create a ballot under the signer's address and return the `BallotID`
    public fun create_ballot<Proposal: store + copy + drop>(
        ballot_account: &signer,
        proposal: Proposal,
        proposal_type: vector<u8>,
        num_votes_required: u64,
        allowed_voters: vector<WeightedVoter>,
        expiration_timestamp_secs: u64
    ): BallotID acquires Ballots, BallotCounter {
        let ballot_address = Signer::address_of(ballot_account);

        assert(DiemTimestamp::now_seconds() < expiration_timestamp_secs, Errors::invalid_argument(EINVALID_TIMESTAMP));

        if (!exists<BallotCounter>(ballot_address)) {
            move_to(ballot_account, BallotCounter {
                counter: 0,
            });
        };
        if (!exists<Ballots<Proposal>>(ballot_address)) {
            move_to(ballot_account, Ballots<Proposal> {
                ballots: Vector::empty(),
                create_ballot_handle: Event::new_event_handle<CreateBallotEvent<Proposal>>(ballot_account),
                remove_ballot_handle: Event::new_event_handle<RemoveBallotEvent>(ballot_account),
                voted_handle: Event::new_event_handle<VotedEvent>(ballot_account),
                ballot_approved_handle: Event::new_event_handle<BallotApprovedEvent>(ballot_account),
            });
        };

        let ballot_data = borrow_global_mut<Ballots<Proposal>>(ballot_address);

        // Remove any expired ballots
        gc_internal<Proposal>(ballot_data);
        let ballots = &mut ballot_data.ballots;

        assert(Vector::length(ballots) < MAX_BALLOTS_PER_PROPOSAL_TYPE_PER_ADDRESS, Errors::limit_exceeded(ETOO_MANY_BALLOTS));
        let ballot_id = new_ballot_id(incr_counter(ballot_account), ballot_address);
        let ballot = Ballot<Proposal> {
            proposal,
            proposal_type,
            num_votes_required,
            allowed_voters,
            votes_received: Vector::empty(),
            total_weighted_votes_received: 0,
            ballot_id: *&ballot_id,
            expiration_timestamp_secs,
        };
        Vector::push_back(ballots, *&ballot);
        Event::emit_event<CreateBallotEvent<Proposal>>(
            &mut ballot_data.create_ballot_handle,
            CreateBallotEvent {
                ballot_id: *&ballot_id,
                ballot,
            },
        );
        ballot_id
    }

    // Checks if a voter is present in the vector<WeightedVoter>
    fun check_voter_present(
        weighted_voters: &vector<WeightedVoter>,
        voter: &vector<u8>,
    ): bool {
        let i = 0;
        let len = Vector::length(weighted_voters);
        while (i < len) {
            if (&Vector::borrow(weighted_voters, i).voter == voter) return true;
            i = i + 1;
        };
        false
    }

    /// Submit a vote from the `voter_account` to `ballot_id`
    /// This also contains the `proposal_type` and `proposal` so that
    /// the voter signs over these when sending their vote.
    /// If this vote causes the ballot to be approved, then the ballot
    /// is removed from the proposers address.
    /// This returns a bool indicating whether this vote moved the ballot
    /// to an approved status. true represents that the ballot got approved
    /// after this vote and false represents that the ballot has not been
    /// approved after this vote
    public fun vote<Proposal: store + drop>(
        voter_account: &signer,
        ballot_id: BallotID,
        proposal_type: vector<u8>,
        proposal: Proposal,
    ): bool acquires Ballots {
        let ballot_data = borrow_global_mut<Ballots<Proposal>>(ballot_id.proposer);

        // Remove any expired ballots
        gc_internal<Proposal>(ballot_data);

        let ballots = &mut ballot_data.ballots;
        let i = 0;
        let len = Vector::length(ballots);
        while (i < len) {
            if (&Vector::borrow(ballots, i).ballot_id == &ballot_id) break;
            i = i + 1;
        };
        assert(i < len, Errors::invalid_state(EBALLOT_NOT_FOUND));
        let ballot_index = i;
        let ballot = Vector::borrow_mut(ballots, ballot_index);

        assert(&ballot.proposal == &proposal, Errors::invalid_argument(EBALLOT_PROPOSAL_MISMATCH));
        assert(&ballot.proposal_type == &proposal_type, Errors::invalid_argument(EBALLOT_PROPOSAL_MISMATCH));

        let voter_address = Signer::address_of(voter_account);
        let voter_address_bcs = BCS::to_bytes(&voter_address);
        let allowed_voters = &ballot.allowed_voters;

        assert(check_voter_present(allowed_voters, &voter_address_bcs), Errors::invalid_state(EINVALID_VOTER));
        assert(DiemTimestamp::now_seconds() <= ballot.expiration_timestamp_secs, Errors::invalid_state(EBALLOT_EXPIRED));

        assert(!check_voter_present(&ballot.votes_received, &voter_address_bcs), Errors::invalid_state(EALREADY_VOTED));

        let i = 0;
        let len = Vector::length(allowed_voters);
        while (i < len) {
            let weighted_voter = Vector::borrow(allowed_voters, i);
            if (&weighted_voter.voter == &voter_address_bcs) {
                Vector::push_back(&mut ballot.votes_received, *weighted_voter);
                ballot.total_weighted_votes_received = ballot.total_weighted_votes_received + weighted_voter.weight;
                Event::emit_event<VotedEvent>(
                    &mut ballot_data.voted_handle,
                    VotedEvent {
                        ballot_id: *&ballot_id,
                        voter: voter_address,
                        vote_weight: weighted_voter.weight,
                    },
                );
                break
            };
            i = i + 1;
        };
        let ballot_approved = ballot.total_weighted_votes_received >= ballot.num_votes_required;
        // If the ballot gets approved, remove the ballot immediately
        if (ballot_approved) {
            Vector::swap_remove(ballots, ballot_index);
            Event::emit_event<BallotApprovedEvent>(
                &mut ballot_data.ballot_approved_handle,
                BallotApprovedEvent {
                    ballot_id,
                },
            );
        };
        ballot_approved
    }

    /// gc_ballots deletes all the expired ballots of the type `Proposal`
    /// under the provided address `addr`. The signer can be anybody
    /// and does not need to have the same address as `addr`
    public(script) fun gc_ballots<Proposal: store + drop>(
        _signer: signer,
        addr: address,
    ) acquires Ballots {
        gc_internal<Proposal>(borrow_global_mut<Ballots<Proposal>>(addr));
    }

    public(friend) fun gc_test_helper<Proposal: store + drop>(
        addr: address,
    ): vector<BallotID>  acquires Ballots {
        gc_internal<Proposal>(borrow_global_mut<Ballots<Proposal>>(addr))
    }

    fun gc_internal<Proposal: store + drop>(
        ballot_data: &mut Ballots<Proposal>,
    ): vector<BallotID> {
        let ballots = &mut ballot_data.ballots;
        let remove_handle = &mut ballot_data.remove_ballot_handle;
        let i = 0;
        let removed_ballots = Vector::empty();
        while (i < Vector::length(ballots)) {
            let ballot = Vector::borrow(ballots, i);
            if (ballot.expiration_timestamp_secs < DiemTimestamp::now_seconds()) {
                let ballot_id = *(&ballot.ballot_id);
                Vector::swap_remove(ballots, i);
                Vector::push_back(&mut removed_ballots, *&ballot_id);
                Event::emit_event<RemoveBallotEvent>(
                    remove_handle,
                    RemoveBallotEvent {
                        ballot_id
                    },
                );
            } else {
                i = i + 1;
            };
        };
        removed_ballots
    }

    public(friend) fun remove_ballot_internal<Proposal: store + drop>(
        account: signer,
        ballot_id: BallotID,
    ) acquires Ballots {
        let addr = Signer::address_of(&account);
        let ballot_data = borrow_global_mut<Ballots<Proposal>>(addr);
        let ballots = &mut ballot_data.ballots;
        let remove_handle = &mut ballot_data.remove_ballot_handle;
        let i = 0;
        let len = Vector::length(ballots);
        while (i < len) {
            if (&Vector::borrow(ballots, i).ballot_id == &ballot_id) {
                Vector::swap_remove(ballots, i);
                Event::emit_event<RemoveBallotEvent>(
                    remove_handle,
                    RemoveBallotEvent {
                        ballot_id
                    },
                );
                return ()
            };
            i = i + 1;
        };
    }

    /// remove_ballot removes the ballot with the provided
    /// `ballot_id` from the address of the provided `account`
    /// If a ballot with `ballot_id` is not found, then it
    /// does nothing
    public(script) fun remove_ballot<Proposal: store + drop>(
        account: signer,
        ballot_id: BallotID,
    ) acquires Ballots {
        remove_ballot_internal<Proposal>(account, ballot_id)
    }

    /// incr_counter increments the counter stored under the signer's
    /// account
    fun incr_counter(account: &signer): u64 acquires BallotCounter {
        let addr = Signer::address_of(account);
        let counter = &mut borrow_global_mut<BallotCounter>(addr).counter;
        let count = *counter;
        *counter = *counter + 1;
        count
    }

///****************************************************************
/// Specs
///****************************************************************

// The following formalizes a symbolic state machine describing the voting protocol.
// There is a state machine for each Proposal type and address, and another state
// machine for each BallotId
// The relevant state for a given ballot_address this module
// consists of structs Ballot<Proposal> and BallotCounter.

spec module {
    invariant<Proposal> forall ballot_address: address
        where exists<Ballot<Proposal>>(ballot_address):
            exists<BallotCounter>(ballot_address);
}

spec fun is_initialized<Proposal>(ballot_address: address): bool {
    exists<Ballots<Proposal>>(ballot_address)
}

spec create_ballot {
    ensures is_initialized<Proposal>(Signer::address_of(ballot_account));
}

}
