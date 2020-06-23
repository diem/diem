script {
use 0x1::TransactionFee;
use 0x1::Roles::{Self, TreasuryComplianceRole};

/// Burn transaction fees that have been collected in the given `currency`
/// and relinquish to the association. The currency must be non-synthetic.
fun burn_txn_fees<CoinType>(tc_account: &signer) {
    let tc_capability =
        Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
    TransactionFee::burn_fees<CoinType>(tc_account, &tc_capability);
    Roles::restore_capability_to_privilege(tc_account, tc_capability)
}
}
