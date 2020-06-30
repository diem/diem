script {
use 0x1::LibraAccount;
use 0x1::Roles::{Self, LibraRootRole};

/// Create an account with the ParentVASP role at `address` with authentication key
/// `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If
/// `add_all_currencies` is true, 0 balances for all available currencies in the system will
/// also be added. This can only be invoked by an Association account.
/// The `human_name`, `base_url`, and compliance_public_key` fields of the
/// ParentVASP are filled in with dummy information.
fun create_testing_account<CoinType>(
    association: &signer,
    new_account_address: address,
    auth_key_prefix: vector<u8>,
    add_all_currencies: bool
) {
    let assoc_root_capability = Roles::extract_privilege_to_capability<LibraRootRole>(association);
    LibraAccount::create_parent_vasp_account<CoinType>(
        association,
        &assoc_root_capability,
        new_account_address,
        auth_key_prefix,
        b"testnet",
        b"https://libra.org",
        // A bogus (but valid ed25519) compliance public key
        x"b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
        add_all_currencies,
    );
    Roles::restore_capability_to_privilege(association, assoc_root_capability);
}
}
