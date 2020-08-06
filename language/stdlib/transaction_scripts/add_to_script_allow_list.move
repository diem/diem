script {
use 0x1::LibraTransactionPublishingOption;
use 0x1::SlidingNonce;

/// Append the `hash` to script hashes list allowed to be executed by the network.
fun add_to_script_allow_list(lr_account: &signer, hash: vector<u8>, sliding_nonce: u64,) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    LibraTransactionPublishingOption::add_to_script_allow_list(lr_account, hash)
}
}
