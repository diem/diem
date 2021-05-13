script {
    use DiemFramework::DiemTransactionPublishingOption;
    fun main(diem_root: signer) {
        DiemTransactionPublishingOption::halt_all_transactions(&diem_root);
    }
}
