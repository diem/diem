script {
use 0x1::LibraAccount;
use 0x1::Roles::{Self, ParentVASPRole};

/// Create a `ChildVASP` account for sender `parent_vasp` at `child_address` with a balance of
/// `child_initial_balance` in `CoinType` and an initial authentication_key
/// `auth_key_prefix | child_address`.
/// If `add_all_currencies` is true, the child address will have a zero balance in all available
/// currencies in the system.
/// This account will a child of the transaction sender, which must be a ParentVASP.
fun create_child_vasp_account<CoinType>(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector<u8>,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    let parent_vasp_capability = Roles::extract_privilege_to_capability<ParentVASPRole>(parent_vasp);
    LibraAccount::create_child_vasp_account<CoinType>(
        parent_vasp,
        &parent_vasp_capability,
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
    Roles::restore_capability_to_privilege(parent_vasp, parent_vasp_capability);
}
}
