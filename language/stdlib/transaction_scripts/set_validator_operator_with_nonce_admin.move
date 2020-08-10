script {
use 0x1::SlidingNonce;
use 0x1::ValidatorConfig;
use 0x1::ValidatorOperatorConfig;
/// Set validator operator as 'operator_account' of validator owner 'account' (via Admin Script).
/// `operator_name` should match expected from operator account. This script also
/// takes `sliding_nonce`, as a unique nonce for this operation. See `Sliding_nonce.move` for details.
fun set_validator_operator_with_nonce_admin(
    lr_account: &signer, account: &signer, sliding_nonce: u64, operator_name: vector<u8>, operator_account: address
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    assert(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
    ValidatorConfig::set_operator(account, operator_account);
}
}
