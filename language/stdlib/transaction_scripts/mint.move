script {
use 0x1::LibraAccount;

/// Create `amount` coins for `payee`.
fun mint<Token>(account: &signer, payee: address, amount: u64) {
  assert(LibraAccount::exists_at(payee), 8000971);
  LibraAccount::mint_to_address<Token>(account, payee, amount);
}
}
