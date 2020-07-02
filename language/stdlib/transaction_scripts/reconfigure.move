script {
    use 0x1::LibraSystem;

    /// Update configs of all the validators and emit reconfiguration event.
    fun reconfigure(lr_account: &signer) {
        LibraSystem::update_and_reconfigure(lr_account);
    }
}
