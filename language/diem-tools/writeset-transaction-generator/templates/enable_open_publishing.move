script {
    use DiemFramework::DiemTransactionPublishingOption;
    fun main(diem_root: signer, _unused_diem_root: signer) {
        DiemTransactionPublishingOption::set_open_script(&diem_root);
        DiemTransactionPublishingOption::set_open_module(&diem_root, true);
    }
}