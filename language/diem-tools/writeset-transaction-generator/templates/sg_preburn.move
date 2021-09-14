script {
    use 0x1::TreasuryComplianceScripts;
    use 0x1::XUS::XUS;

    fun main(_diem_root: signer, acct: signer) {
        TreasuryComplianceScripts::preburn<XUS>(acct, 10000000);
    }
}
