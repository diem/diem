script {
    use 0x1::LibraSystem;
    use 0x1::SlidingNonce;
    use 0x1::ValidatorConfig;

    /// Add `new_validator` to the validator set.
    /// Fails if the `new_validator` address is already in the validator set
    /// or does not have a `ValidatorConfig` resource stored at the address.
    /// Emits a NewEpochEvent.
    fun add_validator_and_reconfigure(
        lr_account: &signer,
        sliding_nonce: u64,
        validator_name: vector<u8>,
        validator_address: address
    ) {
        SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
        assert(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
        LibraSystem::add_validator(lr_account, validator_address);
    }
}
