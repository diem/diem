script {
use 0x1::LibraVersion;
use 0x1::SlidingNonce;

/// Update Libra version.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun update_libra_version(account: &signer, sliding_nonce: u64, major: u64) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    LibraVersion::set(account, major)
}
}
