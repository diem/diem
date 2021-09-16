module DiemFramework::CreateValidatorAccount {

    use Std::Vector;
    use Std::Signer;
    use Std::BCS;
    use DiemFramework::ApprovalPolicy;
    use DiemFramework::Vote;

    const ROOT_ADDR: address = @0x1;

    struct CreateValidatorAccount has store, drop, copy {
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    }

    struct ApprovalPolicy has key {
        approval_policy: ApprovalPolicy::ApprovalPolicy,
    }

    // Stores the initial approval config for CreateValidatorAccount
    public(script) fun initialize_approval_policy(
        account: signer,
    ) {
        // Roles::assert_diem_root(_account);
        let addr = Signer::address_of(&account);
        let approver_address = @0x2;
        let approval_policy = ApprovalPolicy::create_approval_policy(Vector::singleton(BCS::to_bytes(&approver_address)), 1);
        if (!exists<ApprovalPolicy>(addr)) {
            move_to(&account, ApprovalPolicy {
                approval_policy,
            });
        };
    }

    public(script) fun propose_update_approval_policy() {

    }

    public(script) fun propose_create_validator_account(
        proposer: signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) acquires ApprovalPolicy {
        let approval_policy = *(&borrow_global<ApprovalPolicy>(ROOT_ADDR).approval_policy);
        let create_validator_account = CreateValidatorAccount {
            new_account_address,
            auth_key_prefix,
            human_name,
        };
        Vote::create_ballot(
            &proposer,
            create_validator_account,
            b"create_validator_account",
            approval_policy,
            0,
            16313756410,
        );
    }

    public(script) fun vote_on_create_validator_account(
        voter: signer,
        // ballot identifier
        ballot_counter: u64,
        ballot_proposer: address,
        // proposal details
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) {
        let ballot_id = Vote::construct_ballot_id(ballot_counter, ballot_proposer);
        let create_validator_account = CreateValidatorAccount {
            new_account_address,
            auth_key_prefix,
            human_name,
        };
        let voting_success = Vote::vote(
            &voter,
            ballot_id,
            b"create_validator_account",
            create_validator_account,
        );
        //
        if (voting_success) {
            // actually create validator account here
        }

    }

}
