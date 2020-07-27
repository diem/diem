script {
    use 0x1::LibraSystem;
    use 0x1::SlidingNonce;
    use 0x1::ValidatorConfig;

    /// Removes a validator from the validator set.
    /// Fails if the validator_address is not in the validator set.
    /// Emits a NewEpochEvent.
    fun remove_validator_and_reconfigure(
        lr_account: &signer,
        sliding_nonce: u64,
        validator_name: vector<u8>,
        validator_address: address
    ) {
        SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
        assert(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
        LibraSystem::remove_validator(lr_account, validator_address);
    }
}
