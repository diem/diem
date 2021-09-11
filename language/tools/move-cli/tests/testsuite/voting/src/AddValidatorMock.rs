module 0x1::AddValidator {
    use 0x1::Vote;

    struct AddValidatorParams has store {
        // params in here
    }

    struct CurrentValidatorVote has key {
        validation: Vote::BallotValidation<AddValidatorParams>
    }

    public(script) fun propose(account: &signer, params: AddValidatorParams, voting_addrs: vector<address>) {
        let ballot_validation = Vote::create_ballot(account, params, voting_addrs);
        move_to(account, CurrentValidatorVote {
            validation: ballot_validation,
        }
    }

    public fun try_get_params(ballot_address: address): (AddValidatorParams, Vote::BallotIsValidated) {
        let (left, voted) = Vote::get_votes(ballot_address);
        assert(Vector::length(&voted) > Vector::length(&left), 0); // > 50%
        let CurrentValidatorVote { validation } = move_from<CurrentValidatorVote>(ballot_address);
        Vote::validate_and_return_ballot(ballot_address, validation)
    }


    public fun add_validator(params: AddValidatorParams, validation_proof: &Vote::BallotIsValidated) {
        ....
    }

    public(script) fun add_validator_script(account: &signer) {
        let (left, voted) = Vote::get_votes(ballot_address);
        assert(Vector::length(&voted) > Vector::length(&left) || is_diem_root(account), 0); // > 50%
        let CurrentValidatorVote { validation } = move_from<CurrentValidatorVote>(ballot_address);
        let (params, _) = Vote::validate_and_return_ballot(ballot_address, validation);
    }
    public(script) fun add_validator_vote() {

    }

}
