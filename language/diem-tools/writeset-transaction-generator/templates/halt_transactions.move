script {
    use 0x1::DiemTransactionPublishingOption;
    fun main(diem_root: &signer) {
        DiemTransactionPublishingOption::set_open_script(diem_root);
        // Add a null hash to the script allow list to disable execution of all scripts sent to the validator.
        DiemTransactionPublishingOption::add_to_script_allow_list(diem_root, x"0000000000000000000000000000000000000000000000000000000000000000");
    }
}
