script {
use 0x1::TransactionFee;

/// Burn transaction fees that have been collected in the given `currency`
/// and relinquish to the association. The currency must be non-synthetic.
fun burn_txn_fees<CoinType>(tc_account: &signer) {
    TransactionFee::burn_fees<CoinType>(tc_account);
}
}
