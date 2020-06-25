script {
use 0x1::LibraAccount;
use 0x1::Roles::{Self, LibraRootRole};

/// Create `amount` coins for `payee`.
fun mint<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
  if (!LibraAccount::exists_at(payee)) {
      LibraAccount::create_testnet_account<Token>(
        account,
        &assoc_root_role,
        payee,
        auth_key_prefix
      )
  };
  LibraAccount::mint_to_address<Token>(account, payee, amount);
  Roles::restore_capability_to_privilege(account, assoc_root_role);
}
}
