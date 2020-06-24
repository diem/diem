script {
use 0x1::LibraAccount;
use 0x1::Roles::{Self, AssociationRootRole};

/// Create an account with the ParentVASP role at `address` with authentication key
/// `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If
/// `add_all_currencies` is true, 0 balances for all available currencies in the system will
/// also be added. This can only be invoked by an Association account.
fun create_parent_vasp_account<CoinType>(
    association: &signer,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    human_name: vector<u8>,
    base_url: vector<u8>,
    compliance_public_key: vector<u8>,
    add_all_currencies: bool
) {
    let assoc_root_capability = Roles::extract_privilege_to_capability<AssociationRootRole>(association);
    LibraAccount::create_parent_vasp_account<CoinType>(
        association,
        &assoc_root_capability,
        new_account_address,
        auth_key_prefix,
        human_name,
        base_url,
        compliance_public_key,
        add_all_currencies
    );
    Roles::restore_capability_to_privilege(association, assoc_root_capability);
}
}
