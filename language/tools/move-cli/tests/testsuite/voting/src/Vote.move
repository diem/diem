module 0x2::Vote {

    use Std::Signer;
    use Std::Vector;
    // use DiemFramework::DiemTimestamp;

    const MAX_BALLOTS_PER_PROPOSAL_PER_ADDRESS: u64 = 256;

    const ETOO_MANY_BALLOTS: u64 = 1;
    const EBALLOT_NOT_FOUND: u64 = 2;
    const EPROPOSAL_MISMATCH: u64 = 3;
    const EPROPOSAL_TYPE_MISMATCH: u64 = 4;
    const EAPPROVAL_POLICY_MISMATCH: u64 = 5;
    const EVOTER_NOT_ALLOWED: u64 = 6;
    const EVOTING_NOT_STARTED: u64 = 7;
    const EVOTING_EXPIRED: u64 = 8;
    const EALREADY_VOTED: u64 = 9;
    const EBALLOT_ALREADY_COMPLETE: u64 = 10;
    const EINVALID_APPROVAL_POLICY: u64 = 11;
    const EINVALID_BALLOT_TIMESTAMPS: u64 = 12;

    struct ApprovalPolicy has store, drop, copy {
        allowed_voters: vector<address>,
        num_votes_required: u64,
    }

    struct BallotID has store, copy, drop {
        counter: u64,
        proposer: address,
    }

    /// A ballot is a struct parametrized on a Proposal.
    struct Ballot<Proposal: store> has store {
        // details for the proposal being voted on.
        proposal: Proposal,
        // A string type for the proposal - ex: "create_validator_owner",
        // "freeze_account", "create_vasp", etc.
        // This lives outside the proposal: Proposal to make it easy for 
        // indexers to index the ballots which have been proposed and
        // categorize them into "types"
        proposal_type: vector<u8>,
        approval_policy: ApprovalPolicy,
        votes_received: vector<address>,
        // a globally unique ballot id that is created for every proposal
        ballot_id: BallotID,
        // Votes rejected before this time
        start_timestamp_secs: u64,
        // Votes rejected after this time
        expiration_timestamp_secs: u64,
    }

    struct Ballots<Proposal: store> has key {
        ballots: vector<Ballot<Proposal>>
    }

    struct BallotCounter has key {
        counter: u64,
    }

    public fun create_approval_policy(allowed_voters: vector<address>, num_votes_required: u64): ApprovalPolicy {
        assert(num_votes_required <= Vector::length(&allowed_voters), EINVALID_APPROVAL_POLICY);
        assert(num_votes_required > 0, EINVALID_APPROVAL_POLICY);

        // Using a nested loop verify that allowed_voters does
        // not contain duplicates
        let i = 0;
        let len = Vector::length(&allowed_voters);
        while (i < len) {
            let e1 = Vector::borrow(&allowed_voters, i);
            let j = i + 1;
            while (j < len) {
                let e2 = Vector::borrow(&allowed_voters, j);
                assert(e1 != e2, EINVALID_APPROVAL_POLICY);
                j = j + 1;
            };
            i = i + 1;
        };

        ApprovalPolicy {
            allowed_voters,
            num_votes_required,
        }
    }

    fun create_ballot_id(proposer: address): BallotID acquires BallotCounter {
        BallotID {
            counter: incr_counter(proposer),
            proposer,
        }
    }
    
    public fun create_ballot<Proposal: store>(
        ballot_account: &signer,
        proposal: Proposal,
        proposal_type: vector<u8>,
        approval_policy: ApprovalPolicy,
        start_timestamp_secs: u64,
        expiration_timestamp_secs: u64
    ): BallotID acquires Ballots, BallotCounter {
        assert(start_timestamp_secs <= expiration_timestamp_secs, EINVALID_BALLOT_TIMESTAMPS);
        let ballot_address = Signer::address_of(ballot_account);
        if (!exists<BallotCounter>(ballot_address)) {
            move_to(ballot_account, BallotCounter {
                counter: 0,
            });
        };
        if (!exists<Ballots<Proposal>>(ballot_address)) {
            move_to(ballot_account, Ballots<Proposal> {
                ballots: Vector::empty(),
            });
        };
        let ballots = &mut borrow_global_mut<Ballots<Proposal>>(ballot_address).ballots;
        assert(Vector::length(ballots) < MAX_BALLOTS_PER_PROPOSAL_PER_ADDRESS, ETOO_MANY_BALLOTS);
        let ballot_id = create_ballot_id(ballot_address);
        Vector::push_back(ballots, Ballot<Proposal> {
            proposal,
            proposal_type,
            approval_policy,
            votes_received: Vector::empty(),
            ballot_id: *(&ballot_id),
            start_timestamp_secs,
            expiration_timestamp_secs,
        });
        ballot_id
    }

    public fun vote<Proposal: store + drop>(
        voter_account: &signer,
        ballot_id: BallotID,
        proposal: Proposal,
        proposal_type: vector<u8>,
        approval_policy: ApprovalPolicy,
    ): bool acquires Ballots {
        let ballots = &mut borrow_global_mut<Ballots<Proposal>>(ballot_id.proposer).ballots;
        let i = 0;
        let len = Vector::length(ballots);
        while (i < len) {
            if (&Vector::borrow(ballots, i).ballot_id == &ballot_id) break;
            i = i + 1;
        };

        assert(i < len, EBALLOT_NOT_FOUND);
        let ballot = Vector::borrow_mut(ballots, i);
        
        assert(&ballot.proposal == &proposal, EPROPOSAL_MISMATCH);
        assert(&ballot.proposal_type == &proposal_type, EPROPOSAL_TYPE_MISMATCH);
        assert(&ballot.approval_policy == &approval_policy, EAPPROVAL_POLICY_MISMATCH);
        let voter_address = Signer::address_of(voter_account);
        assert(Vector::contains(&ballot.approval_policy.allowed_voters, &voter_address), EVOTER_NOT_ALLOWED);
        // assert(DiemTimestamp::now_seconds() >= ballot.start_timestamp_secs, EVOTING_NOT_STARTED);
        // assert(DiemTimestamp::now_seconds() <= ballot.expiration_timestamp_secs, EVOTING_EXPIRED);
        assert(!Vector::contains(&ballot.votes_received, &voter_address), EALREADY_VOTED);
        assert(Vector::length(&ballot.votes_received) < ballot.approval_policy.num_votes_required, EBALLOT_ALREADY_COMPLETE);
        Vector::push_back(&mut ballot.votes_received, voter_address);
        Vector::length(&ballot.votes_received) >= ballot.approval_policy.num_votes_required
    }


    fun incr_counter(addr: address): u64 acquires BallotCounter {
        let counter = &mut borrow_global_mut<BallotCounter>(addr).counter;
        let count = *counter;
        *counter = *counter + 1;
        count
    }


}
