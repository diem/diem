script {
    use 0x1::LibraSystem;

    /// Adding `to_remove` to the set of pending validator removals. Fails if
    /// the `to_remove` address is already in the validator set or already in the pending removals.
    /// Callable by Validator's operator.
    fun remove_validator(lr_account: &signer, validator_address: address) {
        LibraSystem::remove_validator(lr_account, validator_address);
    }
}
