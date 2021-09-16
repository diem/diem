module DiemFramework::ApprovalPolicy {
    // ApprovalPolicy stores an m-of-n policy. allowed_voters is the list of
    // voters allowed to vote and num_votes_required is the threshold required
    // for approval
    struct ApprovalPolicy has store, drop, copy {
        allowed_voters: vector<vector<u8>>,
        num_votes_required: u64,
    }

    public fun create_approval_policy(
        allowed_voters: vector<vector<u8>>,
        num_votes_required: u64,
    ): ApprovalPolicy {
        ApprovalPolicy {
            allowed_voters,
            num_votes_required,
        }
    }

    public fun get_allowed_voters(policy: &ApprovalPolicy): vector<vector<u8>> {
        *(&policy.allowed_voters)
    }

    public fun get_num_votes_required(policy: &ApprovalPolicy): u64 {
        policy.num_votes_required
    }

}
