script {
use 0x1::LibraAccount::{Self, AccountUnfreezing};
use 0x1::SlidingNonce;
use 0x1::Roles;

/// Script for un-freezing account by authorized initiator
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun unfreeze_account(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    let unfreezing_capability = Roles::extract_privilege_to_capability<AccountUnfreezing>(account);
    LibraAccount::unfreeze_account(account, &unfreezing_capability, to_unfreeze_account);
    Roles::restore_capability_to_privilege(account, unfreezing_capability);
}
}
