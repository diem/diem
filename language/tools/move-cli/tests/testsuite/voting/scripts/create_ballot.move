script {

use Std::Debug;
use Std::Vector;
use Std::Signer;
use 0x2::Vote;
use 0x2::CreateValidatorAccount;

fun main(account: signer) {
    let address = Signer::address_of(&account);
    let proposal = CreateValidatorAccount::create_validator_account(0);
    let v = Vector::empty();
    Vector::push_back(&mut v, address);
    Vector::push_back(&mut v, @0x4);
    let approval_policy = Vote::create_approval_policy(v, 2);
    let ballot_id = Vote::create_ballot(&account, *(&proposal), b"create_validator_account", *(&approval_policy), 0, 16313756410);
    let success = Vote::vote(
        &account,
        ballot_id,
        proposal,
        b"create_validator_account",
        approval_policy
    );
    Debug::print(&success);
}
}
