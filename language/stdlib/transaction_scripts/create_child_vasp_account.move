script {
use 0x0::LibraAccount;

/// Create a `ChildVASP` account for sender `parent_vasp` at `child_address` with a balance of
/// `child_initial_balance` in `CoinType` and an initial authentication_key
/// `auth_key_prefix | child_address`.
/// If `add_all_currencies` is true, the child address will have a zero balance in all available
/// currencies in the system
fun main<CoinType>(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector<u8>,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    LibraAccount::create_child_vasp_account<CoinType>(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    if (child_initial_balance > 0) {
        let vasp_withdrawal_cap = LibraAccount::extract_withdraw_capability(parent_vasp);
        LibraAccount::pay_from<CoinType>(&vasp_withdrawal_cap, child_address, child_initial_balance);
        LibraAccount::restore_withdraw_capability(vasp_withdrawal_cap);
    };
}
}
