script {
    use 0x1::LibraTransactionPublishingOption;
    fun main(libra_root: &signer) {
        LibraTransactionPublishingOption::set_open_script(libra_root);
        // Add a null hash to the script allow list to disable execution of all scripts sent to the validator.
        LibraTransactionPublishingOption::add_to_script_allow_list(libra_root, x"0000000000000000000000000000000000000000000000000000000000000000");
    }
}
