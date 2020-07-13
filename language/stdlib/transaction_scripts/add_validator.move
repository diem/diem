script {
    use 0x1::LibraSystem;

    /// Add `new_validator` to the validator set.
    /// Fails if the `new_validator` address is already in the validator set
    /// or does not have a `ValidatorConfig` resource stored at the address.
    /// Emits a NewEpochEvent.
    /// TODO(valerini): rename to add_validator_and_reconfigure?
    fun add_validator(lr_account: &signer, validator_address: address) {
        LibraSystem::add_validator(lr_account, validator_address);
    }
}
