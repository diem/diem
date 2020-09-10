script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Create an account with the DesignatedDealer role at `addr` with authentication key
/// `auth_key_prefix` | `addr` and a 0 balance of type `Currency`. If `add_all_currencies` is true,
/// 0 balances for all available currencies in the system will also be added. This can only be
/// invoked by an account with the TreasuryCompliance role.
fun create_designated_dealer<Currency>(
    tc_account: &signer,
    sliding_nonce: u64,
    addr: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>,
    add_all_currencies: bool,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    LibraAccount::create_designated_dealer<Currency>(
        tc_account,
        addr,
        auth_key_prefix,
        human_name,
        add_all_currencies
    );
}
}
