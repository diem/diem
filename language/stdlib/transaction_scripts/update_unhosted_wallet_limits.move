script {
use 0x1::AccountLimits;
use 0x1::SlidingNonce;
use 0x1::Roles::{Self, TreasuryComplianceRole};

/// Optionally update global thresholds of max balance, total flow (inflow + outflow) (microLBR)
/// for `LimitsDefinition` bound accounts.
/// If a new threshold is 0, that particular config does not get updated.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun update_unhosted_wallet_limits<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_max_total_flow: u64,
    new_max_holding_balance: u64,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let cap = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
    AccountLimits::update_limits_definition(&cap, new_max_total_flow, new_max_holding_balance);
    Roles::restore_capability_to_privilege(tc_account, cap);
}
}
