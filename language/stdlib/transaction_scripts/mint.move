script {
use 0x0::LibraAccount;
fun main<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!LibraAccount::exists_at(payee)) {
      LibraAccount::create_testnet_account<Token>(account, payee, auth_key_prefix)
  };
  LibraAccount::mint_to_address<Token>(account, payee, amount);
}
}
