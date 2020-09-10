script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Create an account with the ParentVASP role at `address` with authentication key
/// `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If
/// `add_all_currencies` is true, 0 balances for all available currencies in the system will
/// also be added. This can only be invoked by an TreasuryCompliance account.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun create_parent_vasp_account<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>,
    add_all_currencies: bool
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    LibraAccount::create_parent_vasp_account<CoinType>(
        tc_account,
        new_account_address,
        auth_key_prefix,
        human_name,
        add_all_currencies
    );
}
}
