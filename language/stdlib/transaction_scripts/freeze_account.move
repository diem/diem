script {
use 0x1::LibraAccount::{Self, AccountFreezing};
use 0x1::SlidingNonce;
use 0x1::Roles;

/// Freeze account `address`. Initiator must be authorized.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun freeze_account(account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    let freezing_capability = Roles::extract_privilege_to_capability<AccountFreezing>(account);
    LibraAccount::freeze_account(account, &freezing_capability, to_freeze_account);
    Roles::restore_capability_to_privilege(account, freezing_capability);
}
}
