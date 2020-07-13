script {
    use 0x1::LibraSystem;

    /// Removes a validator from the validator set.
    /// Fails if the validator_address is not in the validator set.
    /// Emits a NewEpochEvent.
    /// TODO(valerini): rename to remove_validator_and_reconfigure?
    fun remove_validator(lr_account: &signer, validator_address: address) {
        LibraSystem::remove_validator(lr_account, validator_address);
    }
}
