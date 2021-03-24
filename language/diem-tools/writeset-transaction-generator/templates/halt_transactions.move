script {
    use 0x1::DiemTransactionPublishingOption;
    fun main(diem_root: signer) {
        DiemTransactionPublishingOption::halt_all_transactions(&diem_root);
    }
}
