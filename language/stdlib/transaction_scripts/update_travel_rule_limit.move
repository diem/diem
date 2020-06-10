script {
    use 0x1::SlidingNonce;
    use 0x1::DualAttestationLimit;

    fun main(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64) {
        SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
        DualAttestationLimit::set_microlibra_limit(tc_account, new_micro_lbr_limit)
    }
}
