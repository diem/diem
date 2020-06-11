script {
    use 0x1::LibraSystem;

    // Callable by Validator's operator
    fun main(account: &signer, validator_address: address) {
        LibraSystem::remove_validator(account, validator_address);
    }
}
