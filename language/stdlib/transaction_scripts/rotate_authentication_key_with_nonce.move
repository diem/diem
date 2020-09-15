script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Rotate `account`'s authentication key to `new_key`.
/// `new_key` should be a 256 bit sha3 hash of an ed25519 public key. This script also takes
/// `sliding_nonce`, as a unique nonce for this operation. See sliding_nonce.move for details.
fun rotate_authentication_key_with_nonce(account: &signer, sliding_nonce: u64, new_key: vector<u8>) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
    LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
    LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
spec fun rotate_authentication_key_with_nonce {
    use 0x1::Signer;
    let account_addr = Signer::spec_address_of(account);
    include SlidingNonce::RecordNonceAbortsIf{ seq_nonce: sliding_nonce };
    include LibraAccount::ExtractKeyRotationCapabilityAbortsIf;
    let key_rotation_capability = LibraAccount::spec_get_key_rotation_cap(account_addr);
    include LibraAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

    /// This rotates the authentication key of `account` to `new_key`
    include LibraAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};
}
}
